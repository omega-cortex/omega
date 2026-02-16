# Technical Specification: `omega-skills/src/lib.rs`

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-skills/src/lib.rs` |
| **Crate** | `omega-skills` |
| **Role** | Crate root -- declares submodules and controls public API surface |

## Purpose

This file is the entry point for the `omega-skills` crate. It serves as the foundation for the planned skill and plugin system that will allow Omega to execute structured, composable actions beyond simple AI chat completion. Currently, the crate is scaffolding only -- it declares one private submodule and exposes no public types.

## Module Doc Comment

```rust
//! # omega-skills
//!
//! Skill and plugin system for Omega.
```

## Module Declarations

| Module | Visibility | Status | Description |
|--------|-----------|--------|-------------|
| `builtin` | `mod` (private) | Placeholder | Built-in skills for Omega. Contains only a doc comment in `builtin/mod.rs`; no public types or functions. |

### Source

The complete contents of `lib.rs`:

```rust
//! # omega-skills
//!
//! Skill and plugin system for Omega.

mod builtin;
```

## Re-exports

There are **no** `pub use` re-exports in `lib.rs`. Since the sole module `builtin` is declared with `mod` (not `pub mod`), nothing inside it is reachable from outside the crate.

## Public API Surface

The crate currently exposes **zero** public items. There are no public modules, structs, enums, traits, or functions. Any crate that depends on `omega-skills` can import it, but cannot access anything from it.

| Item | Kind | Access Path | Status |
|------|------|-------------|--------|
| (none) | -- | -- | -- |

## Feature Gates

There are **no** feature gates defined in `lib.rs` or in the crate's `Cargo.toml`. The `builtin` module is compiled unconditionally.

## Dependencies (from Cargo.toml)

All dependencies use workspace versions. They are declared in anticipation of the full skill system implementation.

| Dependency | Intended Usage |
|------------|----------------|
| `omega-core` | Core types, traits, error handling, config. Skills will consume `IncomingMessage`, return `OutgoingMessage`, and use `OmegaError`. |
| `tokio` | Async runtime. Skills will execute asynchronously. |
| `serde` / `serde_json` | Serialization for skill parameters, configuration, and output. |
| `tracing` | Structured logging within skill execution. |
| `thiserror` | Potential skill-specific error types (currently unused). |
| `anyhow` | Flexible error handling during skill development (currently unused). |
| `async-trait` | `#[async_trait]` for any skill trait definitions. |

## Submodule Details

### `builtin`

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-skills/src/builtin/mod.rs` |
| **Visibility** | Private (`mod builtin`) |
| **Status** | Placeholder |

The `builtin` module is organized as a directory module (`builtin/mod.rs`) rather than a single file (`builtin.rs`). This structure anticipates multiple built-in skill files being added as siblings inside `builtin/`, with `mod.rs` re-exporting them.

Complete contents of `builtin/mod.rs`:

```rust
//! Built-in skills for Omega.
```

No types, traits, functions, or sub-declarations exist.

## Implementation Status

| Aspect | Status |
|--------|--------|
| Crate root (`lib.rs`) | Scaffolded -- module doc comment and one `mod` declaration |
| `builtin` module | Scaffolded -- doc comment only |
| Skill trait definition | Not yet defined |
| Skill registry / dispatcher | Not yet defined |
| Built-in skills | Not yet implemented |
| Skill configuration in `config.toml` | Not yet wired (no `SkillsConfig` struct exists in `omega-core`) |
| Integration with gateway | Not yet wired |
| Tests | None |

## Planned Architecture (per CLAUDE.md)

Per the project roadmap, `omega-skills` is scheduled for **Phase 4** development. The intended design includes:

- **Skill trait** -- An async trait (analogous to `Provider` and `Channel` in `omega-core::traits`) that all skills implement. Expected methods: `name()`, `description()`, `execute()`, and possibly `matches()` for intent routing.
- **Built-in skills** -- A set of skills bundled with Omega (e.g., web search, file operations, calendar lookup, system status). These will live in the `builtin/` directory module.
- **Skill registry** -- A runtime registry that collects available skills and dispatches execution requests to the correct handler.
- **Gateway integration** -- The gateway event loop in `src/gateway.rs` will route certain messages or commands to the skill system rather than (or in addition to) the AI provider.

## Relationship to Other Crates

| Crate | Relationship |
|-------|-------------|
| `omega-core` | Dependency. Skills will use core types (`IncomingMessage`, `OutgoingMessage`, `OmegaError`, `Context`) and potentially a new `Skill` trait defined in `omega-core::traits`. |
| `omega-providers` | Peer. Skills may invoke providers for AI-assisted sub-tasks. |
| `omega-channels` | Peer. Skills do not interact with channels directly; the gateway mediates. |
| `omega-memory` | Peer. Skills may read from or write to memory for stateful operations. |
| `omega-sandbox` | Peer (also planned). Skills that execute commands will likely delegate to the sandbox for secure execution. |

## Notes

- The crate compiles and passes `cargo check` despite having no meaningful code. It is included in the workspace so that the project structure is established and the dependency graph is wired before implementation begins.
- The `builtin/mod.rs` directory module pattern suggests the team expects multiple built-in skill files, each declared as a submodule of `builtin`.
- Because the module is private and empty, `omega-skills` adds negligible compilation cost to the workspace.
