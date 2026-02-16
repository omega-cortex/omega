# Technical Specification: Anthropic API Provider

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-providers/src/anthropic.rs` |
| **Crate** | `omega-providers` |
| **Module declaration** | `mod anthropic;` in `crates/omega-providers/src/lib.rs` (private module) |
| **Status** | **Placeholder / Stub** -- file contains only a module-level doc comment |

## Current Contents

```rust
//! Anthropic API provider (placeholder).
```

The file is a single line. No structs, no trait implementations, no functions, and no imports exist yet.

## Required Trait: `Provider`

Defined in `omega-core/src/traits.rs`. Any Anthropic provider struct must implement this trait.

| Method | Signature | Description |
|--------|-----------|-------------|
| `name` | `fn name(&self) -> &str` | Returns a human-readable provider name (e.g. `"anthropic"`). |
| `requires_api_key` | `fn requires_api_key(&self) -> bool` | Must return `true` for the Anthropic API provider. |
| `complete` | `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>` | Sends conversation context to the Anthropic Messages API and returns the assistant response. |
| `is_available` | `async fn is_available(&self) -> bool` | Checks whether the provider is configured and reachable. |

The trait requires `Send + Sync` and uses the `#[async_trait]` attribute macro.

## Expected Struct: `AnthropicProvider`

Not yet defined. Based on the existing configuration struct `AnthropicConfig` (in `omega-core/src/config.rs`) and the reference implementation `ClaudeCodeProvider`, the provider struct should hold:

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `api_key` | `String` | `AnthropicConfig.api_key` | Anthropic API key (required). |
| `model` | `String` | `AnthropicConfig.model` | Model identifier. Default: `"claude-sonnet-4-20250514"`. |
| `client` | `reqwest::Client` | constructed | HTTP client for API requests. |

## Related Configuration Struct: `AnthropicConfig`

Defined in `omega-core/src/config.rs`, already exists and is fully functional.

| Field | Type | Serde Attribute | Default |
|-------|------|-----------------|---------|
| `enabled` | `bool` | `#[serde(default)]` | `false` |
| `api_key` | `String` | `#[serde(default)]` | `""` |
| `model` | `String` | `#[serde(default = "default_anthropic_model")]` | `"claude-sonnet-4-20250514"` |

Config section in `config.toml`:

```toml
[provider.anthropic]
enabled = true
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"
```

## Dependencies on omega-core Types

| Type | Path | Role |
|------|------|------|
| `Context` | `omega-core/src/context.rs` | Input to `complete()`. Contains `system_prompt`, `history` (Vec of `ContextEntry`), and `current_message`. |
| `ContextEntry` | `omega-core/src/context.rs` | A single history turn with `role` (`"user"` or `"assistant"`) and `content`. |
| `OutgoingMessage` | `omega-core/src/message.rs` | Return value of `complete()`. Contains `text`, `metadata` (`MessageMetadata`), and `reply_target`. |
| `MessageMetadata` | `omega-core/src/message.rs` | Provider metadata: `provider_used`, `tokens_used`, `processing_time_ms`, `model`. |
| `OmegaError` | `omega-core/src/error.rs` | Error type. The `OmegaError::Provider(String)` variant is used for provider failures. |

## Module Visibility

The module is declared as `mod anthropic;` (private) in `lib.rs`. All other placeholder providers (ollama, openai, openrouter) are also private. Only `claude_code` is `pub mod`. When this provider is implemented, the module should be made public (`pub mod anthropic;`) so the gateway can construct and use it.

## Anthropic Messages API Reference

The implementation must target the Anthropic Messages API (`POST https://api.anthropic.com/v1/messages`).

### Request Shape

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model` | `string` | yes | Model ID (e.g. `"claude-sonnet-4-20250514"`). |
| `max_tokens` | `integer` | yes | Maximum tokens in the response. |
| `system` | `string` | no | System prompt. Maps from `Context.system_prompt`. |
| `messages` | `array` | yes | Array of `{"role": "user"|"assistant", "content": "..."}`. Maps from `Context.history` + `Context.current_message`. |

### Required Headers

| Header | Value |
|--------|-------|
| `x-api-key` | The API key from config. |
| `anthropic-version` | `"2023-06-01"` (or latest). |
| `content-type` | `"application/json"` |

### Response Shape (relevant fields)

| Field | Type | Description |
|-------|------|-------------|
| `content` | `array` | Array of content blocks; text blocks have `{"type": "text", "text": "..."}`. |
| `model` | `string` | Model used. |
| `usage.input_tokens` | `integer` | Input token count. |
| `usage.output_tokens` | `integer` | Output token count. |
| `stop_reason` | `string` | `"end_turn"`, `"max_tokens"`, `"stop_sequence"`. |

## Reference Implementation Comparison

The existing `ClaudeCodeProvider` serves as the reference pattern.

| Aspect | ClaudeCodeProvider | AnthropicProvider (expected) |
|--------|-------------------|------------------------------|
| Backend | Claude CLI subprocess | Anthropic REST API via HTTP |
| Auth | None (CLI session) | API key in header |
| `requires_api_key()` | `false` | `true` |
| Transport | `tokio::process::Command` | `reqwest::Client` |
| Response parsing | Custom `ClaudeCliResponse` JSON | Anthropic Messages API JSON |
| Token info | Not available | Available from `usage` field |
| `name()` | `"claude-code"` | `"anthropic"` |

## Test Expectations

At minimum, unit tests should cover:

| Test | Description |
|------|-------------|
| `test_provider_name` | `name()` returns `"anthropic"`. |
| `test_requires_api_key` | `requires_api_key()` returns `true`. |
| `test_context_to_messages` | Conversion from `Context` to Anthropic message array format. |
| `test_parse_response` | Parsing a well-formed Anthropic API JSON response. |
| `test_parse_error_response` | Handling API error responses gracefully. |
