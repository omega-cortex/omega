//! SQLite-backed persistent memory store.

use omega_core::{
    config::MemoryConfig,
    context::{Context, ContextEntry},
    error::OmegaError,
    message::{IncomingMessage, OutgoingMessage},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

/// How long (in minutes) before a conversation is considered idle.
const CONVERSATION_TIMEOUT_MINUTES: i64 = 30;

/// Persistent memory store backed by SQLite.
#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
    max_context_messages: usize,
}

impl Store {
    /// Create a new store, running migrations on first use.
    pub async fn new(config: &MemoryConfig) -> Result<Self, OmegaError> {
        let db_path = shellexpand(&config.db_path);

        // Ensure parent directory exists.
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OmegaError::Memory(format!("failed to create data dir: {e}")))?;
        }

        let opts = SqliteConnectOptions::from_str(&format!("sqlite:{db_path}"))
            .map_err(|e| OmegaError::Memory(format!("invalid db path: {e}")))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(opts)
            .await
            .map_err(|e| OmegaError::Memory(format!("failed to connect to sqlite: {e}")))?;

        // Run migrations.
        Self::run_migrations(&pool).await?;

        info!("Memory store initialized at {db_path}");

        Ok(Self {
            pool,
            max_context_messages: config.max_context_messages,
        })
    }

    /// Get a reference to the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run SQL migrations.
    async fn run_migrations(pool: &SqlitePool) -> Result<(), OmegaError> {
        for migration in &[
            include_str!("../migrations/001_init.sql"),
            include_str!("../migrations/002_audit_log.sql"),
            include_str!("../migrations/003_memory_enhancement.sql"),
        ] {
            sqlx::raw_sql(migration)
                .execute(pool)
                .await
                .map_err(|e| OmegaError::Memory(format!("migration failed: {e}")))?;
        }
        Ok(())
    }

    /// Get or create an active conversation for a given channel + sender.
    ///
    /// Only returns conversations that are `active` AND have `last_activity`
    /// within the timeout window. Otherwise creates a new one.
    async fn get_or_create_conversation(
        &self,
        channel: &str,
        sender_id: &str,
    ) -> Result<String, OmegaError> {
        // Find active conversation within the timeout window.
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM conversations \
             WHERE channel = ? AND sender_id = ? AND status = 'active' \
             AND datetime(last_activity) > datetime('now', ? || ' minutes') \
             ORDER BY last_activity DESC LIMIT 1",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(-CONVERSATION_TIMEOUT_MINUTES)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        if let Some((id,)) = row {
            // Update last_activity timestamp.
            sqlx::query(
                "UPDATE conversations SET last_activity = datetime('now'), updated_at = datetime('now') WHERE id = ?",
            )
            .bind(&id)
            .execute(&self.pool)
            .await
            .map_err(|e| OmegaError::Memory(format!("update failed: {e}")))?;
            return Ok(id);
        }

        // Create new conversation.
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO conversations (id, channel, sender_id, status, last_activity) \
             VALUES (?, ?, ?, 'active', datetime('now'))",
        )
        .bind(&id)
        .bind(channel)
        .bind(sender_id)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("insert failed: {e}")))?;

        Ok(id)
    }

    /// Find active conversations that have been idle beyond the timeout.
    pub async fn find_idle_conversations(
        &self,
    ) -> Result<Vec<(String, String, String)>, OmegaError> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT id, channel, sender_id FROM conversations \
             WHERE status = 'active' \
             AND datetime(last_activity) <= datetime('now', ? || ' minutes')",
        )
        .bind(-CONVERSATION_TIMEOUT_MINUTES)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Find all active conversations (for shutdown).
    pub async fn find_all_active_conversations(
        &self,
    ) -> Result<Vec<(String, String, String)>, OmegaError> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT id, channel, sender_id FROM conversations WHERE status = 'active'",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Get all messages for a conversation (for summarization).
    pub async fn get_conversation_messages(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<(String, String)>, OmegaError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT role, content FROM messages \
             WHERE conversation_id = ? ORDER BY timestamp ASC",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Close a conversation with a summary.
    pub async fn close_conversation(
        &self,
        conversation_id: &str,
        summary: &str,
    ) -> Result<(), OmegaError> {
        sqlx::query(
            "UPDATE conversations SET status = 'closed', summary = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(summary)
        .bind(conversation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("update failed: {e}")))?;

        Ok(())
    }

    /// Store a fact (upsert by sender_id + key).
    pub async fn store_fact(
        &self,
        sender_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), OmegaError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO facts (id, sender_id, key, value) VALUES (?, ?, ?, ?) \
             ON CONFLICT(sender_id, key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')",
        )
        .bind(&id)
        .bind(sender_id)
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("upsert fact failed: {e}")))?;

        Ok(())
    }

    /// Get all facts for a sender.
    pub async fn get_facts(&self, sender_id: &str) -> Result<Vec<(String, String)>, OmegaError> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM facts WHERE sender_id = ? ORDER BY key")
                .bind(sender_id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Get recent closed conversation summaries for a sender.
    pub async fn get_recent_summaries(
        &self,
        channel: &str,
        sender_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, String)>, OmegaError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT summary, updated_at FROM conversations \
             WHERE channel = ? AND sender_id = ? AND status = 'closed' AND summary IS NOT NULL \
             ORDER BY updated_at DESC LIMIT ?",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Get memory statistics for a sender.
    pub async fn get_memory_stats(&self, sender_id: &str) -> Result<(i64, i64, i64), OmegaError> {
        let (conv_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE sender_id = ?")
                .bind(sender_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        let (msg_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM messages m \
             JOIN conversations c ON m.conversation_id = c.id \
             WHERE c.sender_id = ?",
        )
        .bind(sender_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        let (fact_count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM facts WHERE sender_id = ?")
                .bind(sender_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok((conv_count, msg_count, fact_count))
    }

    /// Get conversation history (summaries with timestamps) for a sender.
    pub async fn get_history(
        &self,
        channel: &str,
        sender_id: &str,
        limit: i64,
    ) -> Result<Vec<(String, String)>, OmegaError> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT COALESCE(summary, '(no summary)'), updated_at FROM conversations \
             WHERE channel = ? AND sender_id = ? AND status = 'closed' \
             ORDER BY updated_at DESC LIMIT ?",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        Ok(rows)
    }

    /// Delete facts for a sender — all facts if key is None, specific fact if key is Some.
    pub async fn delete_facts(
        &self,
        sender_id: &str,
        key: Option<&str>,
    ) -> Result<u64, OmegaError> {
        let result = if let Some(k) = key {
            sqlx::query("DELETE FROM facts WHERE sender_id = ? AND key = ?")
                .bind(sender_id)
                .bind(k)
                .execute(&self.pool)
                .await
        } else {
            sqlx::query("DELETE FROM facts WHERE sender_id = ?")
                .bind(sender_id)
                .execute(&self.pool)
                .await
        };

        result
            .map(|r| r.rows_affected())
            .map_err(|e| OmegaError::Memory(format!("delete failed: {e}")))
    }

    /// Close the current active conversation for a sender (for /forget).
    pub async fn close_current_conversation(
        &self,
        channel: &str,
        sender_id: &str,
    ) -> Result<bool, OmegaError> {
        let result = sqlx::query(
            "UPDATE conversations SET status = 'closed', updated_at = datetime('now') \
             WHERE channel = ? AND sender_id = ? AND status = 'active'",
        )
        .bind(channel)
        .bind(sender_id)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("update failed: {e}")))?;

        Ok(result.rows_affected() > 0)
    }

    /// Get the database file size in bytes.
    pub async fn db_size(&self) -> Result<u64, OmegaError> {
        let (page_count,): (i64,) = sqlx::query_as("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| OmegaError::Memory(format!("pragma failed: {e}")))?;

        let (page_size,): (i64,) = sqlx::query_as("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| OmegaError::Memory(format!("pragma failed: {e}")))?;

        Ok((page_count * page_size) as u64)
    }

    /// Build a conversation context from memory for the provider.
    pub async fn build_context(&self, incoming: &IncomingMessage) -> Result<Context, OmegaError> {
        let conv_id = self
            .get_or_create_conversation(&incoming.channel, &incoming.sender_id)
            .await?;

        // Load recent messages from this conversation.
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT role, content FROM messages WHERE conversation_id = ? ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(&conv_id)
        .bind(self.max_context_messages as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;

        // Rows come newest-first, reverse for chronological order.
        let history: Vec<ContextEntry> = rows
            .into_iter()
            .rev()
            .map(|(role, content)| ContextEntry { role, content })
            .collect();

        // Fetch facts and summaries for enriched context.
        let facts = self
            .get_facts(&incoming.sender_id)
            .await
            .unwrap_or_default();
        let summaries = self
            .get_recent_summaries(&incoming.channel, &incoming.sender_id, 3)
            .await
            .unwrap_or_default();

        let system_prompt = build_system_prompt(&facts, &summaries, &incoming.text);

        Ok(Context {
            system_prompt,
            history,
            current_message: incoming.text.clone(),
        })
    }

    /// Store a user message and assistant response.
    pub async fn store_exchange(
        &self,
        incoming: &IncomingMessage,
        response: &OutgoingMessage,
    ) -> Result<(), OmegaError> {
        let conv_id = self
            .get_or_create_conversation(&incoming.channel, &incoming.sender_id)
            .await?;

        // Store user message.
        let user_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content) VALUES (?, ?, 'user', ?)",
        )
        .bind(&user_id)
        .bind(&conv_id)
        .bind(&incoming.text)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("insert failed: {e}")))?;

        // Store assistant response.
        let asst_id = Uuid::new_v4().to_string();
        let metadata_json = serde_json::to_string(&response.metadata)
            .map_err(|e| OmegaError::Memory(format!("serialize failed: {e}")))?;

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, metadata_json) VALUES (?, ?, 'assistant', ?, ?)",
        )
        .bind(&asst_id)
        .bind(&conv_id)
        .bind(&response.text)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("insert failed: {e}")))?;

        Ok(())
    }
}

