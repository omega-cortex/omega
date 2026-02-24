# omega-core Cargo.toml -- Technical Specification

## Path

`backend/crates/omega-core/Cargo.toml`

## Purpose

Defines the `omega-core` crate, the foundational library in the Omega workspace. This crate provides core types, traits, configuration parsing, prompt sanitization, and error handling used by every other crate in the workspace.

## Package Metadata

All package metadata fields are inherited from the workspace root (`Cargo.toml` at repository root).

| Field        | Value                | Source              |
|--------------|----------------------|---------------------|
| `name`       | `omega-core`         | Local               |
| `version`    | `0.1.0`              | `workspace = true`  |
| `edition`    | `2021`               | `workspace = true`  |
| `license`    | `MIT OR Apache-2.0`  | `workspace = true`  |
| `repository` | `https://github.com/omega-cortex/omega` | `workspace = true` |
| `description`| Core types, traits, config, and error handling for Omega | Local |

## Dependencies

All dependencies use `workspace = true`, meaning their versions and feature flags are centrally managed in `[workspace.dependencies]` in the root `Cargo.toml`. The crate declares no local version overrides or additional features.

### Dependency Table

| Crate          | Workspace Version | Features Enabled         | Role in omega-core |
|----------------|-------------------|--------------------------|---------------------|
| `tokio`        | `1`               | `full`                   | Async runtime. Provides the executor, timers, I/O primitives, and synchronization used throughout the crate's async trait definitions. |
| `serde`        | `1`               | `derive`                 | Serialization/deserialization framework. The `derive` feature enables `#[derive(Serialize, Deserialize)]` on core types and config structs. |
| `serde_json`   | `1`               | (none)                   | JSON parsing and generation. Used for provider response deserialization and message payloads. |
| `toml`         | `0.8`             | (none)                   | TOML configuration file parsing. Reads `config.toml` into strongly typed Rust structs via serde. |
| `tracing`      | `0.1`             | (none)                   | Structured logging and diagnostics. All logging in core uses `tracing` macros (`info!`, `warn!`, `error!`, `debug!`), never `println!`. |
| `thiserror`    | `2`               | (none)                   | Ergonomic custom error type derivation. Powers the `#[derive(Error)]` macros on `OmegaError` and related types. |
| `anyhow`       | `1`               | (none)                   | Flexible error handling with context. Used for propagating errors with `?` and attaching contextual messages. |
| `uuid`         | `1`               | `v4`, `serde`            | Universally unique identifiers. `v4` enables random UUID generation for message and session IDs. `serde` enables serializing UUIDs in JSON/TOML. |
| `chrono`       | `0.4`             | `serde`                  | Date and time handling. Timestamps on messages, audit events, and conversation boundaries. `serde` enables serializing `DateTime` values. |
| `async-trait`  | `0.1`             | (none)                   | Procedural macro enabling `async fn` in trait definitions. Used for the `Provider` and `Channel` traits that all backends and platforms implement. |

### Dependency Count

- **Direct dependencies:** 10
- **Dev dependencies:** 0
- **Build dependencies:** 0

## Features

The crate declares no Cargo features of its own. All feature configuration is handled at the workspace dependency level.

## Workspace Inheritance

The following fields are inherited from `[workspace.package]`:

- `version`
- `edition`
- `license`
- `repository`

All dependencies are inherited from `[workspace.dependencies]` with no local overrides. This means version bumps and feature changes are performed exclusively in the root `Cargo.toml`.

## Notes

- The crate does **not** depend on `reqwest`, `sqlx`, `clap`, or `tracing-subscriber`. Those are used by other workspace crates (`omega-providers`, `omega-memory`, the root binary) but are not needed at the core type/trait level.
- The crate does **not** declare any `[[bin]]`, `[[example]]`, or `[[bench]]` targets. It is a pure library crate.
- No `[dev-dependencies]` or `[build-dependencies]` sections are present.
