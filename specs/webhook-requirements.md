# Requirements: Inbound Webhook for Utility Tool Integration

## Scope

| File | Change | Risk |
|------|--------|------|
| `backend/src/api.rs` | Add `POST /api/webhook` handler, expand `ApiState` with `tx` sender and `audit` logger | Medium |
| `backend/src/gateway/mod.rs` | Pass `tx.clone()` and `AuditLogger` to `api::serve()` before `drop(tx)` | Low |
| `backend/crates/omega-core/src/message.rs` | Add optional `source` field to `IncomingMessage` | Low |
| `backend/config.example.toml` | Document webhook in API section comments | Low |

## Summary

External tools (TODO apps, monitoring scripts, home automation) can push messages to OMEGA via a simple HTTP POST. Two modes: "direct" sends the text straight to the user's Telegram/WhatsApp (no AI), "ai" injects the message into the full AI pipeline so OMEGA can reason about it. Authentication reuses the existing API bearer token. Everything runs on the existing axum server (port 3000, localhost only).

## User Stories

- As an external utility tool, I want to POST a notification to OMEGA so that the user sees it on Telegram/WhatsApp within seconds without polling.
- As the Omega owner, I want to receive push notifications from my local tools on my preferred messaging channel without configuring each tool separately.
- As a developer integrating a tool, I want the webhook contract to be simple enough that a `curl` one-liner works.
- As an external tool, I want to send a message through the AI pipeline so that OMEGA can reason about the data and respond intelligently to the user.

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| WH-001 | POST /api/webhook endpoint on existing axum server | Must | Endpoint accepts POST with JSON body, returns structured JSON, reuses existing `build_router` pattern |
| WH-002 | Bearer token authentication (reuse existing `check_auth`) | Must | Valid token: 200/202. Invalid/missing: 401. No auth configured: allow all |
| WH-003 | Direct delivery mode | Must | Mode "direct" sends text to channel without AI, uses `OutgoingMessage` + `channel.send()`, returns 200 with `status: "delivered"` |
| WH-004 | Request contract (JSON body) | Must | `source` (required), `message` (required), `mode` (required: "direct"/"ai"), optional `channel` and `target`. Invalid/missing: 400 |
| WH-005 | Default channel/target resolution | Must | Omitted channel: first configured (priority: telegram > whatsapp). Omitted target: first allowed_user. None available: 400 |
| WH-006 | AI pipeline mode | Must | Mode "ai" builds synthetic `IncomingMessage`, sends via `tx`, returns 202 with `status: "queued"` and `request_id`. AI response goes to messaging channel |
| WH-007 | Pass `tx.clone()` to API server | Must | `api::serve()` receives `mpsc::Sender<IncomingMessage>`. Clone before `drop(tx)`. Main loop still exits on shutdown |
| WH-008 | Audit logging for webhook deliveries | Should | Direct mode: AuditEntry with source in sender_name. AI mode: audited by pipeline. Failures: AuditStatus::Error |
| WH-009 | Source tracking on IncomingMessage | Should | Add `source: Option<String>` with `#[serde(default)]`. Non-breaking for all existing paths |
| WH-010 | Error responses | Must | Channel not found: 400. Send failure: 502. Invalid mode: 400. Empty message: 400. Gateway down: 503 |
| WH-011 | Structured response contract | Must | Direct: `{"status":"delivered","channel":"...","target":"..."}`. AI: `{"status":"queued","request_id":"..."}`. Error: `{"error":"..."}` |
| WH-012 | Pass channel config for target resolution | Should | `api::serve()` receives channel config for default target (allowed_users) |
| WH-013 | Config example documentation | Could | Add webhook usage example in `config.example.toml` `[api]` section |
| WH-014 | Rate limiting | Won't | Deferred (localhost only, single user) |
| WH-015 | Attachments in webhook payload | Won't | Deferred (text-only for MVP) |
| WH-016 | Outbound webhooks | Won't | Out of scope |
| WH-017 | Per-tool auth tokens | Won't | Single bearer token suffices |
| WH-018 | Webhook retry/queue | Won't | Tool is responsible for retrying |

