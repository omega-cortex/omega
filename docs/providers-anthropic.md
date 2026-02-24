# Anthropic API Provider

## Overview

The Anthropic provider (`backend/crates/omega-providers/src/anthropic.rs`) is one of five AI provider backends in the Omega workspace. It will connect directly to the Anthropic Messages API over HTTP, allowing Omega to use Claude models without relying on the Claude Code CLI.

**Current status: placeholder.** The file exists but contains only a single doc comment. No code has been written yet. This provider is scheduled for Phase 4.

## Why It Matters

The default provider in Omega is `claude-code`, which shells out to the locally installed `claude` CLI. That works great for a developer workstation, but the Anthropic API provider opens up several advantages:

- **Server deployment** -- No need to install the Claude CLI on a remote host; just configure an API key.
- **Token tracking** -- The Anthropic API returns `input_tokens` and `output_tokens`, which allows Omega to report usage in `MessageMetadata.tokens_used`.
- **Model selection** -- Direct control over which model to use, configured in `config.toml`.
- **Faster cold starts** -- HTTP request vs. subprocess spawn.

## Configuration

The config structure already exists in `backend/crates/omega-core/src/config.rs`. Add this section to your `config.toml`:

```toml
[provider]
default = "anthropic"

[provider.anthropic]
enabled = true
api_key = "sk-ant-api03-..."
model = "claude-sonnet-4-20250514"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `false` | Whether this provider is active. |
| `api_key` | `String` | `""` | Your Anthropic API key. Keep this in `config.toml`, which is gitignored. |
| `model` | `String` | `"claude-sonnet-4-20250514"` | The model identifier to use. |

## The `Provider` Trait

Every AI backend in Omega implements the `Provider` trait defined in `backend/crates/omega-core/src/traits.rs`. Here are the four methods the Anthropic provider must satisfy:

### `fn name(&self) -> &str`

Return a human-readable name for this provider. Should return `"anthropic"`.

### `fn requires_api_key(&self) -> bool`

Return `true`. Unlike the Claude Code CLI provider (which uses the user's existing CLI session), the Anthropic API requires an API key in every request.

### `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>`

This is the main method. It receives a `Context` containing:

- `system_prompt` -- A system-level instruction string.
- `history` -- A `Vec<ContextEntry>`, each with a `role` (`"user"` or `"assistant"`) and `content`.
- `current_message` -- The latest user message.

The method must:

1. Convert the `Context` into the Anthropic Messages API format (a `system` field and a `messages` array).
2. Send a `POST` request to `https://api.anthropic.com/v1/messages` with the appropriate headers (`x-api-key`, `anthropic-version`, `content-type`).
3. Parse the JSON response, extracting the assistant's text from the `content` array.
4. Return an `OutgoingMessage` with:
   - `text` -- The assistant's reply.
   - `metadata.provider_used` -- `"anthropic"`.
   - `metadata.tokens_used` -- Sum of `input_tokens` + `output_tokens` from the response.
   - `metadata.processing_time_ms` -- Wall-clock time for the request.
   - `metadata.model` -- The model string from the response.
   - `reply_target` -- `None` (the gateway sets this from the incoming message).

### `async fn is_available(&self) -> bool`

Check that the provider is usable. At minimum, verify that the API key is non-empty. Optionally, send a lightweight request to validate credentials.

## Implementation Roadmap

Here is a step-by-step guide for implementing this provider:

### 1. Add Dependencies

The `omega-providers` crate will need `reqwest` (with `json` feature) and `serde`/`serde_json` for HTTP and serialization. Check `Cargo.toml` for the crate to ensure they are included.

### 2. Define the Struct

```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}
```

Add a constructor `from_config(config: &AnthropicConfig) -> Self` and a `Default` impl if appropriate.

### 3. Define Request/Response Types

Create serde structs for the Anthropic API request and response:

```rust
#[derive(Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    system: Option<String>,
    messages: Vec<ApiMessage>,
}

#[derive(Serialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    model: String,
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u64,
    output_tokens: u64,
}
```

### 4. Implement the Trait

Follow the pattern established by `ClaudeCodeProvider`:

- Use `tracing::debug!` / `tracing::warn!` for logging (never `println!`).
- Use `?` for error propagation (never `unwrap()`).
- Return `OmegaError::Provider(...)` for all provider-specific failures.
- Measure elapsed time with `std::time::Instant`.

### 5. Make the Module Public

In `backend/crates/omega-providers/src/lib.rs`, change:

```rust
mod anthropic;
```

to:

```rust
pub mod anthropic;
```

### 6. Wire Into the Gateway

Update the gateway (`backend/src/gateway.rs`) to select this provider when `config.provider.default == "anthropic"` and `config.provider.anthropic` is `Some(...)` with `enabled == true`.

### 7. Write Tests

- Unit tests for context-to-request conversion.
- Unit tests for response parsing (use raw JSON fixtures).
- An integration test (gated behind a feature flag or environment variable) that sends a real request.

## Error Handling

All errors should use the `OmegaError::Provider(String)` variant. Common failure modes to handle:

- **Missing API key** -- Return an error at construction time or in `is_available()`.
- **HTTP errors** -- Network failures, timeouts, non-2xx status codes.
- **Malformed responses** -- JSON parsing failures; log a warning and return a meaningful error.
- **Rate limiting** -- HTTP 429 responses; consider surfacing the `retry-after` header in the error message.
- **Overloaded** -- HTTP 529 responses; similar handling to rate limiting.

## Relationship to Other Providers

| Provider | File | Status | Transport |
|----------|------|--------|-----------|
| Claude Code CLI | `claude_code.rs` | Implemented | Subprocess |
| **Anthropic API** | **`anthropic.rs`** | **Placeholder** | **HTTP** |
| OpenAI | `openai.rs` | Placeholder | HTTP |
| Ollama | `ollama.rs` | Placeholder | HTTP |
| OpenRouter | `openrouter.rs` | Placeholder | HTTP |

All five providers are declared in `lib.rs`. Currently only `claude_code` is public and functional. The Anthropic provider, once implemented, can serve as a template for the other three HTTP-based providers since they all follow the same general pattern: build a JSON request, POST it, parse the JSON response.
