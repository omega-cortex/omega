# backend/src/api.rs — HTTP API Server

> Lightweight axum HTTP server for SaaS dashboard integration (WhatsApp QR pairing, health checks) and inbound webhook for external tool integration.

## Purpose

Provides a headless-compatible HTTP API for managing Omega from external dashboards and receiving push notifications from local utility tools. Spawned as a background task in the gateway, same pattern as scheduler/heartbeat.

## Configuration

`ApiConfig` in `backend/crates/omega-core/src/config/mod.rs`:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `false` | Enable API server |
| `host` | `String` | `"127.0.0.1"` | Bind address (localhost only by default) |
| `port` | `u16` | `3000` | Listen port |
| `api_key` | `String` | `""` | Bearer token. Empty = no auth |

## Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/api/health` | Yes | Uptime + WhatsApp connection status |
| `POST` | `/api/pair` | Yes | Trigger pairing, return QR as base64 PNG |
| `GET` | `/api/pair/status` | Yes | Long-poll (60s) for pairing completion |
| `POST` | `/api/webhook` | Yes | Inbound webhook for external tool message delivery (direct or AI mode) |

## Authentication

Bearer token via `Authorization: Bearer <token>` header. When `api_key` is empty (default), all requests are allowed without auth. `check_auth()` validates at the top of each handler.

## State

`ApiState` holds cloned references from Gateway at spawn time:
- `channels: HashMap<String, Arc<dyn Channel>>` — for WhatsApp downcast and webhook delivery
- `api_key: Option<String>` — `None` when empty config
- `uptime: Instant` — gateway start time
- `tx: Option<mpsc::Sender<IncomingMessage>>` — gateway message sender for AI mode webhook injection. `None` when API is not fully wired (tests)
- `audit: Option<AuditLogger>` — audit logger for webhook direct mode. `None` when not wired
- `channel_config: ChannelConfig` — channel configuration for default target resolution (allowed_users)

## Handler Details

### `health`
- Returns JSON: `{ status, uptime_secs, whatsapp }`
- WhatsApp status: `connected`, `disconnected`, `not_configured`, `error`

### `pair`
- Returns `already_paired` if `is_connected()` is true
- Calls `restart_for_pairing()` then `pairing_channels()`
- Waits 30s for QR via receiver, generates PNG with `generate_qr_image()`
- Returns `{ status: "qr_ready", qr_png_base64 }`

### `pair_status`
- Returns immediate `paired` if already connected
- Otherwise calls `pairing_channels()` and long-polls `done_rx` for 60s
- Returns `paired` or `pending`

### `webhook`
Accepts `WebhookRequest` JSON body with fields: `source` (required), `message` (required), `mode` (required: "direct" or "ai"), `channel` (optional), `target` (optional).

**Validation**: source non-empty, message non-empty, mode is "direct" or "ai".

**Channel/target resolution**:
- `resolve_default_channel()`: explicit channel from request, or first by priority (telegram > whatsapp)
- `resolve_default_target()`: explicit target from request, or first `allowed_users` entry from channel config

**Direct mode**: Builds `OutgoingMessage` with message text and resolved reply_target. Calls `channel.send()`. Logs audit entry. Returns 200 with `{ status: "delivered", channel, target }`.

**AI mode**: Builds synthetic `IncomingMessage` with resolved channel/sender_id/reply_target and `source` field. Sends via `tx.send()` into gateway pipeline. Returns 202 with `{ status: "queued", request_id: "<uuid>" }`.

**Error responses**: 400 (validation, channel not found, no default target), 401 (auth), 502 (channel send failure), 503 (gateway unavailable).

## Helper Functions

- `resolve_default_channel(channels, channel_config, explicit)` — resolves channel by name or priority order (telegram > whatsapp)
- `resolve_default_target(channel_config, channel_name, explicit)` — resolves target from explicit value or first allowed_user

## Data Structures

