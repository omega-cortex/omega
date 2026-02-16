# OpenRouter Provider

## What Is This?

The OpenRouter provider (`crates/omega-providers/src/openrouter.rs`) is intended to let Omega route AI requests through [OpenRouter](https://openrouter.ai), a proxy service that provides a single API endpoint to access models from Anthropic, OpenAI, Meta, Google, Mistral, and many others.

## Current Status: Placeholder

The file currently contains only a single doc comment:

```rust
//! OpenRouter proxy provider (placeholder).
```

There are no structs, no trait implementations, and no logic. This is by design -- it was scaffolded during Phase 1 as part of the workspace layout, and actual implementation is planned for **Phase 4**.

The module is declared in `lib.rs` as a private module (`mod openrouter;`), so nothing is exposed to other crates yet.

## Why OpenRouter?

OpenRouter is a particularly useful provider for Omega because:

- **Model flexibility** -- A single API key grants access to dozens of models across multiple vendors. You can switch from Claude to GPT-4o to Llama 3 without changing provider config.
- **OpenAI-compatible API** -- The request/response format follows the OpenAI chat completions spec, which is well-documented and widely supported.
- **Fallback routing** -- OpenRouter can automatically route to alternative models if one is unavailable, adding resilience.
- **Cost tracking** -- The API response includes token usage, making it straightforward to populate `MessageMetadata`.

## What Needs to Be Done

To implement this provider fully, you would need to:

### 1. Define the Provider Struct

Create a struct to hold configuration. At minimum:

```rust
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    base_url: String,       // default: https://openrouter.ai/api/v1
    http_client: reqwest::Client,
}
```

### 2. Implement the `Provider` Trait

The trait lives in `omega-core::traits` and requires four methods:

**`fn name(&self) -> &str`**
Return `"openrouter"` (or similar). This string appears in logs and in `MessageMetadata.provider_used`.

**`fn requires_api_key(&self) -> bool`**
Return `true`. OpenRouter always requires an API key.

**`async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>`**
This is the core method. It should:
1. Convert the `Context` (system prompt, history, current message) into an OpenAI-compatible `messages` array.
2. Send a POST request to `https://openrouter.ai/api/v1/chat/completions` with the `Authorization: Bearer <key>` header.
3. Parse the JSON response to extract the assistant's reply text, model used, and token count.
4. Return an `OutgoingMessage` with the text and populated `MessageMetadata`.

**`async fn is_available(&self) -> bool`**
Check that the API key is set and, optionally, that the endpoint responds. A simple approach is to verify the key is non-empty; a more robust approach is to make a lightweight API call (e.g., list models).

### 3. Handle the OpenRouter Request/Response Format

The request body follows OpenAI's chat completions format:

```json
{
  "model": "anthropic/claude-sonnet-4",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "..."},
    {"role": "assistant", "content": "..."},
    {"role": "user", "content": "..."}
  ]
}
```

The response includes:

```json
{
  "choices": [{"message": {"role": "assistant", "content": "..."}}],
  "model": "anthropic/claude-sonnet-4",
  "usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}
}
```

Map `usage.total_tokens` to `MessageMetadata.tokens_used` and `model` to `MessageMetadata.model`.

### 4. Add Configuration Support

The provider should read its settings from `config.toml`. A reasonable config section might look like:

```toml
[provider.openrouter]
api_key_env = "OPENROUTER_API_KEY"   # Read API key from this env var
model = "anthropic/claude-sonnet-4"
base_url = "https://openrouter.ai/api/v1"
```

Following Omega conventions, never store the actual API key in the config file -- reference an environment variable instead.

### 5. Export the Module

Update `crates/omega-providers/src/lib.rs` to make the module public:

```rust
pub mod openrouter;
```

Or re-export the struct directly:

```rust
mod openrouter;
pub use openrouter::OpenRouterProvider;
```

### 6. Add Error Handling

Use `OmegaError::Provider(String)` for all errors. Follow the project rule of never using `unwrap()` -- propagate errors with `?` and `map_err`. Common failure cases to handle:

- Missing or invalid API key
- Network errors / timeouts
- Non-200 HTTP responses from OpenRouter
- Malformed JSON in the response
- Rate limiting (HTTP 429)

### 7. Add Tests

At minimum, include a unit test verifying default construction and trait method return values (similar to the `ClaudeCodeProvider` tests). Integration tests against the live API should be feature-gated to avoid requiring an API key in CI.

## Reference Implementation

The `ClaudeCodeProvider` in `crates/omega-providers/src/claude_code.rs` is the only fully implemented provider and serves as the reference. While it uses a subprocess (CLI) rather than HTTP, the pattern for constructing an `OutgoingMessage` with `MessageMetadata` and handling errors is directly applicable.

## Crate Design Rules to Follow

- Use `tracing` for all logging (never `println!`).
- All I/O must be async (the `reqwest` client already supports this).
- Add a doc comment to every public function and struct.
- Use `?` for error propagation, never `unwrap()`.
- The `reqwest` dependency is already declared in `Cargo.toml` -- no new dependencies are needed for basic HTTP functionality.
