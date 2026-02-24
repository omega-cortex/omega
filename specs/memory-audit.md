# Specification: omega-memory Audit System

## Path
`backend/crates/omega-memory/src/audit.rs`

## Purpose
Implements the audit log subsystem for Omega. Every interaction that flows through the gateway -- whether it succeeds, fails, or is denied by auth -- is recorded as an immutable row in the `audit_log` SQLite table. The audit module provides the data structures for describing an interaction (`AuditEntry`, `AuditStatus`) and a writer (`AuditLogger`) that inserts entries into the database.

## Dependencies
- **omega_core::error::OmegaError** -- unified error type (uses the `Memory` variant for audit failures)
- **sqlx::SqlitePool** -- shared connection pool for SQLite writes
- **tracing::debug** -- structured debug logging after each successful write
- **uuid::Uuid** -- generates UUIDv4 primary keys for each audit row

## Module Declaration
Declared public in `backend/crates/omega-memory/src/lib.rs`:

```rust
pub mod audit;
pub use audit::AuditLogger;
```

Re-exported types are consumed by the gateway as:

```rust
use omega_memory::audit::{AuditEntry, AuditLogger, AuditStatus};
```

## Data Structures

### `AuditEntry`

```rust
pub struct AuditEntry {
    pub channel: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub input_text: String,
    pub output_text: Option<String>,
    pub provider_used: Option<String>,
    pub model: Option<String>,
    pub processing_ms: Option<i64>,
    pub status: AuditStatus,
    pub denial_reason: Option<String>,
}
```

**Field Reference:**

| Field | Type | Nullable | Description |
|-------|------|----------|-------------|
| `channel` | `String` | No | Messaging platform name (e.g., `"telegram"`, `"whatsapp"`) |
| `sender_id` | `String` | No | Platform-specific user identifier (e.g., Telegram user ID as string) |
| `sender_name` | `Option<String>` | Yes | Human-readable sender name, if available from the channel |
| `input_text` | `String` | No | The user's message text (after sanitization in the gateway pipeline) |
| `output_text` | `Option<String>` | Yes | The AI provider's response text. `None` for denied messages. Contains `"ERROR: {e}"` for provider failures. |
| `provider_used` | `Option<String>` | Yes | Name of the provider that handled the request (e.g., `"Claude Code CLI"`). `None` for denials. |
| `model` | `Option<String>` | Yes | Model identifier from the provider response (e.g., `"claude-opus-4-6"`). `None` for denials and errors. |
| `processing_ms` | `Option<i64>` | Yes | Provider processing time in milliseconds. `None` for denials and errors. |
| `status` | `AuditStatus` | No | Outcome of the interaction: `Ok`, `Error`, or `Denied`. |
| `denial_reason` | `Option<String>` | Yes | Explanation for denial (e.g., `"telegram user 999 not in allowed_users"`). `None` for non-denied entries. |

**Trait Implementations:** None derived. `AuditEntry` is a plain data struct with no `Debug`, `Clone`, `Serialize`, or other derived traits.

### `AuditStatus`

```rust
pub enum AuditStatus {
    Ok,
    Error,
    Denied,
}
```

**Variant Reference:**

| Variant | String Representation | Used When |
|---------|-----------------------|-----------|
| `Ok` | `"ok"` | Provider returned a successful response |
| `Error` | `"error"` | Provider call failed |
| `Denied` | `"denied"` | Auth check rejected the sender |

**Methods:**

#### `fn as_str(&self) -> &'static str`
- **Visibility:** Private (`fn`, not `pub fn`)
- **Parameters:** `&self`
- **Returns:** `&'static str` -- one of `"ok"`, `"error"`, `"denied"`
- **Purpose:** Converts the enum variant to the string value stored in the `status` column of the `audit_log` table. The column has a `CHECK` constraint enforcing these three values.

**Trait Implementations:** None derived. `AuditStatus` has no `Debug`, `Clone`, `PartialEq`, or other derived traits.

### `AuditLogger`

