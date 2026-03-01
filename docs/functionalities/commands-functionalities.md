# Functionalities: Commands

## Overview

Built-in bot commands that provide instant responses without provider calls. 18 commands organized into submodules: status, tasks, settings, learning.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | Command::parse() | Service | `backend/src/commands/mod.rs:54` | Parses command from message text with @botname stripping; returns None for unknown /prefixes | -- |
| 2 | /status | Command | `backend/src/commands/status.rs` | System status: uptime, provider, memory stats | Store |
| 3 | /memory | Command | `backend/src/commands/status.rs` | Memory usage: conversation count, fact count, message count | Store |
| 4 | /history | Command | `backend/src/commands/status.rs` | Recent conversation summaries | Store |
| 5 | /facts | Command | `backend/src/commands/status.rs` | User's stored personal facts | Store |
| 6 | /forget | Command | `backend/src/commands/tasks.rs` | Clear current conversation (handled specially in pipeline) | Store |
| 7 | /tasks | Command | `backend/src/commands/tasks.rs` | List pending scheduled tasks | Store |
| 8 | /cancel | Command | `backend/src/commands/tasks.rs` | Cancel a scheduled task by ID prefix | Store |
| 9 | /language | Command | `backend/src/commands/settings.rs` | Set preferred language | Store |
| 10 | /personality | Command | `backend/src/commands/settings.rs` | Set or reset personality preference | Store |
| 11 | /skills | Command | `backend/src/commands/settings.rs` | List loaded skills | Skills |
| 12 | /projects | Command | `backend/src/commands/settings.rs` | List available projects with active indicator | Store, Projects |
| 13 | /project | Command | `backend/src/commands/settings.rs` | Activate, deactivate, or switch project | Store, Projects |
| 14 | /purge | Command | `backend/src/commands/tasks.rs` | Purge all user facts (preserving system keys) | Store |
| 15 | /whatsapp | Command | `backend/src/commands/settings.rs` | Returns WHATSAPP_QR marker to trigger pairing flow | -- |
| 16 | /heartbeat | Command | `backend/src/commands/settings.rs` | Show heartbeat status and configuration | HeartbeatConfig |
| 17 | /learning | Command | `backend/src/commands/learning.rs` | Show reward outcomes and learned behavioral rules | Store |
| 18 | /setup | Command | `backend/src/commands/mod.rs:138` | Intercepted early in pipeline; fallback shows help | -- |
| 19 | /help | Command | `backend/src/commands/status.rs` | Show available commands | -- |
| 20 | CommandContext | Model | `backend/src/commands/mod.rs:15` | Grouped context for command execution: store, channel, sender, text, uptime, provider, skills, projects, heartbeat settings, active project | -- |

## Internal Dependencies

- Pipeline dispatches to commands::handle() which routes to submodule handlers
- All commands use resolve_lang() for i18n
- /forget is intercepted early in pipeline and calls handle_forget()
- /setup is intercepted early in pipeline and calls start_setup_session()
- /whatsapp returns a magic string handled by pipeline

## Dead Code / Unused

- None detected.
