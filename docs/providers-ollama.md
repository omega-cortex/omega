# Ollama Provider

## Overview

The Ollama provider lets Omega use locally-running large language models through [Ollama](https://ollama.ai). Unlike cloud-based providers (Anthropic, OpenAI, OpenRouter), Ollama requires no API key and keeps all inference on your own machine. This makes it a good fit for privacy-conscious setups or offline usage.

## Current Status: Placeholder

The file at `backend/crates/omega-providers/src/ollama.rs` is currently a placeholder. It contains only a module-level doc comment:

```rust
//! Ollama local model provider (placeholder).
```

No structs, functions, or trait implementations exist yet. This module is part of Phase 4 of the Omega roadmap. The configuration infrastructure (config struct, TOML section, defaults) is already in place -- only the provider implementation itself is missing.

## Configuration (Already Done)

Even though the provider code is not yet implemented, the configuration layer is ready. The `OllamaConfig` struct exists in `omega-core` and the example config file includes a section for it.

In `config.example.toml`:

```toml
[provider.ollama]
enabled = false
base_url = "http://localhost:11434"
model = "llama3"
```

| Field | Default | What It Does |
|-------|---------|-------------|
| `enabled` | `false` | Flip to `true` to activate this provider. |
| `base_url` | `http://localhost:11434` | The URL where your Ollama server is listening. |
| `model` | `llama3` | Which model to use. Must already be pulled in Ollama. |

No API key field is needed since Ollama runs locally.

## The `Provider` Trait

Every AI backend in Omega implements the `Provider` trait from `omega-core`. Here are the four methods the Ollama provider must satisfy:

### `fn name(&self) -> &str`

Return a human-readable name for this provider. Should return `"ollama"`.

This is used in logging, audit records, and the `provider_used` field of `MessageMetadata`.

### `fn requires_api_key(&self) -> bool`

Return `false`. Ollama runs locally and does not require an API key.

### `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>`

This is the main method. It receives a `Context` containing:

- `system_prompt` -- the system-level instructions for the AI
- `history` -- a list of previous `(role, content)` pairs in the conversation
- `current_message` -- the user's latest message

The implementation should:

1. Build a JSON request body mapping the `Context` fields to Ollama's `/api/chat` format.
2. POST that request to `{base_url}/api/chat` with `"stream": false`.
3. Parse the response JSON and extract the assistant's reply text.
4. Return an `OutgoingMessage` with the text and metadata (model name, processing time, token count if available).
5. On failure, return `OmegaError::Provider(...)` with a descriptive message.

### `async fn is_available(&self) -> bool`

Check whether the Ollama server is reachable. A good approach is to send a GET request to `{base_url}/api/tags` and return `true` if the response status is 200. Return `false` on any error (connection refused, timeout, etc.).

## What Needs to Be Done

Here is a step-by-step guide for implementing this provider:

### 1. Define the `OllamaProvider` struct

```rust
pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}
```

Add a constructor that takes an `OllamaConfig`:

```rust
impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &OllamaConfig) -> Self {
        Self::new(config.base_url.clone(), config.model.clone())
    }
}
```

### 2. Define request/response structs

Ollama's `/api/chat` endpoint expects and returns JSON. You will need structs like:

```rust
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    model: String,
    message: ChatMessage,
    total_duration: Option<u64>,
    eval_count: Option<u64>,
}
```

### 3. Implement the `Provider` trait

Use `reqwest` (already in the crate's dependencies) to make HTTP calls. Use `tracing` for logging instead of `println!`. Return `OmegaError::Provider(...)` for any failures.

### 4. Convert `Context` to Ollama messages

Map the `Context` fields to the Ollama chat format:

- `context.system_prompt` becomes a message with `role: "system"`.
- Each entry in `context.history` maps directly (roles are already `"user"` or `"assistant"`).
- `context.current_message` becomes the final `role: "user"` message.

### 5. Export the module

In `backend/crates/omega-providers/src/lib.rs`, change:

```rust
mod ollama;
```

to:

```rust
pub mod ollama;
```

### 6. Wire it into the gateway

In `backend/src/main.rs` or wherever the provider is instantiated, add a branch for `"ollama"` that creates an `OllamaProvider` from the config and passes it to the gateway.

### 7. Write tests

At minimum:

- A unit test that verifies `name()` returns `"ollama"` and `requires_api_key()` returns `false`.
- A test that constructs the provider from config and checks the fields.
- Integration tests (behind a feature flag or `#[ignore]`) that require a running Ollama instance.

## Dependencies

All crate dependencies needed for implementation are already declared in `backend/crates/omega-providers/Cargo.toml`:

| Dependency | Role |
|------------|------|
| `reqwest` | HTTP client for talking to the Ollama server |
| `serde` / `serde_json` | JSON serialization for request and response bodies |
| `async-trait` | Required for implementing the async `Provider` trait |
| `tokio` | Async runtime |
| `tracing` | Structured logging (`debug!`, `warn!`, `error!`) |
| `omega-core` | Core types: `Context`, `OutgoingMessage`, `MessageMetadata`, `OmegaError`, `Provider` trait |

No new dependencies need to be added.

## Reference: How the Claude Code Provider Does It

The Claude Code provider (`backend/crates/omega-providers/src/claude_code.rs`) is a good reference. It follows the same pattern:

1. Struct holds config state (`session_id`, `max_turns`, `allowed_tools`).
2. `complete()` invokes the Claude CLI as a subprocess, captures stdout, parses JSON.
3. Returns an `OutgoingMessage` with `text`, `metadata` (provider name, model, processing time), and `reply_target`.
4. `is_available()` checks if the CLI binary exists by running `claude --version`.

The Ollama provider will follow the same structure, but instead of spawning a subprocess, it will make HTTP requests to the Ollama server.

## Ollama Prerequisites

For the provider to work at runtime, the user must:

1. Install Ollama from [ollama.ai](https://ollama.ai).
2. Start the Ollama service: `ollama serve`.
3. Pull the desired model: `ollama pull llama3`.
4. Set `enabled = true` and (optionally) configure `base_url` and `model` in `config.toml`.
