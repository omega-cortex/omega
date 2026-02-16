# omega-skills: Cargo.toml Guide

This document explains the Cargo manifest for the `omega-skills` crate -- what the crate does, what each dependency is for, how workspace inheritance works, and how to add new dependencies as the skill system grows.

## What Is This Crate?

`omega-skills` is where the skill and plugin system for Omega lives. While the core of Omega handles receiving messages, routing them to an AI provider, and sending back responses, the skills system is designed to give Omega the ability to _do things_ -- execute actions, call APIs, run commands, look up information, and generally extend beyond pure conversation.

The crate is currently in its early stages (Phase 4 of the project roadmap). It has a `builtin` module stubbed out for built-in skills that will ship with Omega, and the dependency set is already in place for building out the full skill framework.

The file lives at:

```
crates/omega-skills/Cargo.toml
```

## How Workspace Inheritance Works

You will notice that almost every field in this manifest says `workspace = true` rather than specifying a value directly:

```toml
[package]
name = "omega-skills"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Skill and plugin system for Omega"
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

Only `name` and `description` are defined locally, since those are unique to each crate and cannot be inherited.

## Dependencies Explained

### omega-core

```toml
omega-core = { workspace = true }
```

This is Omega's own core crate. It provides the shared types, traits, configuration structures, error types, and prompt sanitization logic that every crate in the workspace depends on. Skills need access to core types like messages, configuration, and error handling. The `Skill` trait (or whatever trait the skill system ultimately defines) will live alongside or extend the patterns established by core traits like `Provider` and `Channel`.

### tokio

```toml
tokio = { workspace = true }   # version 1, features = ["full"]
```

The async runtime that powers all of Omega. Skills are async by design -- they may need to make network calls, read files, execute subprocesses, or wait on timers. The `full` feature set ensures all of Tokio's capabilities are available, including `process` (for spawning commands), `net` (for network I/O), `time` (for timeouts and delays), and `fs` (for file system operations).

### serde and serde_json

```toml
serde = { workspace = true }        # version 1, features = ["derive"]
serde_json = { workspace = true }   # version 1
```

Serialization framework. `serde` with the `derive` feature lets you put `#[derive(Serialize, Deserialize)]` on any struct, which you will use for skill configuration, skill metadata, and input/output data structures. `serde_json` handles the actual JSON parsing and generation, which is relevant any time a skill needs to work with structured data from an API or produce structured output.

### tracing

```toml
tracing = { workspace = true }   # version 0.1
```

Structured logging. The project rule is: no `println!`, use `tracing` instead. Typical usage in a skill:

```rust
tracing::info!(skill = "weather", "Executing weather lookup");
tracing::debug!(city = %city, "Querying weather API");
tracing::error!(%err, "Skill execution failed");
```

Note that `tracing-subscriber` (which actually outputs the logs to the console or a file) is **not** a dependency of `omega-skills`. That responsibility belongs to the root binary, which sets up the subscriber once at startup.

### thiserror

```toml
thiserror = { workspace = true }   # version 2
```

Lets you derive `std::error::Error` on your own error enums with minimal boilerplate. You will use this for skill-specific error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Skill not found: {0}")]
    NotFound(String),

    #[error("Skill execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid skill input: {0}")]
    InvalidInput(String),
}
```

### anyhow

```toml
anyhow = { workspace = true }   # version 1
```

Provides `anyhow::Result<T>` for functions where you want to propagate errors without defining a custom error type for every situation. Useful in internal helper functions within skill implementations. For public API boundaries (like the skill trait itself), prefer `thiserror`-based types so callers can match on specific error variants.

### async-trait

```toml
async-trait = { workspace = true }   # version 0.1
```

Enables async functions in trait definitions. The skill system will define traits that skill implementations must satisfy, and those traits will have async methods. For example:

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    /// Human-readable name of this skill.
    fn name(&self) -> &str;

    /// Execute the skill with the given input.
    async fn execute(&self, input: &str) -> Result<String>;
}
```

Every skill implementation uses `#[async_trait]` on its `impl Skill for ...` block, just as providers use it for the `Provider` trait.

## What is NOT Here (and Why)

You might notice that several workspace dependencies are absent from `omega-skills`:

| Crate                | Where it lives instead          | Why |
|----------------------|---------------------------------|-----|
| `reqwest`            | `omega-providers`, `omega-channels` | HTTP is not needed at the skill framework level. When individual skills need to make HTTP calls, reqwest can be added here or gated behind a feature flag. |
| `sqlx`               | `omega-memory`                  | Database access goes through `omega-memory`, not directly from skills. |
| `toml`               | `omega-core`                    | Config parsing is handled by core. Skills receive their configuration already parsed. |
| `uuid`               | `omega-core`                    | Identifier generation happens at the core/gateway level, not in skills. |
| `chrono`             | `omega-core`                    | Timestamp handling is a core concern. Skills receive timestamped messages. |
| `clap`               | Root binary                     | CLI argument parsing happens only in `main.rs`. |
| `tracing-subscriber` | Root binary                     | Log output setup is an application-level concern. |

This separation keeps `omega-skills` lean. As the skill system evolves, some of these dependencies may be added if genuinely needed, but the principle is to start minimal and add only what is required.

## How to Add a New Dependency

Because all dependencies use workspace inheritance, adding a new dependency to `omega-skills` is a two-step process.

**Step 1: Add it to the workspace root first.**

Open the root `Cargo.toml` and add the dependency under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
# ... existing deps ...
my-new-crate = { version = "3.0", features = ["something"] }
```

If the dependency already exists in the workspace root (because another crate uses it), skip this step.

**Step 2: Reference it from this crate.**

In `crates/omega-skills/Cargo.toml`, add:

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

**Step 4: Verify everything compiles.**

```bash
cargo check -p omega-skills
cargo clippy --workspace
cargo test --workspace
```

### Before Adding a Dependency, Ask Yourself

- **Does this belong in `omega-skills`, or should it live in a more specific crate?** If only one skill needs it, consider whether the skill itself should be a separate crate or whether the dependency should be feature-gated.
- **Is this crate well-maintained and widely used?** Prefer established crates from the Rust ecosystem.
- **Does it add significant compile time?** The skill crate is compiled as part of the full workspace, so heavy dependencies affect everyone.
- **Could `omega-core` already provide what you need?** Check core types and traits before pulling in something new.

## Future Considerations

As the skill system matures, there are several patterns that may be introduced:

- **Feature flags for individual skills.** If some skills pull in heavy dependencies (e.g., a skill that does image processing with `image`), gating them behind feature flags allows users to compile only what they need.
- **Dev dependencies for testing.** The crate currently has no `[dev-dependencies]`. When skill tests are added, consider `tokio-test` for async test utilities or `mockall` for mocking core traits.
- **A `[build-dependencies]` section** is unlikely to be needed, but could appear if skill metadata requires build-time code generation.

## Things to Keep in Mind

- Always run `cargo clippy --workspace && cargo test --workspace && cargo fmt --check` before committing changes to any `Cargo.toml` in the workspace.
- The crate is currently a stub with a `lib.rs` and an empty `builtin` module. The dependency set is forward-looking -- it reflects what the skill system will need as it is built out during Phase 4.
- The dependency set closely mirrors `omega-providers` (minus `reqwest`), which is intentional. Skills and providers share a similar architectural pattern: trait-based, async, backed by the core type system, with structured errors and logging.
