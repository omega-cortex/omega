# Technical Specification: omega-channels/src/lib.rs

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-channels/src/lib.rs` |
| **Crate** | `omega-channels` |
| **Role** | Crate root -- declares submodules and controls public API surface |

## Purpose

This file is the entry point for the `omega-channels` crate. It serves two purposes:

1. Declare the submodules that implement individual messaging platform integrations.
2. Control which modules are publicly re-exported to downstream crates (`omega-core`, the gateway, the binary).

## Module Declarations

| Module | Visibility | Status | Description |
|--------|-----------|--------|-------------|
| `telegram` | `pub mod` | Implemented | Telegram Bot API channel using long-polling (`getUpdates`) and `sendMessage`. |
| `whatsapp` | `mod` (private) | Placeholder | WhatsApp bridge channel. Contains only a doc comment; no public types. |

## Re-exports

The file does **not** contain any explicit `pub use` re-exports. Public access to channel implementations is provided solely through module visibility:

| Symbol | Access Path | Notes |
|--------|-------------|-------|
| `TelegramChannel` | `omega_channels::telegram::TelegramChannel` | Public struct; implements `omega_core::traits::Channel`. |
| `split_message` | Not accessible | Private function inside `telegram` module (not marked `pub` at module boundary, but `pub` within the module -- accessible as `omega_channels::telegram::split_message` because the module itself is `pub`). |

Since `whatsapp` is declared with `mod` (no `pub`), nothing inside it is reachable from outside the crate.

## Feature Gates

There are **no** feature gates defined in `lib.rs` or in the crate's `Cargo.toml`. Both modules are compiled unconditionally.

## Dependencies (from Cargo.toml)

| Dependency | Usage |
|------------|-------|
| `omega-core` | Provides `Channel` trait, `IncomingMessage`, `OutgoingMessage`, `OmegaError`, `TelegramConfig` |
| `tokio` | Async runtime, `mpsc` channels, `Mutex`, `sleep` |
| `serde` / `serde_json` | Deserializing Telegram API responses, building JSON request bodies |
| `tracing` | Structured logging (`info!`, `warn!`, `error!`, `debug!`) |
| `thiserror` | Declared as dependency but error types come from `omega-core` |
| `anyhow` | Declared as dependency; not directly used in current code |
| `async-trait` | `#[async_trait]` on the `Channel` impl |
| `reqwest` | HTTP client for Telegram Bot API |
| `uuid` | Generating unique `IncomingMessage.id` values |
| `chrono` | Timestamping incoming messages with `Utc::now()` |

## Public API Surface Summary

| Item | Kind | Module | Description |
|------|------|--------|-------------|
| `telegram` | module | crate root | Public module containing the Telegram integration |
| `TelegramChannel` | struct | `telegram` | Implements `Channel` trait for Telegram Bot API |
| `TelegramChannel::new(config: TelegramConfig) -> Self` | associated fn | `telegram` | Constructor |
| `split_message(text: &str, max_len: usize) -> Vec<&str>` | free fn | `telegram` | Splits long messages to respect Telegram's 4096-char limit |

### `TelegramChannel` -- Channel trait methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `name` | `fn name(&self) -> &str` | Returns `"telegram"` |
| `start` | `async fn start(&self) -> Result<mpsc::Receiver<IncomingMessage>, OmegaError>` | Spawns long-polling task, returns message receiver |
| `send` | `async fn send(&self, message: OutgoingMessage) -> Result<(), OmegaError>` | Sends an outgoing message to the target chat |
| `send_typing` | `async fn send_typing(&self, target: &str) -> Result<(), OmegaError>` | Sends a "typing" chat action indicator |
| `stop` | `async fn stop(&self) -> Result<(), OmegaError>` | Logs shutdown; no-op cleanup |

## Tests

The `telegram` module contains a `#[cfg(test)] mod tests` block with two unit tests:

| Test | Description |
|------|-------------|
| `test_split_short_message` | Asserts a short string returns a single chunk |
| `test_split_long_message` | Asserts a 6000-char string is split into chunks each <= 4096 |
