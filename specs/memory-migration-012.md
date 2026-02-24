# Specification: omega-memory/migrations/012_project_sessions.sql

## Path

`backend/crates/omega-memory/migrations/012_project_sessions.sql`

## Purpose

Creates the `project_sessions` table for SQLite-backed CLI session persistence, and adds a `project` column to the `conversations` table. Before this migration, CLI sessions were stored in an in-memory `HashMap<String, String>` keyed by `channel:sender_id` — sessions were lost on restart and had no project awareness. Conversations were also not scoped to projects.

This migration enables:
- **Session survival across restarts** — sessions persist in SQLite instead of memory.
- **Project-scoped sessions** — each (channel, sender_id, project) tuple gets its own CLI session, so switching projects no longer kills the previous session.
- **Project-scoped conversations** — conversations carry a `project` field for isolation.

## Prerequisites

- Migration `001_init.sql` must have been applied (creates the `conversations` table).
- Migration `011_project_learning.sql` should have been applied (establishes the project-scoping convention).

---

## Schema Changes

### New Table: `project_sessions`

```sql
CREATE TABLE IF NOT EXISTS project_sessions (
    id              TEXT PRIMARY KEY,
    channel         TEXT NOT NULL,
    sender_id       TEXT NOT NULL,
    project         TEXT NOT NULL DEFAULT '',
    session_id      TEXT NOT NULL,
    parent_project  TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(channel, sender_id, project)
);

CREATE INDEX IF NOT EXISTS idx_project_sessions_lookup
    ON project_sessions(channel, sender_id, project);
```

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | `TEXT` | `PRIMARY KEY` | UUID v4 string. |
| `channel` | `TEXT` | `NOT NULL` | Channel name (e.g., `"telegram"`, `"whatsapp"`). |
| `sender_id` | `TEXT` | `NOT NULL` | The user this session belongs to. |
| `project` | `TEXT` | `NOT NULL`, default `''` | Project scope. Empty string = general (no project). |
| `session_id` | `TEXT` | `NOT NULL` | The Claude Code CLI session ID for `--resume`. |
| `parent_project` | `TEXT` | nullable | Reserved for future use (project hierarchy). |
| `created_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the session was first created. |
| `updated_at` | `TEXT` | `NOT NULL`, default `datetime('now')` | When the session was last updated. |

**Unique constraint:** `(channel, sender_id, project)` — one session per channel+sender+project combination.

**Index:**
- `idx_project_sessions_lookup` on `(channel, sender_id, project)` — fast session lookup by the composite key.

### ALTER TABLE: `conversations`

```sql
ALTER TABLE conversations ADD COLUMN project TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_conversations_project
    ON conversations(channel, sender_id, project, status, last_activity);
```

Adds a `project` column with empty string default. Existing rows get `project = ''` (general scope).

**Index:**
- `idx_conversations_project` on `(channel, sender_id, project, status, last_activity)` — efficient lookup for project-scoped conversation queries (get_or_create, find_idle, find_active).

---

## Project Scope Convention

- **Empty string (`''`)** = general OMEGA scope (no project). Consistent with migration 011.
- **Non-empty string** = project-scoped (e.g., `"omega-trader"`).
- All existing data migrates with `project = ''` (backward compatible).

---

## Backward Compatibility

- `project_sessions` is a new table (`CREATE TABLE IF NOT EXISTS`), no existing data affected.
- `ALTER TABLE conversations ADD COLUMN ... DEFAULT ''` preserves all existing conversations with general scope.
- Store functions that previously had no `project` parameter now accept one, but callers pass `""` for general scope — identical behavior to pre-migration.

---

## Migration Tracking

This migration is registered with name `"012_project_sessions"` in the `_migrations` table.

**Migration definitions (compile-time embedded):**
```rust
("012_project_sessions", include_str!("../migrations/012_project_sessions.sql"))
```

---

## Application-Level Usage

### Store Methods (New)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `store_session` | `(channel, sender_id, project, session_id)` | Upsert CLI session. ON CONFLICT updates session_id + updated_at. |
| `get_session` | `(channel, sender_id, project) -> Option<String>` | Look up session_id for a specific project context. |
| `clear_session` | `(channel, sender_id, project)` | Delete session for a specific project. |
| `clear_all_sessions_for_sender` | `(sender_id)` | Delete all sessions for a sender (used by `/forget`). |

### Store Methods (Updated Signatures)

| Method | Change | Purpose |
|--------|--------|---------|
| `get_or_create_conversation` | Added `project: &str` param | Scope conversation lookup/creation to project. |
| `close_current_conversation` | Added `project: &str` param | Close the active conversation for a specific project. |
| `find_idle_conversations` | Returns `project` in tuple | Summarizer uses project to scope closure. |
| `find_all_active_conversations` | Returns `project` in tuple | Background loops can enumerate active project conversations. |
| `store_exchange` | Added `project: &str` param | Route exchange to the correct project-scoped conversation. |

### Gateway Changes

The `cli_sessions: HashMap<String, String>` field was removed from the Gateway struct. All session storage and retrieval now goes through the `Store` methods above. This means:

- Sessions survive process restarts.
- Switching projects preserves the previous project's session — returning to it resumes the CLI session via `--resume`.
- `/forget` and `FORGET_CONVERSATION` clear the session for the current project only; `clear_all_sessions_for_sender` handles full reset.

---

## Relationship to Other Migrations

| Migration | Name | What It Creates |
|-----------|------|----------------|
| `001_init.sql` | `001_init` | `conversations`, `messages`, `facts` tables |
| `011_project_learning.sql` | `011_project_learning` | Project columns on outcomes + lessons + scheduled_tasks |
| **`012_project_sessions.sql`** | **`012_project_sessions`** | **`project_sessions` table, project column on conversations, 2 new indexes** |
