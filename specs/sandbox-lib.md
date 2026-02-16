# Technical Specification: omega-sandbox/src/lib.rs

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-sandbox/src/lib.rs` |
| **Crate** | `omega-sandbox` |
| **Role** | Crate root -- placeholder for the secure command execution environment |

## Purpose

`omega-sandbox` is the secure execution environment for the Omega agent. Its responsibility is to provide a controlled, resource-limited context in which shell commands and scripts can be run on behalf of the user, with strict guardrails around which commands are allowed, which filesystem paths are accessible, and how much time and output a single execution may consume.

The crate is currently a **placeholder**. The `lib.rs` file contains only a module-level doc comment and no types, traits, functions, or submodules. Implementation is planned for Phase 4 of the project roadmap.

## Current Contents

The entire file consists of a single doc comment:

```rust
//! # omega-sandbox
//!
//! Secure execution environment for Omega.
```

### Module Declarations

None.

### Public Types

None.

### Public Functions

None.

### Traits

None.

### Tests

None.

---

## Dependencies (Cargo.toml)

The crate's `Cargo.toml` already declares the dependencies it will need once implementation begins:

| Dependency | Workspace | Planned Usage |
|------------|-----------|---------------|
| `omega-core` | Yes | Access to `SandboxConfig`, `OmegaError::Sandbox`, and shared types |
| `tokio` | Yes | Async command execution (`tokio::process::Command`), timeouts, task spawning |
| `serde` | Yes | Serialization of execution results and configuration |
| `tracing` | Yes | Structured logging of command execution, policy decisions, and errors |
| `thiserror` | Yes | Potential sandbox-specific error subtypes (or direct use of `OmegaError::Sandbox`) |
| `anyhow` | Yes | Ergonomic error handling during development |

---

## Configuration Surface

The sandbox is already fully configurable through `omega-core::config::SandboxConfig`:

```rust
pub struct SandboxConfig {
    pub enabled: bool,                    // default: true
    pub allowed_commands: Vec<String>,    // default: [] (empty = allow all, or deny all -- TBD)
    pub blocked_paths: Vec<String>,       // default: []
    pub max_execution_time_secs: u64,     // default: 30
    pub max_output_bytes: usize,          // default: 1_048_576 (1 MiB)
}
```

The example configuration (`config.example.toml`) demonstrates a typical setup:

```toml
[sandbox]
enabled = true
allowed_commands = ["ls", "cat", "grep", "find", "git", "cargo", "npm", "python"]
blocked_paths = ["/etc/shadow", "/etc/passwd"]
max_execution_time_secs = 30
max_output_bytes = 1048576
```

---

## Error Integration

The unified error enum in `omega-core::error::OmegaError` already includes a `Sandbox` variant:

```rust
#[error("sandbox error: {0}")]
Sandbox(String),
```

This variant is manually constructed (no `#[from]` conversion). All sandbox errors should wrap into this variant using descriptive messages that include the command attempted, the policy that blocked it, and the reason.

---

## Workspace Integration Points

| Integration | Location | Description |
|-------------|----------|-------------|
| Root Cargo.toml | `Cargo.toml` | Listed as workspace member and dependency |
| Config | `omega-core::config::SandboxConfig` | Configuration struct with defaults |
| Error | `omega-core::error::OmegaError::Sandbox` | Error variant for sandbox failures |
| Gateway | `src/gateway.rs` | Will need wiring to route execution requests through the sandbox |
| Skills | `omega-skills` | Future skills may invoke sandbox for command execution |
| Binary | `Cargo.toml` (root) | Already declared as a dependency of the binary |

---

## Planned Architecture

Based on the configuration surface, error integration, project roadmap (Phase 4), and the crate description ("Secure execution environment for Omega"), the following architecture is anticipated:

### Core Trait (planned)

A `Sandbox` trait (or equivalent) that provides an async interface for command execution:

```rust
#[async_trait]
pub trait Sandbox: Send + Sync {
    /// Execute a command within the sandbox constraints.
    async fn execute(&self, command: &str, args: &[&str]) -> Result<ExecutionResult, OmegaError>;

    /// Check if a command is allowed by the current policy.
    fn is_allowed(&self, command: &str) -> bool;

    /// Check if a filesystem path is accessible.
    fn path_allowed(&self, path: &str) -> bool;
}
```

### Execution Result (planned)

A struct to capture command output:

```rust
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub truncated: bool,
}
```

### Policy Engine (planned)

Enforcement logic that maps `SandboxConfig` fields to runtime decisions:

| Config Field | Policy |
|-------------|--------|
| `enabled` | Master toggle -- if `false`, all execution requests are denied |
| `allowed_commands` | Allowlist of executable names; commands not on the list are rejected |
| `blocked_paths` | Denylist of filesystem paths; commands accessing these paths are rejected |
| `max_execution_time_secs` | Timeout after which the child process is killed |
| `max_output_bytes` | Maximum combined stdout + stderr size; output is truncated beyond this limit |

### Security Considerations (planned)

| Concern | Mitigation |
|---------|------------|
| Command injection | Parse and validate command name against allowlist before execution |
| Path traversal | Resolve symlinks and canonicalize paths before checking against blocklist |
| Resource exhaustion (time) | Enforce `max_execution_time_secs` via `tokio::time::timeout` |
| Resource exhaustion (output) | Truncate output at `max_output_bytes` |
| Privilege escalation | Omega already refuses to run as root (guard in `main.rs`) |
| Environment leakage | Strip or filter sensitive environment variables before passing to child process |
| Shell metacharacters | Execute commands directly via `tokio::process::Command` rather than through a shell |

---

## File Size

| Metric | Value |
|--------|-------|
| Lines of code | 3 |
| Public types | 0 |
| Public functions | 0 |
| Tests | 0 |
| Submodules | 0 |

---

## Implementation Status

| Component | Status |
|-----------|--------|
| Crate scaffolding (Cargo.toml, lib.rs) | Complete |
| Configuration (`SandboxConfig`) | Complete (in omega-core) |
| Error variant (`OmegaError::Sandbox`) | Complete (in omega-core) |
| Example config (`config.example.toml` sandbox section) | Complete |
| Sandbox trait / interface | Not started |
| Command execution engine | Not started |
| Policy enforcement (allowlist, blocklist, limits) | Not started |
| Timeout enforcement | Not started |
| Output truncation | Not started |
| Gateway integration | Not started |
| Unit tests | Not started |
| Integration tests | Not started |

This crate is scheduled for Phase 4 of the Omega roadmap, alongside alternative providers, the skills system, a cron scheduler, and WhatsApp integration.
