# Technical Specification: omega-channels/Cargo.toml

## File

| Field       | Value                                                        |
|-------------|--------------------------------------------------------------|
| Path        | `backend/crates/omega-channels/Cargo.toml`                           |
| Crate name  | `omega-channels`                                             |
| Description | Messaging platform integrations for Omega                    |
| Role        | Defines the build manifest for the `omega-channels` crate    |

## Package Metadata

All package metadata fields are inherited from the workspace root (`Cargo.toml`).

| Field        | Value (inherited)                              |
|--------------|------------------------------------------------|
| `version`    | `0.1.0`                                        |
| `edition`    | `2021`                                         |
| `license`    | `MIT OR Apache-2.0`                            |
| `repository` | `https://github.com/omega-cortex/omega`        |

## Workspace Inheritance

The crate uses `workspace = true` for every dependency and all package metadata fields. No dependency versions or feature flags are declared locally; all resolution is deferred to `[workspace.dependencies]` in the root `Cargo.toml`. This guarantees version consistency across the entire workspace.

## Dependencies

### Internal Crate Dependencies

| Dependency   | Workspace Ref          | Resolved Path           |
|--------------|------------------------|-------------------------|
| `omega-core` | `{ workspace = true }` | `backend/crates/omega-core`     |

### External Dependencies

| Dependency    | Workspace Version | Features                  | Purpose in Channel Crate                          |
|---------------|-------------------|---------------------------|---------------------------------------------------|
| `tokio`       | `1`               | `full`                    | Async runtime for all I/O operations              |
| `serde`       | `1`               | `derive`                  | Serialization/deserialization of message types     |
| `serde_json`  | `1`               | --                        | JSON parsing for platform API payloads            |
| `tracing`     | `0.1`             | --                        | Structured logging (project-wide standard)        |
| `thiserror`   | `2`               | --                        | Typed error definitions for channel errors        |
| `anyhow`      | `1`               | --                        | Ergonomic error propagation                       |
| `async-trait` | `0.1`             | --                        | Async method support in trait definitions         |
| `reqwest`     | `0.12`            | `json`, `rustls-tls`      | HTTP client for platform API calls                |
| `uuid`        | `1`               | `v4`, `serde`             | Unique identifiers for messages and conversations |
| `chrono`      | `0.4`             | `serde`                   | Timestamp handling for messages                   |

### Dependencies NOT Used by This Crate

The following workspace dependencies exist but are not declared in `omega-channels`:

| Dependency            | Reason for Exclusion                                         |
|-----------------------|--------------------------------------------------------------|
| `toml`                | Config parsing handled by `omega-core`                       |
| `sqlx`                | Database access handled by `omega-memory`                    |
| `tracing-subscriber`  | Subscriber setup handled by the root binary                  |
| `clap`                | CLI argument parsing handled by the root binary              |

## Feature Configuration

The `omega-channels` crate defines **no local features**. All feature flags are inherited transitively through workspace dependency declarations. There is no `[features]` section in this manifest.

## Dependency Graph (Direct)

```
omega-channels
  +-- omega-core (internal, workspace path)
  +-- tokio 1 [full]
  +-- serde 1 [derive]
  +-- serde_json 1
  +-- tracing 0.1
  +-- thiserror 2
  +-- anyhow 1
  +-- async-trait 0.1
  +-- reqwest 0.12 [json, rustls-tls]
  +-- uuid 1 [v4, serde]
  +-- chrono 0.4 [serde]
```

## Resolver

The workspace uses Cargo resolver version `2` (set in the root `Cargo.toml`), which is required for edition 2021 workspaces and provides improved feature unification behavior.
