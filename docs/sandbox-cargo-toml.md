# omega-sandbox Crate Dependencies

## Path

`crates/omega-sandbox/Cargo.toml`

## What is omega-sandbox?

`omega-sandbox` is Omega's OS-level filesystem enforcement layer. It wraps the AI provider subprocess with platform-native write restrictions — Apple Seatbelt on macOS, Landlock LSM on Linux — so that even a confused AI cannot write files outside permitted directories.

The crate exports a single function, `sandboxed_command()`, which builds a `tokio::process::Command` with appropriate OS enforcement applied based on the sandbox mode.

## How Workspace Inheritance Works

Most dependencies use workspace inheritance:

```toml
tokio = { workspace = true }
```

This means the version, features, and other settings are declared once in the root `Cargo.toml` under `[workspace.dependencies]`. All crates in the workspace use the same version. Upgrades happen in one place.

Package metadata fields (`version`, `edition`, `license`, `repository`) are also inherited.

## Dependencies and What They Do

### Internal

| Dependency   | What It Provides                                                          |
|--------------|---------------------------------------------------------------------------|
| `omega-core` | `SandboxMode` enum, `OmegaError::Sandbox`, shared types and configuration |

### External

| Dependency    | Version | What It Is Used For                                    |
|---------------|---------|--------------------------------------------------------|
| `tokio`       | 1       | Async runtime, `tokio::process::Command` for subprocess management |
| `serde`       | 1       | Serialization of configuration types                    |
| `tracing`     | 0.1     | Structured logging for fallback warnings and sandbox events |
| `thiserror`   | 2       | Typed error definitions for sandbox failures            |
| `anyhow`      | 1       | Ergonomic error propagation in Landlock setup           |

### Platform-Specific

| Target | Dependency | Version | What It Is Used For                                |
|--------|-----------|---------|---------------------------------------------------|
| Linux  | `landlock` | 0.4     | Landlock LSM filesystem restrictions (kernel 5.13+) |

The `landlock` dependency is declared under `[target.'cfg(target_os = "linux")'.dependencies]` and is only compiled on Linux. It uses a direct version declaration because target-specific dependencies cannot currently use workspace inheritance.

The macOS implementation uses `sandbox-exec` which is a built-in macOS binary — no additional crate dependency is needed.

## What is NOT Here (and Why)

| Crate                | Why                                                |
|----------------------|----------------------------------------------------|
| `serde_json`         | No JSON parsing needed; typed structs suffice      |
| `reqwest`            | HTTP is for API calls, not sandbox enforcement     |
| `sqlx`               | Database access is in omega-memory                 |
| `uuid`               | No unique identifiers needed                       |
| `async-trait`        | No trait definitions; concrete functions only       |
| `libc`               | macOS uses sandbox-exec (external binary), Linux uses landlock crate |

## How to Add a New Dependency

### Standard dependency

1. Add to root `Cargo.toml` under `[workspace.dependencies]`
2. Reference in `crates/omega-sandbox/Cargo.toml` with `{ workspace = true }`

### Platform-specific dependency

Use Cargo's target-specific syntax:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
landlock = "0.4"
```

Note: target-specific dependencies cannot use workspace inheritance, so declare the version directly.

## Common Tasks

```bash
# Check that everything compiles
cargo check -p omega-sandbox

# Run clippy on just this crate
cargo clippy -p omega-sandbox

# See the full resolved dependency tree
cargo tree -p omega-sandbox

# Run tests for just this crate
cargo test -p omega-sandbox
```
