# Core Traits: Provider and Channel

## Path
`/Users/isudoajl/ownCloud/Projects/omega/backend/crates/omega-core/src/traits.rs`

## Overview

Omega's architecture is built around two central traits that define the contract between the gateway event loop and all pluggable components:

- **`Provider`** -- the brain. Any AI backend that can take a conversation context and return a response.
- **`Channel`** -- the nervous system. Any messaging platform that can receive messages from users and send responses back.

These traits live in `omega-core` so that every crate in the workspace can depend on them without circular dependencies. The gateway in `backend/src/gateway.rs` consumes both traits as trait objects (`Arc<dyn Provider>` and `Arc<dyn Channel>`), meaning your implementations are wired in at runtime -- no generics, no monomorphization, just clean dynamic dispatch.

```
                       omega-core
                     ┌────────────────┐
                     │  trait Provider │
                     │  trait Channel  │
                     └──────┬─────────┘
                            │
              ┌─────────────┼─────────────┐
              v                           v
     omega-providers              omega-channels
  ┌────────────────────┐    ┌────────────────────────┐
  │ ClaudeCodeProvider │    │   TelegramChannel      │
  │ (future: Anthropic │    │   (future: WhatsApp    │
  │  OpenAI, Ollama)   │    │    Discord, etc.)      │
  └────────────────────┘    └────────────────────────┘
              │                           │
              └─────────────┬─────────────┘
                            v
                     ┌─────────────┐
                     │   Gateway   │
                     └─────────────┘
```

## The `Provider` Trait

A Provider is anything that can take a conversation context and produce a response. Think of it as a uniform wrapper around any AI model, whether it is a local CLI tool, a REST API, or a locally-hosted model.

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Whether this provider requires an API key to function.
    fn requires_api_key(&self) -> bool;

    /// Send a conversation context to the provider and get a response.
    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>;

    /// Check if the provider is available and ready.
    async fn is_available(&self) -> bool;
}
```

### What Each Method Means

**`name()`** returns a short, stable identifier like `"claude-code"` or `"openai"`. The gateway uses this for logging, audit records, and the `/status` bot command. Pick a name that is lowercase, hyphenated, and unlikely to collide with other providers.

**`requires_api_key()`** tells the configuration system whether it needs to prompt for or validate an API key. The Claude Code CLI provider returns `false` here because it piggybacks on the user's existing `claude` authentication. An Anthropic API provider would return `true`.

**`complete(context)`** is the core method. It receives a `Context` struct containing:
- A system prompt
- Conversation history (a list of user/assistant message pairs)
- The current user message

Your job is to send this to your AI backend, wait for a response, and return an `OutgoingMessage`. The outgoing message must include:
- The response `text`
- A `MessageMetadata` struct with `provider_used`, optional `tokens_used`, `processing_time_ms`, and optional `model`

You do **not** need to set `reply_target` on the outgoing message -- the gateway handles routing after your method returns.

**`is_available()`** is a health check. The self-check system and startup validation call this to verify the provider is operational before entering the event loop. For a CLI-based provider, this might check that the binary exists. For an API provider, this might make a lightweight test request.

### Implementing a New Provider

Here is a skeleton for a hypothetical OpenAI provider:

```rust
use async_trait::async_trait;
use omega_core::{
    context::Context,
    error::OmegaError,
    message::{MessageMetadata, OutgoingMessage},
    traits::Provider,
};
use std::time::Instant;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError> {
        let start = Instant::now();

        // Build the API request from context.system_prompt,
        // context.history, and context.current_message.
        // Make the HTTP call, parse the response.

        let response_text = call_openai_api(
            &self.api_key,
            &self.model,
            context,
        ).await.map_err(|e| OmegaError::Provider(format!("openai: {e}")))?;

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(OutgoingMessage {
            text: response_text,
            metadata: MessageMetadata {
                provider_used: "openai".to_string(),
                tokens_used: None, // fill in if the API returns token counts
                processing_time_ms: elapsed,
                model: Some(self.model.clone()),
            },
            reply_target: None, // the gateway sets this
        })
    }

    async fn is_available(&self) -> bool {
        // Could make a lightweight /models request to verify the key works.
        !self.api_key.is_empty()
    }
}
```

### Important Rules for Providers

1. **Always use `OmegaError::Provider(String)` for errors.** The gateway expects this variant and maps it into user-facing error messages and audit log entries.

2. **Never panic.** Use `?` and proper error mapping. The `OmegaError` type is `thiserror`-based and supports `From` conversions for `std::io::Error` and `serde_json::Error`.

3. **Track processing time.** Start a timer at the beginning of `complete()` and include elapsed milliseconds in `MessageMetadata`. The audit system records this.

4. **Use tracing, not println.** Import from the `tracing` crate (`debug!`, `warn!`, `error!`).

5. **The `Context::to_prompt_string()` helper** is available for providers that take a single text input (like the Claude Code CLI). API-based providers that support structured message arrays should read `context.history` and `context.current_message` directly.

## The `Channel` Trait

A Channel is anything that can receive messages from users and send responses back. It abstracts over the specifics of each messaging platform -- polling vs. webhooks, authentication, message formatting, rate limits, etc.

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    /// Human-readable channel name.
    fn name(&self) -> &str;

    /// Start listening for incoming messages.
    async fn start(&self) -> Result<tokio::sync::mpsc::Receiver<IncomingMessage>, OmegaError>;

    /// Send a response back through this channel.
    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>;

    /// Send a typing indicator (optional, default is no-op).
    async fn send_typing(&self, _target: &str) -> Result<(), OmegaError> {
        Ok(())
    }

    /// Graceful shutdown.
    async fn stop(&self) -> Result<(), OmegaError>;
}
```

