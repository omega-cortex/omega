# Technical Specification: `openai.rs`

**File:** `crates/omega-providers/src/openai.rs`
**Crate:** `omega-providers`
**Module path:** `omega_providers::openai`
**Visibility:** Private (`mod openai;` in `lib.rs` -- not `pub mod`)
**Status:** Placeholder (stub). The file contains only a module-level doc comment and no executable code.

---

## Contents

The entire file consists of a single line:

```rust
//! OpenAI-compatible API provider (placeholder).
```

There are no structs, enums, trait implementations, functions, constants, imports, or tests.

---

## Structs

None defined.

---

## Trait Implementations

None defined. A complete implementation would need to implement the `Provider` trait from `omega_core::traits`.

---

## Functions

None defined.

---

## Required Trait: `Provider`

The `Provider` trait is defined in `crates/omega-core/src/traits.rs`. Any OpenAI provider struct must implement the following:

| Method | Signature | Return Type | Description |
|--------|-----------|-------------|-------------|
| `name` | `fn name(&self) -> &str` | `&str` | Human-readable provider identifier (e.g., `"openai"`). |
| `requires_api_key` | `fn requires_api_key(&self) -> bool` | `bool` | Must return `true` for OpenAI (API key is required). |
| `complete` | `async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError>` | `Result<OutgoingMessage, OmegaError>` | Send conversation context to the OpenAI API and return a response. |
| `is_available` | `async fn is_available(&self) -> bool` | `bool` | Check reachability of the OpenAI API endpoint. |

The trait requires `Send + Sync` bounds and uses `#[async_trait]`.

---

## Related Configuration

The config struct for this provider already exists in `crates/omega-core/src/config.rs`:

### `OpenAiConfig`

| Field | Type | Serde Default | Default Value | Description |
|-------|------|---------------|---------------|-------------|
| `enabled` | `bool` | `false` | `false` | Whether the OpenAI provider is active. |
| `api_key` | `String` | `""` | `""` (empty) | API key. Can also come from `OPENAI_API_KEY` env var. |
| `model` | `String` | `default_openai_model()` | `"gpt-4o"` | Model identifier to use. |
| `base_url` | `String` | `default_openai_base_url()` | `"https://api.openai.com/v1"` | API base URL. Supports OpenAI-compatible endpoints. |

Derives: `Debug`, `Clone`, `Serialize`, `Deserialize`.

The config is accessed via `Config.provider.openai` (type `Option<OpenAiConfig>`).

---

## Related Types (from `omega-core`)

### Input: `Context` (`omega_core::context`)

| Field | Type | Description |
|-------|------|-------------|
| `system_prompt` | `String` | System-level instruction prepended to every request. |
| `history` | `Vec<ContextEntry>` | Prior conversation turns (oldest first). |
| `current_message` | `String` | The latest user message. |

### `ContextEntry`

| Field | Type | Description |
|-------|------|-------------|
| `role` | `String` | `"user"` or `"assistant"`. |
| `content` | `String` | Message text. |

### Output: `OutgoingMessage` (`omega_core::message`)

| Field | Type | Description |
|-------|------|-------------|
| `text` | `String` | Response text to send back to the user. |
| `metadata` | `MessageMetadata` | Provider name, token count, timing, model. |
| `reply_target` | `Option<String>` | Platform-specific routing target (set by the gateway, not the provider). |

### `MessageMetadata`

| Field | Type | Description |
|-------|------|-------------|
| `provider_used` | `String` | Should be `"openai"`. |
| `tokens_used` | `Option<u64>` | Token usage from the API response. |
| `processing_time_ms` | `u64` | Wall-clock request duration in milliseconds. |
| `model` | `Option<String>` | Model identifier from the response. |

### Error: `OmegaError` (`omega_core::error`)

Provider errors should use the `OmegaError::Provider(String)` variant.

---

## Module Registration

In `crates/omega-providers/src/lib.rs`:

```rust
mod openai;
```

The module is private. No types are re-exported from the crate root. Once a provider struct is created, it would need to be made `pub` and re-exported (e.g., `pub mod openai;` or a `pub use` statement).

---

## Configuration in `config.example.toml`

```toml
[provider.openai]
enabled = false
api_key = ""  # Or env: OPENAI_API_KEY
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
```

Environment variable override: `OPENAI_API_KEY` maps to `provider.openai.api_key`.

---

## Comparison with Implemented Provider (`ClaudeCodeProvider`)

| Aspect | `ClaudeCodeProvider` (implemented) | `OpenAiProvider` (not yet created) |
|--------|------------------------------------|------------------------------------|
| File size | ~210 lines | 1 line (doc comment only) |
| Struct | `ClaudeCodeProvider` with 3 fields | None |
| `Provider` impl | Complete | None |
| Tests | 1 unit test | None |
| Transport | Subprocess (`tokio::process::Command`) | HTTP (would use `reqwest` or similar) |
| Auth | None (uses local `claude` CLI auth) | API key in `Authorization` header |
| Response parsing | Custom `ClaudeCliResponse` JSON struct | Would parse OpenAI chat completions JSON |
| Module visibility | `pub mod claude_code` | `mod openai` (private) |
