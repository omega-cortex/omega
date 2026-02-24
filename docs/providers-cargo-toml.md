# omega-providers: Cargo.toml Guide

This document explains the Cargo manifest for the `omega-providers` crate -- what each dependency does, how workspace inheritance works, and how to add new dependencies when building out additional AI providers.

## What Is This Crate?

`omega-providers` is the crate where all AI backend adapters live. Each provider takes an Omega request, translates it into whatever format the target AI service expects, makes the call, and returns a normalized response. The current primary provider is the Claude Code CLI, but the architecture supports Anthropic API, OpenAI, Ollama, and OpenRouter as well.

The file lives at:

```
backend/crates/omega-providers/Cargo.toml
```

## How Workspace Inheritance Works

You will notice that almost every field in this manifest says `workspace = true` rather than specifying a value directly:

```toml
[package]
name = "omega-providers"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
```

This means the actual values come from the root `Cargo.toml` under `[workspace.package]`:

```toml
# Root Cargo.toml
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/omega-cortex/omega"
```

The same pattern applies to dependencies. When you write:

```toml
tokio = { workspace = true }
```

Cargo looks up `tokio` in the root `[workspace.dependencies]` table and pulls in the version and feature flags defined there:

```toml
# Root Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
```

This keeps all version pinning in one place. If you need to bump `tokio` from version 1 to version 2 someday, you change it once in the root and every crate in the workspace picks it up.

## Dependencies Explained

### omega-core

```toml
omega-core = { workspace = true }
```

This is Omega's own core crate. It provides the shared types, traits (including the `Provider` trait that all AI backends implement), configuration structures, error types, and prompt sanitization. Every provider depends on it.

### tokio

```toml
tokio = { workspace = true }   # version 1, features = ["full"]
```

The async runtime that powers all of Omega. The Claude Code CLI provider uses Tokio's `process` module to spawn the `claude` binary as a subprocess and read its stdout asynchronously. API-based providers use it for async HTTP calls through reqwest.

### serde and serde_json

```toml
serde = { workspace = true }        # version 1, features = ["derive"]
serde_json = { workspace = true }   # version 1
```

Serialization framework. `serde` with the `derive` feature lets you put `#[derive(Serialize, Deserialize)]` on your structs. `serde_json` handles the actual JSON parsing and generation. You will use these for every provider that communicates via JSON -- which is essentially all of them.

### tracing

```toml
tracing = { workspace = true }   # version 0.1
```

Structured logging. The project rule is: no `println!`, use `tracing` instead. Typical usage in a provider:

```rust
tracing::info!("Sending request to Claude Code CLI");
tracing::debug!(response_len = body.len(), "Received response");
tracing::error!(%err, "Provider call failed");
```

### thiserror

```toml
thiserror = { workspace = true }   # version 2
```

Lets you derive `std::error::Error` on your own error enums with minimal boilerplate:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),
}
```

### anyhow

```toml
anyhow = { workspace = true }   # version 1
```

Provides `anyhow::Result<T>` for functions where you want to propagate errors without defining a custom error type for every situation. Useful in internal helper functions. For public API boundaries, prefer `thiserror`-based types.

### async-trait

```toml
async-trait = { workspace = true }   # version 0.1
```

Enables async functions in trait definitions. The `Provider` trait in `omega-core` uses this:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn send(&self, request: Request) -> Result<Response>;
}
```

Every provider implementation uses `#[async_trait]` on its `impl Provider for ...` block.

### reqwest

```toml
reqwest = { workspace = true }   # version 0.12, features = ["json", "rustls-tls"]
```

HTTP client used by API-based providers (Anthropic, OpenAI, Ollama, OpenRouter). The `json` feature gives you convenient `.json(&payload)` on request builders and `.json::<T>()` on responses. The `rustls-tls` feature means TLS is handled by rustls rather than OpenSSL, so you do not need OpenSSL installed on the build machine.

The Claude Code CLI provider does not use reqwest directly since it communicates via subprocess, but it is available for all HTTP-based providers.

## How to Add a New Dependency

Follow these steps:

**Step 1: Add it to the workspace root first.**

Open the root `Cargo.toml` and add the dependency under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
# ... existing deps ...
my-new-crate = { version = "3.0", features = ["something"] }
```

**Step 2: Reference it from this crate.**

In `backend/crates/omega-providers/Cargo.toml`, add:

```toml
[dependencies]
# ... existing deps ...
my-new-crate = { workspace = true }
```

**Step 3: If only this crate needs extra features,** you can extend the workspace definition:

```toml
my-new-crate = { workspace = true, features = ["extra-feature"] }
```

This adds `extra-feature` on top of whatever features the workspace already specifies.

**Step 4: Run `cargo check -p omega-providers`** to make sure everything resolves correctly.

## Things to Keep in Mind

- **No `[dev-dependencies]`** are defined yet. When you add tests, you may want to add `tokio-test` or `mockall` here.
- **No `[features]`** section exists. If the number of providers grows and you want to let users compile only the providers they need, consider gating each provider behind a feature flag (e.g., `openai`, `ollama`, `anthropic`).
- **No build script** is used. This crate compiles as a straightforward Rust library.
- Always run `cargo clippy --workspace && cargo test --workspace && cargo fmt --check` before committing changes to any `Cargo.toml` in the workspace.
