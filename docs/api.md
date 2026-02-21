# HTTP API

Omega includes a lightweight HTTP API server for SaaS dashboard integration. It allows external systems to trigger WhatsApp QR pairing and monitor health — useful for headless cloud deployments where the terminal-based `omega pair` command isn't available.

## Configuration

Add to `config.toml`:

```toml
[api]
enabled = true
host = "127.0.0.1"    # Localhost only by default
port = 3000
api_key = ""           # Empty = no auth. Set a token for production.
```

## Security

- **Localhost only** by default — external access via reverse proxy (nginx/caddy adds TLS)
- **Bearer token auth** when `api_key` is set — empty means no auth (safe for local-only use)
- Dashboard backend calls the API server-to-server, never exposed to browser directly

## Endpoints

### `GET /api/health`

Health check with uptime and WhatsApp connection status.

```bash
curl http://localhost:3000/api/health
```

Response:
```json
{
  "status": "ok",
  "uptime_secs": 3600,
  "whatsapp": "connected"
}
```

WhatsApp status values: `connected`, `disconnected`, `not_configured`.

### `POST /api/pair`

Trigger WhatsApp pairing. Returns the QR code as a base64-encoded PNG image.

```bash
curl -X POST http://localhost:3000/api/pair
```

Response (QR ready):
```json
{
  "status": "qr_ready",
  "qr_png_base64": "iVBORw0KGgo..."
}
```

Response (already paired):
```json
{
  "status": "already_paired",
  "message": "WhatsApp is already connected"
}
```

### `GET /api/pair/status`

Long-poll (up to 60s) for pairing completion. Use after `/api/pair` to wait for the user to scan the QR code.

```bash
curl http://localhost:3000/api/pair/status
```

Response (success):
```json
{
  "status": "paired",
  "message": "WhatsApp pairing completed"
}
```

Response (timeout):
```json
{
  "status": "pending",
  "message": "Pairing not yet completed"
}
```

## Authentication

When `api_key` is set, all requests require the `Authorization` header:

```bash
curl -H "Authorization: Bearer your-secret-key" http://localhost:3000/api/health
```

Missing or invalid tokens return `401 Unauthorized`.

## Dashboard Integration Flow

1. Dashboard backend calls `POST /api/pair`
2. If `status: "qr_ready"`, decode `qr_png_base64` and display in browser
3. Poll `GET /api/pair/status` until `status: "paired"` or timeout
4. Use `GET /api/health` for ongoing status monitoring
