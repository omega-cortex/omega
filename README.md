# Omega

**Your AI, your server, your rules.**

A personal AI agent that runs on your own hardware. Connects to Telegram, delegates reasoning to Claude Code CLI, and remembers your conversations across sessions. Single Rust binary, no Docker, no cloud dependency.

## What Makes Omega Different

- **Runs locally** — Your messages never touch third-party servers beyond the AI provider
- **Real memory** — Conversations are summarized and recalled across sessions. Omega learns facts about you over time.
- **Zero config AI** — Uses your local `claude` CLI authentication. No API keys to manage.
- **Action-oriented** — Omega does things, not just talks about them
- **2-minute setup** — `omega init` walks you through everything

## Quick Start

```bash
# Build
cargo build --release

# Interactive setup
./target/release/omega init

# Start
./target/release/omega start
```

Or manual setup:

```bash
cp config.example.toml config.toml   # Edit with your settings
./target/release/omega start
```

## How It Works

```
You (Telegram) → Omega Gateway → Claude Code CLI → Response
                      │
                 ┌────┴────┐
              Memory    Audit Log
            (SQLite)    (SQLite)
```

Every message flows through:

1. **Auth** — Only your Telegram user ID gets through
2. **Sanitize** — Prompt injection patterns neutralized
3. **Memory** — Context built from conversation history + facts + past summaries
4. **Provider** — Claude Code CLI processes the request
5. **Store** — Exchange saved, conversation updated
6. **Audit** — Full interaction logged
7. **Respond** — Message sent back with typing indicator

Conversations idle for 30+ minutes are automatically summarized and closed. New conversations include recent summaries for continuity.

## Commands

| Command | Description |
|---------|-------------|
| `/status` | Uptime, provider, database info |
| `/memory` | Your conversation and fact counts |
| `/history` | Last 5 conversation summaries |
| `/facts` | Known facts about you |
| `/forget` | Clear current conversation |
| `/help` | List commands |

Commands are instant (no AI call). Everything else goes to the provider.

## Requirements

- Rust 1.70+
- `claude` CLI installed and authenticated
- Telegram bot token (from [@BotFather](https://t.me/BotFather))

## Configuration

`config.toml` (gitignored):

```toml
[omega]
name = "Omega"

[auth]
enabled = true

[provider]
default = "claude-code"

[provider.claude-code]
max_turns = 10
allowed_tools = ["Bash", "Read", "Write", "Edit"]

[channel.telegram]
enabled = true
bot_token = "YOUR_TOKEN"
allowed_users = [123456789]    # Your Telegram user ID

[memory]
db_path = "~/.omega/memory.db"
max_context_messages = 50
```

## Architecture

Cargo workspace with 6 crates:

| Crate | Purpose |
|-------|---------|
| `omega-core` | Types, traits, config, error handling, prompt sanitization |
| `omega-providers` | AI backends (Claude Code CLI + planned: Anthropic, OpenAI, Ollama) |
| `omega-channels` | Messaging platforms (Telegram + planned: WhatsApp) |
| `omega-memory` | SQLite storage, conversation history, facts, audit log |
| `omega-skills` | Skill/plugin system (planned) |
| `omega-sandbox` | Secure command execution (planned) |

## macOS Service

Run as a persistent LaunchAgent:

```bash
cp com.ilozada.omega.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.ilozada.omega.plist
```

## Development

```bash
cargo clippy --workspace     # Lint (zero warnings required)
cargo test --workspace       # All tests must pass
cargo fmt --check            # Formatting check
cargo build --release        # Optimized binary
```

## License

MIT
