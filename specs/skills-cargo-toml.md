# Technical Specification: omega-skills/Cargo.toml

## File Path

```
crates/omega-skills/Cargo.toml
```

## Purpose

This is the Cargo manifest for the `omega-skills` crate, which defines the skill and plugin system for the Omega agent infrastructure. Skills are modular units of functionality that extend what Omega can do beyond basic message-in/response-out interaction. The crate is currently in early development (Phase 4 of the project roadmap), with a `builtin` module stubbed out for future built-in skill implementations.

## Package Metadata

| Field        | Value                                       | Source    |
|--------------|---------------------------------------------|-----------|
| `name`       | `omega-skills`                              | Local     |
| `version`    | `0.1.0`                                     | Workspace |
| `edition`    | `2021`                                      | Workspace |
| `license`    | `MIT OR Apache-2.0`                         | Workspace |
| `repository` | `https://github.com/omega-cortex/omega`     | Workspace |
| `description`| `Skill and plugin system for Omega`         | Local     |

All fields marked "Workspace" are inherited from `[workspace.package]` in the root `Cargo.toml` using the `field.workspace = true` syntax. Only `name` and `description` are defined locally, as these are unique to this crate.

## Workspace Inheritance

The crate uses Cargo's workspace inheritance feature (stabilized in Rust 1.64). When a dependency is declared as `{ workspace = true }`, its version, features, and other configuration are resolved from the `[workspace.dependencies]` table in the root `Cargo.toml`.

The following package fields are inherited from `[workspace.package]`:

- `version` -- resolves to `0.1.0`
- `edition` -- resolves to `2021`
- `license` -- resolves to `MIT OR Apache-2.0`
- `repository` -- resolves to `https://github.com/omega-cortex/omega`

All dependencies are inherited from `[workspace.dependencies]` with no local overrides. This means version bumps and feature changes are performed exclusively in the root `Cargo.toml`.

## Dependencies

### Internal Dependencies

| Dependency   | Workspace Definition              | Purpose                                    |
|--------------|-----------------------------------|--------------------------------------------|
| `omega-core` | `{ path = "crates/omega-core" }` | Core types, traits, config, error handling, prompt sanitization |

### External Dependencies

| Dependency    | Resolved Version | Feature Flags          | Purpose                                |
|---------------|------------------|------------------------|----------------------------------------|
| `tokio`       | `1`              | `full`                 | Async runtime for skill execution      |
| `serde`       | `1`              | `derive`               | Serialization/deserialization framework |
| `serde_json`  | `1`              | (none)                 | JSON parsing and generation            |
| `tracing`     | `0.1`            | (none)                 | Structured logging and diagnostics     |
| `thiserror`   | `2`              | (none)                 | Derive macro for error types           |
| `anyhow`      | `1`              | (none)                 | Flexible error handling                |
| `async-trait` | `0.1`            | (none)                 | Async functions in trait definitions   |

### Dependency Detail

**omega-core** (internal, path dependency):
The foundational crate of the Omega workspace. Provides the `Provider` and `Channel` traits, message types, configuration structures, error types (`OmegaError`), and prompt sanitization logic. The skills crate depends on it for access to core types and traits that skills will interact with -- particularly message types, the configuration system, and error handling patterns.

**tokio** (version 1, features: `full`):
The `full` feature enables all Tokio sub-features including `rt-multi-thread`, `macros`, `io-util`, `net`, `time`, `process`, `signal`, `sync`, and `fs`. Skills may perform I/O operations (file access, network calls, subprocess execution), all of which require the async runtime. The `process` feature is relevant for skills that need to invoke external commands.

**serde** (version 1, features: `derive`):
The `derive` feature enables `#[derive(Serialize, Deserialize)]` on structs and enums. Used for skill configuration structs, skill metadata, and any data structures that skills need to serialize or deserialize (e.g., skill input/output payloads).

**serde_json** (version 1):
JSON parsing and generation. Skills will likely consume and produce JSON-formatted data when interacting with external APIs or when their input/output is structured data.

**tracing** (version 0.1):
Structured logging framework. The project mandates tracing over `println!` for all logging. Skills use `tracing` macros (`info!`, `warn!`, `error!`, `debug!`) for diagnostic output during execution.

**thiserror** (version 2):
Procedural macro for deriving `std::error::Error` on custom error enums. Used for skill-specific error types that provide structured, typed errors for different failure modes (e.g., skill not found, skill execution failed, invalid skill input).

**anyhow** (version 1):
Provides `anyhow::Result` and `anyhow::Error` for ergonomic error propagation with context attachment via `.context()`. Used in internal skill logic where defining a full custom error type for every helper function would be excessive.

**async-trait** (version 0.1):
Enables `async fn` in trait definitions. The skill system will define one or more traits that skill implementations must satisfy (e.g., a `Skill` trait with an async `execute` method). Each skill implementation uses `#[async_trait]` on its `impl` block.

### Dependency Count

- **Direct dependencies:** 8 (1 internal + 7 external)
- **Dev dependencies:** 0
- **Build dependencies:** 0

## Feature Configuration

The `omega-skills` crate does **not** define any crate-level features (`[features]` section is absent). All skill implementations are compiled unconditionally. As the number of skills grows, there may be value in gating individual skills behind feature flags to allow selective compilation.

## Resolver

The workspace uses Cargo resolver version `2` (set in the root `Cargo.toml` via `resolver = "2"`), which provides improved feature unification behavior for the dependency graph.

## Comparison With Other Workspace Crates

The `omega-skills` dependency set is a subset of what other crates use. Notably absent compared to sibling crates:

| Crate                | Present In             | Why Absent From omega-skills |
|----------------------|------------------------|------------------------------|
| `reqwest`            | `omega-providers`, `omega-channels` | No HTTP calls needed at the skill framework level. Individual skills that need HTTP would add it when the feature-gating pattern is introduced. |
| `sqlx`               | `omega-memory`         | Database access is a storage concern handled by `omega-memory`. Skills interact with storage through core abstractions, not directly. |
| `toml`               | `omega-core`           | Configuration parsing is handled by `omega-core`. Skills receive their configuration through the core config system. |
| `uuid`               | `omega-core`           | Identifier generation is handled at the core level. Skills receive identifiers rather than generating them. |
| `chrono`             | `omega-core`           | Timestamp handling is managed by core types. Skills receive timestamps on messages rather than creating them. |
| `clap`               | Root binary            | CLI argument parsing happens only in `main.rs`. |
| `tracing-subscriber` | Root binary            | Log output setup is an application-level concern. |

## Notes

- The crate has **no dev-dependencies** (`[dev-dependencies]` section is absent). Tests rely solely on the regular dependencies.
- The crate has **no build dependencies** (`[build-dependencies]` section is absent). No build scripts are used.
- The crate has **no binary targets**. It is a pure library crate.
- The crate is currently minimal, with only a `lib.rs` re-exporting a `builtin` module. The dependency set reflects the anticipated needs of the skill/plugin system as it matures during Phase 4 development.
- The dependency set mirrors `omega-providers` closely (minus `reqwest`), suggesting skills and providers share a similar architectural pattern of trait-based async implementations backed by the core type system.
