# Specification: Migration 013 — Multi-Lesson Support

## File Path
`crates/omega-memory/migrations/013_multi_lessons.sql`

## Purpose
Remove the `UNIQUE(sender_id, domain, project)` constraint from the `lessons` table to allow multiple distinct rules per domain per project. Previously, only one rule could exist per (sender, domain, project), forcing the AI to overwrite previous lessons and use HEARTBEAT.md as a scratchpad instead.

## Migration Strategy
SQLite cannot alter constraints in-place. Following the same pattern as `011_project_learning.sql`:
1. Create `lessons_v2` with identical schema but no UNIQUE constraint
2. Copy all existing data via `INSERT INTO lessons_v2 SELECT * FROM lessons`
3. Drop original `lessons` table
4. Rename `lessons_v2` to `lessons`
5. Recreate indexes

## Schema After Migration

```sql
CREATE TABLE lessons (
    id          TEXT PRIMARY KEY,
    sender_id   TEXT NOT NULL,
    domain      TEXT NOT NULL,
    rule        TEXT NOT NULL,
    project     TEXT NOT NULL DEFAULT '',
    occurrences INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

No UNIQUE constraint — multiple rows with the same `(sender_id, domain, project)` are allowed.

## Indexes

| Index | Columns | Purpose |
|-------|---------|---------|
| `idx_lessons_sender` | `(sender_id)` | Per-user lesson queries |
| `idx_lessons_project` | `(sender_id, project)` | Project-scoped lesson queries |
| `idx_lessons_domain` | `(sender_id, domain, project)` | Domain-scoped cap enforcement and lookups |

## Application-Level Guardrails

Since the UNIQUE constraint is removed, deduplication and capping are enforced in `store_lesson()`:

1. **Content dedup**: Before inserting, check if the exact same rule text exists for (sender, domain, project). If yes, bump `occurrences` and `updated_at` instead of inserting a duplicate.
2. **Cap enforcement**: After inserting a new row, delete the oldest rows beyond 10 per (sender, domain, project).

## Data Migration
All existing data is preserved. Existing lessons retain their `occurrences`, `created_at`, and `updated_at` values.

## Backward Compatibility
- `get_lessons()` and `get_all_lessons()` queries are unchanged — they already return `Vec` of results
- `build_enrichment()` and `build_system_prompt()` iterate over the lessons vec — no changes needed
- `process_markers()` calls `store_lesson()` — behavior is enhanced transparently

## Registration
Entry `("013_multi_lessons", include_str!("../../migrations/013_multi_lessons.sql"))` in `crates/omega-memory/src/store/mod.rs` migrations array.
