# Functionalities: Utilities

## Overview
Utility commands and support modules: `open` (open Google resources in browser), `completion` (shell completions), `exit-codes` (display exit codes), `schema` (generate CLI schema for agents), `agent` (agent-mode output), `version`, `time now`, `webhook serve` (push notification test server), and desire-path argument rewriting.

## CLI Commands

| # | Command | Handler | Location | Description |
|---|---------|---------|----------|-------------|
| 1 | `version` | `handle_version` | src/cli/mod.rs:165 | Print version info |
| 2 | `time now` | `handle_time_now` | src/cli/mod.rs:964 | Print current time in configured timezone |
| 3 | `open <target>` | `handle_open` | src/cli/mod.rs:9059 | Open Google resource URL in default browser |
| 4 | `completion <shell>` | `handle_completion` | src/cli/mod.rs:9081 | Generate shell completions (bash/zsh/fish/powershell) |
| 5 | `exit-codes` | `handle_exit_codes` | src/cli/mod.rs:9093 | Display exit code table |
| 6 | `schema [command]` | `handle_schema` | src/cli/mod.rs:9123 | Print CLI schema as JSON |
| 7 | `agent exit-codes` / `agent schema` | `handle_agent` | src/cli/mod.rs:9143 | Agent-oriented commands |
| 8 | `webhook serve` | `webhook::serve` | src/webhook/mod.rs | Start HTTP server for testing Google push notification webhooks; `--port` (default: 8765), `--bind` (default: 0.0.0.0) |

## Core Dispatch

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `execute` | src/cli/mod.rs:45 | Main entry: converts args, applies desire-path rewriting, parses CLI, dispatches |
| 2 | `dispatch_command` | src/cli/mod.rs:126 | Routes Command enum to handler, enforces --enable-commands |

## Open Module (`src/cli/open.rs`)

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `generate_url` | src/cli/open.rs | Generate web URL for a resource ID and type |
| 2 | `detect_from_url` | src/cli/open.rs | Detect resource type from a Google URL |
| 3 | `resolve_target` | src/cli/open.rs | Resolve target (ID or URL) to openable URL |

Supported resource types: Drive file, Folder, Docs, Sheets, Slides, Gmail thread (auto-detected or explicit).

## Completion Module (`src/cli/completion.rs`)

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `generate_completions` | src/cli/completion.rs | Generate completions via clap_complete |

Supported shells: Bash, Zsh, Fish, PowerShell.

## Agent/Schema Module (`src/cli/agent.rs`)

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `exit_code_table` | src/cli/agent.rs | Build structured exit code table |
| 2 | `build_schema` | src/cli/agent.rs | Build CLI command schema from clap Command tree |
| 3 | `generate_schema` | src/cli/agent.rs | Generate and serialize full CLI schema |

## Desire Path Rewriting (`src/cli/mod.rs`)

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `rewrite_desire_path_args` | src/cli/mod.rs:9154 | Rewrite natural CLI patterns to canonical clap form |
| 2 | `enforce_enabled_commands` | src/cli/mod.rs:9322 | Enforce --enable-commands allowlist |

Examples of desire-path rewrites:
- `gmail search foo` -> `gmail search --query foo`
- `calendar events --from tomorrow` -> proper time argument placement
- Command alias expansion (e.g., `cal` -> `calendar`)

## CLI Root Types (`src/cli/root.rs`)

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | Cli | Struct | src/cli/root.rs:31 | Root CLI parser (flags + command) |
| 2 | RootFlags | Struct | src/cli/root.rs:41 | 13 global flags |
| 3 | Command | Enum | src/cli/root.rs:97 | 24 top-level command variants |
| 4 | ResourceType | Enum | src/cli/open.rs | Auto, Drive, Folder, Docs, Sheets, Slides, GmailThread |
| 5 | ShellType | Enum | src/cli/completion.rs | Bash, Zsh, Fish, PowerShell |

## Webhook Module (`src/webhook/mod.rs`)

HTTP server for testing Google push notification webhooks. Receives POST requests, logs X-Goog-* headers and JSON body to stdout.

| # | Function | Location | Description |
|---|----------|----------|-------------|
| 1 | `serve` | src/webhook/mod.rs | Start HTTP server on `bind:port`, accept connections until Ctrl+C |
| 2 | `handle_request` | src/webhook/mod.rs | Route handler — POST returns 200 OK with logged payload, non-POST returns 405 |
| 3 | `format_banner` | src/webhook/mod.rs | Build startup banner string (listening address + routes) |
| 4 | `extract_goog_headers` | src/webhook/mod.rs | Filter X-Goog-* headers from request (case-insensitive) |

**Routes:**
- `POST /` — catch-all
- `POST /webhook/google/gmail` — Gmail push notifications
- `POST /webhook/google/calendar` — Calendar push notifications
- `POST /webhook/google/drive` — Drive change notifications

**Types:**

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | WebhookArgs | Struct | src/cli/root.rs | CLI args for `webhook` |
| 2 | WebhookCommand | Enum | src/cli/root.rs | Subcommands: Serve |
| 3 | WebhookServeArgs | Struct | src/cli/root.rs | `--port` (default: 8765), `--bind` (default: 0.0.0.0) |

## Shared Watch Types (`src/services/common.rs`)

Common types used by Calendar and Drive watch modules:

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | WatchChannelRequest | Struct | src/services/common.rs | Watch registration body (id, type, address, params) |
| 2 | WatchParams | Struct | src/services/common.rs | Watch parameters (ttl in seconds) |
| 3 | WatchChannelResponse | Struct | src/services/common.rs | Watch response (id, resource_id, expiration) |
| 4 | ChannelStopRequest | Struct | src/services/common.rs | Channel stop body (id, resource_id) |
| 5 | StartPageTokenResponse | Struct | src/services/common.rs | Drive start page token response |

## Potential Dead Code

- `src/cli/exit_codes.rs` — re-export module, no unique logic
- `src/cli/desire_paths.rs` — re-export module, no unique logic
- `src/auth/account.rs` — re-export module, no unique logic
- `src/output/mode.rs` — re-export module, no unique logic

These are structural placeholders that exist for module organization but contain no unique functionality.