## Detailed Acceptance Criteria

### WH-001: POST /api/webhook endpoint
- Given API server running, POST to `/api/webhook` with valid JSON is processed
- Given API server running, POST with invalid JSON returns 400
- Given API disabled (`enabled = false`), no webhook endpoint exists

### WH-002: Bearer token authentication
- Given `api_key = "secret"`, `Authorization: Bearer secret` proceeds
- Given `api_key = "secret"`, `Authorization: Bearer wrong` returns 401
- Given `api_key = "secret"`, no Authorization header returns 401
- Given `api_key = ""`, no Authorization header proceeds

### WH-003: Direct delivery mode
- Given mode "direct" and channel "telegram", message appears in Telegram chat
- Delivery succeeds: HTTP 200 with `status: "delivered"`
- `channel.send()` fails: HTTP 502 with error details
- Uses same `OutgoingMessage` construction as scheduler reminders

### WH-004: Request contract
- `{"source":"todo","message":"Buy milk","mode":"direct"}` accepted
- `{"source":"todo","message":"Buy milk","mode":"ai"}` accepted
- Explicit routing: `{"source":"todo","message":"...","mode":"ai","channel":"telegram","target":"842277204"}` accepted
- Missing `source`: 400. Missing `message`: 400. Invalid `mode`: 400. Empty `message`: 400

### WH-005: Default channel/target resolution
- Only Telegram configured, `channel` omitted: Telegram used
- `allowed_users = [842277204]`, `target` omitted: `"842277204"` used
- No channels configured: 400 "no channels configured"
- `allowed_users = []`, `target` omitted: 400 "no default target"

### WH-006: AI pipeline mode
- Synthetic `IncomingMessage` created with: `channel` = resolved, `sender_id` = resolved target, `reply_target` = resolved target, `source` = webhook source
- Sent via `tx.send()` into gateway's mpsc channel
- HTTP 202 with `status: "queued"` (does not wait for AI)
- AI response delivered to messaging channel, not HTTP caller

### WH-007: Pass tx.clone() to API server
- `tx.clone()` created before `drop(tx)` in `gateway/mod.rs`
- API server abort on shutdown drops the clone
- Main loop exits cleanly after all senders dropped

### WH-010: Error responses
- Channel not found: `{"error": "channel 'foo' not configured"}` (400)
- Send failure: `{"error": "delivery failed: <detail>"}` (502)
- Invalid mode: `{"error": "invalid mode 'foo', expected 'direct' or 'ai'"}` (400)
- Empty message: `{"error": "message must not be empty"}` (400)
- Gateway down: `{"error": "gateway unavailable"}` (503)

## Impact Analysis

### Existing Code Affected
- `api.rs`: `ApiState` gets 2 new fields, `serve()` adds 2 params — additive only
- `gateway/mod.rs`: Lines ~249-259 (API spawn) pass `tx.clone()` and audit — 3-line change before `drop(tx)`
- `message.rs`: `IncomingMessage` gets optional `source` field — serde default, non-breaking
- `auth.rs`: No change. Auth uses `incoming.channel` which is a real channel name

### Regression Risk
- Gateway shutdown: Verify Ctrl+C still exits when API holds `tx` clone
- Existing API tests: `test_router()` must include new `ApiState` fields
- Scheduler delivery: Not changed, but webhook mirrors the pattern

## Open Question Resolutions

| Question | Resolution | Rationale |
|----------|-----------|-----------|
| Synthetic channel vs flag | Real channel name + `source` field | `auth.rs` rejects unknown channels. `source` is additive, non-breaking |
| Default target resolution | Priority order (telegram > whatsapp) + first allowed_user | Covers 90% case. Avoids config bloat. 400 if unresolvable |
| AI mode HTTP response | 202 Accepted (async) | Pipeline is 10-60s. Holding HTTP is impractical |
| Rate limiting | Won't (deferred) | Localhost-only. mpsc backpressure sufficient for MVP |

## Traceability Matrix

