# Technical Specification: omega-providers/Cargo.toml

## File Path

```
backend/crates/omega-providers/Cargo.toml
```

## Purpose

This is the Cargo manifest for the `omega-providers` crate, which contains AI provider implementations for the Omega agent infrastructure. Providers are backend adapters that translate Omega's internal request/response types into calls to specific AI services (Claude Code CLI, Anthropic API, OpenAI API, Ollama, OpenRouter).

## Package Metadata

| Field        | Value                                       | Source    |
|--------------|---------------------------------------------|-----------|
| `name`       | `omega-providers`                           | Local     |
| `version`    | `0.1.0`                                     | Workspace |
| `edition`    | `2021`                                      | Workspace |
| `license`    | `MIT OR Apache-2.0`                         | Workspace |
| `repository` | `https://github.com/omega-cortex/omega`     | Workspace |
| `description`| `AI provider implementations for Omega`     | Local     |

All fields marked "Workspace" are inherited from `[workspace.package]` in the root `Cargo.toml` using the `field.workspace = true` syntax. Only `name` and `description` are defined locally, as these are unique to this crate.

## Workspace Inheritance

The crate uses Cargo's workspace inheritance feature (stabilized in Rust 1.64). When a dependency is declared as `{ workspace = true }`, its version, features, and other configuration are resolved from the `[workspace.dependencies]` table in the root `Cargo.toml`.

This means the `omega-providers` crate does **not** specify any dependency versions or feature flags directly. All version pinning and feature selection is centralized at the workspace level.

## Dependencies

### Internal Dependencies

| Dependency   | Workspace Definition              | Purpose                                    |
|--------------|-----------------------------------|--------------------------------------------|
| `omega-core` | `{ path = "crates/omega-core" }` | Core types, traits, config, error handling, prompt sanitization |

### External Dependencies

| Dependency    | Resolved Version | Feature Flags                | Purpose                                |
|---------------|------------------|------------------------------|----------------------------------------|
| `tokio`       | `1`              | `full`                       | Async runtime for subprocess and I/O   |
| `serde`       | `1`              | `derive`                     | Serialization/deserialization framework |
| `serde_json`  | `1`              | (none)                       | JSON parsing and generation            |
| `tracing`     | `0.1`            | (none)                       | Structured logging and diagnostics     |
| `thiserror`   | `2`              | (none)                       | Derive macro for error types           |
| `anyhow`      | `1`              | (none)                       | Flexible error handling                |
| `async-trait` | `0.1`            | (none)                       | Async functions in trait definitions   |
| `reqwest`     | `0.12`           | `json`, `rustls-tls`         | HTTP client for API-based providers    |

### Dependency Detail

**tokio** (version 1, features: `full`):
The `full` feature enables all Tokio sub-features including `rt-multi-thread`, `macros`, `io-util`, `net`, `time`, `process`, `signal`, `sync`, and `fs`. The `process` feature is particularly important for the Claude Code CLI provider, which spawns `claude` as a subprocess.

**serde** (version 1, features: `derive`):
The `derive` feature enables `#[derive(Serialize, Deserialize)]` on structs and enums. Used for provider configuration structs and API request/response payloads.

**serde_json** (version 1):
Used for parsing JSON responses from AI providers. The Claude Code CLI provider specifically parses JSON output structured as `{"type": "result", "subtype": "success", "result": "...", ...}`.

**reqwest** (version 0.12, features: `json`, `rustls-tls`):
- `json` -- enables `.json()` method on request builders and response objects for automatic serialization/deserialization.
- `rustls-tls` -- uses rustls for TLS instead of the system's OpenSSL, ensuring consistent TLS behavior across platforms without requiring OpenSSL to be installed.

**tracing** (version 0.1):
Structured logging framework. The project mandates tracing over `println!` for all logging.

**thiserror** (version 2):
Procedural macro for deriving `std::error::Error` on custom error enums. Used for provider-specific error types.

**anyhow** (version 1):
Provides `anyhow::Result` and `anyhow::Error` for ergonomic error propagation with context attachment via `.context()`.

**async-trait** (version 0.1):
Enables `async fn` in trait definitions. Required because Rust does not yet have native async trait support in stable (as of edition 2021). Used for the provider trait that all AI backends must implement.

## Feature Configuration

The `omega-providers` crate does **not** define any crate-level features (`[features]` section is absent). All provider implementations are compiled unconditionally. There is no conditional compilation gating specific providers behind feature flags.

## Resolver

The workspace uses Cargo resolver version `2` (set in the root `Cargo.toml` via `resolver = "2"`), which provides improved feature unification behavior for the dependency graph.

## Notes

- The crate has **no dev-dependencies** (`[dev-dependencies]` section is absent). Any tests in this crate rely solely on the regular dependencies.
- The crate has **no build dependencies** (`[build-dependencies]` section is absent). No build scripts are used.
- The crate has **no binary targets**. It is a library crate only.
