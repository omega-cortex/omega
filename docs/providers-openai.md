# OpenAI Provider

**File:** `crates/omega-providers/src/openai.rs`
**Crate:** `omega-providers`
**Status:** Placeholder -- not yet implemented.

---

## Current State

The OpenAI provider file currently contains a single line:

```rust
//! OpenAI-compatible API provider (placeholder).
```

There is no struct, no trait implementation, and no executable code. The module is registered in `crates/omega-providers/src/lib.rs` as a private module (`mod openai;`), but nothing is exported from it.

Despite the provider code being a stub, the supporting infrastructure already exists:

- **Configuration struct** (`OpenAiConfig`) is defined in `omega-core` and can be read from `config.toml`.
- **Example config** includes an `[provider.openai]` section with `enabled`, `api_key`, `model`, and `base_url` fields.
- **Environment variable** `OPENAI_API_KEY` is documented as an override for the API key.

This means the config layer is ready -- the provider just needs to be built on top of it.

---

## What the Provider Must Implement

Every AI backend in Omega implements the `Provider` trait (defined in `crates/omega-core/src/traits.rs`). The trait has four methods:

### `fn name(&self) -> &str`

Return a short, stable identifier. For this provider, that should be `"openai"`. The gateway uses this string for logging, audit records, and the `/status` bot command.

### `fn requires_api_key(&self) -> bool`

Return `true`. The OpenAI API requires authentication via an API key.

### `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>`

This is the core method. It receives a `Context` containing:

- A **system prompt** (goes into the `system` message in the OpenAI chat completions format).
- A **conversation history** as a list of `(role, content)` pairs.
- The **current user message**.

The provider must:

1. Build an HTTP request to the OpenAI Chat Completions API (`POST /v1/chat/completions`).
2. Map the `Context` fields to the OpenAI message format (`{"role": "system", "content": "..."}`, etc.).
3. Include the API key in the `Authorization: Bearer <key>` header.
4. Send the request, parse the JSON response, and extract the assistant's reply.
5. Return an `OutgoingMessage` with the response text and metadata (provider name, token count, processing time, model).
6. On failure, return `OmegaError::Provider(...)` with a descriptive message.

### `async fn is_available(&self) -> bool`

Check whether the OpenAI API is reachable. A simple approach is to call the `/v1/models` endpoint with the configured API key and check for a successful response. Alternatively, just verify that an API key is configured and non-empty.

---

## How to Implement It

Here is a step-by-step guide for building the OpenAI provider.

### 1. Add HTTP dependencies

The `omega-providers` crate will need an HTTP client. Add `reqwest` (with JSON support) to `crates/omega-providers/Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
```

You will also need `serde` and `serde_json` for request/response serialization (these are likely already available through `omega-core`).

### 2. Define the provider struct

```rust
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}
```

Add a constructor that reads from `OpenAiConfig`:

```rust
impl OpenAiProvider {
    pub fn from_config(config: &OpenAiConfig) -> Result<Self, OmegaError> {
        if config.api_key.is_empty() {
            return Err(OmegaError::Config(
                "OpenAI API key is required".to_string(),
            ));
        }
        Ok(Self {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config.base_url.clone(),
            client: reqwest::Client::new(),
        })
    }
}
```

### 3. Define request/response types

The OpenAI Chat Completions API expects and returns structured JSON. Define serde structs for at least:

- **Request:** `model`, `messages` (array of `{role, content}`), optional `temperature`, `max_tokens`.
- **Response:** `choices[0].message.content`, `model`, `usage.total_tokens`.

### 4. Implement the `Provider` trait

Map the `Context` to OpenAI's message format:

- `context.system_prompt` becomes `{"role": "system", "content": "..."}`.
- Each entry in `context.history` becomes `{"role": "user"|"assistant", "content": "..."}`.
- `context.current_message` becomes `{"role": "user", "content": "..."}`.

This is more natural than the `to_prompt_string()` flattening used by the Claude Code CLI provider, because the OpenAI API natively supports structured message arrays.

### 5. Make the module public

Change `mod openai;` to `pub mod openai;` in `crates/omega-providers/src/lib.rs`, or add a `pub use` re-export for the struct.

### 6. Wire it into the gateway

In `src/main.rs` (or wherever the provider is constructed), add a branch that instantiates `OpenAiProvider` when the config's `default` provider is `"openai"` and `provider.openai` is `Some(...)` with `enabled = true`.

### 7. Add tests

At minimum:

- Unit test for struct construction and `name()`/`requires_api_key()`.
- Integration test (behind a feature flag or `#[ignore]`) that calls the real API with a valid key.

---

## Configuration

The provider is configured in `config.toml` under `[provider.openai]`:

```toml
[provider]
default = "openai"

[provider.openai]
enabled = true
api_key = ""       # Or set OPENAI_API_KEY env var
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
```

| Field | Default | Notes |
|-------|---------|-------|
| `enabled` | `false` | Must be set to `true` to activate. |
| `api_key` | `""` | Required. Can be provided via `OPENAI_API_KEY` environment variable instead. |
| `model` | `"gpt-4o"` | Any model available on the configured endpoint. |
| `base_url` | `"https://api.openai.com/v1"` | Change this to use OpenAI-compatible services (Azure OpenAI, LocalAI, vLLM, etc.). |

---

## Design Considerations

**Error handling.** Follow the project rule: no `unwrap()`. Use `?` with `OmegaError::Provider(...)` for all fallible operations. Common failure modes include network timeouts, invalid API keys (HTTP 401), rate limiting (HTTP 429), and malformed responses.

**Logging.** Use `tracing` (not `println!`). Log the request model and timing at `debug` level, and errors/warnings at `warn` or `error` level.

**Timeouts.** Configure a reasonable request timeout on the `reqwest::Client` (e.g., 120 seconds for long completions).

**Streaming.** The initial implementation should use the non-streaming endpoint. Streaming (SSE) support can be added later as an enhancement.

**OpenAI-compatible services.** Because the `base_url` is configurable, this provider can work with any service that implements the OpenAI chat completions API: Azure OpenAI, Together AI, Groq, LocalAI, vLLM, and others. The implementation should not hard-code any OpenAI-specific assumptions beyond the API contract.

---

## Relationship to Other Providers

| Provider | Status | Transport | Auth |
|----------|--------|-----------|------|
| Claude Code CLI | Implemented | Local subprocess | CLI session (no key) |
| Anthropic | Placeholder | HTTP (planned) | API key |
| **OpenAI** | **Placeholder** | **HTTP (planned)** | **API key** |
| Ollama | Placeholder | HTTP (planned) | None (local) |
| OpenRouter | Placeholder | HTTP (planned) | API key |

The OpenAI provider is planned for Phase 4 of the project alongside the other alternative providers.
