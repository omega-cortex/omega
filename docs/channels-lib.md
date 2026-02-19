# omega-channels -- Developer Guide

## What is this crate?

`omega-channels` is the messaging platform layer of the Omega workspace. Every external service Omega can talk to -- Telegram today, WhatsApp tomorrow -- lives here as its own module. The crate root (`lib.rs`) is intentionally tiny: it just declares which modules exist and which ones are publicly visible.

## Crate structure

```
crates/omega-channels/
  Cargo.toml
  src/
    lib.rs              <-- you are here
    telegram.rs         <-- Telegram Bot API integration (complete)
    whatsapp.rs         <-- WhatsApp Web protocol integration (complete)
    whatsapp_store.rs   <-- SQLite session persistence for WhatsApp
    whisper.rs          <-- Shared OpenAI Whisper transcription
```

`lib.rs` declares the submodules:

```rust
//! # omega-channels
//!
//! Messaging platform integrations for Omega.

pub mod telegram;
pub mod whatsapp;
pub mod whatsapp_store;
pub mod whisper;
```

That is all it does. The real work happens inside each module file.

## How visibility works

- **`pub mod telegram`** -- The `telegram` module and everything marked `pub` inside it are accessible to any crate that depends on `omega-channels`. In practice, the gateway imports `omega_channels::telegram::TelegramChannel` to wire up the Telegram integration.

- **`pub mod whatsapp`** -- The `whatsapp` module implements WhatsApp Web protocol integration via `whatsapp-rust`. Features: text, image reception, voice transcription (Whisper), photo sending, group chat detection, markdown sanitization, and send retry with exponential backoff. The gateway imports `omega_channels::whatsapp::WhatsAppChannel`.

- **`pub mod whisper`** -- Shared OpenAI Whisper transcription module. Both Telegram and WhatsApp use `whisper::transcribe_whisper()` for voice message processing.

There are no `pub use` re-exports at the crate root. Consumers reach into the specific module they need (e.g., `omega_channels::telegram::TelegramChannel`).

## How a channel module works

Every channel module must provide a struct that implements the `Channel` trait from `omega-core`. Here is the trait contract:

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError>;
    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>;
    async fn send_typing(&self, target: &str) -> Result<(), OmegaError>;  // default no-op
    async fn stop(&self) -> Result<(), OmegaError>;
}
```

The `telegram.rs` module is a complete reference implementation. It:

1. Accepts a `TelegramConfig` in its constructor.
2. Spawns a long-polling loop in `start()` that yields `IncomingMessage` values through an `mpsc` channel.
3. Sends responses via the Telegram Bot API `sendMessage` endpoint.
4. Handles markdown fallback, message chunking (Telegram has a 4096-character limit), auth filtering, and exponential backoff on errors.

## How to add a new channel

Say you want to add a Discord integration. Here is the step-by-step process:

### 1. Create the module file

Create `crates/omega-channels/src/discord.rs` with a doc comment:

```rust
//! Discord channel integration.
```

### 2. Declare it in lib.rs

While it is still in development, keep it private:

```rust
mod discord;
```

Once it is ready for the gateway to use, promote it:

```rust
pub mod discord;
```

### 3. Implement the Channel trait

Your module needs a public struct that implements `omega_core::traits::Channel`. Follow the same pattern as `TelegramChannel`:

```rust
use async_trait::async_trait;
use omega_core::{
    error::OmegaError,
    message::{IncomingMessage, OutgoingMessage},
    traits::Channel,
};
use tokio::sync::mpsc;

pub struct DiscordChannel {
    // your config and state fields
}

impl DiscordChannel {
    pub fn new(/* config */) -> Self {
        // ...
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str { "discord" }

    async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError> {
        // Spawn a task that listens for messages, sends them through the channel
        todo!()
    }

    async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError> {
        // Send the response back to the user
        todo!()
    }

    async fn stop(&self) -> Result<(), OmegaError> {
        // Clean up connections
        Ok(())
    }
}
```

### 4. Add config support (if needed)

If your channel needs configuration (API tokens, allowed users, etc.), add a config struct in `omega-core/src/config.rs` following the pattern of `TelegramConfig`, then accept it in your constructor.

### 5. Wire it into the gateway

The gateway in `src/gateway.rs` is where channels are instantiated and plugged into the event loop. Add your new channel there, gated on its config being present.

## Design notes

- **No feature gates** -- All modules are compiled unconditionally. If compilation cost becomes an issue with many channels, consider adding Cargo features like `telegram` and `discord` so users can compile only what they need.

- **Auth is per-channel** -- Each channel handles its own `allowed_users` check. The Telegram module filters by Telegram user ID. Your channel should do the equivalent for its platform.

- **Errors use `OmegaError::Channel`** -- All channel errors wrap into `OmegaError::Channel(String)`. Keep error messages descriptive and include the channel name for easy log filtering.

- **Tracing, not println** -- Use `tracing::{info, warn, error, debug}` for all logging. This integrates with the structured logging infrastructure configured in main.
