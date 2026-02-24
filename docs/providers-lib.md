# omega-providers Crate Guide

## What This Crate Does

`omega-providers` houses all the AI backend integrations for Omega. Each backend (Claude Code CLI, Anthropic API, OpenAI, Ollama, OpenRouter) lives in its own module. The crate root (`lib.rs`) wires them together and decides which modules are visible to the rest of the workspace.

Right now, only one provider is fully implemented: **Claude Code CLI**. The others exist as placeholder modules, ready for Phase 4 development.

## Crate Structure

```
backend/crates/omega-providers/src/
  lib.rs            # Crate root -- module declarations only
  claude_code.rs    # Claude Code CLI provider (complete)
  anthropic.rs      # Anthropic API (placeholder)
  openai.rs         # OpenAI-compatible API (placeholder)
  ollama.rs         # Ollama local models (placeholder)
  openrouter.rs     # OpenRouter proxy (placeholder)
```

`lib.rs` is intentionally minimal. It declares the five modules and makes only `claude_code` public:

```rust
mod anthropic;
pub mod claude_code;
mod ollama;
mod openai;
mod openrouter;
```

Everything else -- the struct definitions, trait implementations, helper types -- lives inside the individual module files.

## Using a Provider

From any other crate in the workspace:

```rust
use omega_providers::claude_code::ClaudeCodeProvider;

let provider = ClaudeCodeProvider::new();

// Or with custom settings:
let provider = ClaudeCodeProvider::from_config(
    5,                                          // max agentic turns
    vec!["Bash".into(), "Read".into()],         // allowed tools
);

// Check if the `claude` CLI is installed:
if ClaudeCodeProvider::check_cli().await {
    // ready to go
}
```

Every provider implements the `Provider` trait from `omega-core`, so the gateway can work with any backend through a uniform interface:

```rust
use omega_core::traits::Provider;

async fn ask(provider: &dyn Provider, context: &Context) -> Result<OutgoingMessage, OmegaError> {
    provider.complete(context).await
}
```

## How to Add a New Provider

Follow these steps to turn one of the placeholder modules into a working provider (or add an entirely new one).

### 1. Create or edit the module file

If you are implementing an existing placeholder (say `anthropic.rs`), open it and replace the single doc comment with your implementation. If you are adding a brand-new backend, create a new file like `my_provider.rs` in the `backend/src/` directory.

### 2. Define your provider struct

```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,
}
```

Keep the fields private. Expose construction through `new()` and/or `from_config()` methods.

### 3. Implement the `Provider` trait

```rust
use async_trait::async_trait;
use omega_core::{context::Context, error::OmegaError, message::OutgoingMessage, traits::Provider};

#[async_trait]
impl Provider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }
    fn requires_api_key(&self) -> bool { true }

    async fn complete(&self, context: &Context) -> Result<OutgoingMessage, OmegaError> {
        // Call the Anthropic API, parse the response, return an OutgoingMessage.
        todo!()
    }

    async fn is_available(&self) -> bool {
        // Return true if the API key is set and the service is reachable.
        todo!()
    }
}
```

The four trait methods you must implement:

| Method | Purpose |
|--------|---------|
| `name()` | Short, human-readable identifier (used in logs and metadata). |
| `requires_api_key()` | Whether this provider needs an API key to function. |
| `complete()` | Send a `Context` to the backend and return an `OutgoingMessage`. |
| `is_available()` | Health check -- can this provider handle requests right now? |

### 4. Register the module in `lib.rs`

If your module is already listed (like `anthropic`), just change it from `mod` to `pub mod` so downstream crates can use it:

```rust
pub mod anthropic;      // was: mod anthropic;
pub mod claude_code;
mod ollama;
mod openai;
mod openrouter;
```

If you created a brand-new file, add a new `pub mod` line:

```rust
pub mod my_provider;
```

### 5. Add dependencies if needed

If your provider needs an HTTP client, `reqwest` is already in `Cargo.toml`. For anything else, add it to `backend/crates/omega-providers/Cargo.toml` using workspace dependencies where possible.

### 6. Write tests

Add a `#[cfg(test)] mod tests` block at the bottom of your module file. At minimum, test construction and the metadata methods (`name`, `requires_api_key`). See `claude_code.rs` for an example.

## Design Decisions

**Why is only `claude_code` public?** The other four modules are empty placeholders. Keeping them private avoids exposing an empty API surface and signals to consumers that those providers are not yet ready.

**Why no `pub use` re-exports?** The crate currently has only one public module, so a re-export would save very little typing. When multiple providers are implemented, it may make sense to add convenience re-exports like `pub use claude_code::ClaudeCodeProvider;` at the crate root.

**Why no feature gates?** With only one real provider, conditional compilation adds complexity without benefit. As the provider count grows, feature gates (e.g., `features = ["anthropic", "openai"]`) could be introduced to keep the dependency tree lean for users who only need one backend.

**Why `reqwest` if only `claude_code` uses subprocesses?** It is declared in `Cargo.toml` in anticipation of the HTTP-based providers (Anthropic, OpenAI, Ollama, OpenRouter). The Claude Code provider does not use it.
