# Scheduled Tasks Table (Migration 005)

## Path

`backend/crates/omega-memory/migrations/005_scheduled_tasks.sql`

## What This Migration Does

Migration 005 gives Omega a task queue. Before this migration, Omega was purely reactive -- it only responded when you sent a message. Now, Omega can schedule reminders and recurring tasks that fire at specific times, delivered through your messaging channel without you having to ask again.

This works using a new SQLite table (`scheduled_tasks`) that the background scheduler loop polls for due tasks.

## Migration Sequence

| Order | File | What It Creates |
|-------|------|----------------|
| 1 | `001_init.sql` | Core tables: `conversations`, `messages`, `facts` |
| 2 | `002_audit_log.sql` | Audit trail: `audit_log` |
| 3 | `003_memory_enhancement.sql` | Conversation lifecycle + per-user facts |
| 4 | `004_fts5_recall.sql` | FTS5 search index + auto-sync triggers |
| **5** | **`005_scheduled_tasks.sql`** | **Task queue: `scheduled_tasks` table + indexes** |

Migrations run automatically when the memory store initializes. Each migration runs exactly once.

## The scheduled_tasks Table

```sql
CREATE TABLE IF NOT EXISTS scheduled_tasks (
    id           TEXT PRIMARY KEY,
    channel      TEXT NOT NULL,
    sender_id    TEXT NOT NULL,
    reply_target TEXT NOT NULL,
    description  TEXT NOT NULL,
    due_at       TEXT NOT NULL,
    repeat       TEXT,
    status       TEXT NOT NULL DEFAULT 'pending',
    created_at   TEXT NOT NULL DEFAULT (datetime('now')),
    delivered_at TEXT
);
```

### Column Explanations

| Column | What It Stores |
|--------|---------------|
| `id` | UUID v4 generated at task creation time. The first 8 characters serve as the short ID shown to users. |
| `channel` | The messaging channel where the task was created and should be delivered (e.g., `"telegram"`). |
| `sender_id` | The user who created the task. Used for ownership checks in `/tasks` and `/cancel`. |
| `reply_target` | Platform-specific delivery target (e.g., Telegram chat ID). Ensures the reminder is delivered to the right place. |
| `description` | The human-readable reminder text (e.g., "Call John"). This is what appears in the delivered message. |
| `due_at` | When the task should fire, stored as ISO 8601 text (e.g., `"2026-02-17T15:00:00"`). The scheduler compares this against `datetime('now')`. |
| `repeat` | How the task recurs. `NULL` or absent for one-shot tasks. Valid values: `"daily"`, `"weekly"`, `"monthly"`, `"weekdays"`. |
| `status` | Current state of the task. See lifecycle below. |
| `created_at` | When the task was created. Auto-populated by SQLite's `datetime('now')`. |
| `delivered_at` | When the task was last delivered. Set when a one-shot task completes. For recurring tasks, this tracks the most recent delivery. |

### Indexes

```sql
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_due ON scheduled_tasks(status, due_at);
CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_sender ON scheduled_tasks(sender_id, status);
```

- **`idx_scheduled_tasks_due`** -- Used by `get_due_tasks()` every poll cycle. The composite index on `(status, due_at)` makes the query efficient: it first narrows to `status = 'pending'`, then scans by `due_at`.
- **`idx_scheduled_tasks_sender`** -- Used by `get_tasks_for_sender()` (the `/tasks` command) and `cancel_task()` (the `/cancel` command). Narrows by sender first, then filters by status.

## Status Lifecycle

A task moves through these states:

```
                       ┌─────────────┐
                       │   pending   │ ← Initial state
                       └──────┬──────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              v               v               v
       ┌────────────┐  ┌───────────┐   ┌────────────┐
       │ delivered   │  │ cancelled │   │  pending   │
       │ (one-shot)  │  │ (/cancel) │   │ (recurring │
       └────────────┘  └───────────┘   │ due_at     │
                                        │ advanced)  │
                                        └────────────┘
```

- **pending** -- The task is waiting for its due time. All new tasks start here.
- **delivered** -- The task fired and the reminder was sent. Only one-shot tasks reach this state.
- **cancelled** -- The user cancelled the task via `/cancel`. The task remains in the database for audit purposes.

For recurring tasks, the status stays `'pending'` after delivery. Instead, the `due_at` column is advanced to the next occurrence.

## Repeat Types

| Type | Advance Rule | Example |
|------|-------------|---------|
| `NULL` / `"once"` | No advance. Status becomes `'delivered'`. | Fire once at 3pm, then done. |
| `"daily"` | `due_at` moves forward by 1 day. | Fire every day at 9am. |
| `"weekly"` | `due_at` moves forward by 7 days. | Fire every Monday at 9am. |
| `"monthly"` | `due_at` moves forward by 1 month. | Fire on the 1st of each month. |
| `"weekdays"` | `due_at` moves forward by 1 day, skipping Saturday and Sunday. | Fire Mon-Fri at 8:30am. |

The weekday skip is handled by the `complete_task()` method in the store. When a weekday task is delivered on Friday, the next `due_at` is set to Monday. On all other weekdays, it advances by exactly 1 day.

## How the Scheduler Uses This Table

Every `poll_interval_secs` seconds (default: 60), the scheduler loop:

1. **Queries** -- `SELECT id, channel, reply_target, description, repeat FROM scheduled_tasks WHERE status = 'pending' AND due_at <= datetime('now')`.
2. **Delivers** -- For each result, sends `"Reminder: {description}"` via the task's channel.
3. **Completes** -- Calls `complete_task()` which either marks the task as `'delivered'` (one-shot) or advances `due_at` (recurring).

The two indexes ensure both the polling query and the user-facing commands (`/tasks`, `/cancel`) are fast, even as the table grows.

## Schema Overview After All Migrations

After migration 005, the database has the following objects:

| Object | Type | Created By |
|--------|------|------------|
| `conversations` | Table | 001 + 003 |
| `messages` | Table | 001 |
| `facts` | Table | 001 + 003 |
| `audit_log` | Table | 002 |
| `_migrations` | Table | store.rs |
| `messages_fts` | Virtual table (FTS5) | 004 |
| `messages_fts_insert` | Trigger | 004 |
| `messages_fts_delete` | Trigger | 004 |
| `messages_fts_update` | Trigger | 004 |
| `scheduled_tasks` | Table | **005** |
| `idx_scheduled_tasks_due` | Index | **005** |
| `idx_scheduled_tasks_sender` | Index | **005** |
