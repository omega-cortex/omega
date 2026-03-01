# OMEGA

**Your AI, your server, your rules.**

A personal AI agent infrastructure written in Rust. Omega connects to messaging platforms, delegates reasoning to configurable AI backends, and acts autonomously on your behalf. Single binary, no Docker, no cloud dependency.

## What Makes Omega Different

- **Autonomous, not assistive** -- Omega executes tasks, schedules follow-ups, and closes its own loops. It doesn't wait to be asked twice.
- **6 AI providers** -- Claude Code CLI, Anthropic API, OpenAI, Ollama, OpenRouter, Gemini. Swap with one config line.
- **Smart model routing** -- Every message is classified by complexity. Simple tasks use a fast model (Sonnet); complex work is decomposed into steps and executed by a powerful model (Opus). Automatic, no user intervention.
- **Real memory** -- SQLite-backed conversations, facts, summaries, and FTS5 semantic search. Omega learns who you are across sessions.
- **OS-level sandbox** -- Seatbelt (macOS) / Landlock (Linux) filesystem enforcement. Not just prompt-based -- three layers of protection.
- **Skill system** -- Extensible skills loaded from `~/.omega/skills/*/SKILL.md` with trigger-based MCP server activation.
- **Project system** -- Domain contexts loaded from `~/.omega/projects/*/ROLE.md` with per-project heartbeats, skills, and session isolation.
- **Reward-based learning** -- Detects its own mistakes, fixes them immediately, stores outcomes and lessons so they never repeat.
- **Multi-language** -- Full i18n for 8 languages: English, Spanish, Portuguese, French, German, Italian, Dutch, Russian.
- **Quantitative trading** -- External [`omega-trader`](https://github.com/omgagi/omega-trader) CLI with Kalman filter, HMM regime detection, Kelly sizing, IBKR TWS integration.
- **Runs locally** -- Your messages never touch third-party servers beyond the AI provider.

## Architecture

```
You (Telegram / WhatsApp)
        |
        v
  +-----------+     +----------------+     +-------------+
  |  Gateway   |---->|  AI Provider   |---->|   Response   |
  |            |     | (Claude Code,  |     |   + Markers  |
  |  Auth      |     |  Ollama, etc.) |     +------+------+
  |  Sanitize  |     +----------------+            |
  |  Classify  |                              +----v----+
  |  Route     |<-----------------------------| Process  |
  |  Audit     |      Memory (SQLite)         | Markers  |
  +-----------+      Facts, Summaries         +---------+
        |             Scheduled Tasks          SCHEDULE:
        v             Audit Log                SKILL_IMPROVE:
  +----------+                                 PROJECT_ACTIVATE:
  | Channels |                                 BUILD_PROPOSAL:
  | Telegram |                                 HEARTBEAT_ADD:
  | WhatsApp |                                 ...
  +----------+
```

Cargo workspace with 6 crates:

| Crate | Purpose |
|-------|---------|
| `omega-core` | Types, traits, config, error handling, prompt sanitization |
| `omega-providers` | 6 AI backends with unified `Provider` trait + agentic tool loop (bash/read/write/edit) + MCP client |
| `omega-channels` | Telegram (voice transcription, photo support) + WhatsApp (voice, images, groups, markdown) |
| `omega-memory` | SQLite storage, conversation history, facts, scheduled tasks, sessions, outcomes, audit log |
| `omega-skills` | Skill loader with TOML/YAML frontmatter, project system, trigger-based MCP server activation |
| `omega-sandbox` | Seatbelt (macOS) / Landlock (Linux) filesystem enforcement with 3-level isolation |

## Quick Start

```bash
curl -fsSL https://raw.githubusercontent.com/omgagi/omega/main/install.sh | bash
```

This downloads the latest release binary for your platform, installs it to `~/.local/bin/`, and runs `omega init` to walk you through setup.

Or build from source:

```bash
cd backend && cargo +nightly build --release
./target/release/omega init
./target/release/omega start
```

## How It Works

Every message flows through a deterministic pipeline:

1. **Dispatch** -- Concurrent per-sender. If you're already waiting for a response, new messages are buffered with an ack.
2. **Auth** -- Only allowed user IDs get through.
3. **Sanitize** -- Prompt injection patterns neutralized before reaching the AI (role tags, override phrases, zero-width bypasses).
4. **Identity** -- Cross-channel user identity resolved via alias system. New users auto-detected with language detection.
5. **Context** -- Conversation history + user facts + active project + skills injected into system prompt.
6. **Keywords** -- 9 keyword categories gate conditional prompt sections, reducing token usage by ~55-70%.
7. **Classify** -- Fast model (Sonnet) decides: simple task = direct response, complex work = step-by-step plan.
8. **Route** -- Simple tasks handled by Sonnet. Complex tasks decomposed and executed by Opus with progress updates.
9. **Markers** -- AI emits protocol markers (`SCHEDULE:`, `SKILL_IMPROVE:`, etc.) that the gateway processes and strips before delivery.
10. **Store** -- Exchange saved, conversation updated, facts extracted.
11. **Audit** -- Full interaction logged with model, tokens, processing time.
12. **Respond** -- Clean message sent back to user.

### Background Loops

- **Scheduler** -- Polls every 60s for due tasks. Reminders are delivered as text; action tasks invoke the AI with full tool access and retry logic. Supports quiet hours deferral.
- **Heartbeat** -- Clock-aligned periodic check-in. Classifies checklist items into domain groups, executes each group in parallel (Opus). Per-project heartbeats supported.
- **Summarizer** -- Conversations idle for 2+ hours are automatically summarized with fact extraction and closed.
- **CLAUDE.md maintenance** -- Background loop refreshes workspace CLAUDE.md every 24h, preserving dynamic content.

### Marker Protocol

The AI communicates with the gateway through protocol markers emitted in response text:

| Marker | Purpose |
|--------|---------|
| `SCHEDULE: desc \| datetime \| repeat` | Schedule a reminder |
| `SCHEDULE_ACTION: desc \| datetime \| repeat` | Schedule an autonomous action |
| `PROJECT_ACTIVATE: name` | Activate a project context |
| `PROJECT_DEACTIVATE` | Deactivate current project |
| `LANG_SWITCH: language` | Switch conversation language |
| `PERSONALITY: style` | Set behavior style |
| `SKILL_IMPROVE: skill \| lesson` | Update skill with learned lesson |
| `HEARTBEAT_ADD: item` | Add item to monitoring checklist |
| `HEARTBEAT_REMOVE: item` | Remove item from checklist |
| `HEARTBEAT_INTERVAL: minutes` | Change heartbeat frequency |
| `BUG_REPORT: description` | Log a self-detected bug |
| `BUILD_PROPOSAL: spec` | Propose a build for user confirmation |
| `REWARD: outcome` | Store a reward outcome |
| `LESSON: rule` | Store a learned behavioral rule |
| `CANCEL_TASK: id` | Cancel a scheduled task |
| `UPDATE_TASK: id \| changes` | Modify a scheduled task |
| `FORGET_CONVERSATION` | Close current conversation |
| `PURGE_FACTS` | Purge all user facts |

All markers are extracted, processed, and stripped before the response reaches the user. Anti-hallucination confirmations verify task creation against the actual database.

### Build Pipeline

Topology-driven multi-phase builds orchestrated via Claude Code CLI agent mode:

1. **Discovery** -- Brain agent analyzes the request, asks clarifying questions
2. **ParseBrief** -- Analyst agent enriches the brief
3. **Standard** -- Architect, test-writer, developer agents execute in sequence
4. **CorrectiveLoop** -- QA agent validates, developer fixes issues
5. **ParseSummary** -- Delivery agent generates build summary

### Project Setup

Interactive `/setup` command creates new projects with:
- Multi-round Brain agent questioning (max 3 rounds)
- HEARTBEAT.md generation for autonomous monitoring
- ROLE.md generation with domain-specific instructions

## Providers

| Provider | Type | Auth | Notes |
|----------|------|------|-------|
| `claude-code` | CLI subprocess | Local `claude` auth | Default. Auto-resume on max_turns. MCP server injection. |
| `anthropic` | HTTP | `x-api-key` header | Direct Anthropic API with agentic tool loop |
| `openai` | HTTP | Bearer token | Works with any OpenAI-compatible endpoint |
| `ollama` | HTTP | None | Local models (llama3.1, mistral, etc.) |
| `openrouter` | HTTP | Bearer token | Access 100+ models via single API |
| `gemini` | HTTP | x-goog-api-key header | Google Gemini API |

All HTTP providers include an agentic tool-execution loop (bash, read, write, edit) and MCP client support.

## Commands

| Command | Description |
|---------|-------------|
| `/status` | Uptime, provider, database info |
| `/memory` | Conversation and fact counts |
| `/history` | Last 5 conversation summaries |
| `/facts` | Known facts about you |
| `/forget` | Clear current conversation |
| `/tasks` | List scheduled tasks |
| `/cancel <id>` | Cancel a scheduled task |
| `/language` | Show or set language |
| `/personality` | Show or set behavior style |
| `/skills` | List available skills |
| `/projects` | List projects |
| `/project <name>` | Activate or deactivate a project |
| `/setup <desc>` | Create a new project interactively |
| `/heartbeat` | Show heartbeat status |
| `/learning` | Show reward outcomes and rules |
| `/purge` | Purge all user facts |
| `/whatsapp` | Start WhatsApp QR pairing |
| `/help` | Show all commands |

## HTTP API

Lightweight axum server for dashboard integration:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check with uptime, channel status |
| `/api/pair` | POST | Trigger WhatsApp pairing, returns QR as base64 PNG |
| `/api/webhook` | POST | Inbound message injection (inject or forward mode) |

Bearer token authentication with constant-time comparison.

## Quantitative Trading

Trading is handled by the standalone [`omega-trader`](https://github.com/omgagi/omega-trader) binary, invoked via the `ibkr-trader` skill.

## System Service

```bash
omega service install    # macOS LaunchAgent or Linux systemd (auto-start, restart on crash)
omega service status     # Check if running
omega service uninstall  # Remove
```

## Configuration

`config.toml` (gitignored):

```toml
[omega]
name = "Omega"
data_dir = "~/.omega"

[auth]
enabled = true

[provider]
default = "claude-code"

[provider.claude-code]
enabled = true
max_turns = 15
allowed_tools = ["Bash", "Read", "Write", "Edit"]

[provider.ollama]
enabled = true
base_url = "http://localhost:11434"
model = "llama3.1:8b"

[channel.telegram]
enabled = true
bot_token = "YOUR_TOKEN"
allowed_users = [123456789]

[memory]
db_path = "~/.omega/memory.db"
max_context_messages = 50

[heartbeat]
enabled = true
interval_minutes = 60
active_start = "09:00"
active_end = "23:00"

[scheduler]
enabled = true
poll_interval_secs = 60

[api]
enabled = false
host = "127.0.0.1"
port = 3000
```

## Requirements

- Rust nightly (for WhatsApp dependency)
- `claude` CLI installed and authenticated (for default provider)
- Telegram bot token (from [@BotFather](https://t.me/BotFather))

## Development

```bash
cd backend
cargo clippy --workspace     # Lint (zero warnings required)
cargo test --workspace       # All tests must pass
cargo fmt --check            # Formatting check
cargo build --release        # Optimized binary
```

## Codebase Stats

- **154 functionalities** across 18 modules
- **6 library crates** + 1 binary crate
- **18 bot commands**, **21 protocol markers**, **9 keyword categories**
- **8 languages** supported
- Full inventory: [`docs/functionalities/FUNCTIONALITIES.md`](docs/functionalities/FUNCTIONALITIES.md)

## License

MIT
