# Specification: omega-memory/migrations/011_project_learning.sql

## Path

`backend/crates/omega-memory/migrations/011_project_learning.sql`

## Purpose

Adds a `project` column to the `outcomes`, `lessons`, and `scheduled_tasks` tables, enabling per-project isolation of learning data and tasks. Before this migration, all outcomes, lessons, and tasks were global — there was no way to scope learning data to a specific project context.

## Prerequisites

- Migration `010_outcomes.sql` must have been applied (creates `outcomes` and `lessons` tables).
- Migration `005_scheduled_tasks.sql` must have been applied (creates `scheduled_tasks` table).

---

## Schema Changes

### ALTER TABLE: `outcomes`

```sql
ALTER TABLE outcomes ADD COLUMN project TEXT NOT NULL DEFAULT '';
```

Adds a `project` column with empty string default. Existing rows get `project = ''` (general scope).

### Recreated Table: `lessons`

SQLite cannot ALTER unique constraints, so the table is recreated with the new column and constraint.

```sql
CREATE TABLE lessons_new (
    id          TEXT PRIMARY KEY,
    sender_id   TEXT NOT NULL,
    domain      TEXT NOT NULL,
    rule        TEXT NOT NULL,
    project     TEXT NOT NULL DEFAULT '',
    occurrences INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(sender_id, domain, project)
);

INSERT INTO lessons_new (id, sender_id, domain, rule, project, occurrences, created_at, updated_at)
    SELECT id, sender_id, domain, rule, '', occurrences, created_at, updated_at FROM lessons;

DROP TABLE lessons;
ALTER TABLE lessons_new RENAME TO lessons;
```

**Key change:** UNIQUE constraint changed from `(sender_id, domain)` to `(sender_id, domain, project)`. This allows the same domain to have different lessons per project.

### ALTER TABLE: `scheduled_tasks`

```sql
ALTER TABLE scheduled_tasks ADD COLUMN project TEXT NOT NULL DEFAULT '';
```

Adds a `project` column with empty string default. Existing rows get `project = ''` (general scope).

### New Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_outcomes_project ON outcomes (sender_id, project, timestamp);
CREATE INDEX IF NOT EXISTS idx_lessons_project ON lessons (sender_id, project);
CREATE INDEX IF NOT EXISTS idx_lessons_sender ON lessons (sender_id);
```

| Index | Table | Columns | Purpose |
|-------|-------|---------|---------|
| `idx_outcomes_project` | `outcomes` | `(sender_id, project, timestamp)` | Project-scoped outcome queries. |
| `idx_lessons_project` | `lessons` | `(sender_id, project)` | Project-scoped lesson queries. |
| `idx_lessons_sender` | `lessons` | `(sender_id)` | Per-sender lesson queries (recreated after table rebuild). |

---

## Project Scope Convention

- **Empty string (`''`)** = general OMEGA scope (no project). This avoids NULL-in-UNIQUE edge cases.
- **Non-empty string** = project-scoped (e.g., `"my-app"`, `"trading-bot"`).
- All existing data migrates with `project = ''` (backward compatible).

---

## Backward Compatibility

- `ALTER TABLE ... ADD COLUMN ... DEFAULT ''` preserves all existing data with general scope.
- The lessons table is fully rebuilt via `CREATE ... INSERT ... DROP ... RENAME`, preserving all existing data with `project = ''`.
- Store functions that previously had no `project` parameter now accept one, but callers pass `""` for general scope — identical behavior to pre-migration.

---

## Migration Tracking

This migration is registered with name `"011_project_learning"` in the `_migrations` table.

**Migration definitions (compile-time embedded):**
```rust
("011_project_learning", include_str!("../migrations/011_project_learning.sql"))
```

---

## Application-Level Usage

### Store Methods (Updated Signatures)

| Method | New Parameter | Purpose |
|--------|--------------|---------|
| `store_outcome(sender_id, domain, score, lesson, source, project)` | `project: &str` | Tag outcome with project scope. |
| `get_recent_outcomes(sender_id, limit, project)` | `project: Option<&str>` | Filter outcomes by project (None = all). |
| `get_all_recent_outcomes(hours, limit, project)` | `project: Option<&str>` | Filter outcomes by project (None = all). |
| `store_lesson(sender_id, domain, rule, project)` | `project: &str` | Upsert by `(sender_id, domain, project)`. |
| `get_lessons(sender_id, project)` | `project: Option<&str>` | Project-specific first, then general. |
| `get_all_lessons(project)` | `project: Option<&str>` | Project-specific + general (None = all). |
| `create_task(channel, sender_id, ..., project)` | `project: &str` | Tag task with project scope. |
| `get_due_tasks()` | (return type extended) | Returns project in tuple. |
| `get_tasks_for_sender(sender_id)` | (return type extended) | Returns project in tuple. |

### Context Injection

- **Regular messages:** `build_context()` accepts `active_project: Option<&str>`. When a project is active, outcomes are filtered to that project, and lessons are layered (project-specific first, then general fill).
- **Heartbeat:** Per-project heartbeat loop reads `~/.omega/projects/<name>/HEARTBEAT.md` and scopes enrichment to the project.
- **Scheduler:** Action tasks carry their `project` field through to all nested operations (create_task, store_outcome, store_lesson).

---

## Relationship to Other Migrations

| Migration | Name | What It Creates |
|-----------|------|----------------|
| `010_outcomes.sql` | `010_outcomes` | `outcomes` + `lessons` tables, 3 indexes |
| **`011_project_learning.sql`** | **`011_project_learning`** | **project columns on outcomes + lessons + scheduled_tasks, recreated lessons UNIQUE, 3 new indexes** |
