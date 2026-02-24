//! Project-scoped CLI session persistence.
//!
//! Replaces the in-memory `HashMap<String, String>` with SQLite-backed sessions
//! that survive restarts and are scoped per (channel, sender_id, project).

use super::Store;
use omega_core::error::OmegaError;
use uuid::Uuid;

impl Store {
    /// Upsert a CLI session for a (channel, sender_id, project) tuple.
    ///
    /// If a session already exists for the same key, updates the session_id.
    pub async fn store_session(
        &self,
        channel: &str,
        sender_id: &str,
        project: &str,
        session_id: &str,
    ) -> Result<(), OmegaError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO project_sessions (id, channel, sender_id, project, session_id) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(channel, sender_id, project) \
             DO UPDATE SET session_id = excluded.session_id, updated_at = datetime('now')",
        )
        .bind(&id)
        .bind(channel)
        .bind(sender_id)
        .bind(project)
        .bind(session_id)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("store_session failed: {e}")))?;

        Ok(())
    }

    /// Look up the CLI session_id for a (channel, sender_id, project) tuple.
    pub async fn get_session(
        &self,
        channel: &str,
        sender_id: &str,
        project: &str,
    ) -> Result<Option<String>, OmegaError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT session_id FROM project_sessions \
             WHERE channel = ? AND sender_id = ? AND project = ?",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(project)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("get_session failed: {e}")))?;

        Ok(row.map(|(sid,)| sid))
    }

    /// Delete the CLI session for a specific (channel, sender_id, project).
    pub async fn clear_session(
        &self,
        channel: &str,
        sender_id: &str,
        project: &str,
    ) -> Result<(), OmegaError> {
        sqlx::query(
            "DELETE FROM project_sessions \
             WHERE channel = ? AND sender_id = ? AND project = ?",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(project)
        .execute(&self.pool)
        .await
        .map_err(|e| OmegaError::Memory(format!("clear_session failed: {e}")))?;

        Ok(())
    }

    /// Delete all CLI sessions for a sender (used by /forget-all scenarios).
    pub async fn clear_all_sessions_for_sender(&self, sender_id: &str) -> Result<(), OmegaError> {
        sqlx::query("DELETE FROM project_sessions WHERE sender_id = ?")
            .bind(sender_id)
            .execute(&self.pool)
            .await
            .map_err(|e| OmegaError::Memory(format!("clear_all_sessions failed: {e}")))?;

        Ok(())
    }
}
