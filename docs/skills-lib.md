# omega-skills -- Developer Guide

## What is this crate?

`omega-skills` is the skill and plugin layer of the Omega workspace. It will provide a framework for defining, registering, and executing structured actions that go beyond basic AI chat completion -- things like web searches, file operations, system queries, and any other composable task Omega needs to perform.

Today, the crate is a **placeholder**. It compiles, it is part of the workspace, and its dependency graph is wired, but it contains no executable code. It is scheduled for implementation in Phase 4 of the Omega roadmap.

## Crate Structure

```
crates/omega-skills/
  Cargo.toml
  src/
    lib.rs              <-- you are here
    builtin/
      mod.rs            <-- built-in skills (placeholder)
```

`lib.rs` is four lines long:

```rust
//! # omega-skills
//!
//! Skill and plugin system for Omega.

mod builtin;
```

That is all it does right now. The `builtin` module is equally minimal -- a single doc comment.

## Current Status

| What | Status |
|------|--------|
| Crate structure | Set up and compiling |
| Public API | None -- nothing is exported |
| Skill trait | Not yet defined |
| Built-in skills | Not yet implemented |
| Config integration | Not yet wired |
| Gateway integration | Not yet wired |
| Tests | None |

The crate exists so that the workspace structure is established early. All dependencies (`omega-core`, `tokio`, `serde`, `async-trait`, etc.) are declared in `Cargo.toml` and ready for use.

## How Visibility Works

- **`mod builtin`** -- The `builtin` module is private. It compiles, but nothing inside it is reachable from outside the crate. This follows the same convention used in `omega-providers` and `omega-channels` for modules that are still under development.

There are no `pub mod` declarations and no `pub use` re-exports. Any crate that depends on `omega-skills` can import the crate, but cannot access any types from it yet.

## What Will This Crate Provide?

Based on the project architecture (see `CLAUDE.md`), the planned components are:

### A Skill Trait

An async trait similar to `Provider` and `Channel` in `omega-core::traits`. Each skill will implement this trait. Expected shape:

```rust
use async_trait::async_trait;
use omega_core::error::OmegaError;

#[async_trait]
pub trait Skill: Send + Sync {
    /// Human-readable name of this skill.
    fn name(&self) -> &str;

    /// Short description of what this skill does.
    fn description(&self) -> &str;

    /// Check whether the skill can handle the given input.
    fn matches(&self, input: &str) -> bool;

    /// Execute the skill and return a result string.
    async fn execute(&self, input: &str) -> Result<String, OmegaError>;
}
```

This is speculative -- the actual trait will be defined during Phase 4 implementation.

### Built-in Skills

The `builtin/` directory module is structured to hold multiple skill files. Each built-in skill will live in its own file inside `builtin/`, declared as a submodule in `builtin/mod.rs`. Possible candidates:

- System status / health check
- Web search
- File operations (read, write, list)
- Timer / reminder
- Calculator / unit conversion

### Skill Registry

A runtime component that:

1. Collects all available skills (both built-in and potentially user-defined).
2. Matches incoming requests to the appropriate skill.
3. Dispatches execution and returns results.

### Gateway Integration

The gateway event loop (`src/gateway.rs`) will gain a skill dispatch step. Certain messages or commands (e.g., `/search`, `/status`) will route to the skill system instead of -- or in addition to -- the AI provider.

## How to Add a Skill (Future)

When the skill system is implemented, the process for adding a new built-in skill will follow a pattern similar to adding providers or channels:

### 1. Create the skill file

Create a new file inside the `builtin/` directory:

```
crates/omega-skills/src/builtin/my_skill.rs
```

### 2. Implement the Skill trait

```rust
use async_trait::async_trait;
use omega_core::error::OmegaError;
use crate::Skill; // once the trait exists

pub struct MySkill {
    // configuration fields
}

impl MySkill {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
}

#[async_trait]
impl Skill for MySkill {
    fn name(&self) -> &str { "my-skill" }

    fn description(&self) -> &str { "Does something useful" }

    fn matches(&self, input: &str) -> bool {
        input.starts_with("/my-skill")
    }

    async fn execute(&self, input: &str) -> Result<String, OmegaError> {
        // Your skill logic here
        Ok("Result".to_string())
    }
}
```

### 3. Register it in `builtin/mod.rs`

```rust
pub mod my_skill;
pub use my_skill::MySkill;
```

### 4. Add it to the skill registry

The registry (once it exists) will collect all built-in skills during initialization. Your skill will be instantiated and registered there.

## Dependencies

All dependencies are declared at workspace level:

| Dependency | Why it is here |
|------------|----------------|
| `omega-core` | Core types and traits. Skills will use `OmegaError`, `IncomingMessage`, `OutgoingMessage`, and `Context`. |
| `tokio` | Async runtime for skill execution. |
| `serde` / `serde_json` | Serialization for skill parameters and output. |
| `tracing` | Structured logging within skills. |
| `thiserror` | For defining skill-specific error types if needed. |
| `anyhow` | Flexible error handling. |
| `async-trait` | Enables async methods in trait definitions. |

## Design Notes

- **Directory module for builtins** -- The `builtin/mod.rs` pattern (rather than a single `builtin.rs` file) is deliberate. It signals that multiple skill files will be added as the system grows, each as a submodule of `builtin`.

- **No feature gates** -- Like the other crates in the workspace, all modules compile unconditionally. Feature gates may be introduced if the skill set grows large and users want to select which skills to include.

- **Private until ready** -- The `builtin` module is private, following the same convention as placeholder modules in `omega-providers` (e.g., `mod anthropic`) and `omega-channels` (e.g., `mod whatsapp`). It will be promoted to `pub mod` once it contains working code.

- **Errors use `OmegaError`** -- The crate does not define its own error type. Like all other Omega crates, it will use `OmegaError` from `omega-core`, most likely the `OmegaError::Provider(String)` variant or a new skill-specific variant if one is added.

- **Relationship to `omega-sandbox`** -- Skills that execute system commands will likely delegate to `omega-sandbox` for secure, sandboxed execution rather than running commands directly. Both crates are scheduled for Phase 4.

## Quick Reference

| You want to... | Where to look |
|----------------|---------------|
| Understand the project roadmap | `CLAUDE.md` -- Phase 4 section |
| See similar crate patterns | `omega-providers/src/lib.rs`, `omega-channels/src/lib.rs` |
| See the core trait interfaces | `omega-core/src/traits.rs` -- `Provider` and `Channel` traits |
| Check config structure | `omega-core/src/config.rs` -- no `SkillsConfig` yet |
| See how gateway dispatches | `src/gateway.rs` -- skill dispatch will be added here |
