# Task Retry Columns (Migration 009)

## Path

`backend/crates/omega-memory/migrations/009_task_retry.sql`

## What This Migration Does

Migration 009 adds `retry_count` and `last_error` columns to the `scheduled_tasks` table. Before this migration, failed action tasks were either silently dropped or left pending indefinitely after a provider error with no audit trail and no retry mechanism.

Now, when an action task fails:
- The `retry_count` is incremented and the error is stored in `last_error`
- If retries remain, the task is rescheduled 2 minutes into the future
- If max retries (3) are exhausted, the task is permanently marked as `failed`
- Every execution (success or failure) is logged to the `audit_log` table

## Migration Sequence

| Order | File | What It Creates |
|-------|------|----------------|
| 5 | `005_scheduled_tasks.sql` | Task queue: `scheduled_tasks` table + indexes |
| 7 | `007_task_type.sql` | Task type: `task_type` column on `scheduled_tasks` |
| 8 | `008_user_aliases.sql` | Cross-channel user aliases: `user_aliases` table |
| **9** | **`009_task_retry.sql`** | **Retry support: `retry_count` + `last_error` columns** |

## The Change

```sql
ALTER TABLE scheduled_tasks ADD COLUMN retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE scheduled_tasks ADD COLUMN last_error TEXT;
```

### Column Explanations

| Column | What It Stores |
|--------|---------------|
| `retry_count` | Number of times this task has been retried after failure. Starts at 0, incremented by `fail_task()`. |
| `last_error` | Most recent error message from the last failed attempt. `NULL` for tasks that have never failed. |

## How Retry Works

1. Action task comes due and the scheduler invokes the provider.
2. The provider response includes an `ACTION_OUTCOME:` marker (`success` or `failed | reason`).
3. On **success**: task is completed as before (status = `delivered` or due_at advanced for recurring).
4. On **failure** (or provider error): `fail_task()` is called.
   - If `retry_count < 3`: task stays `pending`, `due_at` is pushed 2 minutes forward, user is notified of retry.
   - If `retry_count >= 3`: task status becomes `failed`, user is notified of permanent failure.
5. Every execution is audit-logged with `[ACTION]` prefix in `input_text`.

## Backward Compatibility

Both columns have safe defaults (`0` and `NULL`). Existing tasks are unaffected. Tasks that never fail will always have `retry_count = 0` and `last_error = NULL`.

## Schema Overview After Migration 009

The `scheduled_tasks` table has these columns:

| Column | Type | Created By |
|--------|------|------------|
| `id` | TEXT PRIMARY KEY | 005 |
| `channel` | TEXT NOT NULL | 005 |
| `sender_id` | TEXT NOT NULL | 005 |
| `reply_target` | TEXT NOT NULL | 005 |
| `description` | TEXT NOT NULL | 005 |
| `due_at` | TEXT NOT NULL | 005 |
| `repeat` | TEXT | 005 |
| `status` | TEXT NOT NULL DEFAULT 'pending' | 005 |
| `created_at` | TEXT NOT NULL DEFAULT datetime('now') | 005 |
| `delivered_at` | TEXT | 005 |
| `task_type` | TEXT NOT NULL DEFAULT 'reminder' | 007 |
| `retry_count` | INTEGER NOT NULL DEFAULT 0 | **009** |
| `last_error` | TEXT | **009** |
