# Gateway Architecture & Message Flow

## Overview

The **Gateway** is the central orchestrator of Omega's event loop. It sits at the intersection of:
- **Messaging channels** (Telegram, WhatsApp) — where users send messages.
- **AI providers** (Claude Code CLI, Anthropic API, etc.) — where reasoning happens.
- **Memory store** (SQLite) — where conversation history and user facts are persisted.
- **Audit system** — where all interactions are logged for security and debugging.

The gateway's job is simple: listen for messages, process them through a deterministic pipeline, get a response from an AI provider, store the exchange, and send the response back to the user.

## Conceptual Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GATEWAY EVENT LOOP                          │
│                                                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ Incoming Messages (via MPSC)                                  │  │
│  │                                                                 │  │
│  │ Telegram → Channel Listener → ┐                               │  │
│  │ WhatsApp → Channel Listener → ├→ MPSC Queue → Main Loop       │  │
│  │                                                                 │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ Main Event Loop (tokio::select!)                              │  │
│  │ • Wait for message from MPSC                                  │  │
│  │ • Wait for Ctrl+C shutdown signal                             │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │ Background Tasks (concurrent, non-blocking)                   │  │
│  │ • Conversation Summarizer (every 60s)                         │  │
│  │ • Typing Indicators (every 5s per message)                    │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## The Message Processing Pipeline

When a user sends a message, it travels through the gateway in eight sequential stages. Understanding each stage is key to understanding how Omega works.

### Stage 1: Authentication Check

**What happens:** The gateway verifies that the sender is authorized to use Omega.

**Implementation:**
- Calls `check_auth()` which examines:
  - Which channel the message came from (Telegram, WhatsApp, etc.).
  - Per-channel allow-lists (e.g., Telegram user IDs).
- Empty allow-lists default to "allow all" (useful for testing).
- Non-empty allow-lists are strict whitelists.

**On Failure:**
- The message is rejected immediately.
- A denial message is sent back to the user.
- The denial is logged in the audit system with `AuditStatus::Denied`.
- The message never reaches the provider.

**Security Model:**
This is a simple but effective defense. Omega will not process messages from unauthorized users, preventing unauthorized access to your AI assistant.

### Stage 2: Input Sanitization

**What happens:** User input is cleaned to prevent injection attacks and prompt manipulation.

**Implementation:**
- Calls `sanitize()` from `omega_core`.
- Detects patterns that could break out of the system prompt or manipulate the AI backend.
- Returns the cleaned text and a list of detected issues.

**Examples of What Gets Sanitized:**
- Control sequences (newlines in unexpected places).
- Attempts to override the system prompt.
- Shell metacharacters if the backend were to execute commands.

**Result:**
- Input text is replaced with the sanitized version.
- If sanitization modified the text, a warning is logged.
- All subsequent processing uses the clean text.

**Security Model:**
Sanitization is a defense-in-depth measure. Even if an injection pattern gets through, it's neutralized before reaching the AI provider.

### Stage 3: Command Dispatch

**What happens:** The gateway checks if the input is a bot command rather than a regular message.

**Implementation:**
- Calls `commands::Command::parse()` to extract command intent.
- Built-in commands include:
  - `/uptime` - How long Omega has been running.
  - `/help` - List available commands.
  - `/status` - System health information.
  - `/facts` - Retrieve stored facts about the user.
  - `/memory` - Retrieve conversation history.

**On Command Match:**
- The command is handled locally without calling the AI provider.
- A response is returned immediately.
- The message processing stops here (provider is never called).

**Why This Exists:**
Commands are fast, deterministic, and don't require AI reasoning. They provide system introspection without API latency or cost.

### Stage 4: Typing Indicator

**What happens:** The gateway tells the channel that Omega is thinking.

**Implementation:**
- Gets the channel that received the message.
- Sends an initial typing action immediately.
- Spawns a background task that repeats the typing action every 5 seconds.
- The repeater runs concurrently while processing the message.

**Why This Exists:**
Users expect to see "typing" indicators on messaging platforms. Without them, it looks like Omega is broken or hung. The repeater keeps the indicator visible during long provider calls.

**Cleanup:**
- When the response is ready, the repeater task is aborted.
- If an error occurs during processing, the repeater is aborted early.