```rust
pub struct AuditLogger {
    pool: SqlitePool,
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `pool` | `SqlitePool` | Shared SQLite connection pool, cloned from the `Store`'s pool at gateway construction time |

**Trait Implementations:** None derived.

## Public Methods

### `AuditLogger::new`

```rust
pub fn new(pool: SqlitePool) -> Self
```

**Parameters:**
- `pool: SqlitePool` -- a connection pool to the SQLite database that already has the `audit_log` table (created by migration `002_audit_log`)

**Returns:** `AuditLogger`

**Logic:**
- Stores the pool in the struct. No validation, no I/O.

**Error Handling:** None (infallible).

### `AuditLogger::log`

```rust
pub async fn log(&self, entry: &AuditEntry) -> Result<(), OmegaError>
```

**Parameters:**
- `entry: &AuditEntry` -- the interaction to record

**Returns:** `Result<(), OmegaError>` where the error variant is always `OmegaError::Memory`

**Logic:**
1. Generate a new UUIDv4 string for the row's primary key.
2. Execute an `INSERT INTO audit_log` query binding all 11 columns (id + 10 from `AuditEntry`).
3. On success, emit a `debug!` trace with channel, sender_id, status, and first 80 characters of input text.
4. Return `Ok(())`.

**SQL Query:**
```sql
INSERT INTO audit_log
  (id, channel, sender_id, sender_name, input_text, output_text,
   provider_used, model, processing_ms, status, denial_reason)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
