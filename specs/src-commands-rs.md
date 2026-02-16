# Specification: src/commands.rs

## Overview

**File Path:** `src/commands.rs`

**Purpose:** Implements built-in bot commands for Omega. Commands are instant-response operations that bypass the AI provider entirely. They directly query and manipulate the memory store, providing users with immediate access to system status, conversation history, facts, and memory management.

**Key Characteristic:** All commands execute asynchronously with no external provider invocation.

---

## Command Enum

### Variants

```rust
pub enum Command {
    Status,
    Memory,
    History,
    Facts,
    Forget,
    Help,
}
```

| Command | Line | Purpose |
|---------|------|---------|
| `Status` | 8 | System uptime, active provider, and database size |
| `Memory` | 9 | User-specific stats: conversations, messages, facts count |
| `History` | 10 | Last 5 conversation summaries with timestamps |
| `Facts` | 11 | List of known facts about the user |
| `Forget` | 12 | Close and clear the current active conversation |
| `Help` | 13 | Display all available commands |

---

## Command Parsing

### Function: `Command::parse(text: &str) -> Option<Self>`

**Location:** Lines 19–30

**Behavior:**
- Extracts the first whitespace-delimited token from message text
- Matches against known command prefixes (all start with `/`)
- Returns `None` for unknown `/` prefixes, allowing them to pass through to the provider as regular messages
- Case-sensitive matching

**Command Prefixes Recognized:**
- `/status` → `Command::Status`
- `/memory` → `Command::Memory`
- `/history` → `Command::History`
- `/facts` → `Command::Facts`
- `/forget` → `Command::Forget`
- `/help` → `Command::Help`

**Example Behavior:**
```
"/status" → Some(Status)
"/help foobar" → Some(Help)  // whitespace-delimited, so "foobar" ignored
"/unknown" → None  // unknown commands pass through
"hello" → None  // non-command text returns None
```

---

## Command Handler

### Function: `handle(cmd, store, channel, sender_id, uptime, provider_name) -> String`

**Location:** Lines 34–50

**Signature:**
```rust
pub async fn handle(
    cmd: Command,
    store: &Store,
    channel: &str,
    sender_id: &str,
    uptime: &Instant,
    provider_name: &str,
) -> String
```

**Parameters:**
- `cmd`: The parsed command enum variant
- `store`: Reference to the memory store (SQLite-backed)
- `channel`: Messaging channel identifier (e.g., "telegram", "whatsapp")
- `sender_id`: User identifier within the channel
- `uptime`: Process start time (for elapsed duration calculation)
- `provider_name`: Active AI provider name (e.g., "Claude Code CLI")

**Return:** Formatted response text to send back to the user

**Dispatch:** Routes each command variant to its handler function (lines 43–48)

---

## Individual Command Handlers

### /status — `handle_status(store, uptime, provider_name)`

**Location:** Lines 52–70

**Behavior:**
- Calculates elapsed time since `uptime` in hours, minutes, seconds
- Queries `store.db_size()` for database file size
- Formats size using `format_bytes()`
- Returns multi-line response with three fields

**Response Format:**
```
Omega Status
Uptime: 1h 23m 45s
Provider: Claude Code CLI
Database: 2.3 MB
```

**Error Handling:** If `db_size()` fails, displays "unknown" instead of panicking

---

### /memory — `handle_memory(store, sender_id)`

**Location:** Lines 72–84

**Behavior:**
- Calls `store.get_memory_stats(sender_id)` (async)
- Retrieves tuple: `(convos: i64, msgs: i64, facts: i64)`
- Formats response with three counts

**Response Format (Success):**
```
Your Memory
Conversations: 5
Messages: 47
Facts: 3
```

**Response Format (Error):**
```
Error: [error description]
```

---

### /history — `handle_history(store, channel, sender_id)`

**Location:** Lines 86–98

**Behavior:**
- Calls `store.get_history(channel, sender_id, 5)` to fetch last 5 conversations
- Returns `Vec<(summary: String, timestamp: String)>`
- Iterates through entries, formatting each with timestamp and summary
- Handles empty history gracefully

**Response Format (With History):**
```
Recent Conversations

[2025-02-16 14:30:15]
Discussed project architecture and design patterns

[2025-02-16 13:15:22]
Reviewed Rust async/await best practices
```

**Response Format (Empty):**
```
No conversation history yet.
```

---

