# omega-sandbox Crate Dependencies

## Path

`crates/omega-sandbox/Cargo.toml`

## What is omega-sandbox?

`omega-sandbox` is Omega's secure execution environment. Its job is to run commands and code in an isolated context so that the AI agent can perform actions on the host system without unrestricted access. Think of it as a safety layer between "the AI wants to run a shell command" and "that command actually executes on your machine."

This crate is currently a **Phase 4 scaffold** -- the `Cargo.toml` and a stub `lib.rs` are in place, but the implementation has not been built yet. The dependency list represents the minimal foundation needed to begin development. As the crate is implemented, additional dependencies will be added for process management, filesystem sandboxing, and resource limits.

The planned responsibilities include:

- **Process isolation** -- spawning commands in restricted environments with limited filesystem and network access.
- **Resource limits** -- enforcing timeouts, memory caps, and CPU constraints on executed commands.
- **Output capture** -- collecting stdout, stderr, and exit codes from sandboxed processes.
- **Security policy enforcement** -- checking commands against allow/deny lists before execution.

## How Workspace Inheritance Works

You will notice that every dependency in this crate looks like this:

```toml
tokio = { workspace = true }
```

This means the version, features, and other settings are **not** declared here. Instead, they are declared once in the root `Cargo.toml` under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
```

When you write `{ workspace = true }` in a crate, Cargo pulls in whatever version and features the workspace root defines. This gives us two important guarantees:

1. **Every crate in the workspace uses the same version** of a given dependency. No version conflicts.
2. **Upgrades happen in one place.** Bump the version in the root `Cargo.toml` and every crate picks it up.

Package metadata fields (`version`, `edition`, `license`, `repository`) are also inherited the same way:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/omega-cortex/omega"
```

## Dependencies and What They Do

### Internal

| Dependency   | What It Provides                                                          |
|--------------|---------------------------------------------------------------------------|
| `omega-core` | Shared types (`Message`, `Config`, `OmegaError`), traits, and configuration structures |

This is the only internal crate that `omega-sandbox` depends on. It does not depend on `omega-providers`, `omega-channels`, or `omega-memory` -- the gateway in `src/gateway.rs` wires those together at a higher level.

### External

| Dependency    | Version | What It Is Used For                                                                                  |
|---------------|---------|------------------------------------------------------------------------------------------------------|
| `tokio`       | 1       | The async runtime. Sandboxed processes will be spawned and managed asynchronously using `tokio::process::Command`, timers for enforcing timeouts, and I/O streams for capturing output. |
| `serde`       | 1       | Deriving `Serialize` and `Deserialize` on sandbox configuration structs, execution requests, and result types. |
| `tracing`     | 0.1     | Structured logging for sandbox events. The project uses `tracing` everywhere instead of `println!` or `log`. Sandbox operations will emit spans and events for process start, completion, timeout, and policy violations. |
| `thiserror`   | 2       | Defining typed error enums for sandbox-specific failure modes. Examples: `SandboxError::PermissionDenied`, `SandboxError::Timeout`, `SandboxError::ResourceLimitExceeded`. |
| `anyhow`      | 1       | Quick error propagation with `?` in functions where detailed error typing is not needed.              |

### The Minimal Set

You might notice that `omega-sandbox` has fewer dependencies than other Omega crates. This is intentional. The current manifest includes only what every Omega crate needs as a baseline:

- **`omega-core`** for shared types
- **`tokio`** for async
- **`serde`** for serialization
- **`tracing`** for logging
- **`thiserror`** and **`anyhow`** for error handling

These six dependencies form the "foundation set" that nearly every crate in the workspace declares. As implementation progresses, the sandbox will likely gain additional dependencies for its specific concerns (process isolation, filesystem restrictions, resource enforcement).

## What is NOT Here (and Why)

