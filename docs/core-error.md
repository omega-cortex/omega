# Developer Guide: Error Handling in Omega

## Path
`crates/omega-core/src/error.rs`

## Overview

Omega uses a single error enum -- `OmegaError` -- for every fallible operation in the entire workspace. Whether a database query fails, a Telegram message cannot be delivered, or the Claude CLI crashes, the error that bubbles up is always an `OmegaError`. This guide explains the design philosophy, the available variants, and how you should create and propagate errors when working on the codebase.

## Philosophy

Omega follows three principles for error handling:

1. **Never panic.** Production code must not call `.unwrap()`, `.expect()`, or use any pattern that could panic. Always use the `?` operator to propagate errors or handle them gracefully.

2. **One error type for the whole workspace.** Instead of each crate defining its own error enum, every crate returns `Result<T, OmegaError>`. This keeps trait signatures simple and avoids nested error conversions.

3. **Descriptive messages at the source.** When you convert a foreign error (e.g., from SQLite, reqwest, or serde) into an `OmegaError`, include enough context in the message string that a developer or operator can diagnose the problem without a debugger.

## The OmegaError Enum

Here is the full definition:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmegaError {
    #[error("provider error: {0}")]
    Provider(String),

    #[error("channel error: {0}")]
    Channel(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("memory error: {0}")]
    Memory(String),

    #[error("sandbox error: {0}")]
    Sandbox(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

Each variant represents a domain of the system. When something goes wrong, the variant tells you *where* the problem originated, and the inner value tells you *what* happened.

## Choosing the Right Variant

Use this decision guide when you need to return an error:

| You are working in... | Use this variant | Example |
|-----------------------|------------------|---------|
| An AI provider (Claude, OpenAI, Ollama) | `Provider(String)` | CLI not found, bad API response, timeout |
| A messaging channel (Telegram, WhatsApp) | `Channel(String)` | Failed to send message, invalid chat ID, polling error |
| Configuration loading or validation | `Config(String)` | Missing file, invalid TOML, unsupported value |
| Database / memory operations | `Memory(String)` | Connection failed, query error, migration problem |
| Sandbox / command execution | `Sandbox(String)` | Command timed out, permission denied |
| Raw file I/O (and the error is `std::io::Error`) | Just use `?` | The `Io` variant is created automatically |
| JSON serialization/deserialization | Just use `?` | The `Serialization` variant is created automatically |

## How to Create Errors

### Pattern 1: `.map_err()` with format string (most common)

This is the bread-and-butter pattern. You call a fallible function, and if it fails you wrap the error into the appropriate `OmegaError` variant with a descriptive message.

```rust
use omega_core::error::OmegaError;

// In omega-memory
sqlx::query("SELECT ...")
    .fetch_all(&self.pool)
    .await
    .map_err(|e| OmegaError::Memory(format!("query failed: {e}")))?;
```

```rust
// In omega-channels
reqwest::Client::new()
    .post(&url)
    .json(&body)
    .send()
    .await
    .map_err(|e| OmegaError::Channel(format!("telegram send failed: {e}")))?;
```

```rust
// In omega-providers
let output = tokio::process::Command::new("claude")
    .args(&args)
    .output()
    .await
    .map_err(|e| OmegaError::Provider(format!("failed to run claude CLI: {e}")))?;
```

**Why format the error into a string?** Because the original error types (sqlx::Error, reqwest::Error, etc.) are foreign to omega-core. Wrapping them as strings keeps `OmegaError` independent of those dependencies.

### Pattern 2: Direct construction with `Err()`

When a condition check fails (not a foreign error), create the error directly:

```rust
if status.code() != Some(0) {
    return Err(OmegaError::Provider(format!(
        "claude exit code {}: {}",
        status.code().unwrap_or(-1),
        stderr
    )));
}
```

### Pattern 3: `.ok_or_else()` for Option-to-Result conversion

When you have an `Option` that should not be `None`:

```rust
let chat_id_str = message
    .reply_target
    .as_deref()
    .ok_or_else(|| OmegaError::Channel("no reply_target on outgoing message".into()))?;
```

### Pattern 4: Automatic `From` conversion with `?`

For `std::io::Error` and `serde_json::Error`, you do not need `.map_err()` at all. The `#[from]` attribute generates a `From` implementation, so the `?` operator converts them automatically:

```rust
// std::io::Error is automatically wrapped into OmegaError::Io
let contents = std::fs::read_to_string("some_file.txt")?;

// serde_json::Error is automatically wrapped into OmegaError::Serialization
let parsed: MyStruct = serde_json::from_str(&json_str)?;
```

Use this pattern when the bare I/O or JSON error is descriptive enough on its own and you do not need to add extra context.

## How Errors Propagate

Errors flow upward through the system, from the originating crate to the user:

```
1. A low-level operation fails (SQLite, HTTP, file I/O)
       |
2. The crate wraps it:  .map_err(|e| OmegaError::Memory(...))?
       |
3. The trait boundary passes it through:  Result<T, OmegaError>
       |
4. The gateway (src/gateway.rs) catches it, logs it, decides to retry or propagate
       |
5. main.rs converts to anyhow::Error and displays it to the user
```

At step 4, the gateway has full discretion. For some errors (like a transient network failure when sending a Telegram message), it may log and retry. For others (like a corrupt database), it may propagate and shut down.

At step 5, `anyhow` automatically uses the `Display` implementation of `OmegaError`, so the user sees the human-readable message, such as:

```
Error: memory error: failed to connect to sqlite: ...
```

## Where OmegaError Lives in the Trait System

The `Provider` and `Channel` traits in `omega-core/src/traits.rs` use `OmegaError` as their error type. This means every provider and channel implementation must return `OmegaError`:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>;
    // ...
}

#[async_trait]
pub trait Channel: Send + Sync {
    async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError>;
    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>;
    async fn stop(&self) -> Result<(), OmegaError>;
    // ...
}
```

If you are implementing a new provider or channel, your methods must return `Result<T, OmegaError>` and use the corresponding variant (`Provider` or `Channel`).

## Writing Good Error Messages

When constructing an error message string, follow these guidelines:

**Do include:**
- What operation failed: `"failed to connect to sqlite"`, `"telegram send failed"`, `"failed to run claude CLI"`
- The original error: always include the foreign error with `{e}` or `{err}` in the format string
- Relevant context: file paths, chat IDs, migration names, anything that helps diagnosis

**Do not include:**
- Stack traces (tracing handles that)
- The word "error" redundantly (the variant prefix already says "memory error:" or "provider error:")
- Sensitive data (tokens, passwords, full message contents)

**Good examples:**
```rust
OmegaError::Memory(format!("failed to create data dir: {e}"))
OmegaError::Channel(format!("invalid telegram chat_id '{target}': {e}"))
OmegaError::Config(format!("failed to read {}: {}", path.display(), e))
OmegaError::Memory(format!("migration {name} failed: {e}"))
```

**Bad examples (do not do this):**
```rust
OmegaError::Memory("error".into())                    // Too vague
OmegaError::Memory(format!("memory error: {e}"))      // Redundant "memory error"
OmegaError::Channel(format!("token={}", bot_token))   // Leaks secret
```

## Adding a New Variant

If Omega gains a new subsystem that does not fit any existing variant, you can add one to the enum. The steps:

1. Add the variant to `OmegaError` in `crates/omega-core/src/error.rs`
2. Add a `#[error("yourprefix error: {0}")]` attribute
3. Decide whether the inner type should be `String` (domain-specific, manually constructed) or a foreign error type with `#[from]`
4. Run `cargo check --workspace` to confirm compilation across all crates

Since `OmegaError` is not `#[non_exhaustive]`, adding a variant may break exhaustive `match` statements elsewhere. In practice, the codebase does not match exhaustively on `OmegaError` (errors are propagated with `?` or logged), so this is not a concern.

## Relationship to `anyhow`

The binary crate (`src/main.rs`) uses `anyhow::Result<()>` as its return type. `OmegaError` implements `std::error::Error`, so it converts to `anyhow::Error` automatically via `?`. This means:

- Library crates (`omega-core`, `omega-providers`, `omega-channels`, `omega-memory`) use `Result<T, OmegaError>`
- The binary crate (`src/main.rs`) uses `anyhow::Result<T>` and can propagate `OmegaError` with `?`
- The gateway (`src/gateway.rs`) works at the `OmegaError` boundary and decides what to do with each error

You should never use `anyhow` inside the library crates. Keep `anyhow` confined to the binary.

## Quick Reference

| Task | Pattern |
|------|---------|
| Wrap a foreign error | `.map_err(|e| OmegaError::Variant(format!("what failed: {e}")))?` |
| Fail on a condition | `return Err(OmegaError::Variant("what went wrong".into()))` |
| Convert Option to Result | `.ok_or_else(|| OmegaError::Variant("missing X".into()))?` |
| Propagate std::io::Error | `?` (automatic via `From`) |
| Propagate serde_json::Error | `?` (automatic via `From`) |
| Import the type | `use omega_core::error::OmegaError;` |

## Summary

Omega's error handling is deliberately simple: one enum, seven variants, descriptive string messages. The `?` operator handles propagation, `.map_err()` adds context, and thiserror generates the boilerplate. When in doubt, wrap the error with a clear message explaining what operation failed and include the original error. Your future self (and the tracing logs) will thank you.
