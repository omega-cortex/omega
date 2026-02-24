# Specification: omega-memory/migrations/009_task_retry.sql

## Path

`backend/crates/omega-memory/migrations/009_task_retry.sql`

## Purpose

Adds `retry_count` and `last_error` columns to the `scheduled_tasks` table to support retry logic for failed action tasks. Before this migration, action tasks had no failure tracking — the scheduler would either silently drop failures or leave tasks pending indefinitely after provider errors.

With these columns, the `fail_task()` method can increment the retry count, store the error reason, and either reschedule the task (with a 2-minute delay) or permanently mark it as `failed` once the maximum retry count is reached.

## Prerequisites

- Migration `005_scheduled_tasks.sql` must have been applied (creates the `scheduled_tasks` table).

---

## Schema Changes

### ALTER TABLE: `scheduled_tasks`

```sql
ALTER TABLE scheduled_tasks ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE scheduled_tasks ADD COLUMN last_error TEXT;
```

### Column Descriptions

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `retry_count` | `INTEGER` | `NOT NULL`, default `0` | Number of times this task has been retried after failure. Incremented by `fail_task()`. |
| `last_error` | `TEXT` | nullable | Most recent error message from the last failed attempt. Set by `fail_task()`. |

---

## Backward Compatibility

Both columns have safe defaults (`0` and `NULL`), so existing rows are unaffected. The `DEFAULT 0` on `retry_count` ensures all existing tasks start with zero retries. The nullable `last_error` column starts as `NULL` for tasks that have never failed.

---

## Migration Tracking

This migration is registered with name `"009_task_retry"` in the `_migrations` table.

**Migration definitions (compile-time embedded):**
```rust
("009_task_retry", include_str!("../migrations/009_task_retry.sql"))
```

---

## Application-Level Usage

### `Store::fail_task(id, error, max_retries)`

New method that handles action task failures:
- Increments `retry_count`
- If `retry_count < max_retries`: keeps `status = 'pending'`, sets `due_at = datetime('now', '+2 minutes')`, stores error in `last_error`. Returns `true` (will retry).
- If `retry_count >= max_retries`: sets `status = 'failed'`, stores error in `last_error`. Returns `false` (permanently failed).

### Scheduler Loop

The scheduler now:
1. Injects an `ACTION_OUTCOME:` verification instruction into action task system prompts
2. Parses the `ACTION_OUTCOME: success` or `ACTION_OUTCOME: failed | <reason>` marker from provider responses
3. On success: calls `complete_task()` as before
4. On failure: calls `fail_task()`, notifies user of retry or permanent failure
5. On provider error: calls `fail_task()` instead of leaving task in limbo
6. Logs every action execution to the `audit_log` table with `[ACTION]` prefix

---

## Relationship to Other Migrations

| Migration | Name | What It Creates |
|-----------|------|----------------|
| `005_scheduled_tasks.sql` | `005_scheduled_tasks` | `scheduled_tasks` table, 2 indexes |
| `007_task_type.sql` | `007_task_type` | ALTER `scheduled_tasks` (+1 col: `task_type`) |
| `008_user_aliases.sql` | `008_user_aliases` | `user_aliases` table |
| **`009_task_retry.sql`** | **`009_task_retry`** | **ALTER `scheduled_tasks` (+2 cols: `retry_count`, `last_error`)** |