### /facts — `handle_facts(store, sender_id)`

**Location:** Lines 100–112

**Behavior:**
- Calls `store.get_facts(sender_id)` (async)
- Returns `Vec<(key: String, value: String)>` of fact key-value pairs
- Iterates and formats each fact as a bulleted list
- Handles empty facts gracefully

**Response Format (With Facts):**
```
Known Facts

- favorite_language: Rust
- location: San Francisco
- timezone: PST
```

**Response Format (Empty):**
```
No facts stored yet.
```

---

### /forget — `handle_forget(store, channel, sender_id)`

**Location:** Lines 114–120

**Behavior:**
- Calls `store.close_current_conversation(channel, sender_id)` (async)
- Closes and clears the active conversation for the user in the specified channel
- Returns boolean: `true` if a conversation was closed, `false` if none was active

**Response Format (Success - Conversation Cleared):**
```
Conversation cleared. Starting fresh.
```

**Response Format (Success - No Active Conversation):**
```
No active conversation to clear.
```

**Response Format (Error):**
```
Error: [error description]
```

---

### /help — `handle_help()`

**Location:** Lines 122–132

**Behavior:**
- No async operations or external calls
- Returns hardcoded help text with all six commands and brief descriptions
- Single-threaded, pure function

**Response Format:**
```
Omega Commands

/status  — Uptime, provider, database info
/memory  — Your conversation and facts stats
/history — Last 5 conversation summaries
/facts   — List known facts about you
/forget  — Clear current conversation
/help    — This message
```

---

## Helper Functions

### `format_bytes(bytes: u64) -> String`

**Location:** Lines 134–143

**Purpose:** Convert byte counts to human-readable format

**Logic:**
- `< 1024 B` → Display as bytes: `"512 B"`
- `< 1 MB` → Display as KB (1 decimal): `"2.5 KB"`
- `≥ 1 MB` → Display as MB (1 decimal): `"3.2 MB"`

**Examples:**
- `512` → `"512 B"`
- `2560` → `"2.5 KB"`
- `5242880` → `"5.0 MB"`

---

## Memory Store Integration

All command handlers interact with the `omega_memory::Store` trait/type:

| Handler | Method | Return Type | Purpose |
|---------|--------|-------------|---------|
| `handle_status()` | `store.db_size()` | `Result<u64>` | Get database file size |
| `handle_memory()` | `store.get_memory_stats(sender_id)` | `Result<(i64, i64, i64)>` | Count conversations, messages, facts |
| `handle_history()` | `store.get_history(channel, sender_id, 5)` | `Result<Vec<(String, String)>>` | Fetch last 5 conversation summaries |
| `handle_facts()` | `store.get_facts(sender_id)` | `Result<Vec<(String, String)>>` | Fetch all facts for user |
| `handle_forget()` | `store.close_current_conversation(channel, sender_id)` | `Result<bool>` | Close active conversation |

All store operations are async and return `Result` types with proper error handling.

---

## Design Patterns

### Pattern 1: Async Without External I/O

All handlers are `async` even though most only interact with local SQLite. This allows for:
- Consistent async interface with rest of Omega
- Future extensibility (e.g., remote status checks)
- Non-blocking database queries

### Pattern 2: Error Handling

- `Result` types from store methods are unwrapped with `match` expressions
- No `.unwrap()` or `.expect()` calls
- Errors formatted as-is in responses: `"Error: {e}"`
- Graceful degradation (e.g., `"unknown"` for db_size failures)

### Pattern 3: Separation of Concerns

- Parsing (`Command::parse()`) → Dispatch (`handle()`) → Execution (individual handlers)
- Each handler has single responsibility
- No business logic in enum definition

---

## Integration Points

**Called From:** `src/gateway.rs` (event loop)

**Flow:**
1. Message arrives (Telegram/WhatsApp)
2. Text is checked via `Command::parse(text)`
3. If `Some(cmd)`, invoke `Command::handle()` with parsed command
4. Send response immediately without calling AI provider
5. If `None`, pass message to provider for reasoning

---

## Notes

- Command names are hardcoded; no dynamic command registration system
- All commands are synchronous from user perspective (no long-running operations)
- Commands are scoped to user + channel (e.g., `/forget` clears only the user's conversation in that channel)
- No command aliases or variations (e.g., `/help` and `/h` are different; only `/help` is recognized)
- Fact keys and values are opaque strings managed by the memory system