/// Expand `~` to home directory.
fn shellexpand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}/{rest}", home.to_string_lossy());
        }
    }
    path.to_string()
}

/// Build a dynamic system prompt enriched with facts and conversation history.
fn build_system_prompt(
    facts: &[(String, String)],
    summaries: &[(String, String)],
    current_message: &str,
) -> String {
    let mut prompt = String::from(
        "You are Omega, a personal AI agent running on the owner's infrastructure.\n\
         You are NOT a chatbot. You are an agent that DOES things.\n\n\
         Rules:\n\
         - When asked to DO something, DO IT. Don't explain how.\n\
         - Answer concisely. No preamble.\n\
         - Speak the same language the user uses.\n\
         - Reference past conversations naturally when relevant.\n\
         - Never apologize unnecessarily.",
    );

    if !facts.is_empty() {
        prompt.push_str("\n\nKnown facts about this user:");
        for (key, value) in facts {
            prompt.push_str(&format!("\n- {key}: {value}"));
        }
    }

    if !summaries.is_empty() {
        prompt.push_str("\n\nRecent conversation history:");
        for (summary, timestamp) in summaries {
            prompt.push_str(&format!("\n- [{timestamp}] {summary}"));
        }
    }

    if likely_spanish(current_message) {
        prompt.push_str("\n\nRespond in Spanish.");
    }

    prompt
}

/// Simple heuristic to detect if a message is likely in Spanish.
fn likely_spanish(text: &str) -> bool {
    let lower = text.to_lowercase();
    let markers = [
        " que ", " por ", " para ", " como ", " con ", " una ", " los ", " las ", " del ",
        " tiene ", " hace ", " esto ", " esta ", " pero ", "hola", "gracias", "buenos", "buenas",
        "dime", "necesito", "quiero", "puedes", "puedo",
    ];
    let count = markers.iter().filter(|m| lower.contains(**m)).count();
    count >= 3
}