### Stage 5: Context Building

**What happens:** The gateway builds a rich context for the AI provider, including conversation history and user facts.

**Implementation:**
- Calls `memory.build_context(&incoming)`.
- The context includes:
  - The user's current message.
  - Recent conversation history (previous exchanges in the same thread).
  - Stored facts about the user (name, preferences, etc.).
  - A system prompt guiding the AI to be helpful and safe.

**Why This Exists:**
Raw AI models are stateless. They have no memory of previous conversations. The context gives the AI a chance to be conversational and personalized.

**Example:**
```
# System Prompt
You are a helpful AI assistant named Omega...

# User Facts
- Name: Alice
- Timezone: America/Los_Angeles
- Preference: Brief, direct responses

# Recent History
User: What's the weather?
Assistant: I don't have real-time weather data, but you can check...
User: What about next week?
Assistant: You'd need to check a weather service like...

# Current Message
User: Thanks. What about my location?
```

**Error Handling:**
If context building fails (e.g., database error), an error message is sent immediately and the message is dropped. The provider is never called.

### Stage 6: Provider Call

**What happens:** The gateway sends the enriched context to the AI provider and gets a response.

**Implementation:**
- Calls `provider.complete(&context)`.
- The provider is typically the Claude Code CLI but can be swapped (OpenAI, Anthropic, Ollama, etc.).
- The provider returns a `Response` with:
  - `text`: The assistant's answer.
  - `metadata.provider_used`: Which provider generated this (for audit logging).
  - `metadata.model`: Which model was used (e.g., "claude-opus-4-6").
  - `metadata.processing_time_ms`: How long the request took.

**Why This Exists:**
This is where the actual AI reasoning happens. Everything else in the pipeline is infrastructure.

**Error Handling:**
If the provider fails, an error message is sent to the user and the message is dropped. The error is logged with full details.

**Performance:**
Provider calls are the slowest part of the pipeline (typically 2-30 seconds). Everything else is near-instant.

### Stage 7: Memory Storage

**What happens:** The exchange (user input + AI response) is saved to the SQLite database.

**Implementation:**
- Calls `memory.store_exchange(&incoming, &response)`.
- This saves:
  - The user's text (sanitized).
  - The AI's response.
  - Metadata (channel, sender_id, timestamp).
  - Links the exchange to the conversation thread.

**Why This Exists:**
Without persistent memory, Omega forgets every message after it's processed. Storage enables continuity and allows the context builder to fetch history for future messages.

**Error Handling:**
If storage fails, the error is logged but does not block the response. The user gets their answer even if the database is temporarily unavailable. This is intentional: providing service is more important than logging.

### Stage 8: Audit Logging

**What happens:** The interaction is logged for security, compliance, and debugging.

**Implementation:**
- Calls `audit.log(&AuditEntry)` with:
  - Channel name, sender_id, sender_name.
  - Input text and output text (the actual exchange).
  - Provider name and model used.
  - Processing time.
  - Status (Ok, Denied, Error).

**Why This Exists:**
Audit logs answer critical questions:
- Who said what and when?
- Which provider answered which question?
- Were there any errors or denials?
- Is there a pattern of misuse?

**Privacy Note:**
Audit logs include the actual message text. Store them securely and comply with data retention laws.

### Stage 9: Send Response

**What happens:** The response is sent back to the user via the channel that received the message.

**Implementation:**
- Gets the channel by name.
- Calls `channel.send(response)`.
- If the send fails, the error is logged but processing is complete.

**Why This Exists:**
The message must be delivered to the user. If the channel fails (e.g., Telegram API is down), there's nothing to do but log it.

**Error Handling:**
Send errors are logged but do not cause a retry or escalation. The assumption is that the channel will handle retries internally if needed.

## Full Pipeline Diagram