### What Each Method Means

**`name()`** works the same as on `Provider` -- a short, stable identifier like `"telegram"` or `"whatsapp"`. The gateway uses this as the key in its channel map and for routing responses back to the correct platform.

**`start()`** is called once during gateway startup. It must:
1. Create a `tokio::sync::mpsc::channel` (suggested buffer size: 64).
2. Spawn a background task that listens for messages on the platform (long polling, websocket, webhook server, etc.).
3. Convert each platform-native message into an `IncomingMessage` and send it through the channel's `tx` end.
4. Return the `rx` end to the gateway.

The gateway then forwards all received messages into a unified mpsc queue, regardless of which channel they came from.

**`send(message)`** delivers a response back to the user. The `OutgoingMessage` will have its `reply_target` field set by the gateway -- this carries whatever platform-specific routing information you need (e.g., a Telegram chat ID, a WhatsApp phone number). Your implementation should read `reply_target`, parse it into whatever your platform expects, and deliver the message.

**`send_typing(target)`** is optional. The default implementation is a no-op. If your platform supports typing indicators (like Telegram's "typing..." status), override this method. The gateway calls it before invoking the AI provider and repeats it every 5 seconds until the response is ready. The `target` parameter is the same platform-specific routing identifier from `reply_target`.

**`stop()`** is called during graceful shutdown. Use it to clean up resources -- cancel polling loops, close connections, flush pending messages.

### Implementing a New Channel

Here is a skeleton for a hypothetical WhatsApp channel:

```rust
use async_trait::async_trait;
use omega_core::{
    error::OmegaError,
    message::{IncomingMessage, OutgoingMessage},
    traits::Channel,
};
use tokio::sync::mpsc;

pub struct WhatsAppChannel {
    phone_number_id: String,
    access_token: String,
    client: reqwest::Client,
}

#[async_trait]
impl Channel for WhatsAppChannel {
    fn name(&self) -> &str {
        "whatsapp"
    }

    async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError> {
        let (tx, rx) = mpsc::channel(64);

        // Clone what the background task needs.
        let client = self.client.clone();
        let token = self.access_token.clone();

        tokio::spawn(async move {
            // Set up a webhook listener or poll the WhatsApp Business API.
            // For each incoming message:
            //   1. Parse the platform-native payload
            //   2. Convert to IncomingMessage (set reply_target to the sender's phone)
            //   3. tx.send(incoming).await
            //
            // If tx.send fails, the receiver was dropped -- break the loop.
        });

        Ok(rx)
    }

    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError> {
        let phone = message.reply_target.as_deref()
            .ok_or_else(|| OmegaError::Channel("no reply_target".into()))?;

        // Send via WhatsApp Business API using self.client.
        send_whatsapp_message(&self.client, &self.access_token, phone, &message.text)
            .await
            .map_err(|e| OmegaError::Channel(format!("whatsapp send: {e}")))?;

        Ok(())
    }

    async fn send_typing(&self, target: &str) -> Result<(), OmegaError> {
        // WhatsApp supports read receipts but not typing indicators
        // in the Business API, so we leave this as a no-op or send
        // a "mark as read" action.
        Ok(())
    }

    async fn stop(&self) -> Result<(), OmegaError> {
        tracing::info!("WhatsApp channel stopped");
        Ok(())
    }
}
```

### Important Rules for Channels

1. **Always use `OmegaError::Channel(String)` for errors.**

2. **Set `reply_target` on every `IncomingMessage`.** The gateway copies this field onto the `OutgoingMessage` before calling `send()`. Without it, the response cannot be routed back.

3. **Handle auth in the polling loop.** The `TelegramChannel` implementation checks `allowed_users` before forwarding messages to the gateway. This is a good pattern -- reject unauthorized messages early and log a warning.

4. **Use exponential backoff for polling errors.** Network failures are inevitable. The Telegram implementation uses a backoff that doubles from 1 second up to 60 seconds, resetting on success.

5. **Respect platform rate limits.** Telegram limits messages to 4096 characters; the `TelegramChannel` splits long messages automatically. Each platform will have its own constraints.

6. **Populate `IncomingMessage` fully.** At minimum: `id` (use `Uuid::new_v4()`), `channel` (your channel name), `sender_id`, `text`, `timestamp` (use `chrono::Utc::now()`), and `reply_target`. The `sender_name`, `reply_to`, and `attachments` fields are optional but valuable for the memory system.

## How the Gateway Consumes These Traits

The gateway holds both traits as trait objects behind `Arc`:

```rust
pub struct Gateway {
    provider: Arc<dyn Provider>,
    channels: HashMap<String, Arc<dyn Channel>>,
    // ...
}
```

The message processing pipeline is:

1. **Start channels** -- call `channel.start()` for each channel, spawn forwarders into a unified mpsc queue.
2. **Receive message** -- `rx.recv()` yields the next `IncomingMessage` from any channel.
3. **Auth check** -- verify the sender is authorized for this channel.
4. **Sanitize** -- neutralize prompt injection patterns in the message text.
5. **Command check** -- intercept bot commands like `/status` and `/history`.
6. **Typing indicator** -- call `channel.send_typing(&target)`, repeat every 5 seconds.
7. **Build context** -- load conversation history and user facts from memory.
8. **Provider complete** -- call `provider.complete(&context)` to get the AI response.
9. **Store exchange** -- persist the user message and assistant response in memory.
10. **Audit log** -- record the full interaction with metadata.
11. **Send response** -- call `channel.send(response)` to deliver the reply.
12. **Shutdown** -- call `channel.stop()` for each channel during graceful shutdown.

This pipeline is the same regardless of which provider or channel is active. That is the power of the trait abstraction: the gateway does not know or care whether it is talking to Claude Code or OpenAI, Telegram or WhatsApp.

## The `Send + Sync` Requirement

Both traits require `Send + Sync`. This is not optional -- it is a hard requirement imposed by the async runtime:

- **`Send`** is needed because the gateway passes `Arc<dyn Provider>` and `Arc<dyn Channel>` into `tokio::spawn`, which requires the future (and everything it captures) to be `Send`.
- **`Sync`** is needed because `Arc<T>` only implements `Send` when `T: Send + Sync`. Since the gateway shares providers and channels via `Arc`, the inner type must be `Sync`.

In practice, this means your implementation struct must not contain `Rc`, `Cell`, `RefCell`, or other non-thread-safe types. Use `Arc<Mutex<_>>` or `Arc<RwLock<_>>` from `tokio::sync` if you need interior mutability.

## The `async_trait` Macro

Both traits use the `#[async_trait]` macro from the `async_trait` crate. This is necessary because `async fn` in traits cannot yet be used with dynamic dispatch (`dyn Trait`). The macro rewrites each async method into a regular method returning `Pin<Box<dyn Future + Send>>`.

When you implement the trait, you must also annotate your `impl` block with `#[async_trait]`:

```rust
#[async_trait]
impl Provider for MyProvider {
    // async fn works normally here thanks to the macro
    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError> {
        // ...
    }
}
```

## Checklist: Adding a New Provider

1. Create a new file in `backend/crates/omega-providers/src/` (e.g., `openai.rs`).
2. Define your struct with whatever configuration it needs.
3. Implement `Provider` with `#[async_trait]`.
4. Return `OmegaError::Provider(...)` for all errors.
5. Track `processing_time_ms` in `complete()`.
6. Expose the struct from `backend/crates/omega-providers/src/lib.rs`.
7. Wire it into `backend/src/main.rs` based on config selection.
8. Add tests: at minimum, test `name()`, `requires_api_key()`, and a mock `complete()`.

## Checklist: Adding a New Channel

1. Create a new file in `backend/crates/omega-channels/src/` (e.g., `whatsapp.rs`).
2. Define your struct with platform-specific config.
3. Implement `Channel` with `#[async_trait]`.
4. In `start()`: create an mpsc channel, spawn a listener task, return the receiver.
5. In `send()`: read `reply_target` from the outgoing message and deliver it.
6. Override `send_typing()` if the platform supports it.
7. In `stop()`: clean up resources.
8. Return `OmegaError::Channel(...)` for all errors.
9. Set `reply_target` on every `IncomingMessage` you produce.
10. Expose the struct from `backend/crates/omega-channels/src/lib.rs`.
11. Wire it into `backend/src/main.rs` and `backend/src/gateway.rs`.
12. Add tests: at minimum, test `name()` and message splitting/formatting logic.
