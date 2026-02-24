# Specification: backend/crates/omega-core/src/traits.rs

## Path
`/Users/isudoajl/ownCloud/Projects/omega/backend/crates/omega-core/src/traits.rs`

## Purpose
Defines the two foundational trait abstractions for the Omega agent: `Provider` (AI backends) and `Channel` (messaging platforms). Every AI provider and every messaging channel in the system must implement one of these traits, which together form the contract between the gateway event loop and all pluggable components.

## Dependencies

### Crate-Internal
```rust
use crate::{
    context::Context,
    error::OmegaError,
    message::{IncomingMessage, OutgoingMessage},
};
```

### External
```rust
use async_trait::async_trait;
```

The `async_trait` procedural macro is used because Rust does not natively support `async fn` in trait definitions (stabilized in Rust 1.75 but not yet compatible with dynamic dispatch via `dyn Trait`). The macro desugars async methods into methods returning `Pin<Box<dyn Future>>`.

## Trait: `Provider`

### Declaration
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn requires_api_key(&self) -> bool;
    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>;
    async fn is_available(&self) -> bool;
}
```

### Supertraits
| Supertrait | Reason |
|------------|--------|
| `Send` | Allows the provider to be sent across thread boundaries (required for `tokio::spawn`). |
| `Sync` | Allows shared references (`&self`) to be used from multiple threads concurrently. |

### Methods

| Method | Async | Signature | Return Type | Description |
|--------|-------|-----------|-------------|-------------|
| `name` | No | `fn name(&self) -> &str` | `&str` | Human-readable identifier for the provider (e.g. `"claude-code"`). Used in logging, audit entries, and metadata. |
| `requires_api_key` | No | `fn requires_api_key(&self) -> bool` | `bool` | Whether the provider needs an API key. Used by the init wizard and configuration validation. |
| `complete` | Yes | `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>` | `Result<OutgoingMessage, OmegaError>` | Core reasoning method. Accepts a full conversation `Context` (system prompt, history, current message) and returns an `OutgoingMessage` containing the response text and metadata. |
| `is_available` | Yes | `async fn is_available(&self) -> bool` | `bool` | Health check. Returns `true` if the provider is ready to accept requests. Used by self-check and startup validation. |

### Key Types

#### Input: `Context`
```rust
pub struct Context {
    pub system_prompt: String,
    pub history: Vec<ContextEntry>,
    pub current_message: String,
}
```
The `Context` struct carries the full conversation state. Providers that accept a single text input can call `context.to_prompt_string()` to flatten it into a formatted string.

#### Output: `OutgoingMessage`
```rust
pub struct OutgoingMessage {
    pub text: String,
    pub metadata: MessageMetadata,
    pub reply_target: Option<String>,
}
```
The `reply_target` field is typically set by the gateway after `complete()` returns, not by the provider itself. Providers populate `text` and `metadata`.

#### Error: `OmegaError`
Providers should return `OmegaError::Provider(String)` for all provider-specific errors.

### Known Implementations

| Struct | Crate | File |
|--------|-------|------|
| `ClaudeCodeProvider` | `omega-providers` | `backend/crates/omega-providers/src/claude_code.rs` |

### Gateway Usage Pattern
```rust
// In Gateway struct:
provider: Arc<dyn Provider>,

// Calling complete:
let response = self.provider.complete(&context).await?;