| Crate                | Where it lives instead              | Why                                                |
|----------------------|-------------------------------------|----------------------------------------------------|
| `serde_json`         | `omega-core`, `omega-memory`        | No JSON parsing needed yet; typed structs suffice. |
| `reqwest`            | `omega-providers`, `omega-channels` | HTTP is for API calls, not command execution.      |
| `sqlx`               | `omega-memory`                      | Database access is a storage concern.              |
| `uuid`               | `omega-core`, `omega-memory`        | May be added later for sandbox session IDs.        |
| `chrono`             | `omega-core`, `omega-memory`        | May be added later for execution timing.           |
| `async-trait`        | `omega-core`, `omega-channels`      | No trait definitions yet; will be added if needed. |
| `toml`               | `omega-core`                        | Config parsing is not a sandbox concern.           |
| `clap`               | Root binary                         | CLI argument parsing happens only in `main.rs`.    |
| `tracing-subscriber` | Root binary                         | Log output setup is an application-level concern.  |

This separation keeps `omega-sandbox` focused on one job: secure command execution. It does not parse config, handle HTTP, or manage databases.

## Dependencies That Will Likely Be Added

As Phase 4 implementation progresses, the following dependencies are anticipated:

| Crate      | Why It Will Be Needed                                                                 |
|------------|---------------------------------------------------------------------------------------|
| `uuid`     | Generating unique identifiers for sandbox sessions and execution requests.            |
| `chrono`   | Tracking execution start/end times, enforcing time-based limits.                      |
| `nix`      | Unix-specific process management: signals, namespaces, `setrlimit`, `seccomp`.        |
| `tempfile`  | Creating temporary directories for sandboxed filesystem roots.                       |
| `serde_json`| Serializing execution results if they need to be passed as JSON to the gateway.      |

Platform-specific dependencies may also be needed:

- **macOS**: Apple's `sandbox_init` API or the `sandbox-exec` profile system.
- **Linux**: `seccomp`, `cgroups`, or container runtimes.

## How to Add a New Dependency

Because all dependencies use workspace inheritance, adding a new dependency to `omega-sandbox` is a two-step process:

### Step 1: Add it to the workspace root

Open the root `Cargo.toml` and add the dependency under `[workspace.dependencies]`:

```toml
[workspace.dependencies]
# ... existing entries ...
my-new-crate = { version = "2.0", features = ["some-feature"] }
```

### Step 2: Reference it in the crate

Open `crates/omega-sandbox/Cargo.toml` and add:

```toml
[dependencies]
my-new-crate = { workspace = true }
```

That is it. The version and features come from the workspace definition.

### Step 3: If only this crate needs a feature

If `omega-sandbox` needs a feature that no other crate needs, you can add it locally without overriding the workspace version:

```toml
[dependencies]
my-new-crate = { workspace = true, features = ["extra-feature"] }
```

This merges `extra-feature` with whatever features the workspace already declares.

### Step 4: If the dependency is platform-specific

Sandbox implementations are often platform-specific. Use Cargo's target-specific dependency syntax:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
nix = { version = "0.28", features = ["process", "signal"] }
```

Note that target-specific dependencies cannot currently use workspace inheritance, so you will need to declare the version directly.

## When You Might Modify This File

Common scenarios where you would touch `crates/omega-sandbox/Cargo.toml`:

- **Implementing process isolation** -- adding `nix`, `libc`, or platform-specific crates for spawning restricted processes.
- **Adding filesystem sandboxing** -- adding `tempfile` for temporary sandbox roots, or filesystem overlay crates.
- **Adding resource limiting** -- adding crates for cgroups, `setrlimit`, or similar resource control APIs.
- **Adding test infrastructure** -- adding `[dev-dependencies]` for integration testing of sandbox behavior (e.g., `assert_cmd`, `predicates`).
- **Integrating with audit** -- if sandbox execution events need to be logged to the database, you might add `omega-memory` as a dependency (though this is more likely handled at the gateway level).

Before adding a dependency, ask yourself:

- Does this belong in `omega-sandbox`, or should it live in a more specific crate?
- Is this crate well-maintained and widely used?
- Does it add significant compile time?
- Is it cross-platform, or does it need target-specific handling?

Keeping `omega-sandbox` focused on secure execution benefits the entire workspace.

## Common Tasks

**Check that everything compiles:**

```bash
cargo check -p omega-sandbox
```

**Run clippy on just this crate:**

```bash
cargo clippy -p omega-sandbox
```

**See the full resolved dependency tree:**

```bash
cargo tree -p omega-sandbox
```

This shows every transitive dependency, which is useful for debugging version conflicts or understanding binary size.

**Run tests for just this crate:**

```bash
cargo test -p omega-sandbox
```
