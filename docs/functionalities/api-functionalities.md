# Functionalities: HTTP API

## Overview

Lightweight HTTP API server built with axum for SaaS dashboard integration. Provides health checks, WhatsApp QR pairing, and inbound webhooks.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | serve() | Service | `backend/src/api.rs` | Starts axum HTTP server on configured host:port with shared state | axum, ApiConfig |
| 2 | GET /api/health | Endpoint | `backend/src/api.rs:118` | Health check returning uptime, channel status, WhatsApp connection state | ApiState |
| 3 | POST /api/pair | Endpoint | `backend/src/api.rs:149` | Triggers WhatsApp pairing, returns QR as base64 PNG, waits for completion | WhatsAppChannel |
| 4 | POST /api/webhook | Endpoint | `backend/src/api.rs` | Inbound message injection with two modes: inject (into gateway pipeline) and forward (direct channel send) | IncomingMessage, Channel |
| 5 | constant_time_eq() | Utility | `backend/src/api.rs:54` | Constant-time string comparison for API token validation (prevents timing attacks) | -- |
| 6 | check_auth() | Utility | `backend/src/api.rs:69` | Bearer token authentication for API endpoints | -- |
| 7 | ApiState | Model | `backend/src/api.rs:31` | Shared state: channels, api_key, uptime, tx (gateway sender), audit, channel_config | -- |
| 8 | WebhookRequest | Model | `backend/src/api.rs:41` | Webhook request body: source, message, mode (inject/forward), channel, target | -- |

## Internal Dependencies

- API server spawned by Gateway::run() as background task
- Inject mode sends IncomingMessage to gateway via mpsc channel
- Forward mode sends directly via Channel::send()
- Uses AuditLogger for webhook audit trails

## Dead Code / Unused

- None detected.
