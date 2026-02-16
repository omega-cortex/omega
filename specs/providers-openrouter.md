# Technical Specification: OpenRouter Provider

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-providers/src/openrouter.rs` |
| **Crate** | `omega-providers` |
| **Module declaration** | `mod openrouter;` in `crates/omega-providers/src/lib.rs` (private) |
| **Module doc comment** | `//! OpenRouter proxy provider (placeholder).` |
| **Status** | **Placeholder / Stub** -- the file contains only a module-level doc comment and no executable code. |

## Current Contents

The entire file consists of a single line:

```rust
//! OpenRouter proxy provider (placeholder).
```

No structs, enums, traits, functions, constants, or imports are defined.

## Structs

None.

## Trait Implementations

None. When implemented, this module must provide a struct that implements the `Provider` trait from `omega-core::traits`.

## Required Trait: `Provider`

Defined in `crates/omega-core/src/traits.rs`. Any OpenRouter provider struct must satisfy the following interface:

| Method | Signature | Description |
|--------|-----------|-------------|
| `name` | `fn name(&self) -> &str` | Returns a human-readable provider name (e.g., `"openrouter"`). |
| `requires_api_key` | `fn requires_api_key(&self) -> bool` | Must return `true` -- OpenRouter requires an API key. |
| `complete` | `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>` | Sends conversation context to the OpenRouter API and returns the model response. |
| `is_available` | `async fn is_available(&self) -> bool` | Checks whether the provider can currently handle requests (e.g., API key is set, endpoint is reachable). |

The trait is gated behind `#[async_trait]` from the `async-trait` crate and requires `Send + Sync`.

## Dependencies Available in Crate

The `omega-providers` crate already declares these workspace dependencies, all of which would be used by a full implementation:

| Dependency | Relevance to OpenRouter |
|------------|------------------------|
| `omega-core` | `Provider` trait, `Context`, `OutgoingMessage`, `MessageMetadata`, `OmegaError` |
| `tokio` | Async runtime |
| `reqwest` | HTTP client for OpenRouter REST API |
| `serde` / `serde_json` | Serialize request bodies and deserialize JSON responses |
| `tracing` | Structured logging |
| `async-trait` | `#[async_trait]` macro for the `Provider` trait |
| `thiserror` / `anyhow` | Error handling utilities |

## Module Visibility

The module is declared as `mod openrouter;` (private) in `lib.rs`. It is **not** re-exported with `pub mod` or `pub use`. To expose it to the rest of the workspace, `lib.rs` would need to be updated to either `pub mod openrouter;` or add a `pub use openrouter::OpenRouterProvider;` (or equivalent) re-export.

## Types the Implementation Must Use

### Input: `Context` (`omega-core::context`)

| Field | Type | Description |
|-------|------|-------------|
| `system_prompt` | `String` | System prompt prepended to every request. |
| `history` | `Vec<ContextEntry>` | Conversation history, oldest first. Each entry has `role: String` and `content: String`. |
| `current_message` | `String` | The current user message. |

### Output: `OutgoingMessage` (`omega-core::message`)

| Field | Type | Description |
|-------|------|-------------|
| `text` | `String` | The response text. |
| `metadata` | `MessageMetadata` | Provider name, token count, processing time, model identifier. |
| `reply_target` | `Option<String>` | Platform-specific routing target (typically `None` from the provider). |

### Errors: `OmegaError` (`omega-core::error`)

The `complete` method returns `Result<OutgoingMessage, OmegaError>`. Provider-level errors use the `OmegaError::Provider(String)` variant.

## Relationship to Other Placeholder Providers

All four non-Claude-Code providers are currently identical placeholders:

| File | Doc Comment |
|------|-------------|
| `anthropic.rs` | `//! Anthropic API provider (placeholder).` |
| `openai.rs` | `//! OpenAI-compatible API provider (placeholder).` |
| `ollama.rs` | `//! Ollama local model provider (placeholder).` |
| `openrouter.rs` | `//! OpenRouter proxy provider (placeholder).` |

Only `claude_code.rs` contains a working implementation. It serves as the canonical reference for how to implement the `Provider` trait in this codebase.

## OpenRouter API Reference (External)

OpenRouter provides an OpenAI-compatible REST API at `https://openrouter.ai/api/v1/chat/completions`. Key characteristics:

| Aspect | Detail |
|--------|--------|
| **Auth** | `Authorization: Bearer <OPENROUTER_API_KEY>` header |
| **Endpoint** | `POST https://openrouter.ai/api/v1/chat/completions` |
| **Request format** | OpenAI-compatible: `model`, `messages` array, optional `temperature`, `max_tokens`, etc. |
| **Response format** | OpenAI-compatible: `choices[0].message.content`, `usage.total_tokens`, `model` |
| **Model selection** | Via the `model` field (e.g., `"anthropic/claude-sonnet-4"`, `"openai/gpt-4o"`, `"meta-llama/llama-3-70b"`) |
| **Custom headers** | `HTTP-Referer` and `X-Title` recommended for ranking/identification |

## Planned Phase

Per `CLAUDE.md`, OpenRouter falls under **Phase 4** ("Alternative providers, skills system, sandbox, cron scheduler, WhatsApp").
