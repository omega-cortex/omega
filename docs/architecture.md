# Architecture

## System Design

Omega is a personal AI agent infrastructure written in Rust. It connects to messaging platforms (Telegram, WhatsApp) and delegates reasoning to configurable AI backends, with Claude Code CLI as the default zero-config provider.

The system is a Cargo workspace with 7 crates. See `CLAUDE.md` for the full crate table and gateway event loop.

---

## Session-Based Prompt Persistence

The Claude Code CLI provider supports session-based prompt persistence to avoid re-sending the full system prompt and conversation history on every message. This yields ~90-99% token savings on continuation messages.

### How It Works

1. **First message:** The gateway builds a full `Context` (system prompt + history + current message) with `session_id: None`. The provider sends the entire flattened prompt to `claude -p`. The CLI response includes a `session_id` in its JSON output, which the provider returns in `MessageMetadata.session_id`.

2. **Subsequent messages:** The gateway sets `Context.session_id` to the stored session ID. `to_prompt_string()` detects this and skips the system prompt and history (already in the CLI session), emitting only a minimal context update (current time, keyword-gated conditional sections) prepended to the user message. The provider passes `--session-id` to the CLI.

3. **Invalidation:** The gateway clears the stored session ID when any of these occur:
   - `/forget` command or `FORGET_CONVERSATION` marker
   - Idle timeout (conversation goes stale)
   - Provider error (session may be corrupt or expired)

4. **Fallback:** If a session-based call fails, the gateway retries with a fresh full-context call, creating a new session. The user never sees the failure.

### Scope

- **Claude Code CLI only.** HTTP-based providers (OpenAI, Anthropic, Ollama, OpenRouter, Gemini) always receive the full context on every call — they have no session mechanism.
- **Complementary to memory.** The SQLite conversation history (omega-memory) provides cross-session persistence. CLI sessions provide within-conversation token savings. Both systems work together.

### Data Flow

```
Gateway                          Provider (Claude Code CLI)
  |                                    |
  |-- Context(session_id: None) ------>|  First call: full prompt
  |<-- MessageMetadata(session_id) ----|  Returns session_id
  |                                    |
  |  [stores session_id per user]      |
  |                                    |
  |-- Context(session_id: "abc") ----->|  Continuation: minimal prompt
  |<-- MessageMetadata(session_id) ----|  Same or new session_id
  |                                    |
  |  [/forget or error]                |
  |  [clears session_id]               |
  |                                    |
  |-- Context(session_id: None) ------>|  Fresh full prompt (new session)
```
