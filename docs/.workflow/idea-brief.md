# Idea Brief: Inbound Webhook for Utility Tool Integration

## One-Line Summary
A local HTTP endpoint on the existing API server that lets external utility tools push notifications and instructions to OMEGA for delivery via messaging channels.

## Problem Statement
OMEGA currently relies on scheduled action tasks (polling) to interact with external tools. A TODO app, CRM, or monitoring tool must either be polled on an interval or shoehorned into the scheduler. This is wasteful: a tool that detects "meeting in 5 minutes" should push that notification immediately, not wait for the next 60-second scheduler poll cycle. The push model is both more efficient and more timely.

## Current State
Today, external tools integrate with OMEGA in three ways -- all with limitations:

1. **Scheduler injection** -- Insert a row into `scheduled_tasks` SQLite table with `task_type = 'action'`. Works, but requires direct SQLite access (fragile, no auth, no standard contract) and still waits for the scheduler poll interval.
2. **Skills** -- Teach the AI about a tool via `SKILL.md`. But skills are AI-initiated (the AI calls the tool), not tool-initiated.
3. **HTTP API** -- Axum server on port 3000 exists but only serves health checks and WhatsApp QR pairing. No general-purpose message ingestion.

None of these give an external tool a clean, authenticated, standardized way to say: "Deliver this message to the user right now."

## Proposed Solution
Add a `POST /api/webhook` endpoint to the existing axum API server (`backend/src/api.rs`). External tools POST a JSON payload following a standard contract. OMEGA delivers the message via the configured channel (Telegram/WhatsApp).

Two delivery modes:
- **"direct"** -- Pass-through: the message text is sent directly to the user via the channel. No AI involved. Fast, cheap, predictable.
- **"ai"** -- Pipeline: the message is injected into the gateway's message pipeline as a synthetic `IncomingMessage`, processed by the AI (with full context, tools, markers), and the AI's response is delivered. Useful when the tool wants OMEGA to reason about the data.

## Target Users
- **Primary**: External utility tools (TODO apps, CRM systems, monitoring scripts, home automation) running as background processes on the same machine.
- **Secondary**: The Omega owner (the human) -- they receive the notifications on Telegram/WhatsApp.

## Success Criteria
- A utility tool running locally can POST to `http://127.0.0.1:3000/api/webhook` and the user sees the message on Telegram within seconds.
- In "direct" mode, delivery is near-instant (no AI latency).
- In "ai" mode, the message goes through the full gateway pipeline as if the user had typed it.
- Failed deliveries return clear HTTP error responses (not silent failures).
- The contract is simple enough that a bash script with `curl` can use it.

## MVP Scope
1. **Single endpoint**: `POST /api/webhook` on the existing axum server.
2. **Standard contract**: JSON body with `source`, `message`, `mode` ("direct" or "ai"), optional `channel` and `target`.
3. **Bearer auth**: Reuse the existing `api_key` from `[api]` config.
4. **Direct mode**: Look up the delivery channel and target, build an `OutgoingMessage`, call `channel.send()`.
5. **AI mode**: Build a synthetic `IncomingMessage` and inject it into the gateway's `mpsc::Sender<IncomingMessage>`.
6. **Audit logging**: Log webhook deliveries in the audit table.
7. **Structured response**: Return JSON with delivery status (`delivered`, `queued`, `error`).

## Explicitly Out of Scope
- Outbound webhooks (OMEGA calling external tools on events)
- Tool registration/discovery (tools just know the endpoint and bearer token)
- Internet exposure (localhost only)
- Webhook retry/queue (tool is responsible for retrying)
- Per-tool auth tokens
- Attachments (text payloads only for MVP)

## Key Decisions
- **Reuse existing API server**: No new HTTP listener. Just another route on axum port 3000.
- **Two modes, not one**: "direct" avoids AI latency; "ai" gives full pipeline access.
- **Inbound only**: Tools push to OMEGA. OMEGA does not push to tools.
- **Channel/target routing**: Default to first configured channel + first allowed user. Optional explicit routing.

## Open Questions for Analyst
1. **Synthetic channel name vs. flag**: Should webhook messages use `channel: "webhook"` (pseudo-channel) or `channel: "telegram"` with `is_webhook: true`?
2. **Default target resolution**: When channel/target omitted, use (a) first configured channel + first allowed user, (b) configurable default, or (c) require always?
3. **AI mode HTTP response**: Return 202 immediately since AI pipeline is async?
4. **Rate limiting**: Prevent runaway tools from flooding the user?

## Key Files
- `backend/src/api.rs` -- New `POST /api/webhook` handler, expanded `ApiState`
- `backend/src/gateway/mod.rs` -- Pass `tx` sender to API server
- `backend/crates/omega-core/src/message.rs` -- Webhook source marker on `IncomingMessage`

## Existing Patterns to Follow
- Direct delivery: same as scheduler reminders in `gateway/scheduler.rs`
- AI pipeline injection: same as channel message forwarding in `gateway/mod.rs`
- Auth: same bearer token check in `api.rs` `check_auth()`

## Risks
- **AI mode gateway integration**: Synthetic `IncomingMessage` needs sensible sender_id, reply_target for pipeline routing
- **Response routing**: AI response must land on the right channel, not back to the webhook caller
- **Silent delivery failures**: Channel might be down but webhook returns 200/202