| Req ID | Priority | Test IDs | Arch Section | Module |
|--------|----------|----------|-------------|--------|
| WH-001 | Must | T-WH-001, test_webhook_invalid_json_returns_400, test_webhook_get_method_returns_405 | Module 1: Webhook Handler, Handler Logic | `api.rs` |
| WH-002 | Must | T-WH-002 (test_webhook_invalid_auth_returns_401, test_webhook_missing_auth_returns_401, test_webhook_no_auth_configured_allows_all) | Module 1: Handler Logic step 1 (reuses `check_auth`) | `api.rs` |
| WH-003 | Must | T-WH-001, T-WH-006 (test_webhook_direct_valid_request_returns_200, test_webhook_direct_mode_calls_channel_send, test_webhook_direct_mode_send_failure_returns_502) | Module 1: Direct Mode OutgoingMessage Construction | `api.rs` |
| WH-004 | Must | T-WH-003, T-WH-004, T-WH-005, T-WH-011 (test_webhook_missing_source/message/mode_returns_400, test_webhook_invalid_mode_returns_400, test_webhook_empty_message_returns_400, test_webhook_empty_source_returns_400, test_webhook_explicit_channel_overrides_default) | Module 1: Data Structures (WebhookRequest) | `api.rs` |
| WH-005 | Must | T-WH-009, T-WH-010, T-WH-011 (test_webhook_default_channel_prefers_telegram, test_webhook_default_channel_falls_back_to_whatsapp, test_webhook_default_target_uses_first_allowed_user, test_webhook_no_channels_returns_400, test_webhook_no_default_target_returns_400, test_webhook_explicit_channel_overrides_default) | Module 1: Default Channel Resolution, Default Target Resolution | `api.rs` |
| WH-006 | Must | T-WH-007, T-WH-008 (test_webhook_ai_mode_returns_202_with_request_id, test_webhook_ai_mode_no_gateway_returns_503, test_webhook_ai_mode_sends_incoming_message_via_tx, test_webhook_ai_mode_sender_name_includes_source) | Module 1: AI Mode IncomingMessage Construction | `api.rs` |
| WH-007 | Must | T-WH-008 (test_webhook_ai_mode_sends_incoming_message_via_tx, test_webhook_ai_mode_dropped_receiver_returns_503) | Module 2: Gateway Plumbing, Change Location | `gateway/mod.rs` |
| WH-008 | Should | T-WH-013 (test_webhook_direct_mode_returns_correct_response_shape) | Module 1: Direct Mode (AuditEntry), Design Decisions | `api.rs` |
| WH-009 | Should | T-WH-012 (test_webhook_ai_mode_preserves_source_field) | Module 3: Source Field | `message.rs` |
| WH-010 | Must | T-WH-004, T-WH-005, T-WH-010 (test_webhook_invalid_mode_returns_400, test_webhook_empty_message_returns_400, test_webhook_no_channels_returns_400, test_webhook_direct_mode_send_failure_returns_502, test_webhook_ai_mode_no_gateway_returns_503, test_webhook_ai_mode_dropped_receiver_returns_503, test_webhook_explicit_channel_not_configured_returns_400) | Module 1: HTTP Response Contract (error codes) | `api.rs` |
| WH-011 | Must | T-WH-001, T-WH-007, T-WH-013 (test_webhook_direct_valid_request_returns_200, test_webhook_ai_mode_returns_202_with_request_id, test_webhook_direct_mode_returns_correct_response_shape) | Module 1: HTTP Response Contract (success/error shapes) | `api.rs` |
| WH-012 | Should | (Covered by T-WH-009 default resolution tests using ChannelConfig) | Module 2: Gateway Plumbing (channel_config param) | `api.rs`, `gateway/mod.rs` |
| WH-013 | Could | N/A (documentation, no code test needed) | Config Example Documentation | `config.example.toml` |

## Risks

- **AI mode silent failure**: `tx.send()` succeeds (202) but pipeline fails later. User sees error on messaging channel. Acceptable for MVP.
- **HashMap ordering**: Default channel resolution uses priority order (telegram > whatsapp), not HashMap iteration.
- **Gateway overload**: mpsc backpressure handles this for MVP.
