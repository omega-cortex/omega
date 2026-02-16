# Specification: omega-memory/src/store.rs

## File Path
`/Users/isudoajl/ownCloud/Projects/omega/crates/omega-memory/src/store.rs`

## Purpose
SQLite-backed persistent memory store for Omega. Manages conversation lifecycle (creation, activity tracking, idle detection, closure), message storage, user fact persistence, context building for AI providers, and memory statistics. This is the central data layer that enables Omega to maintain conversation continuity, user personalization, and long-term memory across sessions.

## Architecture Overview

### Core Responsibility
The store owns all read/write access to the SQLite database for conversation data, messages, and facts. It is consumed by the gateway (for the message pipeline) and by background tasks (for summarization and shutdown). The store does **not** manage the audit log -- that is handled by `AuditLogger` which shares the same database pool.

### Database Location
Default: `~/.omega/memory.db` (configurable via `MemoryConfig.db_path`). The `~` prefix is expanded at runtime via the `shellexpand()` helper.

## Constants

| Name | Type | Value | Description |
|------|------|-------|-------------|
| `CONVERSATION_TIMEOUT_MINUTES` | `i64` | `30` | Minutes of inactivity before a conversation is considered idle and eligible for summarization/closure. |

## Data Structures

### Store

```rust
#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
    max_context_messages: usize,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `pool` | `SqlitePool` | SQLite connection pool (max 4 connections, WAL journal mode). |
| `max_context_messages` | `usize` | Maximum number of recent messages to include in context for the provider. Sourced from `MemoryConfig.max_context_messages` (default: 50). |

**Traits derived:** `Clone`.

**Thread safety:** `SqlitePool` is `Send + Sync`, so `Store` can be safely shared across tokio tasks via cloning.

## Database Schema

The store manages three tables created across three migrations. A fourth table (`_migrations`) tracks migration state.

### Table: `_migrations`

Created directly in `run_migrations()`, not via a migration file.

```sql
CREATE TABLE IF NOT EXISTS _migrations (
    name       TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `name` | `TEXT` | `PRIMARY KEY` | Migration identifier (e.g., `"001_init"`). |
| `applied_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | ISO-8601 timestamp of when the migration was applied. |

### Table: `conversations`

Created by `001_init.sql`, extended by `003_memory_enhancement.sql`.

```sql
CREATE TABLE IF NOT EXISTS conversations (
    id            TEXT PRIMARY KEY,
    channel       TEXT NOT NULL,
    sender_id     TEXT NOT NULL,
    started_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    -- Added by 003_memory_enhancement:
    summary       TEXT,
    last_activity TEXT NOT NULL DEFAULT (datetime('now')),
    status        TEXT NOT NULL DEFAULT 'active'
);
```

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `TEXT` | `PRIMARY KEY` | UUID v4 string. |
| `channel` | `TEXT` | `NOT NULL` | Channel name (e.g., `"telegram"`, `"whatsapp"`). |
| `sender_id` | `TEXT` | `NOT NULL` | Platform-specific user identifier. |
| `started_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the conversation was created. |
| `updated_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | Last modification timestamp. |
| `summary` | `TEXT` | nullable | AI-generated 1-2 sentence summary, set when conversation is closed. |
| `last_activity` | `TEXT` | `NOT NULL`, default `datetime('now')` | Timestamp of most recent message exchange. Used for idle detection. |
| `status` | `TEXT` | `NOT NULL`, default `'active'` | Either `'active'` or `'closed'`. |

**Indexes:**
- `idx_conversations_channel_sender` on `(channel, sender_id)` -- for lookup by user.
- `idx_conversations_status` on `(status, last_activity)` -- for idle conversation queries.

### Table: `messages`

Created by `001_init.sql`.

```sql
CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id),
    role            TEXT NOT NULL CHECK (role IN ('user', 'assistant')),
    content         TEXT NOT NULL,
    timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
    metadata_json   TEXT
);
```

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `TEXT` | `PRIMARY KEY` | UUID v4 string. |
| `conversation_id` | `TEXT` | `NOT NULL`, FK to `conversations(id)` | The conversation this message belongs to. |
| `role` | `TEXT` | `NOT NULL`, CHECK `IN ('user', 'assistant')` | Who sent this message. |
| `content` | `TEXT` | `NOT NULL` | The message text. |
| `timestamp` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the message was stored. |
| `metadata_json` | `TEXT` | nullable | JSON-serialized `MessageMetadata` for assistant messages. |

**Index:**
- `idx_messages_conversation` on `(conversation_id, timestamp)` -- for conversation history queries.

### Table: `facts`

Created by `001_init.sql`, **dropped and recreated** by `003_memory_enhancement.sql` to add `sender_id` scoping.

```sql
CREATE TABLE facts (
    id                TEXT PRIMARY KEY,
    sender_id         TEXT NOT NULL,
    key               TEXT NOT NULL,
    value             TEXT NOT NULL,
    source_message_id TEXT REFERENCES messages(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(sender_id, key)
);
```

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `TEXT` | `PRIMARY KEY` | UUID v4 string. |
| `sender_id` | `TEXT` | `NOT NULL` | The user this fact belongs to. |
| `key` | `TEXT` | `NOT NULL` | Fact key (e.g., `"name"`, `"timezone"`, `"preference"`). |
| `value` | `TEXT` | `NOT NULL` | Fact value (e.g., `"Alice"`, `"America/New_York"`). |
| `source_message_id` | `TEXT` | FK to `messages(id)`, nullable | The message that originated this fact (currently unused by store code). |
| `created_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the fact was first stored. |
| `updated_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the fact was last updated (via upsert). |

**Unique constraint:** `(sender_id, key)` -- each user can have at most one value per key.

### Table: `audit_log`

Created by `002_audit_log.sql`. Not accessed by `Store` directly (managed by `AuditLogger`), but shares the same database pool.

## Migrations

### Migration Tracking

Migrations are tracked via the `_migrations` table. The system handles three scenarios:

1. **Fresh database** -- No tables exist. All migrations run in order.
2. **Pre-tracking database** -- Tables exist from before migration tracking was added. The `run_migrations()` method detects this by checking if the `conversations` table has the `summary` column (added in migration 003). If so, all three migrations are marked as applied without re-running.
3. **Normal operation** -- Each migration is checked against `_migrations` and skipped if already applied.

### Migration Files

| Name | File | Purpose |
|------|------|---------|
| `001_init` | `migrations/001_init.sql` | Creates `conversations`, `messages`, `facts` tables with indexes. |
| `002_audit_log` | `migrations/002_audit_log.sql` | Creates `audit_log` table with indexes. |
| `003_memory_enhancement` | `migrations/003_memory_enhancement.sql` | Adds `summary`, `last_activity`, `status` to `conversations`. Recreates `facts` with `sender_id` scoping. |

### Bootstrap Detection Logic

```
IF _migrations table is empty:
    IF conversations table has "summary" column:
        → Mark 001_init, 002_audit_log, 003_memory_enhancement as applied
    ELSE:
        → All migrations will run normally
```

**SQL used for detection:**
```sql
SELECT COUNT(*) FROM _migrations
SELECT sql FROM sqlite_master WHERE type='table' AND name='conversations'
```

The `sql` column from `sqlite_master` contains the CREATE TABLE statement. The code checks if it contains the substring `"summary"`.

## Functions

### Public Methods

#### `async fn new(config: &MemoryConfig) -> Result<Self, OmegaError>`

**Purpose:** Create a new store instance, initializing the database and running migrations.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `config` | `&MemoryConfig` | Memory configuration with `db_path` and `max_context_messages`. |

**Returns:** `Result<Self, OmegaError>`.

**Logic:**
1. Expand `~` in `config.db_path` via `shellexpand()`.
2. Create the parent directory if it does not exist.
3. Build `SqliteConnectOptions` with:
   - `create_if_missing(true)` -- create the database file if absent.
   - `journal_mode(Wal)` -- WAL mode for concurrent reads.
4. Create a connection pool with `max_connections(4)`.
5. Run all migrations via `run_migrations()`.
6. Log initialization with `info!`.
7. Return the new `Store`.

**Error conditions:**
- Parent directory creation fails.
- Invalid database path.
- SQLite connection failure.
- Migration failure.

---

#### `fn pool(&self) -> &SqlitePool`

**Purpose:** Get a reference to the underlying connection pool for direct SQL access.

**Parameters:** None.

**Returns:** `&SqlitePool`.

**Usage:** Called by `AuditLogger` construction and by `gateway.rs` for direct queries (e.g., fetching `sender_id` from conversations during summarization).

---

#### `async fn find_idle_conversations(&self) -> Result<Vec<(String, String, String)>, OmegaError>`

**Purpose:** Find active conversations that have been idle beyond the timeout threshold.

**Parameters:** None.

**Returns:** `Result<Vec<(String, String, String)>, OmegaError>` where each tuple is `(conversation_id, channel, sender_id)`.

**SQL:**
```sql
SELECT id, channel, sender_id FROM conversations
WHERE status = 'active'
AND datetime(last_activity) <= datetime('now', ? || ' minutes')
```

**Bind parameters:**
- `?` = `-CONVERSATION_TIMEOUT_MINUTES` (i.e., `-30`), which SQLite interprets as "30 minutes ago".

**Called by:** `gateway.rs::background_summarizer()` every 60 seconds.

---

#### `async fn find_all_active_conversations(&self) -> Result<Vec<(String, String, String)>, OmegaError>`

**Purpose:** Find all currently active conversations, regardless of idle time.

**Parameters:** None.

**Returns:** `Result<Vec<(String, String, String)>, OmegaError>` where each tuple is `(conversation_id, channel, sender_id)`.

**SQL:**
```sql
SELECT id, channel, sender_id FROM conversations WHERE status = 'active'
```

**Called by:** `gateway.rs::shutdown()` to summarize all conversations before exit.

---

#### `async fn get_conversation_messages(&self, conversation_id: &str) -> Result<Vec<(String, String)>, OmegaError>`

**Purpose:** Get all messages for a conversation, ordered chronologically.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `conversation_id` | `&str` | The conversation UUID. |

**Returns:** `Result<Vec<(String, String)>, OmegaError>` where each tuple is `(role, content)`.

**SQL:**
```sql
SELECT role, content FROM messages
WHERE conversation_id = ? ORDER BY timestamp ASC
```

**Called by:** `gateway.rs::summarize_conversation()` to build a transcript for the AI summarizer.

---

#### `async fn close_conversation(&self, conversation_id: &str, summary: &str) -> Result<(), OmegaError>`

**Purpose:** Mark a conversation as closed and store its summary.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `conversation_id` | `&str` | The conversation UUID. |
| `summary` | `&str` | AI-generated summary text. |

**Returns:** `Result<(), OmegaError>`.

**SQL:**
```sql
UPDATE conversations SET status = 'closed', summary = ?, updated_at = datetime('now') WHERE id = ?
```

**Called by:** `gateway.rs::summarize_conversation()` after summarization and fact extraction.

---

#### `async fn store_fact(&self, sender_id: &str, key: &str, value: &str) -> Result<(), OmegaError>`

**Purpose:** Store a user fact, upserting on `(sender_id, key)` conflict.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `sender_id` | `&str` | The user this fact belongs to. |
| `key` | `&str` | Fact key (e.g., `"name"`, `"timezone"`). |
| `value` | `&str` | Fact value (e.g., `"Alice"`, `"America/New_York"`). |

**Returns:** `Result<(), OmegaError>`.

**SQL:**
```sql
INSERT INTO facts (id, sender_id, key, value) VALUES (?, ?, ?, ?)
ON CONFLICT(sender_id, key) DO UPDATE SET value = excluded.value, updated_at = datetime('now')
```

**Behavior:** If a fact with the same `(sender_id, key)` already exists, the value is updated and `updated_at` is refreshed. Otherwise, a new row is inserted with a fresh UUID.

**Called by:** `gateway.rs::summarize_conversation()` for each extracted fact.

---

#### `async fn get_facts(&self, sender_id: &str) -> Result<Vec<(String, String)>, OmegaError>`

**Purpose:** Get all stored facts for a user.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `sender_id` | `&str` | The user whose facts to retrieve. |

**Returns:** `Result<Vec<(String, String)>, OmegaError>` where each tuple is `(key, value)`, ordered alphabetically by key.

**SQL:**
```sql
SELECT key, value FROM facts WHERE sender_id = ? ORDER BY key
```

**Called by:** `build_context()` for enriching the system prompt, and `commands.rs` for the `/facts` command.

---

#### `async fn get_recent_summaries(&self, channel: &str, sender_id: &str, limit: i64) -> Result<Vec<(String, String)>, OmegaError>`

**Purpose:** Get recent closed conversation summaries for a user on a specific channel.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `channel` | `&str` | Channel name (e.g., `"telegram"`). |
| `sender_id` | `&str` | The user whose summaries to retrieve. |
| `limit` | `i64` | Maximum number of summaries to return. |

**Returns:** `Result<Vec<(String, String)>, OmegaError>` where each tuple is `(summary, updated_at)`, ordered newest first.

**SQL:**
```sql
SELECT summary, updated_at FROM conversations
WHERE channel = ? AND sender_id = ? AND status = 'closed' AND summary IS NOT NULL
ORDER BY updated_at DESC LIMIT ?
```

**Called by:** `build_context()` with `limit = 3` to include the 3 most recent conversation summaries in the system prompt.

---

#### `async fn get_memory_stats(&self, sender_id: &str) -> Result<(i64, i64, i64), OmegaError>`

**Purpose:** Get aggregate memory statistics for a user.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `sender_id` | `&str` | The user to get stats for. |

**Returns:** `Result<(i64, i64, i64), OmegaError>` where the tuple is `(conversation_count, message_count, fact_count)`.

**SQL (three queries):**

```sql
-- Conversation count
SELECT COUNT(*) FROM conversations WHERE sender_id = ?

-- Message count (via join)
SELECT COUNT(*) FROM messages m
JOIN conversations c ON m.conversation_id = c.id
WHERE c.sender_id = ?

-- Fact count
SELECT COUNT(*) FROM facts WHERE sender_id = ?
```

**Called by:** `commands.rs` for the `/status` command.

---

#### `async fn get_history(&self, channel: &str, sender_id: &str, limit: i64) -> Result<Vec<(String, String)>, OmegaError>`

**Purpose:** Get conversation history (summaries with timestamps) for a user.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `channel` | `&str` | Channel name. |
| `sender_id` | `&str` | The user whose history to retrieve. |
| `limit` | `i64` | Maximum number of history entries. |

**Returns:** `Result<Vec<(String, String)>, OmegaError>` where each tuple is `(summary_or_fallback, updated_at)`, ordered newest first.

**SQL:**
```sql
SELECT COALESCE(summary, '(no summary)'), updated_at FROM conversations
WHERE channel = ? AND sender_id = ? AND status = 'closed'
ORDER BY updated_at DESC LIMIT ?
```

**Note:** Uses `COALESCE` to handle conversations that were closed without a summary (returns `"(no summary)"` as fallback).

**Called by:** `commands.rs` for the `/memory` or `/history` command.

---

#### `async fn delete_facts(&self, sender_id: &str, key: Option<&str>) -> Result<u64, OmegaError>`

**Purpose:** Delete facts for a user -- either all facts or a specific fact by key.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `sender_id` | `&str` | The user whose facts to delete. |
| `key` | `Option<&str>` | If `Some(k)`, delete only the fact with that key. If `None`, delete all facts for the user. |

**Returns:** `Result<u64, OmegaError>` -- the number of rows deleted.

**SQL (conditional):**
```sql
-- When key is Some(k):
DELETE FROM facts WHERE sender_id = ? AND key = ?

-- When key is None:
DELETE FROM facts WHERE sender_id = ?
```

**Called by:** `commands.rs` for the `/forget` command (fact deletion variant).

---

#### `async fn close_current_conversation(&self, channel: &str, sender_id: &str) -> Result<bool, OmegaError>`

**Purpose:** Close the active conversation for a user without a summary (for the `/forget` command).

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `channel` | `&str` | Channel name. |
| `sender_id` | `&str` | The user whose conversation to close. |

**Returns:** `Result<bool, OmegaError>` -- `true` if a conversation was closed, `false` if none was active.

**SQL:**
```sql
UPDATE conversations SET status = 'closed', updated_at = datetime('now')
WHERE channel = ? AND sender_id = ? AND status = 'active'
```

**Note:** Does not set a summary. The conversation is simply marked as closed.

**Called by:** `commands.rs` for the `/forget` command (conversation reset variant).

---

#### `async fn db_size(&self) -> Result<u64, OmegaError>`

**Purpose:** Get the database file size in bytes.

**Parameters:** None.

**Returns:** `Result<u64, OmegaError>`.

**SQL (two PRAGMA queries):**
```sql
PRAGMA page_count
PRAGMA page_size
```

**Calculation:** `page_count * page_size` cast to `u64`.

**Called by:** `commands.rs` for the `/status` command.

---

#### `async fn build_context(&self, incoming: &IncomingMessage) -> Result<Context, OmegaError>`

**Purpose:** Build a complete conversation context for the AI provider from the current conversation state, user facts, and recent summaries.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `incoming` | `&IncomingMessage` | The incoming message (provides `channel`, `sender_id`, `text`). |

**Returns:** `Result<Context, OmegaError>`.

**Logic:**
1. Call `get_or_create_conversation(channel, sender_id)` to get the active conversation ID.
2. Fetch recent messages from the conversation (newest first, then reversed to chronological order).
3. Fetch all facts for the sender (errors suppressed, default to empty).
4. Fetch the 3 most recent closed conversation summaries (errors suppressed, default to empty).
5. Build a dynamic system prompt via `build_system_prompt()`.
6. Return a `Context` with the system prompt, history, and current message.

**SQL (step 2):**
```sql
SELECT role, content FROM messages WHERE conversation_id = ? ORDER BY timestamp DESC LIMIT ?
```

**Bind parameters:** `conversation_id`, `max_context_messages`.

**Context construction flow:**
```
get_or_create_conversation(channel, sender_id)
          |
          v
    conversation_id
          |
    ┌─────┴─────────────────────────┐
    v                                v
SELECT messages                 get_facts(sender_id)
(newest N, reversed)                 |
    |                                v
    |                         get_recent_summaries(channel, sender_id, 3)
    |                                |
    v                                v
history: Vec<ContextEntry>     build_system_prompt(facts, summaries, text)
    |                                |
    v                                v
    └───────────┬───────────────────┘
                v
         Context {
           system_prompt,
           history,
           current_message: incoming.text
         }
```

**Error handling:**
- `get_or_create_conversation` failure propagates as `OmegaError`.
- Message query failure propagates as `OmegaError`.
- `get_facts` failure silently defaults to empty vec (`unwrap_or_default()`).
- `get_recent_summaries` failure silently defaults to empty vec (`unwrap_or_default()`).

---

#### `async fn store_exchange(&self, incoming: &IncomingMessage, response: &OutgoingMessage) -> Result<(), OmegaError>`

**Purpose:** Store a user message and the corresponding assistant response in the database.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `incoming` | `&IncomingMessage` | The user's message. |
| `response` | `&OutgoingMessage` | The assistant's response. |

**Returns:** `Result<(), OmegaError>`.

**Logic:**
1. Call `get_or_create_conversation(channel, sender_id)` to get the conversation ID.
2. Insert the user message with `role = 'user'`.
3. Serialize `response.metadata` to JSON.
4. Insert the assistant message with `role = 'assistant'` and `metadata_json`.

**SQL (two inserts):**
```sql
-- User message
INSERT INTO messages (id, conversation_id, role, content) VALUES (?, ?, 'user', ?)

-- Assistant message
INSERT INTO messages (id, conversation_id, role, content, metadata_json) VALUES (?, ?, 'assistant', ?, ?)
```

**Called by:** `gateway.rs::handle_message()` after a successful provider call.

### Private Methods

#### `async fn run_migrations(pool: &SqlitePool) -> Result<(), OmegaError>`

**Purpose:** Run SQL migrations with tracking to avoid re-execution.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `pool` | `&SqlitePool` | The database connection pool. |

**Returns:** `Result<(), OmegaError>`.

**Logic:**
1. Create `_migrations` table if it does not exist.
2. Check if any migrations have been recorded.
3. If no migrations recorded, check for pre-tracking schema and bootstrap if needed.
4. Iterate through all migration definitions.
5. For each migration, check if it has been applied. If not, execute the SQL and record it.

**Migration definitions (compile-time embedded):**
```rust
let migrations: &[(&str, &str)] = &[
    ("001_init", include_str!("../migrations/001_init.sql")),
    ("002_audit_log", include_str!("../migrations/002_audit_log.sql")),
    ("003_memory_enhancement", include_str!("../migrations/003_memory_enhancement.sql")),
];
```

---

#### `async fn get_or_create_conversation(&self, channel: &str, sender_id: &str) -> Result<String, OmegaError>`

**Purpose:** Get the active conversation for a user/channel pair, or create a new one if none exists or the existing one has timed out.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `channel` | `&str` | Channel name. |
| `sender_id` | `&str` | User identifier. |

**Returns:** `Result<String, OmegaError>` -- the conversation UUID.

**Logic:**
1. Query for an active conversation within the timeout window.
2. If found, update `last_activity` and `updated_at` to now, return the ID.
3. If not found, generate a new UUID, insert a new conversation, return the new ID.

**SQL (lookup):**
```sql
SELECT id FROM conversations
WHERE channel = ? AND sender_id = ? AND status = 'active'
AND datetime(last_activity) > datetime('now', ? || ' minutes')
ORDER BY last_activity DESC LIMIT 1
```

**SQL (touch):**
```sql
UPDATE conversations SET last_activity = datetime('now'), updated_at = datetime('now') WHERE id = ?
```

**SQL (create):**
```sql
INSERT INTO conversations (id, channel, sender_id, status, last_activity)
VALUES (?, ?, ?, 'active', datetime('now'))
```

**Conversation boundary logic:**

```
User sends message
       |
       v
Query: active + within 30min?
       |
  ┌────┴────┐
  |         |
 Yes       No
  |         |
  v         v
Touch    Create new
activity conversation
  |         |
  v         v
Return   Return
same ID  new ID
```

## Private Free Functions

### `fn shellexpand(path: &str) -> String`

**Purpose:** Expand `~` prefix to the user's home directory.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | `&str` | File path that may start with `~/`. |

**Returns:** `String` with `~` expanded to `$HOME`.

**Logic:**
1. If path starts with `~/`, replace `~` with the value of the `HOME` environment variable.
2. If `HOME` is not set or path does not start with `~/`, return the path unchanged.

---

### `fn build_system_prompt(facts: &[(String, String)], summaries: &[(String, String)], current_message: &str) -> String`

**Purpose:** Build a dynamic system prompt enriched with user facts, conversation summaries, and language detection.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `facts` | `&[(String, String)]` | User facts as `(key, value)` pairs. |
| `summaries` | `&[(String, String)]` | Recent conversation summaries as `(summary, timestamp)` pairs. |
| `current_message` | `&str` | The user's current message (used for language detection). |

**Returns:** `String` -- the complete system prompt.

**Output structure:**
```
You are Omega, a personal AI agent running on the owner's infrastructure.
You are NOT a chatbot. You are an agent that DOES things.

Rules:
- When asked to DO something, DO IT. Don't explain how.
- Answer concisely. No preamble.
- Speak the same language the user uses.
- Reference past conversations naturally when relevant.
- Never apologize unnecessarily.

Known facts about this user:          ← (only if facts is non-empty)
- name: Alice
- timezone: America/New_York

Recent conversation history:          ← (only if summaries is non-empty)
- [2024-01-15 14:30:00] User asked about Rust async patterns.
- [2024-01-14 09:15:00] User discussed project architecture.

Respond in Spanish.                   ← (only if likely_spanish() returns true)
```

**Conditional sections:**
- Facts section: appended only if `facts` is non-empty.
- Summaries section: appended only if `summaries` is non-empty.
- Spanish directive: appended only if `likely_spanish(current_message)` returns true.

---

### `fn likely_spanish(text: &str) -> bool`

**Purpose:** Simple heuristic to detect if a message is likely in Spanish.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `text` | `&str` | The text to analyze. |

**Returns:** `bool` -- `true` if 3 or more Spanish markers are found.

**Logic:**
1. Convert text to lowercase.
2. Check against a list of 21 Spanish marker words/phrases:
   - ` que `, ` por `, ` para `, ` como `, ` con `, ` una `, ` los `, ` las `, ` del `
   - ` tiene `, ` hace `, ` esto `, ` esta `, ` pero `
   - `hola`, `gracias`, `buenos`, `buenas`, `dime`, `necesito`, `quiero`, `puedes`, `puedo`
3. Count how many markers appear in the text.
4. Return `true` if count >= 3.

**Note:** This is a simple heuristic, not a language detection library. It may produce false positives for Portuguese or other Romance languages, and false negatives for short Spanish messages.

## Context Building Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     build_context()                              │
│                                                                  │
│  IncomingMessage { channel, sender_id, text }                   │
│         |                                                        │
│         v                                                        │
│  get_or_create_conversation(channel, sender_id)                 │
│         |                                                        │
│         v                                                        │
│  ┌──────────────────────────────┐                               │
│  │ Active conversation exists?  │                               │
│  │ (within 30min timeout)       │                               │
│  └──────┬───────────┬──────────┘                               │
│         |           |                                            │
│        Yes         No                                            │
│         |           |                                            │
│         v           v                                            │
│    Touch activity  Create new                                    │
│         |           |                                            │
│         └─────┬─────┘                                           │
│               v                                                  │
│        conversation_id                                           │
│               |                                                  │
│    ┌──────────┼──────────────────┐                              │
│    v          v                  v                                │
│  SELECT     get_facts()    get_recent_summaries()               │
│  messages   (sender_id)    (channel, sender_id, 3)              │
│  (DESC,     └──┬───┘       └──────┬──────────┘                  │
│   LIMIT N)     |                  |                              │
│    |           v                  v                              │
│    v       facts[]          summaries[]                          │
│  Reverse                         |                               │
│  to ASC                          v                               │
│    |           build_system_prompt(facts, summaries, text)       │
│    v                             |                               │
│  history[]                       v                               │
│    |                       system_prompt                         │
│    |                             |                               │
│    └──────────┬──────────────────┘                              │
│               v                                                  │
│         Context {                                                │
│           system_prompt,                                         │
│           history,                                               │
│           current_message: text                                  │
│         }                                                        │
└─────────────────────────────────────────────────────────────────┘
```

## Conversation Lifecycle Diagram

```
┌───────────────────────────────────────────────────────┐
│              CONVERSATION LIFECYCLE                     │
│                                                         │
│  User sends first message                              │
│         |                                               │
│         v                                               │
│  ┌─────────────┐                                       │
│  │   CREATED    │  (status='active', new UUID)         │
│  └──────┬──────┘                                       │
│         |                                               │
│         v                                               │
│  ┌─────────────┐  User sends message  ┌────────────┐  │
│  │   ACTIVE     │◄────────────────────│ Touch      │  │
│  │              │     (< 30min)       │ activity   │  │
│  └──────┬──────┘                      └────────────┘  │
│         |                                               │
│         | (30+ minutes idle)                           │
│         v                                               │
│  ┌─────────────────────────────┐                       │
│  │ background_summarizer finds │                       │
│  │ idle conversation           │                       │
│  └──────┬──────────────────────┘                       │
│         |                                               │
│         v                                               │
│  ┌─────────────┐                                       │
│  │ SUMMARIZE   │  AI generates summary                 │
│  │ + EXTRACT   │  AI extracts facts                    │
│  └──────┬──────┘                                       │
│         |                                               │
│         v                                               │
│  ┌─────────────┐                                       │
│  │   CLOSED     │  (status='closed', summary stored)   │
│  └─────────────┘                                       │
│                                                         │
│  ── Alternative closure paths ──                       │
│                                                         │
│  /forget command → close_current_conversation()        │
│    (no summary, just status='closed')                  │
│                                                         │
│  Shutdown → summarize_conversation() for all active    │
│    (summary stored, then status='closed')              │
│                                                         │
│  User sends message after 30min → new conversation     │
│    (old one stays 'active' until summarizer finds it)  │
└───────────────────────────────────────────────────────┘
```

## Error Handling Strategy

### Error Types
All errors are wrapped in `OmegaError::Memory(String)` with descriptive messages.

### Error Propagation

| Method | Error Behavior |
|--------|---------------|
| `new()` | Propagates all errors (fatal -- store cannot function without database). |
| `run_migrations()` | Propagates all errors (fatal -- schema must be correct). |
| `get_or_create_conversation()` | Propagates all errors (called internally). |
| `build_context()` | Propagates conversation/message errors. Suppresses fact/summary errors (defaults to empty). |
| `store_exchange()` | Propagates all errors (caller in gateway logs but continues). |
| `find_idle_conversations()` | Propagates errors (caller in background task logs and continues). |
| `close_conversation()` | Propagates errors. |
| `store_fact()` | Propagates errors. |
| `get_facts()` | Propagates errors. |
| `delete_facts()` | Propagates errors. |
| `close_current_conversation()` | Propagates errors. |
| `get_memory_stats()` | Propagates errors. |
| `get_history()` | Propagates errors. |
| `get_recent_summaries()` | Propagates errors. |
| `db_size()` | Propagates errors. |

### Resilience in build_context()
Facts and summaries are fetched with `unwrap_or_default()`. This ensures that a failure in retrieving personalization data does not prevent the provider from receiving a valid context. The conversation will still work; it will just lack facts and summaries.

## Dependencies

### External Crates
- `sqlx` -- SQLite driver, connection pool, query execution.
- `uuid` -- UUID v4 generation for all primary keys.
- `tracing` -- Structured logging (`info!` macro).
- `serde_json` -- Serialization of `MessageMetadata` to JSON.

### Internal Dependencies
- `omega_core::config::MemoryConfig` -- Configuration struct.
- `omega_core::context::{Context, ContextEntry}` -- Context types returned by `build_context()`.
- `omega_core::error::OmegaError` -- Error type used for all results.
- `omega_core::message::{IncomingMessage, OutgoingMessage}` -- Message types used by `build_context()` and `store_exchange()`.

## Invariants

1. Every conversation has a UUID v4 as its primary key.
2. Every message has a UUID v4 as its primary key.
3. Every fact has a UUID v4 as its primary key.
4. A conversation is either `'active'` or `'closed'` -- no other statuses exist.
5. Message roles are constrained to `'user'` or `'assistant'` by a CHECK constraint.
6. Facts are unique per `(sender_id, key)` -- duplicate inserts update the existing value.
7. `build_context()` always returns history in chronological order (oldest first).
8. `build_context()` never fails due to fact/summary retrieval errors.
9. The conversation timeout is 30 minutes -- this is a compile-time constant, not configurable.
10. Migrations are idempotent -- running them multiple times has no effect.
11. The database is created with WAL journal mode for concurrent read access.
12. Connection pool is limited to 4 connections maximum.
