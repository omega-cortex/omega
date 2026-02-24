# FTS5 Cross-Conversation Recall (Migration 004)

## Path

`backend/crates/omega-memory/migrations/004_fts5_recall.sql`

## What This Migration Does

Migration 004 gives Omega long-term memory that doesn't lose details. Before this migration, when a conversation closed, its content was compressed into a 1-2 sentence summary — specific commands, code snippets, and configurations were lost. Now, Omega can search ALL past messages to find relevant details from any previous conversation.

This works using SQLite's built-in FTS5 (Full-Text Search) engine. No new dependencies are added.

## Migration Sequence

| Order | File | What It Creates |
|-------|------|----------------|
| 1 | `001_init.sql` | Core tables: `conversations`, `messages`, `facts` |
| 2 | `002_audit_log.sql` | Audit trail: `audit_log` |
| 3 | `003_memory_enhancement.sql` | Conversation lifecycle + per-user facts |
| **4** | **`004_fts5_recall.sql`** | **FTS5 search index + auto-sync triggers** |

Migrations run automatically when the memory store initializes. Each migration runs exactly once.

## How Cross-Conversation Recall Works

When you send a message to Omega, the context builder now does an additional step:

```
User sends message: "How did I configure nginx last time?"
       |
       v
  [Existing steps: fetch conversation history, facts, summaries]
       |
       v
  [NEW] FTS5 search: find past messages matching "configure nginx"
       |
       v
  Results from 3 weeks ago:
  - "I need to set up nginx reverse proxy for port 8080"
  - "The SSL cert is at /etc/letsencrypt/live/example.com"
       |
       v
  These are included in the system prompt as "Related past context"
       |
       v
  Omega can now reference specific details from past conversations
```

### What Gets Searched

Only **user messages** are indexed. Assistant responses are excluded because:
- Your messages contain the intent — what you asked about, what you were working on.
- Including assistant responses would add noise and double the index size.

### Security

Users can only recall their own messages. The search always filters by `sender_id`, so one user cannot see another user's past conversations.

### Resilience

If the FTS search fails for any reason (corrupted index, unexpected query syntax), context building still works — it just lacks the recalled messages. The same `unwrap_or_default()` resilience pattern used for facts and summaries applies here.

## What Changed in the Database

### New: `messages_fts` Virtual Table

A FTS5 virtual table that indexes the `content` column of user messages. It uses "content-sync" mode, meaning it stores only the search index (not a copy of message text), keeping storage overhead minimal.

### New: 3 Auto-Sync Triggers

| Trigger | When It Fires | What It Does |
|---------|---------------|-------------|
| `messages_fts_insert` | After a user message is inserted | Adds it to the search index |
| `messages_fts_delete` | After a user message is deleted | Removes it from the search index |
| `messages_fts_update` | After a user message's content is updated | Re-indexes it |

These triggers keep the search index in sync automatically. No changes to `store_exchange()` were needed.

### Backfill

When the migration runs, all existing user messages are indexed. This is a one-time operation.

## How Recalled Messages Appear in Context

Recalled messages are added to the system prompt in a new section after summaries:

```
Recent conversation history:
- [2025-06-10 14:30:00] User discussed deploying Rust service with nginx.

Related past context:
- [2025-05-20 09:15:00] User: I need to set up nginx reverse proxy for port 8080...
- [2025-05-20 09:18:00] User: The SSL cert is at /etc/letsencrypt/live/example.com...
```

Key details:
- Up to 5 recalled messages are included (ranked by relevance).
- Messages are truncated to 200 characters to avoid bloating the prompt.
- Messages from the current conversation are excluded (they are already in the history).

## Performance

- **Search speed:** Sub-millisecond for typical database sizes.
- **Storage overhead:** FTS5 content-sync stores only the index (~10-30% of indexed text size).
- **Insert overhead:** Negligible — the trigger adds microseconds to each `store_exchange()` call.

## Schema Overview After All Migrations

After migration 004, the database has the following objects:

| Object | Type | Created By |
|--------|------|------------|
| `conversations` | Table | 001 + 003 |
| `messages` | Table | 001 |
| `facts` | Table | 001 + 003 |
| `audit_log` | Table | 002 |
| `_migrations` | Table | store.rs |
| `messages_fts` | Virtual table (FTS5) | **004** |
| `messages_fts_insert` | Trigger | **004** |
| `messages_fts_delete` | Trigger | **004** |
| `messages_fts_update` | Trigger | **004** |
