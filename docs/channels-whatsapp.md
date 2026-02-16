# WhatsApp Channel -- Developer Documentation

## Current Status

The WhatsApp channel is a **placeholder**. The file `crates/omega-channels/src/whatsapp.rs` contains only a module-level doc comment:

```rust
//! WhatsApp bridge channel (placeholder).
```

There is no struct, no trait implementation, and no logic yet. However, the surrounding infrastructure is already in place:

- The module is declared in `crates/omega-channels/src/lib.rs` as `mod whatsapp;`.
- A `WhatsAppConfig` struct exists in `crates/omega-core/src/config.rs` with `enabled`, `bridge_url`, and `phone_number` fields.
- A `[channel.whatsapp]` section is present in `config.example.toml`.

This is a Phase 4 task on the Omega roadmap.

---

## What the Channel Trait Requires

Every messaging platform in Omega must implement the `Channel` trait (defined in `crates/omega-core/src/traits.rs`). Here is what a WhatsApp implementation needs to provide:

### `fn name(&self) -> &str`

Return `"whatsapp"`. This string is used to identify the channel in logs, the gateway routing table, and message metadata.

### `async fn start(&self) -> Result<Receiver<IncomingMessage>, OmegaError>`

This is the heart of the channel. It must set up a mechanism to receive incoming WhatsApp messages and feed them into a `tokio::sync::mpsc` channel. The gateway calls this once at startup and then reads from the returned receiver in a loop.

For Telegram, this is done via long polling against the Bot API. For WhatsApp, the approach depends on the integration method chosen (see the section on architecture choices below).

### `async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>`

