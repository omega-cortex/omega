# Project-Scoped Learning (Migration 011)

## Path

`crates/omega-memory/migrations/011_project_learning.sql`

## What This Migration Does

Migration 011 adds a `project` column to three tables (`outcomes`, `lessons`, `scheduled_tasks`) to enable project-scoped learning isolation. Before this migration, all outcomes, lessons, and tasks existed in a single global namespace. Now each can be tagged with a project name, allowing OMEGA to learn and behave differently per project context.

## Migration Sequence

| Order | File | What It Creates |
|-------|------|----------------|
| 9 | `009_task_retry.sql` | Retry support: `retry_count` + `last_error` columns |
| 10 | `010_outcomes.sql` | Reward-based learning: `outcomes` + `lessons` tables |
| **11** | **`011_project_learning.sql`** | **Project column on `outcomes`, `lessons`, `scheduled_tasks`** |

## The Change

### outcomes table

```sql
ALTER TABLE outcomes ADD COLUMN project TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_outcomes_project ON outcomes (sender_id, project, timestamp);
```

Simple column addition. Existing rows get `project = ''` (general scope).

### lessons table

SQLite cannot alter unique constraints, so the table is recreated:

```sql
CREATE TABLE lessons_new (
    ...
    project     TEXT NOT NULL DEFAULT '',
    UNIQUE(sender_id, domain, project)   -- was UNIQUE(sender_id, domain)
);

INSERT INTO lessons_new ... SELECT ... FROM lessons;
DROP TABLE lessons;
ALTER TABLE lessons_new RENAME TO lessons;
```

The unique constraint changes from `(sender_id, domain)` to `(sender_id, domain, project)`. This allows the same domain to have different lessons per project (e.g., a "trading" lesson for project "omega-trader" and a different "trading" lesson for general scope).

### scheduled_tasks table

```sql
ALTER TABLE scheduled_tasks ADD COLUMN project TEXT NOT NULL DEFAULT '';
```

Simple column addition. Existing tasks get `project = ''` (general scope).

## Why Empty String Instead of NULL

The `project` column uses `''` (empty string) for general scope instead of `NULL`. This avoids `NULL`-in-`UNIQUE` edge cases in SQLite -- `NULL != NULL` in unique constraints, which would allow duplicate `(sender_id, domain, NULL)` rows in lessons. Empty string participates in uniqueness checks correctly.

## Backward Compatibility

- All existing data gets `project = ''`, preserving current behavior
- Queries that don't filter by project see all data (general scope)
- Project-scoped queries use layered loading: project-specific first, general fills the rest
- No data is lost or modified -- only new columns and indexes are added (lessons table is recreated with identical data)
