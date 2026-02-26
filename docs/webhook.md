# Webhook -- Inbound Push Notifications

External tools can push messages to OMEGA via `POST /api/webhook`. Two modes are supported: **direct** (bypass AI, send text straight to user) and **ai** (inject into the full AI pipeline for reasoning).

## Prerequisites

- API server enabled in `config.toml`:
  ```toml
  [api]
  enabled = true
  host = "127.0.0.1"
  port = 3000
  api_key = "your-secret-token"  # Empty = no auth
  ```
- At least one channel configured (Telegram or WhatsApp)

## Request Contract

```
POST /api/webhook
Authorization: Bearer <api_key>
Content-Type: application/json
```

### Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `source` | string | Yes | Tool identifier (e.g., "todo-app", "monitoring") |
| `message` | string | Yes | Message text (must not be empty) |
| `mode` | string | Yes | `"direct"` or `"ai"` |
| `channel` | string | No | Target channel: `"telegram"` or `"whatsapp"`. Omit for auto-detection |
| `target` | string | No | Platform-specific user ID (e.g., Telegram chat ID). Omit for first allowed_user |

### Default Resolution

When `channel` is omitted, the first configured channel is used with priority: **telegram > whatsapp**.

When `target` is omitted, the first entry in the channel's `allowed_users` list is used.

## Response Contract

### Direct Mode -- Success (200)

```json
{
  "status": "delivered",
  "channel": "telegram",
  "target": "842277204"
}
```

### AI Mode -- Success (202)

```json
{
  "status": "queued",
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
}
```

The AI response is delivered to the user's messaging channel asynchronously. The HTTP caller does not receive the AI's response.

### Error Responses

| Status | Condition | Body |
|--------|-----------|------|
| 400 | Missing or empty `source` | `{"error": "source must not be empty"}` |
| 400 | Missing or empty `message` | `{"error": "message must not be empty"}` |
| 400 | Invalid mode | `{"error": "invalid mode 'foo', expected 'direct' or 'ai'"}` |
| 400 | Channel not found | `{"error": "channel 'foo' not configured"}` |
| 400 | No channels configured | `{"error": "no channels configured"}` |
| 400 | No default target | `{"error": "no default target for channel 'telegram'"}` |
| 401 | Missing auth header | `{"error": "missing Authorization header"}` |
| 401 | Invalid token | `{"error": "invalid token"}` |
| 502 | Channel delivery failed | `{"error": "delivery failed: <detail>"}` |
| 503 | Gateway unavailable (AI mode) | `{"error": "gateway unavailable"}` |

## curl Examples

### Direct Mode -- Send a Notification

```bash
curl -X POST http://127.0.0.1:3000/api/webhook \
  -H "Authorization: Bearer your-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"source":"todo-app","message":"Buy milk","mode":"direct"}'
```

Response:
```json
{"status":"delivered","channel":"telegram","target":"842277204"}
```

### AI Mode -- Ask OMEGA to Reason About Data

```bash
curl -X POST http://127.0.0.1:3000/api/webhook \
  -H "Authorization: Bearer your-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"source":"monitoring","message":"CPU at 95% for 10 minutes on prod-1. Should I scale up?","mode":"ai"}'
```

Response:
```json
{"status":"queued","request_id":"a1b2c3d4-e5f6-7890-abcd-ef1234567890"}
```

OMEGA will reason about the alert and send its analysis to the user's Telegram/WhatsApp.

### Explicit Channel and Target

```bash
curl -X POST http://127.0.0.1:3000/api/webhook \
  -H "Authorization: Bearer your-secret-token" \
  -H "Content-Type: application/json" \
  -d '{"source":"home","message":"Front door opened","mode":"direct","channel":"whatsapp","target":"5511999887766"}'
```

### No Auth (api_key is empty in config)

```bash
curl -X POST http://127.0.0.1:3000/api/webhook \
  -H "Content-Type: application/json" \
  -d '{"source":"test","message":"Hello from webhook","mode":"direct"}'
```

## Integration Guide for Tool Developers

### Choosing a Mode

- **Direct mode** (`"direct"`): For notifications that should appear as-is. No AI processing, no delay. Think: alerts, reminders from external systems, status updates. The message text is delivered verbatim.

- **AI mode** (`"ai"`): For data that needs reasoning. OMEGA will process the message through its full AI pipeline -- context, memory, skills, and tools. The AI's response goes to the messaging channel. Think: monitoring alerts that need analysis, data summaries that need interpretation.

### Best Practices

1. **Use meaningful source names**: `"home-assistant"`, `"todo-app"`, `"monitoring-prod"`. These appear in audit logs and help OMEGA understand where messages come from.

2. **Include context in AI mode messages**: Instead of `"alert fired"`, send `"CPU alert on prod-1: 95% for 10 minutes. Historical avg is 45%. Last deployment was 2 hours ago."` -- give the AI enough data to reason.

3. **Handle 502 with retry**: If the messaging channel is temporarily unavailable, retry with exponential backoff (1s, 2s, 4s, max 30s).

4. **Don't wait for AI mode responses**: The 202 means OMEGA accepted the message. The AI response goes to the user's messaging app, not back to your HTTP client.

5. **Omit channel/target for single-user setups**: If you have one Telegram bot with one allowed user, the defaults work automatically.

### Example: Shell Script Integration

```bash
#!/bin/bash
# notify.sh -- send a notification to OMEGA
OMEGA_URL="http://127.0.0.1:3000/api/webhook"
OMEGA_TOKEN="your-secret-token"

notify() {
  local source="$1"
  local message="$2"
  local mode="${3:-direct}"

  curl -s -X POST "$OMEGA_URL" \
    -H "Authorization: Bearer $OMEGA_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"source\":\"$source\",\"message\":\"$message\",\"mode\":\"$mode\"}"
}

# Usage:
# notify "backup" "Daily backup completed successfully"
# notify "monitoring" "Disk usage at 90% on /data" "ai"
```

### Example: Python Integration

```python
import requests

OMEGA_URL = "http://127.0.0.1:3000/api/webhook"
OMEGA_TOKEN = "your-secret-token"

def notify(source: str, message: str, mode: str = "direct"):
    resp = requests.post(
        OMEGA_URL,
        headers={
            "Authorization": f"Bearer {OMEGA_TOKEN}",
            "Content-Type": "application/json",
        },
        json={"source": source, "message": message, "mode": mode},
    )
    resp.raise_for_status()
    return resp.json()

# Usage:
# notify("cron-job", "ETL pipeline completed: 1.2M rows processed")
# notify("security", "Failed SSH login from 192.168.1.100", "ai")
```

## Architecture Notes

- The webhook runs on the same axum server as `/api/health` and `/api/pair` (port 3000 by default)
- Direct mode uses the same `OutgoingMessage` pattern as the scheduler (reminders)
- AI mode injects a synthetic `IncomingMessage` into the gateway's mpsc channel -- the same path as real Telegram/WhatsApp messages
- Auth for the AI pipeline uses the real channel name and a real `allowed_user` as `sender_id`, so existing gateway auth checks pass without modification
- The `source` field on `IncomingMessage` tracks webhook origin without breaking existing message handling
- Audit logging: direct mode creates an explicit audit entry; AI mode is audited by the existing pipeline
