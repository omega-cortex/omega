# Technical Specification: WhatsApp Channel

**File:** `crates/omega-channels/src/whatsapp.rs`
**Crate:** `omega-channels`
**Module:** `whatsapp` (private, declared as `mod whatsapp` in `lib.rs`)
**Status:** Placeholder -- no structs, no trait implementation, no logic.

---

## Overview

The WhatsApp channel module is intended to provide WhatsApp messaging integration for Omega, mirroring the role that `telegram.rs` fills for Telegram. As of the current codebase, it contains only a single-line doc comment and no executable code.

### File Contents

```rust
//! WhatsApp bridge channel (placeholder).
```

That is the entire file. No imports, no structs, no functions, no trait implementations.

---

## Required Trait: `Channel`

Defined in `crates/omega-core/src/traits.rs`, the `Channel` trait is the contract that every messaging platform must satisfy.

| Method | Signature | Default Impl | Description |
|--------|-----------|--------------|-------------|
| `name` | `fn name(&self) -> &str` | No | Returns a human-readable channel name (e.g., `"whatsapp"`). |
| `start` | `async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError>` | No | Begins listening for incoming messages. Returns an `mpsc::Receiver` that yields `IncomingMessage` values. |
| `send` | `async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>` | No | Sends a response back through the channel. |
| `send_typing` | `async fn send_typing(&self, _target: &str) -> Result<(), OmegaError>` | Yes (no-op) | Sends a typing/composing indicator. Has a default no-op implementation. |
| `stop` | `async fn stop(&self) -> Result<(), OmegaError>` | No | Performs graceful shutdown of the channel. |

The trait requires `Send + Sync` and uses `#[async_trait]`.

---

## Configuration Struct

Defined in `crates/omega-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bridge_url: String,
    #[serde(default)]
    pub phone_number: String,
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `false` | Whether the WhatsApp channel is active. |
| `bridge_url` | `String` | `""` | URL of the WhatsApp bridge/gateway server. |
| `phone_number` | `String` | `""` | Phone number associated with the WhatsApp Business account. |

The config is nested under `[channel.whatsapp]` in `config.toml`. Example from `config.example.toml`:

```toml
[channel.whatsapp]
enabled = false
bridge_url = "http://localhost:3000"
phone_number = ""
```

The `ChannelConfig` struct holds it as `pub whatsapp: Option<WhatsAppConfig>`.

---

## Expected Structs (Not Yet Implemented)

Based on the Telegram reference implementation, the WhatsApp channel would need at minimum:

| Struct | Purpose | Status |
|--------|---------|--------|
| `WhatsAppChannel` | Main struct holding config, HTTP client, and connection state. Implements `Channel`. | Not implemented |
| WhatsApp API response types | Deserialization structs for webhook payloads and API responses. | Not implemented |

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Module declaration in `lib.rs` | Done | `mod whatsapp;` (private, not `pub mod`) |
| `WhatsAppConfig` in `omega-core` | Done | Fields: `enabled`, `bridge_url`, `phone_number` |
| Config example entry | Done | `[channel.whatsapp]` section in `config.example.toml` |
| `WhatsAppChannel` struct | Not started | -- |
| `Channel` trait impl | Not started | -- |
| `Channel::name()` | Not started | -- |
| `Channel::start()` | Not started | -- |
| `Channel::send()` | Not started | -- |
| `Channel::send_typing()` | Not started | -- |
| `Channel::stop()` | Not started | -- |
| Authentication/allowed users | Not started | `WhatsAppConfig` has no `allowed_users` field yet |
| Message chunking | Not started | WhatsApp has a 4096-character limit for text messages |
| Webhook server or polling | Not started | Depends on bridge architecture chosen |
| Integration tests | Not started | -- |

---

## Message Types

The channel must produce `IncomingMessage` and consume `OutgoingMessage`, both defined in `crates/omega-core/src/message.rs`.

### IncomingMessage

| Field | Type | WhatsApp Mapping |
|-------|------|-----------------|
| `id` | `Uuid` | Generate via `Uuid::new_v4()` |
| `channel` | `String` | `"whatsapp"` |
| `sender_id` | `String` | WhatsApp phone number or user ID |
| `sender_name` | `Option<String>` | Profile name if available |
| `text` | `String` | Message body text |
| `timestamp` | `DateTime<Utc>` | Message timestamp |
| `reply_to` | `Option<Uuid>` | If replying to a previous Omega message |
| `attachments` | `Vec<Attachment>` | Media attachments (images, documents, etc.) |
| `reply_target` | `Option<String>` | Phone number or chat ID for routing the response |

### OutgoingMessage

| Field | Type | WhatsApp Mapping |
|-------|------|-----------------|
| `text` | `String` | Message body to send |
| `metadata` | `MessageMetadata` | Provider info, timing, model used |
| `reply_target` | `Option<String>` | Phone number or chat ID for routing |

---

## Module Visibility

The module is currently declared as `mod whatsapp;` (private) in `lib.rs`, unlike `telegram` which is `pub mod telegram;`. This means `WhatsAppChannel` would not be accessible outside the crate until the declaration is changed to `pub mod whatsapp;`.

---

## Dependencies Required

Based on the Telegram implementation, the WhatsApp channel will likely need:

| Dependency | Purpose |
|------------|---------|
| `reqwest` | HTTP client for API calls |
| `serde` / `serde_json` | Serialization/deserialization |
| `async_trait` | Async trait support |
| `tokio` | Async runtime, mpsc channels |
| `tracing` | Structured logging |
| `uuid` | Message ID generation |
| `chrono` | Timestamps |

All of these are already dependencies of the `omega-channels` crate.