// Health check:
info!("provider: {}", self.provider.name());
```

The gateway holds the provider as `Arc<dyn Provider>`, enabling shared ownership across the main event loop and background summarizer task.

---

## Trait: `Channel`

### Declaration
```rust
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<tokio::sync::mpsc::Receiver<IncomingMessage>, OmegaError>;
    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>;
    async fn send_typing(&self, _target: &str) -> Result<(), OmegaError> {
        Ok(())
    }
    async fn send_photo(&self, _target: &str, _image: &[u8], _caption: &str) -> Result<(), OmegaError> {
        Ok(())
    }
    async fn stop(&self) -> Result<(), OmegaError>;
    fn as_any(&self) -> &dyn Any;
}
```

### Supertraits
| Supertrait | Reason |
|------------|--------|
| `Send` | Allows the channel to be sent across thread boundaries (required for `tokio::spawn`). |
| `Sync` | Allows shared references (`&self`) to be used from multiple threads (the gateway holds `Arc<dyn Channel>`). |

### Methods

| Method | Async | Has Default | Signature | Return Type | Description |
|--------|-------|-------------|-----------|-------------|-------------|
| `name` | No | No | `fn name(&self) -> &str` | `&str` | Human-readable identifier for the channel (e.g. `"telegram"`). Used in logging and routing. |
| `start` | Yes | No | `async fn start(&self) -> Result<Receiver<IncomingMessage>, OmegaError>` | `Result<mpsc::Receiver<IncomingMessage>, OmegaError>` | Starts the channel's listener (e.g. long polling, webhook). Returns an mpsc `Receiver` that yields `IncomingMessage` values. The channel internally spawns a tokio task for polling. |
| `send` | Yes | No | `async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>` | `Result<(), OmegaError>` | Sends a response back through the platform. The `OutgoingMessage.reply_target` field carries the platform-specific routing info (e.g. Telegram chat_id). |
| `send_typing` | Yes | **Yes** | `async fn send_typing(&self, _target: &str) -> Result<(), OmegaError>` | `Result<(), OmegaError>` | Sends a typing indicator. Default implementation is a no-op that returns `Ok(())`. Channels that support typing indicators override this. |
| `send_photo` | Yes | **Yes** | `async fn send_photo(&self, _target: &str, _image: &[u8], _caption: &str) -> Result<(), OmegaError>` | `Result<(), OmegaError>` | Sends a photo (PNG bytes) with a caption. Default implementation is a no-op. Used by the gateway for QR codes and workspace images. |
| `stop` | Yes | No | `async fn stop(&self) -> Result<(), OmegaError>` | `Result<(), OmegaError>` | Graceful shutdown. Called during gateway shutdown to clean up resources. |
| `as_any` | No | No | `fn as_any(&self) -> &dyn Any` | `&dyn Any` | Returns a reference to the concrete type for downcasting. Enables the gateway to access channel-specific methods (e.g., `WhatsAppChannel::pairing_channels()`). |

### Default Method
`send_typing` has a default implementation:
```rust
async fn send_typing(&self, _target: &str) -> Result<(), OmegaError> {
    Ok(())
}
```
This makes the typing indicator opt-in. Channels that do not support typing (or where it is irrelevant) need not implement it.

### Key Types

#### `IncomingMessage`
```rust
pub struct IncomingMessage {
    pub id: Uuid,
    pub channel: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<Uuid>,
    pub attachments: Vec<Attachment>,
    pub reply_target: Option<String>,
}
```

#### Error
Channels should return `OmegaError::Channel(String)` for all channel-specific errors.

### Known Implementations

| Struct | Crate | File |
|--------|-------|------|
| `TelegramChannel` | `omega-channels` | `backend/crates/omega-channels/src/telegram.rs` |

### Gateway Usage Pattern
```rust
// In Gateway struct:
channels: HashMap<String, Arc<dyn Channel>>,

// Starting all channels:
for (name, channel) in &self.channels {
    let mut channel_rx = channel.start().await?;
    // Forward messages into unified mpsc
}

// Sending responses:
if let Some(channel) = self.channels.get(&incoming.channel) {
    channel.send(response).await?;
}

// Typing indicators:
let _ = ch.send_typing(&target).await;

// Shutdown:
for (name, channel) in &self.channels {
    channel.stop().await?;
}
```

The gateway holds channels in a `HashMap<String, Arc<dyn Channel>>`, keyed by channel name. This enables runtime routing: when a response is ready, the gateway looks up the originating channel by name and calls `send()` on it.

---

## Design Notes

### Object Safety
Both traits are object-safe, which is required because the gateway uses them as trait objects (`dyn Provider`, `dyn Channel`). This means:
- No generic methods.
- No methods returning `Self`.
- All methods take `&self` (not `self` by value).

### Concurrency Model
The `Send + Sync` bounds on both traits, combined with `Arc` wrapping in the gateway, allow:
1. A single provider instance shared between the main event loop and the background summarizer task.
2. Multiple channel instances stored in a `HashMap` and accessed concurrently.
3. All async methods called from `tokio::spawn` contexts.

### Error Strategy
Both traits use `Result<_, OmegaError>` as the error type. Convention:
- `Provider` methods return `OmegaError::Provider(String)`.
- `Channel` methods return `OmegaError::Channel(String)`.
- The gateway maps these errors into user-facing messages and audit log entries.

### `async_trait` Macro
The `#[async_trait]` attribute on both trait definitions and their `impl` blocks desugars async methods into:
```rust
fn complete<'life0, 'life1, 'async_trait>(
    &'life0 self,
    context: &'life1 Context,
) -> Pin<Box<dyn Future<Output = Result<OutgoingMessage, OmegaError>> + Send + 'async_trait>>
```
This enables dynamic dispatch through `dyn Provider` / `dyn Channel` while maintaining `Send` on the returned future.

## File Statistics
- **Lines of code:** 65
- **Traits defined:** 2 (`Provider`, `Channel`)
- **Total methods:** 11 (4 on `Provider`, 7 on `Channel`)
- **Default implementations:** 2 (`Channel::send_typing`, `Channel::send_photo`)
- **External dependencies:** `async_trait`, `std::any::Any`
