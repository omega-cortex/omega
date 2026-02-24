# Specification: Memory Migration 001 — Initial Schema

## File Path

`backend/crates/omega-memory/migrations/001_init.sql`

## Purpose

Establishes the foundational database schema for Omega's memory system. This migration creates the three core tables -- `conversations`, `messages`, and `facts` -- that support conversation tracking, message persistence, and user fact extraction. All tables use `IF NOT EXISTS` guards for idempotent execution.

## Migration Sequence

This is the **first** migration in the sequence:

| Migration | File | Purpose |
|-----------|------|---------|
| **001** | **`001_init.sql`** | **Core schema: conversations, messages, facts** |
| 002 | `002_audit_log.sql` | Audit log table for interaction tracking |
| 003 | `003_memory_enhancement.sql` | Conversation boundaries, summaries, facts scoping |

Migration 003 modifies two of the three tables created here (`conversations` and `facts`), so this migration defines the baseline that subsequent migrations evolve.

---

## Database Engine

SQLite (via `sqlx` with the `sqlite` feature). All datetime values are stored as ISO 8601 text strings using SQLite's `datetime('now')` function. Primary keys are application-generated text UUIDs, not auto-incrementing integers.

---

## Table: `conversations`

Tracks conversation sessions between a user and Omega on a specific channel. Each conversation groups related message exchanges into a logical thread.

### CREATE Statement

