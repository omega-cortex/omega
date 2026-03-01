# Functionalities: omega-memory

## Overview

SQLite-backed persistent memory store with 13 tracked migrations. Manages conversations, messages (with FTS5 search), user facts, scheduled tasks, sessions, outcomes, and audit logging.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | Store | Model | `backend/crates/omega-memory/src/store/mod.rs:34` | SQLite memory store with connection pool and max_context_messages config | sqlx |
| 2 | Store::new() | Service | `backend/crates/omega-memory/src/store/mod.rs:41` | Creates store, ensures directory, connects SQLite with WAL mode, runs migrations | -- |
| 3 | run_migrations() | Service | `backend/crates/omega-memory/src/store/mod.rs:93` | Runs 13 SQL migrations with tracking table, bootstraps pre-tracking schemas | -- |
| 4 | Conversations submodule | Service | `backend/crates/omega-memory/src/store/conversations.rs` | Conversation lifecycle: create, find_active, find_idle (2h timeout), close, summaries, find_all_active | -- |
| 5 | Messages submodule | Service | `backend/crates/omega-memory/src/store/messages.rs` | Message storage: store_exchange(), get_conversation_messages(), FTS5 semantic search | -- |
| 6 | Facts submodule | Service | `backend/crates/omega-memory/src/store/facts.rs` | User facts CRUD: store_fact, get_fact, get_facts, delete_fact, delete_facts; aliases: create_alias, find_canonical_user, resolve_sender_id, is_new_user | -- |
| 7 | Tasks submodule | Service | `backend/crates/omega-memory/src/store/tasks.rs` | Scheduled task CRUD: create_task, get_due_tasks, complete_task (with repeat scheduling), cancel_task, update_task, defer_task, get_tasks_for_sender; DueTask struct | -- |
| 8 | Context submodule | Service | `backend/crates/omega-memory/src/store/context.rs` | Context building: build_context() combines system prompt + history + recall + tasks + profile + summaries + outcomes; format_user_profile(); detect_language() | -- |
| 9 | Sessions submodule | Service | `backend/crates/omega-memory/src/store/sessions.rs` | CLI session persistence: store_session, get_session, clear_session (project-scoped) | -- |
| 10 | Outcomes submodule | Service | `backend/crates/omega-memory/src/store/outcomes.rs` | Reward-based learning: store_outcome, get_recent_outcomes, store_lesson, get_lessons | -- |
| 11 | Context helpers | Service | `backend/crates/omega-memory/src/store/context_helpers.rs` | Onboarding stages, system prompt composition, language detection (8 languages) | -- |
| 12 | AuditLogger | Service | `backend/crates/omega-memory/src/audit.rs` | SQLite-backed audit log: AuditEntry (channel, sender, input, output, provider, model, timing, status), AuditStatus (Ok, Error, Denied) | -- |
| 13 | detect_language() | Utility | `backend/crates/omega-memory/src/store/context.rs` | Detects language from text (8 languages supported) | -- |
| 14 | DueTask | Model | `backend/crates/omega-memory/src/store/tasks.rs` | Due task struct: id, channel, sender_id, reply_target, description, task_type, repeat, project | -- |

## Internal Dependencies

- Gateway calls Store methods for all memory operations
- build_context() assembles the full conversation context from all submodules
- AuditLogger uses the same SQLite pool as Store
- Sessions are project-scoped for Claude Code CLI session_id persistence

## Dead Code / Unused

- None detected.
