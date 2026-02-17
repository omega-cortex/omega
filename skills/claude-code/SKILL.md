---
name: claude-code
description: Claude Code CLI — AI coding agent. Use for running Claude as a subprocess for code generation, analysis, and automation tasks from the workspace.
requires: [claude]
---

# Claude Code CLI

Use `claude` for AI-powered coding, analysis, and automation tasks.

## Modes

- **Interactive** (default): `claude` — starts a REPL session
- **Non-interactive**: `claude -p "prompt"` — one-shot, pipe-friendly, returns result to stdout
- **Continue last**: `claude -c` — resume the most recent conversation
- **Resume session**: `claude -r <session-id>` — resume a specific session

## Key Options

| Flag | Description |
|------|-------------|
| `-p, --print` | Non-interactive mode (one-shot query, no REPL) |
| `-c, --continue` | Continue most recent conversation |
| `-r, --resume <id>` | Resume a specific session by ID |
| `--output-format json\|stream-json\|text` | Output format (`json` for structured parsing) |
| `--model <name>` | Model selection (e.g. `sonnet`, `opus`) |
| `--system-prompt <text>` | Override system prompt |
| `--append-system-prompt <text>` | Append to default system prompt |
| `--allowedTools <tools>` | Restrict to specific tools (e.g. `"Bash(git:*) Edit Read"`) |
| `--disallowedTools <tools>` | Block specific tools |
| `--permission-mode <mode>` | `plan` (read-only), `default`, `bypassPermissions` |
| `--max-turns <n>` | Limit agentic turns in non-interactive mode |
| `--mcp-config <file>` | Load MCP server configuration from JSON file |
| `--verbose` | Enable verbose logging |

## JSON Output

With `--output-format json`, the response structure is:
```json
{"type": "result", "subtype": "success", "result": "...", "model": "...", "session_id": "..."}
```
When `subtype` is `error_max_turns`, extract `result` if available.

## Subcommands

| Command | Description |
|---------|-------------|
| `claude auth login` | Authenticate with Anthropic |
| `claude auth status` | Check authentication status |
| `claude mcp list` | List configured MCP servers |
| `claude mcp add <name> -- <cmd>` | Add an MCP server |
| `claude update` | Update Claude Code to latest version |
| `claude doctor` | Health check and diagnostics |

## Common Patterns

One-shot query with JSON output:
```bash
claude -p "explain this function" --output-format json
```

Scoped tools for safe operation:
```bash
claude -p "review the code" --allowedTools "Read Grep Glob" --permission-mode plan
```

Continue a session with model override:
```bash
claude -c --model opus
```

Pipe input for analysis:
```bash
cat file.rs | claude -p "find bugs in this code"
```

MCP server configuration:
```bash
claude --mcp-config servers.json -p "query the database"
```

## Notes

- Default timeout is 10 minutes for subprocess invocations
- Use `--max-turns` to bound agentic loops in automation
- `--permission-mode bypassPermissions` skips all confirmation prompts (use with caution)
- Session IDs from JSON output can be used with `-r` to continue conversations
