# Specification: omega-memory/migrations/004_fts5_recall.sql

## Path

`crates/omega-memory/migrations/004_fts5_recall.sql`

## Purpose

Adds FTS5 full-text search capability to the messages table, enabling cross-conversation recall. When building context for a new message, the store can now search ALL past user messages across every conversation to find relevant prior context. This eliminates the information loss that occurs when conversations are compressed into 1-2 sentence summaries.

This migration was introduced alongside Phase 3+ cross-conversation recall. Only user messages are indexed — assistant responses are derived content and do not need to be searchable.

## Prerequisites

- Migration `001_init.sql` must have been applied (creates `messages` table).
- Migration `003_memory_enhancement.sql` must have been applied (adds conversation lifecycle columns).
- FTS5 must be available in the SQLite build (confirmed enabled in Nix-provided SQLite).

---

## Schema Changes

### CREATE VIRTUAL TABLE: `messages_fts`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    content,
    content='messages',
    content_rowid='rowid'
);
```

| Property | Value |
|----------|-------|
| Type | FTS5 virtual table |
| Mode | Content-sync (external content) |
| Content table | `messages` |
| Rowid mapping | `messages.rowid` |
| Indexed columns | `content` (message text) |

**Content-sync design:** The `content='messages'` and `content_rowid='rowid'` options tell FTS5 to store only the search index, not a copy of the data. When a query matches, FTS5 returns the `rowid` which is then joined back to the `messages` table to retrieve the full row. This avoids doubling storage for message content.

**`IF NOT EXISTS`:** Makes the statement idempotent when FTS5 is available.

---

### Backfill: Existing User Messages

```sql
INSERT INTO messages_fts(rowid, content)
    SELECT rowid, content FROM messages WHERE role = 'user';
```

| Property | Value |
|----------|-------|
| Scope | All existing rows in `messages` where `role = 'user'` |
| Purpose | Populate the FTS index with historical data |

**Note:** Only user messages are indexed. Assistant messages are excluded because:
1. User messages are the intent signal — they contain the questions, commands, and topics the user cares about.
2. Assistant messages are derived content that would add noise to search results.

---

### Trigger: `messages_fts_insert`

```sql
CREATE TRIGGER messages_fts_insert AFTER INSERT ON messages WHEN NEW.role = 'user'
BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (NEW.rowid, NEW.content);
END;
```

| Property | Value |
|----------|-------|
| Event | `AFTER INSERT ON messages` |
| Condition | `NEW.role = 'user'` |
| Action | Insert the new row into the FTS index |

**Purpose:** Automatically indexes new user messages as they are stored by `store_exchange()`. No changes to `store_exchange()` are needed — the trigger handles sync transparently.

---

### Trigger: `messages_fts_delete`

```sql
CREATE TRIGGER messages_fts_delete AFTER DELETE ON messages WHEN OLD.role = 'user'
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', OLD.rowid, OLD.content);
END;
```

| Property | Value |
|----------|-------|
| Event | `AFTER DELETE ON messages` |
| Condition | `OLD.role = 'user'` |
| Action | Remove the deleted row from the FTS index |

**FTS5 delete syntax:** The special `INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', ...)` syntax is how FTS5 content-sync tables handle deletions. The first column name matching the table name signals a control operation.

---

### Trigger: `messages_fts_update`

```sql
CREATE TRIGGER messages_fts_update AFTER UPDATE OF content ON messages WHEN NEW.role = 'user'
BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', OLD.rowid, OLD.content);
    INSERT INTO messages_fts(rowid, content) VALUES (NEW.rowid, NEW.content);
END;
```

| Property | Value |
|----------|-------|
| Event | `AFTER UPDATE OF content ON messages` |
| Condition | `NEW.role = 'user'` |
| Action | Delete old index entry, insert new one |

**Purpose:** Handles content updates by removing the old index entry and inserting the new one. In practice, Omega does not update message content, but the trigger ensures index consistency if it ever does.

---

## Migration Tracking

This migration is registered with name `"004_fts5_recall"` in the `_migrations` table. The migration runner in `Store::run_migrations()` checks for this name and skips execution if already applied.

**Migration definitions (compile-time embedded):**
```rust
("004_fts5_recall", include_str!("../migrations/004_fts5_recall.sql"))
```

---

## Application-Level Usage

### `Store::search_messages()`

The primary consumer of the FTS index.

```rust
pub async fn search_messages(
    &self,
    query: &str,
    exclude_conversation_id: &str,
    sender_id: &str,
    limit: i64,
) -> Result<Vec<(String, String, String)>, OmegaError>
```

**SQL:**
```sql
SELECT m.role, m.content, m.timestamp
FROM messages_fts fts
JOIN messages m ON m.rowid = fts.rowid
JOIN conversations c ON c.id = m.conversation_id
WHERE messages_fts MATCH ?
AND m.conversation_id != ?
AND c.sender_id = ?
ORDER BY rank
LIMIT ?
```

**Query flow:**
1. FTS5 `MATCH` finds matching rows by content relevance (BM25 ranking).
2. `JOIN messages` retrieves the full message row via rowid.
3. `JOIN conversations` enables filtering by `sender_id` (security: users only recall their own messages).
4. `conversation_id != ?` excludes the current conversation (already in history).
5. `ORDER BY rank` returns best matches first (BM25 relevance score).
6. `LIMIT ?` caps results (default: 5).

**Called by:** `build_context()` with the incoming message text as the query.

---

## Relationship to Other Migrations

| Migration | Name | What It Creates |
|-----------|------|----------------|
| `001_init.sql` | `001_init` | `conversations`, `messages`, `facts` (original) |
| `002_audit_log.sql` | `002_audit_log` | `audit_log` |
| `003_memory_enhancement.sql` | `003_memory_enhancement` | ALTER `conversations` (+3 cols, +1 idx), DROP+CREATE `facts` |
| **`004_fts5_recall.sql`** | **`004_fts5_recall`** | **`messages_fts` virtual table, 3 sync triggers, backfill** |

---

## Idempotency

- `CREATE VIRTUAL TABLE IF NOT EXISTS` is idempotent.
- The `INSERT INTO messages_fts` backfill is **not** idempotent — running it twice would produce duplicate index entries. Idempotency is handled by the migration tracker.
- `CREATE TRIGGER` is **not** idempotent (would error on duplicate). Idempotency is handled by the migration tracker.

---

## Performance Considerations

- **Index size:** FTS5 content-sync mode stores only the inverted index, not message text. Typically 10-30% of the indexed text size.
- **Insert overhead:** The `messages_fts_insert` trigger adds a small overhead to every `store_exchange()` call (microseconds per insert).
- **Search latency:** FTS5 MATCH queries are sub-millisecond for typical database sizes (thousands of messages).
- **Backfill:** One-time cost at migration. For a database with 10,000 user messages, this takes under a second.