```

**Column-to-Field Binding Order:**

| Bind Position | Column | Source Field |
|---------------|--------|-------------|
| 1 | `id` | Generated UUIDv4 |
| 2 | `channel` | `entry.channel` |
| 3 | `sender_id` | `entry.sender_id` |
| 4 | `sender_name` | `entry.sender_name` |
| 5 | `input_text` | `entry.input_text` |
| 6 | `output_text` | `entry.output_text` |
| 7 | `provider_used` | `entry.provider_used` |
| 8 | `model` | `entry.model` |
| 9 | `processing_ms` | `entry.processing_ms` |
| 10 | `status` | `entry.status.as_str()` |
| 11 | `denial_reason` | `entry.denial_reason` |

**Error Handling:**
- `sqlx` errors are mapped to `OmegaError::Memory(format!("audit log write failed: {e}"))`.

## Private Functions

### `truncate`

```rust
fn truncate(s: &str, max: usize) -> &str
```

**Parameters:**
- `s: &str` -- the string to truncate
- `max: usize` -- maximum byte length

**Returns:** `&str` -- a slice of the original string, at most `max` bytes

**Logic:**
- If `s.len() <= max`, returns `s` unchanged.
- Otherwise, returns `&s[..max]`.

**Note:** This function slices on byte boundaries, not character boundaries. If `max` falls in the middle of a multi-byte UTF-8 character, this will panic. In practice, the only caller passes `max = 80` on user input text, and the function is used only for debug logging, so the risk is low but not zero.

**Used by:** `AuditLogger::log` to truncate `input_text` for the `debug!` trace message.

## SQL Schema

### Table: `audit_log`

Created by migration `002_audit_log.sql`:

```sql
CREATE TABLE IF NOT EXISTS audit_log (
    id              TEXT PRIMARY KEY,
    timestamp       TEXT NOT NULL DEFAULT (datetime('now')),
    channel         TEXT NOT NULL,
    sender_id       TEXT NOT NULL,
    sender_name     TEXT,
    input_text      TEXT NOT NULL,
    output_text     TEXT,
    provider_used   TEXT,
    model           TEXT,
    processing_ms   INTEGER,
    status          TEXT NOT NULL DEFAULT 'ok' CHECK (status IN ('ok', 'error', 'denied')),
    denial_reason   TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_sender ON audit_log(channel, sender_id);
```

**Column Reference:**

| Column | Type | Constraints | Default | Set By |
|--------|------|------------|---------|--------|
| `id` | TEXT | PRIMARY KEY | -- | Rust code (UUIDv4) |
| `timestamp` | TEXT | NOT NULL | `datetime('now')` | SQLite default |
| `channel` | TEXT | NOT NULL | -- | `AuditEntry.channel` |
| `sender_id` | TEXT | NOT NULL | -- | `AuditEntry.sender_id` |
| `sender_name` | TEXT | nullable | -- | `AuditEntry.sender_name` |
| `input_text` | TEXT | NOT NULL | -- | `AuditEntry.input_text` |
| `output_text` | TEXT | nullable | -- | `AuditEntry.output_text` |
| `provider_used` | TEXT | nullable | -- | `AuditEntry.provider_used` |
| `model` | TEXT | nullable | -- | `AuditEntry.model` |
| `processing_ms` | INTEGER | nullable | -- | `AuditEntry.processing_ms` |
| `status` | TEXT | NOT NULL, CHECK | `'ok'` | `AuditStatus.as_str()` |
| `denial_reason` | TEXT | nullable | -- | `AuditEntry.denial_reason` |

**Note:** The `timestamp` column is not set by Rust code. It relies on the SQLite `DEFAULT (datetime('now'))` clause and is populated automatically at insert time.

### Indexes

| Index Name | Columns | Purpose |
|------------|---------|---------|
| `idx_audit_log_timestamp` | `timestamp` | Efficient time-range queries (e.g., "show last 24 hours") |
| `idx_audit_log_sender` | `channel, sender_id` | Efficient per-user queries (e.g., "all interactions by user X on Telegram") |

## Gateway Integration

The audit system is used at three points in the gateway's `handle_message()` pipeline in `backend/src/gateway.rs`:

### 1. Auth Denial (Pipeline Stage 1)

When `check_auth()` returns `Some(reason)`, the gateway logs a denied entry:

```rust
self.audit.log(&AuditEntry {
    channel: incoming.channel.clone(),
    sender_id: incoming.sender_id.clone(),
    sender_name: incoming.sender_name.clone(),
    input_text: incoming.text.clone(),
    output_text: None,
    provider_used: None,
    model: None,
    processing_ms: None,
    status: AuditStatus::Denied,
    denial_reason: Some(reason),
}).await;
```

**Characteristics:**
- `output_text`: `None` (no response was generated)
- `provider_used`: `None` (provider was never called)
- `model`: `None`
- `processing_ms`: `None`
- `denial_reason`: Contains the auth failure reason

### 2. Provider Error (Pipeline Stage 6)

When `provider.complete()` returns `Err(e)`, the gateway logs an error entry:

```rust
self.audit.log(&AuditEntry {
    channel: incoming.channel.clone(),
    sender_id: incoming.sender_id.clone(),
    sender_name: incoming.sender_name.clone(),
    input_text: incoming.text.clone(),
    output_text: Some(format!("ERROR: {e}")),
    provider_used: Some(self.provider.name().to_string()),
    model: None,
    processing_ms: None,
    status: AuditStatus::Error,
    denial_reason: None,
}).await;
```

**Characteristics:**
- `output_text`: Contains `"ERROR: {error_message}"` for debugging
- `provider_used`: Set to the provider name (the provider was attempted)
- `model`: `None` (no model info available on error)
- `processing_ms`: `None` (timing not captured on error)
- `denial_reason`: `None`

### 3. Successful Exchange (Pipeline Stage 8)

After the provider responds successfully and the exchange is stored in memory:

```rust
self.audit.log(&AuditEntry {
    channel: incoming.channel.clone(),
    sender_id: incoming.sender_id.clone(),
    sender_name: incoming.sender_name.clone(),
    input_text: incoming.text.clone(),
    output_text: Some(response.text.clone()),
    provider_used: Some(response.metadata.provider_used.clone()),
    model: response.metadata.model.clone(),
    processing_ms: Some(response.metadata.processing_time_ms as i64),
    status: AuditStatus::Ok,
    denial_reason: None,
}).await;
```

**Characteristics:**
- All fields populated from the response metadata
- `processing_ms`: Cast from `u32` to `i64` to match the `Option<i64>` field type
- `denial_reason`: `None`

### Error Suppression in Gateway

In all three cases, the gateway discards the audit result:

```rust
let _ = self.audit.log(&entry).await;
```

Audit failures do not propagate or affect message processing. The `let _ =` pattern intentionally ignores the `Result`. This is a deliberate design decision: delivering the user's response is more important than logging it.

### AuditLogger Construction

The `AuditLogger` is constructed in `Gateway::new()`:

```rust
let audit = AuditLogger::new(memory.pool().clone());
```

It clones the `SqlitePool` from the `Store`, sharing the same connection pool and database file. The audit logger lives as a field on the `Gateway` struct for the lifetime of the process.

## Pipeline Position

```
Message In
    |
    v
[1. Auth Check] ----denied----> [Audit: Denied] --> Send deny msg --> Done
    |
    allowed
    |
    v
[2. Sanitize] --> [3. Command?] --> [4. Typing] --> [5. Context]
    |
    v
[6. Provider] ----error-----> [Audit: Error] --> Send error msg --> Done
    |
    success
    |
    v
[7. Memory Store] --> [8. Audit: Ok] --> [9. Send Response] --> Done
```

## Audit Event Matrix

| Scenario | `status` | `output_text` | `provider_used` | `model` | `processing_ms` | `denial_reason` |
|----------|----------|---------------|-----------------|---------|-----------------|-----------------|
| Auth denied | `"denied"` | `None` | `None` | `None` | `None` | `Some(reason)` |
| Provider error | `"error"` | `Some("ERROR: ...")` | `Some(name)` | `None` | `None` | `None` |
| Success | `"ok"` | `Some(response)` | `Some(name)` | `Some(model)` | `Some(ms)` | `None` |

## Not Audited

The following interactions are **not** recorded in the audit log:

1. **Bot commands** (`/uptime`, `/help`, `/status`, `/facts`, `/memory`, `/forget`) -- handled before the audit stage, return early without audit.
2. **Context build failures** -- if `memory.build_context()` fails, an error message is sent to the user but no audit entry is created.
3. **Channel send failures** -- if the final `channel.send()` fails, no separate audit entry is created (the success audit entry was already written).
4. **Background summarization** -- conversation summarization and fact extraction are not audited.

## Design Notes

1. **Append-only.** The audit module only inserts. There are no update, delete, or query methods. The audit log is an immutable append-only journal.

2. **No reads.** The audit module provides no methods for reading audit data. Querying is done directly via SQL against the `audit_log` table.

3. **Shared pool.** The `AuditLogger` shares the same `SqlitePool` as the `Store`. There is no separate database or connection for audit data.

4. **Timestamp from SQLite.** The `timestamp` column is set by SQLite's `datetime('now')` default, not by Rust code. This ensures the timestamp reflects the database write time, not the time the `AuditEntry` struct was constructed.

5. **UUIDv4 primary keys.** Each audit row gets a random UUID, consistent with the rest of the schema (conversations, messages, facts all use UUIDv4).

6. **No batching.** Each call to `log()` executes a single `INSERT`. There is no write buffer or batch mechanism.

7. **Fire-and-forget in gateway.** The gateway uses `let _ =` to discard audit write results, making audit a best-effort system that never blocks or degrades user-facing operations.

8. **Byte-boundary truncation.** The private `truncate()` function slices on byte boundaries. This could panic on multi-byte UTF-8 input if the cut point falls mid-character, though the risk is limited to debug trace output.

## Lines of Code

| Item | Lines |
|------|-------|
| `AuditEntry` struct | 12 |
| `AuditStatus` enum | 6 |
| `AuditStatus::as_str` | 7 |
| `AuditLogger` struct | 3 |
| `AuditLogger::new` | 3 |
| `AuditLogger::log` | 33 |
| `truncate` function | 6 |
| Module doc comment | 1 |
| **Total** | **93** |

## Summary Table

| Aspect | Detail |
|--------|--------|
| File | `backend/crates/omega-memory/src/audit.rs` |
| Migration | `backend/crates/omega-memory/migrations/002_audit_log.sql` |
| Public types | `AuditEntry`, `AuditStatus`, `AuditLogger` |
| Public methods | `AuditLogger::new`, `AuditLogger::log` |
| SQL operations | 1 (`INSERT INTO audit_log`) |
| Table | `audit_log` (12 columns, 2 indexes) |
| Error variant | `OmegaError::Memory` |
| Gateway integration points | 3 (auth denial, provider error, success) |
| Suppressed scenarios | Bot commands, context build failures |
| Logging | `debug!` on successful write |
| Lines of code | 93 |
