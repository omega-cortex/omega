# backend/src/api.rs — HTTP API Server

> Lightweight axum HTTP server for SaaS dashboard integration (WhatsApp QR pairing, health checks).

## Purpose

Provides a headless-compatible HTTP API for managing Omega from external dashboards. Spawned as a background task in the gateway, same pattern as scheduler/heartbeat.

## Configuration

`ApiConfig` in `backend/crates/omega-core/src/config.rs`:

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

## Authentication

Bearer token via `Authorization: Bearer <token>` header. When `api_key` is empty (default), all requests are allowed without auth. `check_auth()` validates at the top of each handler.

## State

`ApiState` holds cloned references from Gateway at spawn time:
- `channels: HashMap<String, Arc<dyn Channel>>` — for WhatsApp downcast
- `api_key: Option<String>` — `None` when empty config
- `uptime: Instant` — gateway start time

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

## Tests (6)

- `test_health_no_auth` — no auth configured, health returns OK
- `test_health_valid_auth` — correct bearer token accepted
- `test_health_bad_auth` — wrong token returns 401
- `test_health_missing_auth` — missing header returns 401
- `test_pair_no_whatsapp` — no WhatsApp channel returns 400
- `test_pair_status_no_whatsapp` — no WhatsApp channel returns 400