```
User sends message on Telegram
         ↓
    [Channel Listener] spawns task to forward to gateway
         ↓
    [MPSC Queue] receives message
         ↓
    [Main Event Loop] selects message from queue
         ↓
┌─────────────────────────────────────────┐
│        handle_message() executes         │
├─────────────────────────────────────────┤
│                                          │
│ Stage 1: check_auth()                   │
│  ✓ Allowed? → Continue                  │
│  ✗ Denied?  → Send deny, audit, return  │
│                                          │
│ Stage 2: sanitize()                     │
│  • Clean input                          │
│  • Replace text with sanitized version  │
│                                          │
│ Stage 3: commands::parse()              │
│  ✓ Is command? → Handle locally, return │
│  ✗ Not command? → Continue              │
│                                          │
│ Stage 4: send_typing()                  │
│  • Spawn repeater task (every 5s)       │
│                                          │
│ Stage 5: memory.build_context()         │
│  • Fetch history + facts                │
│  ✓ Success? → Continue                  │
│  ✗ Error? → Send error, audit, return   │
│                                          │
│ Stage 6: provider.complete()            │
│  • Call Claude Code CLI (or other)      │
│  ✓ Success? → Continue                  │
│  ✗ Error? → Send error, audit, return   │
│                                          │
│ Stage 7: memory.store_exchange()        │
│  • Save to SQLite (best-effort)         │
│                                          │
│ Stage 8: audit.log()                    │
│  • Log to SQLite                        │
│                                          │
│ Stage 9: channel.send()                 │
│  • Send response via Telegram/WhatsApp  │
│  • Abort typing repeater task           │
│                                          │
└─────────────────────────────────────────┘
         ↓
    Message complete, ready for next message
```

## Conversation Lifecycle

Messages are grouped into conversations. A conversation is a thread of related exchanges between a user and Omega.

### Conversation Boundaries

Conversations are isolated by:
- **User** (sender_id).
- **Channel** (Telegram, WhatsApp, etc.).
- **Time** — After a period of inactivity (threshold TBD), a conversation is closed.

### Conversation Summarization

Every 60 seconds, the background summarizer runs:

1. **Find idle conversations** — Find all conversations inactive for N minutes.
2. **Summarize each** — Call the provider to generate a 1-2 sentence summary.
3. **Extract facts** — Call the provider to extract user facts (name, preferences, etc.).
4. **Store facts** — Save extracted facts to the user profile.
5. **Close conversation** — Mark the conversation as closed and store the summary.

**Why Summarization?**

- **Memory efficiency** — Summaries are short; full history is long.
- **Context window management** — Older conversations are summarized into facts, not kept in full.
- **User profiling** — Facts extracted from conversations are reused in future exchanges.

**Example:**

```
Conversation 1 (inactive, 30+ minutes):
User: What's your favorite food?
Assistant: As an AI, I don't eat, but I find it interesting that...
User: Do you think AI will replace humans?
Assistant: It's complex. AI augments human capability...

→ Summarization triggered
→ Summary: "User interested in AI ethics and food. Thoughtful questions."
→ Facts extracted:
   - interested_in: "AI ethics"
   - question_style: "philosophical"

Conversation 2 (current):
User: Any good book recommendations?
Assistant: [builds context with previous facts about philosophical interests]
```

## Error Recovery & Resilience

The gateway is designed to be resilient:

### Non-Fatal Errors
- Database temporarily unavailable → Store fails, but response still sent.
- Audit logging fails → Logged and ignored, processing continues.
- Channel send fails → Logged and ignored, pipeline completes.
- Provider returns an error → Error message sent, audit logged, pipeline stops.

### Fatal Errors
- Channel startup fails → Gateway initialization fails, Omega exits.
- Auth denied → Message dropped, pipeline stops.

### Graceful Shutdown
When Omega receives Ctrl+C:
1. Main event loop breaks.
2. Background summarizer is aborted.
3. All active conversations are summarized (preserving memory).
4. All channels are stopped cleanly.
5. Omega exits.

This ensures no in-flight conversations are lost.

## Concurrency Model

The gateway uses a **single-threaded, async architecture**:

- **One main thread** — Processes messages sequentially on the main event loop.
- **Multiple background tasks** — Channel listeners, typing repeaters, summarizer run in separate tokio tasks.
- **No locks** — All access is through `Arc` shared references. No Mutex or RwLock needed.

**Why this design?**

- **Simplicity** — No race conditions to reason about.
- **Efficiency** — Message processing is I/O-bound (network, database), not CPU-bound. Concurrency is achieved through async/await, not threads.
- **Scalability** — Can handle many concurrent channels and users without thread overhead.