```sql
CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY,
    channel     TEXT NOT NULL,
    sender_id   TEXT NOT NULL,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Column Definitions

| Column | Type | Nullable | Default | Constraints | Description |
|--------|------|----------|---------|-------------|-------------|
| `id` | `TEXT` | No | -- | `PRIMARY KEY` | Application-generated UUID identifying the conversation. |
| `channel` | `TEXT` | No | -- | `NOT NULL` | Messaging platform name (e.g., `"telegram"`, `"whatsapp"`). |
| `sender_id` | `TEXT` | No | -- | `NOT NULL` | Platform-specific user identifier (e.g., Telegram numeric user ID stored as text). |
| `started_at` | `TEXT` | No | `datetime('now')` | `NOT NULL`, `DEFAULT` | ISO 8601 timestamp of when the conversation was created. |
| `updated_at` | `TEXT` | No | `datetime('now')` | `NOT NULL`, `DEFAULT` | ISO 8601 timestamp of the last activity in this conversation. |

### Indexes

| Index Name | Columns | Purpose |
|------------|---------|---------|
| `idx_conversations_channel_sender` | `(channel, sender_id)` | Accelerates lookups for active conversations by channel and user. Used by the store's `build_context()` and `find_idle_conversations()` methods. |

### Foreign Keys

None. This is a root-level table.

### Notes

- Migration 003 later adds three columns to this table: `summary` (TEXT, nullable), `last_activity` (TEXT, NOT NULL, defaults to `datetime('now')`), and `status` (TEXT, NOT NULL, defaults to `'active'`).
- The composite index on `(channel, sender_id)` is the primary lookup path -- the gateway finds active conversations for a user on a specific channel.
- `started_at` and `updated_at` are both set to `datetime('now')` on creation. The application is responsible for updating `updated_at` on subsequent messages.

---

## Table: `messages`

Stores individual messages within conversations. Each message is either from the user (`'user'`) or the AI assistant (`'assistant'`).

### CREATE Statement

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

### Column Definitions

| Column | Type | Nullable | Default | Constraints | Description |
|--------|------|----------|---------|-------------|-------------|
| `id` | `TEXT` | No | -- | `PRIMARY KEY` | Application-generated UUID identifying the message. |
| `conversation_id` | `TEXT` | No | -- | `NOT NULL`, `REFERENCES conversations(id)` | Foreign key linking this message to its parent conversation. |
| `role` | `TEXT` | No | -- | `NOT NULL`, `CHECK (role IN ('user', 'assistant'))` | Identifies the message sender. Constrained to exactly two values. |
| `content` | `TEXT` | No | -- | `NOT NULL` | The message text. For user messages, this is the sanitized input. For assistant messages, this is the provider's response. |
| `timestamp` | `TEXT` | No | `datetime('now')` | `NOT NULL`, `DEFAULT` | ISO 8601 timestamp of when the message was stored. |
| `metadata_json` | `TEXT` | Yes | `NULL` | -- | Optional JSON blob for provider metadata (model used, processing time, etc.). Nullable because not all messages have associated metadata. |

### Indexes

| Index Name | Columns | Purpose |
|------------|---------|---------|
| `idx_messages_conversation` | `(conversation_id, timestamp)` | Accelerates chronological retrieval of messages within a conversation. Used by `get_conversation_messages()` and `build_context()`. |

### Foreign Keys

| Column | References | On Delete | On Update |
|--------|------------|-----------|-----------|
| `conversation_id` | `conversations(id)` | Not specified (SQLite default: no action) | Not specified |

### CHECK Constraints

| Column | Constraint | Allowed Values |
|--------|-----------|----------------|
| `role` | `CHECK (role IN ('user', 'assistant'))` | `'user'`, `'assistant'` |

### Notes

- The `role` CHECK constraint enforces a strict two-party conversation model. There is no `'system'` role -- system prompts are injected at the context level, not stored as messages.
- The composite index `(conversation_id, timestamp)` is ordered for chronological queries within a conversation. This supports both full conversation retrieval and "last N messages" queries.
- `metadata_json` is a schemaless JSON column. The application currently stores provider metadata (provider name, model, processing time) but the schema imposes no structure.
- The foreign key to `conversations(id)` does not specify cascade behavior. SQLite's default is `NO ACTION`, meaning a conversation cannot be deleted while it has messages (unless foreign keys are not enforced at the connection level).

---

## Table: `facts`

Stores key-value facts about users, extracted from conversations by the AI provider during summarization. Facts enable personalization across conversation boundaries.

### CREATE Statement (as created by 001)

```sql
CREATE TABLE IF NOT EXISTS facts (
    id                TEXT PRIMARY KEY,
    key               TEXT NOT NULL UNIQUE,
    value             TEXT NOT NULL,
    source_message_id TEXT REFERENCES messages(id),
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at        TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Column Definitions

| Column | Type | Nullable | Default | Constraints | Description |
|--------|------|----------|---------|-------------|-------------|
| `id` | `TEXT` | No | -- | `PRIMARY KEY` | Application-generated UUID identifying the fact. |
| `key` | `TEXT` | No | -- | `NOT NULL`, `UNIQUE` | The fact key (e.g., `"name"`, `"timezone"`, `"preference"`). Globally unique in this version. |
| `value` | `TEXT` | No | -- | `NOT NULL` | The fact value (e.g., `"Alice"`, `"America/Los_Angeles"`, `"brief responses"`). |
| `source_message_id` | `TEXT` | Yes | `NULL` | `REFERENCES messages(id)` | Optional reference to the message from which this fact was extracted. Nullable for manually inserted facts. |
| `created_at` | `TEXT` | No | `datetime('now')` | `NOT NULL`, `DEFAULT` | ISO 8601 timestamp of when the fact was first stored. |
| `updated_at` | `TEXT` | No | `datetime('now')` | `NOT NULL`, `DEFAULT` | ISO 8601 timestamp of when the fact was last modified. |

### Indexes

None defined explicitly. The `UNIQUE` constraint on `key` implicitly creates an index.

### Foreign Keys

| Column | References | On Delete | On Update |
|--------|------------|-----------|-----------|
| `source_message_id` | `messages(id)` | Not specified (SQLite default: no action) | Not specified |

### UNIQUE Constraints

| Column(s) | Purpose |
|-----------|---------|
| `key` | Ensures only one fact per key globally. A new fact with the same key replaces the existing one. |

### Notes

- **This table is replaced by migration 003.** The 003 migration drops this table and recreates it with `sender_id` scoping and a composite `UNIQUE(sender_id, key)` constraint instead of `UNIQUE(key)`. The original design used a global namespace for facts; migration 003 scopes facts per user.
- The `source_message_id` foreign key traces the provenance of extracted facts back to the specific message. This is useful for auditing which conversation produced which facts.
- The `UNIQUE(key)` constraint in this version means facts are global to the entire system, not per-user. This was a design limitation corrected in migration 003.

---

## Schema Relationships

```
conversations (1) ───< messages (N)
                         │
                         │ source_message_id (optional)
                         ▼
                       facts (N)
```

- One conversation has many messages (via `messages.conversation_id`).
- A fact may optionally reference its source message (via `facts.source_message_id`).
- There is no direct relationship between `conversations` and `facts` in this migration. Migration 003 adds `sender_id` to `facts`, creating an implicit link through the user identity.

---

## Design Decisions

### TEXT Primary Keys

All tables use application-generated TEXT UUIDs rather than SQLite `INTEGER PRIMARY KEY AUTOINCREMENT`. This supports:
- Deterministic ID generation in application code.
- Conflict-free ID generation across distributed systems (future-proofing).
- Consistent ID format across all tables.

### TEXT Timestamps with `datetime('now')`

Timestamps are stored as ISO 8601 text strings rather than Unix epoch integers. This provides:
- Human-readable values when querying the database directly.
- Consistent format with SQLite's built-in datetime functions.
- Timezone-agnostic UTC storage.

### `IF NOT EXISTS` Guards

All `CREATE TABLE` and `CREATE INDEX` statements use `IF NOT EXISTS`. This makes the migration idempotent -- running it multiple times does not fail or corrupt data. This is a safety measure for the migration runner.

### Nullable `metadata_json`

The `metadata_json` column on `messages` is the only nullable column in the schema (aside from foreign key references). This is intentional: user messages may not have provider metadata, and forcing a default would introduce misleading data.

### CHECK Constraint on `role`

The `role` column uses a CHECK constraint rather than a separate lookup table. This is appropriate because:
- The set of valid roles is small and fixed (`'user'`, `'assistant'`).
- A lookup table would add unnecessary complexity for two values.
- The CHECK constraint is enforced at the database level, not just in application code.

---

## SQLite-Specific Behavior

### Foreign Key Enforcement

SQLite does not enforce foreign keys by default. The `omega-memory` crate must execute `PRAGMA foreign_keys = ON;` on each connection for the `REFERENCES` constraints to be enforced. Without this pragma, referencing a non-existent `conversation_id` or `source_message_id` would succeed silently.

### Default Expressions

The `DEFAULT (datetime('now'))` syntax uses SQLite's expression-based defaults (parenthesized expression). This computes the current UTC datetime at insertion time. It is not a static default value.

### Index Behavior

- `CREATE INDEX IF NOT EXISTS` prevents errors on re-execution.
- SQLite automatically creates an index for `PRIMARY KEY` columns.
- The `UNIQUE` constraint on `facts.key` implicitly creates a unique index.