### `WebhookRequest`
```rust
pub struct WebhookRequest {
    pub source: String,    // tool identifier (e.g., "todo-app")
    pub message: String,   // text to deliver
    pub mode: String,      // "direct" or "ai"
    pub channel: Option<String>,  // explicit channel override
    pub target: Option<String>,   // explicit target override
}
```

### `WebhookResponse` (via `serde_json::json!`)
- Direct success: `{ "status": "delivered", "channel": "...", "target": "..." }`
- AI queued: `{ "status": "queued", "request_id": "..." }`
- Error: `{ "error": "..." }`

## Tests (38)

### Existing (6)
- `test_health_no_auth` — no auth configured, health returns OK
- `test_health_valid_auth` — correct bearer token accepted
- `test_health_bad_auth` — wrong token returns 401
- `test_health_missing_auth` — missing header returns 401
- `test_pair_no_whatsapp` — no WhatsApp channel returns 400
- `test_pair_status_no_whatsapp` — no WhatsApp channel returns 400

### Webhook (32)
- `test_webhook_direct_valid_request` — valid direct request returns 200 (WH-001, WH-003)
- `test_webhook_invalid_auth` — wrong bearer returns 401 (WH-002)
- `test_webhook_missing_auth` — missing header returns 401 (WH-002)
- `test_webhook_no_auth_configured` — empty api_key allows requests (WH-002)
- `test_webhook_missing_source` — missing source returns 400 (WH-004)
- `test_webhook_missing_message` — missing message returns 400 (WH-004)
- `test_webhook_missing_mode` — missing mode returns 400 (WH-004)
- `test_webhook_invalid_mode` — invalid mode returns 400 (WH-004, WH-010)
- `test_webhook_empty_message` — empty message returns 400 (WH-004, WH-010)
- `test_webhook_whitespace_message` — whitespace-only message returns 400 (WH-004, WH-010)
- `test_webhook_direct_sends_via_channel` — verifies channel.send() called with correct OutgoingMessage (WH-003)
- `test_webhook_direct_send_failure` — channel.send() error returns 502 (WH-003, WH-010)
- `test_webhook_ai_returns_202` — AI mode returns 202 with request_id UUID (WH-006, WH-011)
- `test_webhook_ai_gateway_unavailable` — dropped tx returns 503 (WH-006, WH-010)
- `test_webhook_ai_sends_incoming_message` — verifies IncomingMessage sent via tx (WH-006, WH-007)
- `test_webhook_ai_message_fields` — verifies IncomingMessage fields (WH-006, WH-007)
- `test_webhook_default_channel_telegram` — defaults to telegram (WH-005)
- `test_webhook_default_channel_whatsapp_fallback` — falls back to whatsapp (WH-005)
- `test_webhook_default_channel_priority` — telegram > whatsapp priority (WH-005)
- `test_webhook_no_channels` — no channels returns 400 (WH-005, WH-010)
- `test_webhook_no_default_target` — empty allowed_users returns 400 (WH-005, WH-010)
- `test_webhook_explicit_channel` — explicit channel overrides default (WH-004, WH-005)
- `test_webhook_explicit_channel_unknown` — unknown channel returns 400 (WH-004, WH-005)
- `test_webhook_source_on_incoming_message` — source field preserved in AI mode (WH-009)
- `test_webhook_direct_response_shape` — verifies JSON response structure (WH-008, WH-011)
- `test_webhook_invalid_json` — malformed JSON returns 400/422 (edge case)
- `test_webhook_unicode_message_accepted` — unicode/emoji preserved (edge case)
- `test_webhook_large_message_accepted` — 10KB message accepted (edge case)
- `test_webhook_dropped_receiver` — dropped rx returns 503 (edge case)
- `test_webhook_empty_source` — empty source returns 400 (edge case)
- `test_webhook_get_method_not_allowed` — GET returns 405 (edge case)
- `test_webhook_explicit_target` — explicit target overrides default (WH-004, WH-005)
