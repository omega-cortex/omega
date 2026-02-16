# Technical Specification: `omega-providers/src/lib.rs`

## File

| Field | Value |
|-------|-------|
| Path | `crates/omega-providers/src/lib.rs` |
| Crate | `omega-providers` |
| Role | Crate root -- declares modules and controls public API surface |

## Purpose

`lib.rs` is the entry point for the `omega-providers` crate. It serves two purposes:

1. Declare each AI-provider module (one per backend).
2. Control which modules are publicly accessible to downstream crates.

The file itself contains no types, traits, or functions. Its entire job is module wiring and visibility.

## Module Declarations

| Module | Visibility | Status | Description |
|--------|-----------|--------|-------------|
| `anthropic` | `mod` (private) | Placeholder | Anthropic API provider. File contains only a doc comment. |
| `claude_code` | `pub mod` (public) | Implemented | Claude Code CLI provider. Fully functional -- invokes `claude` as a subprocess. |
| `ollama` | `mod` (private) | Placeholder | Ollama local model provider. File contains only a doc comment. |
| `openai` | `mod` (private) | Placeholder | OpenAI-compatible API provider. File contains only a doc comment. |
| `openrouter` | `mod` (private) | Placeholder | OpenRouter proxy provider. File contains only a doc comment. |

## Re-exports

There are **no** explicit `pub use` re-exports in `lib.rs`. The only module reachable from outside the crate is `claude_code`, which is declared with `pub mod`. Downstream consumers access its contents as:

```rust
use omega_providers::claude_code::ClaudeCodeProvider;
```

## Public API Surface

Because `claude_code` is the sole public module, the entire external API surface of this crate is determined by the public items in `claude_code.rs`. Those are:

| Item | Kind | Signature |
|------|------|-----------|
| `ClaudeCodeProvider` | `struct` | `pub struct ClaudeCodeProvider { .. }` (fields are private) |
| `ClaudeCodeProvider::new` | `fn` | `pub fn new() -> Self` |
| `ClaudeCodeProvider::from_config` | `fn` | `pub fn from_config(max_turns: u32, allowed_tools: Vec<String>) -> Self` |
| `ClaudeCodeProvider::check_cli` | `async fn` | `pub async fn check_cli() -> bool` |
| `Provider` impl | trait impl | `impl Provider for ClaudeCodeProvider` (methods: `name`, `requires_api_key`, `complete`, `is_available`) |
| `Default` impl | trait impl | `impl Default for ClaudeCodeProvider` (delegates to `new()`) |

The internal struct `ClaudeCliResponse` and the `#[cfg(test)] mod tests` are **not** part of the public API.

## Feature Gates

There are **no** Cargo feature gates defined in `omega-providers/Cargo.toml`. All modules are compiled unconditionally. No `#[cfg(feature = "...")]` attributes appear in `lib.rs` or any of the submodules.

## Dependencies

Declared in `Cargo.toml` (all workspace-level):

| Dependency | Usage |
|------------|-------|
| `omega-core` | `Provider` trait, `Context`, `OmegaError`, `OutgoingMessage`, `MessageMetadata` |
| `tokio` | `tokio::process::Command` for async subprocess execution |
| `serde` / `serde_json` | Deserialize JSON output from the `claude` CLI |
| `tracing` | `debug!` and `warn!` log macros |
| `thiserror` | (available but unused in current code) |
| `anyhow` | (available but unused in current code) |
| `async-trait` | `#[async_trait]` attribute on the `Provider` impl |
| `reqwest` | (available for HTTP-based providers; unused by `claude_code`) |

## Notes

- The four placeholder modules (`anthropic`, `ollama`, `openai`, `openrouter`) are kept private. They each contain a single doc comment and no executable code. They exist as scaffolding for Phase 4 development.
- Because these modules are private and empty, they do not increase binary size or affect compilation in any meaningful way.
- The crate does not define its own error type; it uses `OmegaError` from `omega-core`.
