# Technical Specification: `providers-ollama.md`

## File

| Property | Value |
|----------|-------|
| **Path** | `crates/omega-providers/src/ollama.rs` |
| **Crate** | `omega-providers` |
| **Module** | `ollama` (private, not re-exported) |
| **Status** | Placeholder / Stub |
| **Phase** | Phase 4 (planned) |

## Purpose

This module is reserved for the Ollama local model provider. Ollama allows running large language models locally without external API keys. When implemented, this provider will communicate with a local Ollama server over HTTP to perform text completions.

## Current Contents

The file contains exactly one line:

```rust
//! Ollama local model provider (placeholder).
```

No structs, traits, functions, or imports are defined. The module is declared in `crates/omega-providers/src/lib.rs` as a private module:

```rust
mod ollama;
```

## Structs

**None defined.** The following struct would need to be created:

### Expected: `OllamaProvider`

When implemented, this struct should hold the runtime state needed to communicate with the Ollama server.

| Field (expected) | Type (expected) | Description |
|------------------|-----------------|-------------|
| `base_url` | `String` | Ollama server endpoint (default: `http://localhost:11434`) |
| `model` | `String` | Model name to use (default: `llama3`) |
| `client` | `reqwest::Client` | HTTP client for API requests |

## Trait Implementations

**None defined.** The following trait must be implemented:

### Required: `Provider` (from `omega_core::traits`)

The `Provider` trait is defined as:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn requires_api_key(&self) -> bool;
    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>;
    async fn is_available(&self) -> bool;
}
```

| Method | Return Type | Expected Implementation |
|--------|-------------|------------------------|
| `name()` | `&str` | Return `"ollama"` |
| `requires_api_key()` | `bool` | Return `false` (Ollama runs locally, no API key needed) |
| `complete(&self, context: &Context)` | `Result<OutgoingMessage, OmegaError>` | POST to Ollama `/api/chat` or `/api/generate` endpoint, parse response, return `OutgoingMessage` |
| `is_available(&self)` | `bool` | HTTP GET to `{base_url}/api/tags` or similar health endpoint to confirm the server is running |

## Function Signatures

**None defined.** Expected functions:

| Function | Signature (expected) | Description |
|----------|---------------------|-------------|
| `OllamaProvider::new` | `fn new(base_url: String, model: String) -> Self` | Constructor with explicit parameters |
| `OllamaProvider::from_config` | `fn from_config(config: &OllamaConfig) -> Self` | Constructor from the existing `OllamaConfig` struct |

## Related Configuration

The `OllamaConfig` struct already exists in `omega-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_ollama_base_url")]
    pub base_url: String,
    #[serde(default = "default_ollama_model")]
    pub model: String,
}
```

| Field | Type | Default Value | Serde Attribute |
|-------|------|---------------|-----------------|
| `enabled` | `bool` | `false` | `#[serde(default)]` |
| `base_url` | `String` | `"http://localhost:11434"` | `#[serde(default = "default_ollama_base_url")]` |
| `model` | `String` | `"llama3"` | `#[serde(default = "default_ollama_model")]` |

### Config Example (`config.example.toml`)

```toml
[provider.ollama]
enabled = false
base_url = "http://localhost:11434"
model = "llama3"
```

## Dependencies Available

The `omega-providers` crate already has all dependencies needed for implementation:

| Dependency | Use |
|------------|-----|
| `reqwest` | HTTP client for Ollama API calls |
| `serde` / `serde_json` | Serialize/deserialize request and response JSON |
| `async-trait` | `#[async_trait]` for the `Provider` trait |
| `tokio` | Async runtime |
| `tracing` | Structured logging |
| `omega-core` | `Context`, `OutgoingMessage`, `MessageMetadata`, `OmegaError`, `Provider` trait |

## Ollama API Surface (Reference)

The Ollama HTTP API endpoints relevant for implementation:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `POST /api/generate` | POST | Single-turn text generation |
| `POST /api/chat` | POST | Multi-turn chat completion (preferred) |
| `GET /api/tags` | GET | List available models (useful for `is_available()`) |

### Expected Request Body (`/api/chat`)

```json
{
  "model": "llama3",
  "messages": [
    {"role": "system", "content": "..."},
    {"role": "user", "content": "..."},
    {"role": "assistant", "content": "..."}
  ],
  "stream": false
}
```

### Expected Response Body (`/api/chat`)

```json
{
  "model": "llama3",
  "message": {
    "role": "assistant",
    "content": "..."
  },
  "total_duration": 12345678,
  "eval_count": 42
}
```

## Module Visibility

| Item | Visibility |
|------|------------|
| Module `ollama` in `lib.rs` | `mod ollama;` (private) |
| Expected struct `OllamaProvider` | `pub struct` (needs to be re-exported or made `pub mod` in `lib.rs`) |

When implemented, `lib.rs` should be updated from `mod ollama;` to `pub mod ollama;` to expose the provider.

## Cross-References

| File | Relationship |
|------|-------------|
| `crates/omega-providers/src/lib.rs` | Declares the `ollama` module |
| `crates/omega-core/src/traits.rs` | Defines the `Provider` trait this module must implement |
| `crates/omega-core/src/config.rs` | Defines `OllamaConfig` used for configuration |
| `crates/omega-core/src/context.rs` | Defines `Context` passed to `complete()` |
| `crates/omega-core/src/message.rs` | Defines `OutgoingMessage` and `MessageMetadata` returned by `complete()` |
| `crates/omega-core/src/error.rs` | Defines `OmegaError::Provider(String)` for error reporting |
| `config.example.toml` | Contains the `[provider.ollama]` configuration section |
| `crates/omega-providers/src/claude_code.rs` | Reference implementation of the `Provider` trait |