## Configuration

The gateway accepts two config sources:

### AuthConfig
```toml
[auth]
enabled = true
deny_message = "Sorry, you're not authorized to use Omega."
```

Controls whether authentication is enforced globally.

### ChannelConfig
```toml
[telegram]
token = "YOUR_BOT_TOKEN"
allowed_users = [123456789, 987654321]  # Empty = allow all
```

Controls per-channel settings. For Telegram, the allowed_users list is a whitelist. An empty list allows anyone (useful for testing).

## Observability

### Logging
- **INFO** — Gateway startup, messages received, responses sent, summaries completed.
- **WARN** — Auth denials, input sanitization warnings, errors during background tasks.
- **ERROR** — Provider failures, database errors, channel failures.

### Audit Trail
Every interaction is logged to SQLite with full context. Query the audit table to see:
- Who said what and when.
- Which provider answered.
- How long it took.
- Whether there were any errors.

### Example Audit Query
```sql
SELECT channel, sender_id, input_text, output_text, model, processing_ms, status
FROM audit_log
WHERE sender_id = '123456789'
ORDER BY created_at DESC
LIMIT 10;
```

## Performance Characteristics

### Latency
- **Auth check** — <1ms (in-memory comparison).
- **Sanitization** — <1ms (regex scan).
- **Context building** — 10-50ms (database query, history fetch).
- **Provider call** — 2,000-30,000ms (API request).
- **Memory storage** — 10-100ms (database insert).
- **Audit logging** — <1ms (queued insert).
- **Response send** — 100-1000ms (network, channel API).

**Total:** Dominated by provider call (2-30 seconds).

### Throughput
- The main loop processes one message at a time (sequential).
- While one message is being processed, other incoming messages wait in the MPSC queue (capacity 256).
- If the queue fills (256 messages waiting), new messages are blocked until space opens.

**Recommended:** Keep the queue from filling by ensuring provider calls complete in <30 seconds.

### Memory
- Gateway struct stores references (Arc) to channels, provider, memory.
- No per-message allocations that aren't freed.
- MPSC queue holds up to 256 IncomingMessage objects in memory.

## Security Posture

1. **Auth Enforcement** — Messages from unauthorized users are rejected immediately.
2. **Input Sanitization** — Injection patterns are neutralized before provider call.
3. **Audit Logging** — All interactions are logged for intrusion detection.
4. **Error Suppression** — Detailed errors are logged internally but generic messages are sent to users (no info leaks).
5. **Graceful Degradation** — If components fail, the gateway degrades gracefully (e.g., storage failure doesn't block user response).

## Design Rationale

### Why MPSC Channel?
All incoming messages funnel through a single MPSC queue. This ensures:
- Messages are processed in order (no race conditions).
- The main loop can wait on a single receiver (tokio::select!).
- Backpressure is built-in (queue fills if processing is slow).

### Why Arc for Shared References?
Provider and channels are wrapped in Arc to:
- Allow cloning without deep copying (cheap clones for spawned tasks).
- Enable thread-safe access without locking (Arc is read-only).
- Avoid lifetime issues in async code (Arc lives as long as all clones exist).

### Why Background Summarization?
Summarization runs in a separate task to:
- Not block the main event loop.
- Preserve memory across conversation boundaries.
- Extract user facts for personalization.

### Why Graceful Shutdown?
On Ctrl+C, Omega summarizes all active conversations to:
- Avoid losing context from in-flight exchanges.
- Cleanly close all database connections.
- Stop all background tasks.

## Next Steps & Future Enhancements

### Phase 4 (Planned)
- Alternative providers — Direct integration with OpenAI, Anthropic APIs.
- Skills system — Plugins for custom functions (weather, calendar, etc.).
- Sandbox environment — Safe execution of user code.
- Cron scheduler — Scheduled tasks and reminders.
- WhatsApp support — Full WhatsApp channel implementation.

### Possible Improvements
- Adaptive summarization — Summarize based on content, not just time.
- Conversation branching — Support multiple concurrent threads from the same user.
- Streaming responses — Send response text incrementally instead of waiting for completion.
- Retry logic — Exponential backoff for transient failures.