Send a text response back to the user. The `OutgoingMessage` includes a `reply_target` field that should contain whatever identifier is needed to route the message (e.g., the recipient's phone number or a chat ID from the bridge).

### `async fn send_typing(&self, target: &str) -> Result<(), OmegaError>`

Send a typing or "composing" indicator. The WhatsApp Business API supports this via the `/messages` endpoint with `type: "reaction"` or status read receipts. A bridge may have its own mechanism. This method has a default no-op implementation, so it is optional to override.

### `async fn stop(&self) -> Result<(), OmegaError>`

Graceful shutdown. Close any webhook server, clean up connections, log that the channel has stopped.

---

## Architecture Choices

The `WhatsAppConfig` has a `bridge_url` field, which suggests the original design intent was to use a **bridge** rather than the WhatsApp Business API directly. There are several viable approaches:

### Option A: WhatsApp Business API (Cloud API)

Meta's official Cloud API. Requires a Meta Business account, a verified phone number, and a webhook endpoint for incoming messages.

- **Pros:** Official, well-documented, reliable, supports all message types.
- **Cons:** Requires a publicly reachable webhook URL (or a tunnel like ngrok), Meta business verification, and costs money for conversation-based pricing.
- **Incoming messages:** Received via webhook POST to your server.
- **Outgoing messages:** Sent via `POST https://graph.facebook.com/v18.0/{phone_number_id}/messages`.
- **Auth token:** A long-lived system user token or a temporary token from the Meta dashboard.

### Option B: WhatsApp Bridge (e.g., whatsapp-web.js, Baileys, or a self-hosted bridge)

A bridge server that connects to WhatsApp Web and exposes a local HTTP API.

- **Pros:** No Meta business account needed, works with a personal WhatsApp number, no webhook URL required.
- **Cons:** Unofficial, may break when WhatsApp updates their protocol, potential ToS violations, requires maintaining the bridge process.
- **Incoming messages:** Polled or pushed from the bridge's API (depends on the bridge implementation).
- **Outgoing messages:** Sent via the bridge's HTTP API.

### Option C: Hybrid

Use the bridge for development and testing, and the official Business API for production. The `bridge_url` config field can serve as the base URL for either backend.

---

## Implementation Guide

Here is a step-by-step outline for implementing the WhatsApp channel:

### 1. Define the struct

```rust
pub struct WhatsAppChannel {
    config: WhatsAppConfig,
    client: reqwest::Client,
}
```

### 2. Add a constructor

```rust
impl WhatsAppChannel {
    pub fn new(config: WhatsAppConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }
}
```

### 3. Implement the Channel trait

Follow the Telegram implementation in `crates/omega-channels/src/telegram.rs` as a reference. The key differences will be:

- **`start()`**: Instead of long-polling the Telegram Bot API, you will either:
  - Start an HTTP server to receive webhooks (for the Business API), or
  - Poll a bridge's API endpoint for new messages.
- **`send()`**: POST to the WhatsApp API or bridge to deliver the response.
- **`send_typing()`**: Send a composing presence indicator if the API/bridge supports it.

### 4. Expand the config

The current `WhatsAppConfig` is minimal. You will likely need to add:

| Field | Purpose |
|-------|---------|
| `api_token` | Authentication token for the Business API or bridge |
| `phone_number_id` | The WhatsApp Business phone number ID (for Cloud API) |
| `verify_token` | Webhook verification token (for Cloud API) |
| `allowed_users` | List of allowed phone numbers for auth enforcement |
| `webhook_port` | Local port for the webhook server |

### 5. Make the module public

Change `mod whatsapp;` to `pub mod whatsapp;` in `crates/omega-channels/src/lib.rs` so the gateway can access `WhatsAppChannel`.

### 6. Wire it into the gateway

In `src/gateway.rs`, add logic to instantiate `WhatsAppChannel` when `config.channel.whatsapp` is `Some(cfg)` and `cfg.enabled` is `true`, then register it alongside the Telegram channel.

### 7. Handle message formatting

WhatsApp has a 4096-character limit for text messages. You will need a `split_message()` function similar to the one in the Telegram channel. WhatsApp also supports its own flavor of formatting (bold with `*`, italic with `_`, strikethrough with `~`, monospace with `` ``` ``).

### 8. Add authentication

The Telegram channel filters messages by `allowed_users` (a list of Telegram user IDs). The WhatsApp equivalent would be a list of allowed phone numbers. Add an `allowed_users: Vec<String>` field to `WhatsAppConfig` and filter incoming messages in `start()`.

---

## WhatsApp Business API Integration Points

If using the official Cloud API, these are the key endpoints:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/v18.0/{phone_number_id}/messages` | POST | Send a text message, media, or template |
| `/v18.0/{phone_number_id}/messages` | POST | Send typing indicator (via `messaging_product: "whatsapp"`, `status: "read"`) |
| Webhook URL (your server) | POST | Receive incoming messages and status updates |
| Webhook URL (your server) | GET | Webhook verification handshake from Meta |

### Webhook Payload Structure (Incoming Message)

```json
{
  "object": "whatsapp_business_account",
  "entry": [{
    "id": "BUSINESS_ACCOUNT_ID",
    "changes": [{
      "value": {
        "messaging_product": "whatsapp",
        "metadata": {
          "display_phone_number": "15550001234",
          "phone_number_id": "PHONE_NUMBER_ID"
        },
        "contacts": [{
          "profile": { "name": "Sender Name" },
          "wa_id": "15550005678"
        }],
        "messages": [{
          "from": "15550005678",
          "id": "wamid.xxx",
          "timestamp": "1677000000",
          "text": { "body": "Hello Omega" },
          "type": "text"
        }]
      },
      "field": "messages"
    }]
  }]
}
```

### Send Message Payload

```json
{
  "messaging_product": "whatsapp",
  "to": "15550005678",
  "type": "text",
  "text": { "body": "Response from Omega" }
}
```

---

## Testing Strategy

- **Unit tests:** Test message parsing, chunking, and config loading.
- **Integration tests:** Use a mock HTTP server (e.g., `wiremock`) to simulate the WhatsApp API or bridge.
- **Manual testing:** Use a bridge in development, or the Meta test phone number for the Business API.

---

## Reference

- Telegram channel implementation: `crates/omega-channels/src/telegram.rs`
- Channel trait definition: `crates/omega-core/src/traits.rs`
- Config struct: `crates/omega-core/src/config.rs` (`WhatsAppConfig`)
- Message types: `crates/omega-core/src/message.rs`
- Example config: `config.example.toml`
- WhatsApp Cloud API docs: https://developers.facebook.com/docs/whatsapp/cloud-api
