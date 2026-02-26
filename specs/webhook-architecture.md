# Architecture: Inbound Webhook

## Scope

This document covers the design of `POST /api/webhook` -- an HTTP endpoint that allows external tools (TODO apps, monitoring scripts, home automation) to push messages into OMEGA. Two delivery modes: "direct" (text straight to user's messaging channel) and "ai" (inject into the full AI pipeline).

**Files modified:**
- `backend/src/api.rs` -- new handler, expanded `ApiState`, expanded `serve()` signature
- `backend/src/gateway/mod.rs` -- `tx.clone()` and audit logger wiring before `drop(tx)`
- `backend/crates/omega-core/src/message.rs` -- `source` field on `IncomingMessage`
- `backend/config.example.toml` -- webhook documentation in `[api]` section

## Overview

```
External Tool                  Omega (axum server)              Gateway Loop
     |                              |                               |
     |-- POST /api/webhook -------->|                               |
     |                              |-- check_auth() ------------->|
     |                              |                               |
     |   mode: "direct"             |-- channel.send(Outgoing) --->| (bypass AI)
     |<-- 200 {"delivered"} --------|                               |
     |                              |                               |
     |   mode: "ai"                 |-- tx.send(Incoming) -------->| (enters pipeline)
     |<-- 202 {"queued"} -----------|                               |
     |                              |                               |
     |                              |                     dispatch_message()
     |                              |                     handle_message()
     |                              |                     ... AI response ...
     |                              |                     channel.send() to user
```

## Modules

### Module 1: Webhook Handler (`api.rs`)

- **Responsibility**: Accept HTTP POST, validate request, resolve channel/target, dispatch to direct delivery or AI pipeline.
- **Public interface**: `POST /api/webhook` endpoint (added to `build_router`)
- **Dependencies**: `ApiState` (channels, tx, audit, channel_config), `check_auth()`, `OutgoingMessage`, `IncomingMessage`
- **Implementation order**: 1

#### Data Structures

**Request body** (`WebhookRequest`):
```rust
#[derive(Debug, Deserialize)]
struct WebhookRequest {
    source: String,
    message: String,
    mode: String,         // "direct" or "ai"
    channel: Option<String>,
    target: Option<String>,
}
```

**Expanded `ApiState`**:
```rust
#[derive(Clone)]
pub struct ApiState {
    channels: HashMap<String, Arc<dyn Channel>>,
    api_key: Option<String>,
    uptime: Instant,
    // --- New fields ---
    tx: Option<mpsc::Sender<IncomingMessage>>,
    audit: Option<AuditLogger>,
    channel_config: ChannelConfig,
}
```

All three new fields are `Option` to maintain backward compatibility with tests and the case where the webhook is running on an API server without gateway plumbing. When `tx` is `None` and mode is "ai", return 503.

**`serve()` signature expansion**:
```rust
pub async fn serve(
    config: ApiConfig,
    channels: HashMap<String, Arc<dyn Channel>>,
    uptime: Instant,
    // --- New params ---
    tx: mpsc::Sender<IncomingMessage>,
    audit: AuditLogger,
    channel_config: ChannelConfig,
)
```

#### Handler Logic (`webhook`)

```
POST /api/webhook
  1. check_auth() -- reuse existing function
  2. Parse JSON body into WebhookRequest
  3. Validate: source non-empty, message non-empty, mode is "direct" or "ai"
  4. Resolve channel: explicit > default (telegram > whatsapp)
  5. Resolve target: explicit > first allowed_user from channel config
  6. Verify resolved channel exists in channels HashMap
  7. Branch on mode:
     a. "direct" -> build OutgoingMessage, channel.send(), audit, return 200
     b. "ai"     -> build IncomingMessage, tx.send(), return 202
```

#### Default Channel Resolution

Priority order is explicit, not HashMap iteration:

```rust
fn resolve_default_channel(
    channels: &HashMap<String, Arc<dyn Channel>>,
) -> Option<String> {
    // Priority: telegram > whatsapp
    if channels.contains_key("telegram") {
        return Some("telegram".to_string());
    }
    if channels.contains_key("whatsapp") {
        return Some("whatsapp".to_string());
    }
    None
}
```

#### Default Target Resolution

Reads `allowed_users` from `ChannelConfig` with the same priority:

```rust
fn resolve_default_target(
    channel_name: &str,
    channel_config: &ChannelConfig,
) -> Option<String> {
    match channel_name {
        "telegram" => channel_config
            .telegram
            .as_ref()
            .and_then(|tg| tg.allowed_users.first())
            .map(|id| id.to_string()),
        "whatsapp" => channel_config
            .whatsapp
            .as_ref()
            .and_then(|wa| wa.allowed_users.first())
            .map(|s| s.clone()),
        _ => None,
    }
}
```

#### Direct Mode: OutgoingMessage Construction

Mirrors the scheduler pattern (`gateway/scheduler.rs` lines 103-107):

```rust
let msg = OutgoingMessage {
    text: request.message.clone(),
    metadata: MessageMetadata::default(),
    reply_target: Some(resolved_target.clone()),
};
channel.send(msg).await?;
```

#### AI Mode: IncomingMessage Construction

```rust
let incoming = IncomingMessage {
    id: Uuid::new_v4(),
    channel: resolved_channel.clone(),
    sender_id: resolved_target.clone(),
    sender_name: Some(format!("webhook:{}", request.source)),
    text: format!("[webhook:{}] {}", request.source, request.message),
    timestamp: Utc::now(),
    reply_to: None,
    attachments: vec![],
    reply_target: Some(resolved_target.clone()),
    is_group: false,
    source: Some(request.source.clone()),
};
```

Key design decisions for AI mode:
- `channel` is the real channel name (e.g., "telegram") so `auth.rs` recognizes it
- `sender_id` is the resolved target (an actual allowed user) so auth passes
- `sender_name` includes webhook source for tracing
- `text` is prefixed with `[webhook:source]` so the AI has context about the origin
- `source` field enables downstream differentiation without breaking existing paths

#### HTTP Response Contract

**Direct success (200)**:
```json
{"status": "delivered", "channel": "telegram", "target": "842277204"}
```

**AI queued (202)**:
```json
{"status": "queued", "request_id": "a1b2c3d4-..."}
```

**Validation errors (400)**:
```json
{"error": "message must not be empty"}
{"error": "invalid mode 'foo', expected 'direct' or 'ai'"}
{"error": "channel 'foo' not configured"}
{"error": "no channels configured"}
{"error": "no default target for channel 'telegram'"}
```

**Auth failure (401)**:
```json
{"error": "missing Authorization header"}
{"error": "invalid token"}
```

**Delivery failure (502)**:
```json
{"error": "delivery failed: connection reset"}
```

**Gateway unavailable (503)**:
```json
{"error": "gateway unavailable"}
```

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| `tx.send()` fails | Gateway receiver dropped (shutdown) | `SendError` from mpsc | Return 503 "gateway unavailable" | AI mode unavailable, direct mode unaffected |
| `channel.send()` fails | Telegram/WhatsApp API error | `OmegaError` from send | Return 502 with error detail | Direct delivery fails, caller retries |
| No channels configured | Config issue | Empty channels HashMap | Return 400 "no channels configured" | All modes unavailable |
| No allowed_users | Config has empty list | `resolve_default_target` returns None | Return 400 "no default target" | Target must be explicit |
| Invalid JSON body | Malformed request | axum deserialization error | Return 400 with parse error | Caller fixes request |
| AI pipeline fails after 202 | Provider error downstream | Audited by pipeline | Error appears on messaging channel | Webhook caller unaware (accepted design) |

#### Security Considerations

- **Trust boundary**: HTTP request body is untrusted input
- **Sensitive data**: Bearer token in Authorization header -- same security as existing API
- **Attack surface**: Message text could contain prompt injection -- mitigated by existing `sanitize()` in the AI pipeline (Phase 3.2). Direct mode sends raw text (intentional -- it's the tool's own message)
- **Mitigations**: Bearer token auth (existing `check_auth`), localhost-only binding (config default), input validation (non-empty source/message, valid mode enum), no file upload (text only)

#### Performance Budget

- **Latency target**: < 50ms p99 for direct mode (just HTTP + channel.send), < 10ms p99 for AI mode (just HTTP + mpsc send)
- **Memory budget**: Negligible -- one `WebhookRequest` struct per request, no buffering
- **Complexity target**: O(1) -- hash lookup for channel, simple branching
- **Throughput target**: Bounded by mpsc channel capacity (256) for AI mode, unbounded for direct mode

### Module 2: Gateway Plumbing (`gateway/mod.rs`)

- **Responsibility**: Wire `tx.clone()`, `AuditLogger`, and `ChannelConfig` into `api::serve()` before `drop(tx)`
- **Public interface**: None (internal wiring change)
- **Dependencies**: `mpsc::Sender<IncomingMessage>`, `AuditLogger`, `ChannelConfig`
- **Implementation order**: 2

#### Change Location

In `gateway/mod.rs`, the current code around lines 133-259:

```rust
// Line 133
let (tx, mut rx) = mpsc::channel::<IncomingMessage>(256);

// Lines 135-153: channel start loops, each gets tx.clone()

// Line 155
drop(tx);   // <--- CRITICAL: clones must happen BEFORE this

// ... background tasks ...

// Lines 249-259: API server spawn
let api_handle = if self.api_config.enabled {
    let api_cfg = self.api_config.clone();
    let api_channels = self.channels.clone();
    let api_uptime = self.uptime;
    Some(tokio::spawn(async move {
        crate::api::serve(api_cfg, api_channels, api_uptime).await;
    }))
} else {
    None
};
```

**Required change**: Move the API spawn block BEFORE `drop(tx)` and add the new params:

```rust
// Lines 135-153: channel start loops (unchanged)

// Spawn HTTP API server (BEFORE drop(tx) so we can clone the sender).
let api_handle = if self.api_config.enabled {
    let api_cfg = self.api_config.clone();
    let api_channels = self.channels.clone();
    let api_uptime = self.uptime;
    let api_tx = tx.clone();
    let api_audit = AuditLogger::new(self.memory.pool().clone());
    let api_channel_config = self.channel_config.clone();
    Some(tokio::spawn(async move {
        crate::api::serve(
            api_cfg,
            api_channels,
            api_uptime,
            api_tx,
            api_audit,
            api_channel_config,
        )
        .await;
    }))
} else {
    None
};

drop(tx);  // Now safe -- all clones are made
```

This is a 3-line addition (`api_tx`, `api_audit`, `api_channel_config`) plus moving the block up. The critical invariant is preserved: `drop(tx)` happens after all clones, so the main loop still exits when all channels close.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| `tx.clone()` before `drop(tx)` ordering violated | Code change error | Main loop never exits (hangs on shutdown) | Compile-time review, test shutdown behavior | Graceful shutdown broken |
| API holds `tx` clone after Ctrl+C | Normal behavior | API abort in `shutdown()` drops the clone | Existing `api_handle.abort()` in shutdown | No impact -- already handled |

#### Security Considerations

- No new trust boundaries introduced
- `AuditLogger` uses the same SQLite pool -- no new connections
- `ChannelConfig` is read-only, cloned from the gateway

### Module 3: Source Field (`message.rs`)

- **Responsibility**: Add optional `source` field to `IncomingMessage` for webhook origin tracking
- **Public interface**: `IncomingMessage.source: Option<String>`
- **Dependencies**: None (additive only)
- **Implementation order**: 3 (can be done in parallel with Module 1)

#### Change

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingMessage {
    // ... existing fields ...

    /// Origin identifier for webhook-injected messages.
    /// None for channel-originated messages, Some("source_name") for webhooks.
    #[serde(default)]
    pub source: Option<String>,
}
```

- `#[serde(default)]` ensures backward compatibility -- existing serialized messages without the field deserialize as `None`
- All existing code that constructs `IncomingMessage` (telegram, whatsapp) does NOT need changes because Rust requires all fields, and they will set `source: None`

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Existing deserialization breaks | Missing `#[serde(default)]` | Unit test | Add the attribute | None with correct attribute |

## Failure Modes (System-Level)

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| Gateway shutdown during webhook call | Module 1 (API handler) | `tx.send()` returns `SendError` | Return 503, caller retries | AI mode unavailable, direct mode still works |
| Telegram/WhatsApp API down | Module 1 (direct mode) | `channel.send()` returns error | Return 502, caller retries | No delivery, but clear error to caller |
| SQLite pool exhausted | Module 1 (audit), Module 2 | `AuditLogger::log()` fails | Log error, do not fail the webhook response | Delivery succeeds, audit entry missing |
| AI pipeline silent failure after 202 | Module 1 (AI mode) | Pipeline audit log shows error | Error message sent to user's messaging channel | Webhook caller unaware -- accepted for MVP |
| mpsc channel full (256 capacity) | Module 1 (AI mode) | `tx.send()` blocks briefly, then succeeds | mpsc backpressure handles this | Slight latency increase for webhook caller |

## Security Model

### Trust Boundaries

- **HTTP request -> axum handler**: Untrusted. All fields validated (non-empty, valid enum).
- **axum handler -> channel.send()**: Trusted. OutgoingMessage constructed internally.
- **axum handler -> tx.send()**: Trusted. IncomingMessage enters the same pipeline as real channel messages (auth check, sanitization, etc.).

### Data Classification

| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| Bearer token | Secret | config.toml (gitignored) | API key comparison in `check_auth()` |
| Webhook message text | Internal | Audit log (direct), conversation store (AI) | SQLite file permissions |
| Source identifier | Public | Audit log sender_name field | No restriction |
| Channel/target routing | Internal | Not persisted beyond request lifecycle | Config-derived |

### Attack Surface

- **Prompt injection via message text**: Risk: AI mode forwards text to provider -- Mitigation: existing `sanitize()` in gateway pipeline neutralizes injection patterns. Direct mode sends raw (intentional).
- **Auth bypass**: Risk: missing or weak token -- Mitigation: same `check_auth()` as all other endpoints. Default config has localhost-only binding.
- **Resource exhaustion via rapid POSTs**: Risk: fill mpsc channel or overload provider -- Mitigation: mpsc backpressure (256 buffer), localhost-only. Rate limiting deferred (WH-014).
- **Channel enumeration**: Risk: attacker probes channel names -- Mitigation: auth check before any logic. Error messages are generic for unauthorized requests.

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| Gateway main loop (tx receiver) | AI mode: 202 queued, processed async | tx dropped: 503 "gateway unavailable" | AI mode unavailable, direct still works |
| Telegram API | Direct mode: 200 delivered | API error: 502 with detail | Caller retries, user doesn't get message |
| WhatsApp connection | Direct mode: 200 delivered | Disconnected: 502 with detail | Caller retries |
| SQLite (audit) | Audit entry logged | Pool error: warning logged, response still sent | Audit gap, no user impact |

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| Webhook direct mode | < 20ms | < 50ms | < 1KB per request | Dominated by channel.send() |
| Webhook AI mode | < 5ms | < 10ms | < 1KB per request | Just mpsc send, no waiting |
| Default resolution | < 1ms | < 1ms | Zero allocation | HashMap lookup + config read |
| Auth check | < 1ms | < 1ms | Zero allocation | String comparison |

## Data Flow

### Direct Mode Sequence

```
External Tool              api.rs                 Channel (Telegram/WA)      Audit
     |                       |                           |                     |
     |-- POST /webhook ----->|                           |                     |
     |                       |-- check_auth() ---------> (pass)               |
     |                       |-- validate request -----> (ok)                 |
     |                       |-- resolve channel/target  |                     |
     |                       |                           |                     |
     |                       |-- OutgoingMessage ------->|                     |
     |                       |                           |-- send to user      |
     |                       |                           |<-- ok               |
     |                       |                           |                     |
     |                       |-- AuditEntry -------------|-------------------->|
     |                       |                           |                     |-- log
     |<-- 200 delivered -----|                           |                     |
```

### AI Mode Sequence

```
External Tool              api.rs            mpsc channel         Gateway           Channel
     |                       |                    |                  |                 |
     |-- POST /webhook ----->|                    |                  |                 |
     |                       |-- check_auth()     |                  |                 |
     |                       |-- validate req     |                  |                 |
     |                       |-- resolve ch/tgt   |                  |                 |
     |                       |                    |                  |                 |
     |                       |-- IncomingMessage->|                  |                 |
     |<-- 202 queued --------|                    |                  |                 |
     |                       |                    |-- recv() ------->|                 |
     |                       |                    |                  |-- auth check    |
     |                       |                    |                  |-- sanitize      |
     |                       |                    |                  |-- build context |
     |                       |                    |                  |-- provider call |
     |                       |                    |                  |-- process resp  |
     |                       |                    |                  |                 |
     |                       |                    |                  |-- OutgoingMsg ->|
     |                       |                    |                  |                 |-- to user
```

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| `Option` fields on `ApiState` for tx/audit/channel_config | Required fields (break tests), separate state struct | `Option` is least invasive; tests set `None`, production sets `Some`. No test rewrites needed beyond adding the fields. |
| Real channel name for AI mode (not "webhook") | Pseudo-channel "webhook" | `auth.rs` rejects unknown channels. Using the real name means the message flows through existing auth/pipeline without changes. |
| `sender_id` set to resolved target user | Synthetic webhook ID | Auth checks `sender_id` against `allowed_users`. Using a real user ID means the message passes auth. Webhook origin tracked via `source` field. |
| Text prefix `[webhook:source]` in AI mode | Separate metadata field only | The AI needs to know the message origin to reason about it. The prefix is visible in the prompt. The `source` field is machine-readable. |
| `request_id` is `Uuid::new_v4()` for AI mode | Sequential counter, timestamp hash | UUIDs are standard, collision-free, and already a dependency. |
| Move API spawn block before `drop(tx)` | Pass `tx` through `ApiState` constructor separately | Moving the block is simpler and makes the ordering constraint visible in code. |
| Priority-based default channel (telegram > whatsapp) | HashMap iteration, config option | Deterministic behavior. HashMap ordering is undefined. Adding a config option is over-engineering for single-user. |
| Audit only in direct mode | Audit both modes | AI mode is already audited by the pipeline. Double-auditing wastes storage and creates confusion. |

## External Dependencies

No new dependencies. All types and functions already exist:
- `axum` (routing, JSON extraction) -- already used in api.rs
- `uuid` (Uuid::new_v4) -- already used in message.rs
- `chrono` (Utc::now) -- already used in message.rs
- `serde` / `serde_json` -- already used throughout
- `tokio::sync::mpsc` -- already used in gateway
- `omega_memory::audit::AuditLogger` -- already used in gateway
- `omega_core::config::ChannelConfig` -- already used in gateway

## Requirement Traceability

| Requirement ID | Architecture Section | Module(s) |
|---------------|---------------------|-----------|
| WH-001 | Module 1: Webhook Handler, Handler Logic | `api.rs` |
| WH-002 | Module 1: Webhook Handler, Handler Logic step 1 | `api.rs` (reuses `check_auth`) |
| WH-003 | Module 1: Direct Mode OutgoingMessage Construction | `api.rs` |
| WH-004 | Module 1: Data Structures (WebhookRequest) | `api.rs` |
| WH-005 | Module 1: Default Channel Resolution, Default Target Resolution | `api.rs` |
| WH-006 | Module 1: AI Mode IncomingMessage Construction | `api.rs` |
| WH-007 | Module 2: Gateway Plumbing, Change Location | `gateway/mod.rs` |
| WH-008 | Module 1: Direct Mode (AuditEntry), Design Decisions (audit) | `api.rs` |
| WH-009 | Module 3: Source Field | `message.rs` |
| WH-010 | Module 1: HTTP Response Contract (error codes) | `api.rs` |
| WH-011 | Module 1: HTTP Response Contract (success/error shapes) | `api.rs` |
| WH-012 | Module 2: Gateway Plumbing (channel_config param) | `api.rs`, `gateway/mod.rs` |
| WH-013 | Config Example Documentation | `config.example.toml` |

## Test Strategy

### Unit Tests (in `api.rs` `#[cfg(test)]`)

The existing `test_router()` helper must be updated to include the new `ApiState` fields (all set to `None`):

```rust
fn test_router(api_key: Option<String>) -> Router {
    let state = ApiState {
        channels: HashMap::new(),
        api_key,
        uptime: Instant::now(),
        tx: None,
        audit: None,
        channel_config: ChannelConfig::default(),
    };
    build_router(state)
}
```

**New tests:**

| Test | Asserts |
|------|---------|
| `test_webhook_missing_source` | POST with `{}` returns 400 |
| `test_webhook_empty_message` | POST with empty message returns 400 |
| `test_webhook_invalid_mode` | POST with mode "foo" returns 400 |
| `test_webhook_no_channels` | POST with valid body but no channels returns 400 "no channels configured" |
| `test_webhook_auth_required` | POST without bearer token when api_key set returns 401 |
| `test_webhook_ai_no_gateway` | POST mode "ai" with tx=None returns 503 |

### Integration Tests (manual via curl)

See `docs/webhook.md` for curl examples covering both modes.

## Line Count Estimate

The webhook handler adds approximately:
- `WebhookRequest` struct: ~8 lines
- `resolve_default_channel()`: ~10 lines
- `resolve_default_target()`: ~15 lines
- `webhook()` handler: ~90 lines
- `build_router` route addition: 1 line
- `ApiState` expansion: 3 lines
- `serve()` param expansion: 6 lines
- New tests: ~80 lines

Total addition to `api.rs`: ~213 lines. Current file is 342 lines. New total: ~555 lines. This is over the 500-line limit (excluding tests). However, the ~80 lines of tests are excluded from the count per project rules, bringing the non-test total to ~475 lines -- within limits.

If the file exceeds 500 non-test lines during implementation, extract `resolve_default_channel()` and `resolve_default_target()` into a `webhook.rs` submodule. But this is unlikely given the estimates.
