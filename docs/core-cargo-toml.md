# omega-core Crate Dependencies

## Path

`crates/omega-core/Cargo.toml`

## What is omega-core?

`omega-core` is the foundation of the Omega workspace. Every other crate in the project depends on it. It defines the shared vocabulary of the system: message types, provider and channel traits, configuration structures, error types, and prompt sanitization logic.

Because it sits at the bottom of the dependency tree, `omega-core` is deliberately conservative in what it pulls in. It includes only what is genuinely needed to define types, parse config, handle errors, and express async interfaces.

## Why These Dependencies?

### Async Runtime -- `tokio`

Omega is fully async. The `tokio` dependency (with the `full` feature set) provides the async runtime that underpins every I/O operation. Even though `omega-core` mostly defines traits rather than performing I/O itself, it needs `tokio` types available so that async trait methods can reference them.

### Serialization -- `serde`, `serde_json`, `toml`

Configuration is stored in TOML files. Messages and provider responses are exchanged as JSON. The `serde` ecosystem handles both:

- **`serde`** with the `derive` feature lets you write `#[derive(Serialize, Deserialize)]` on any struct, which is used extensively on config types, message types, and provider response types.
- **`serde_json`** handles JSON parsing for provider responses (e.g., Claude Code CLI outputs JSON).
- **`toml`** handles reading `config.toml` into Rust structs.

### Logging -- `tracing`

The project rule is: no `println!` in production code. All logging goes through the `tracing` crate, which provides structured, leveled logging. `omega-core` uses `tracing` macros like `info!`, `warn!`, and `error!` in its config loading and sanitization logic.

Note that `tracing-subscriber` (which actually outputs the logs to the console or a file) is **not** a dependency of `omega-core`. That responsibility belongs to the root binary, which sets up the subscriber once at startup.

### Error Handling -- `thiserror`, `anyhow`

Omega uses a two-layer error strategy:

- **`thiserror`** is for defining structured, typed error enums (like `OmegaError`). It generates `Display` and `Error` implementations from `#[error("...")]` attributes.
- **`anyhow`** is for propagating errors with context in application-level code. It lets you write `.context("failed to load config")?` to add human-readable context to any error.

Both are lightweight and widely used in the Rust ecosystem.

### Identifiers -- `uuid`

Messages, sessions, and conversations need unique identifiers. The `uuid` crate with the `v4` feature generates random UUIDs. The `serde` feature means these UUIDs can be serialized directly into JSON and TOML without manual conversion.

### Timestamps -- `chrono`

Every message and audit event carries a timestamp. `chrono` provides `DateTime<Utc>` and related types. The `serde` feature enables automatic serialization of timestamps.

### Async Traits -- `async-trait`

Rust does not yet fully support `async fn` in traits in stable Rust without some help. The `async-trait` macro bridges this gap, allowing trait definitions like:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn ask(&self, prompt: &str, context: &[Message]) -> Result<Response>;
}
```

This is used for both the `Provider` trait (AI backends) and the `Channel` trait (messaging platforms).

## What is NOT Here (and Why)

You might notice that several workspace dependencies are absent from `omega-core`:

| Crate                | Where it lives instead | Why |
|----------------------|----------------------|-----|
| `reqwest`            | `omega-providers`, `omega-channels` | HTTP is for API calls, not core types. |
| `sqlx`               | `omega-memory` | Database access is a storage concern. |
| `clap`               | Root binary | CLI argument parsing happens only in `main.rs`. |
| `tracing-subscriber` | Root binary | Log output setup is an application-level concern. |

This separation keeps `omega-core` lean. Fewer dependencies means faster compile times and a clearer boundary of responsibility.

## How to Add a New Dependency

Because all dependencies use workspace inheritance, adding a new dependency to `omega-core` is a two-step process:

1. **Add it to the workspace root** (`Cargo.toml` at the repo root) under `[workspace.dependencies]`:

   ```toml
   # In the root Cargo.toml
   [workspace.dependencies]
   regex = "1"
   ```

2. **Reference it from omega-core** (`crates/omega-core/Cargo.toml`) under `[dependencies]`:

   ```toml
   # In crates/omega-core/Cargo.toml
   [dependencies]
   regex = { workspace = true }
   ```

After adding, run `cargo check` to verify the dependency resolves correctly, then `cargo clippy --workspace` to confirm no new warnings are introduced.

Before adding a dependency, ask yourself:

- Does this belong in `omega-core`, or should it live in a more specific crate?
- Is this crate well-maintained and widely used?
- Does it add significant compile time?

Keeping `omega-core` minimal benefits the entire workspace.
