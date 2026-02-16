# omega-sandbox -- Developer Guide

## What is this crate?

`omega-sandbox` is the secure command execution layer of the Omega workspace. When Omega needs to run shell commands or scripts on the host machine -- whether triggered by the AI provider, a skill, or a user request -- those commands will pass through the sandbox. The sandbox enforces security policies: which commands are allowed, which filesystem paths are off-limits, how long a command can run, and how much output it can produce.

Today the crate is a **placeholder**. It contains only a doc comment and no implementation. The supporting infrastructure (configuration, error handling) already exists in `omega-core`, so when implementation begins, the sandbox can be built on a solid foundation.

## Crate structure

```
crates/omega-sandbox/
  Cargo.toml
  src/
    lib.rs          <-- you are here (placeholder)
```

`lib.rs` is currently three lines:

```rust
//! # omega-sandbox
//!
//! Secure execution environment for Omega.
```

No types, no functions, no submodules.

---

## Why does this crate exist?

Omega is an AI agent that receives natural language requests and delegates reasoning to AI backends. Some of those requests may involve running commands on the host machine -- checking system status, running build tools, querying databases, or executing scripts.

Running arbitrary commands is inherently dangerous. The sandbox exists to ensure that:

1. **Only approved commands can run.** A configurable allowlist controls which executables are permitted.
2. **Sensitive paths are protected.** A configurable blocklist prevents access to files like `/etc/shadow`.
3. **Runaway processes are killed.** A timeout prevents commands from running indefinitely.
4. **Output is bounded.** A size limit prevents memory exhaustion from verbose commands.
5. **No privilege escalation.** Omega already refuses to run as root, and the sandbox will add further guardrails.

---

## What is already in place?

Even though the sandbox itself is unimplemented, the surrounding infrastructure is ready:

### Configuration

`omega-core::config::SandboxConfig` is fully defined and integrated into the config system:

```rust
pub struct SandboxConfig {
    pub enabled: bool,                    // default: true
    pub allowed_commands: Vec<String>,    // allowlist of executable names
    pub blocked_paths: Vec<String>,       // denylist of filesystem paths
    pub max_execution_time_secs: u64,     // default: 30 seconds
    pub max_output_bytes: usize,          // default: 1 MiB
}
```

Users can configure the sandbox in `config.toml`:

```toml
[sandbox]
enabled = true
allowed_commands = ["ls", "cat", "grep", "find", "git", "cargo", "npm", "python"]
blocked_paths = ["/etc/shadow", "/etc/passwd"]
max_execution_time_secs = 30
max_output_bytes = 1048576
```

### Error handling

`OmegaError::Sandbox(String)` is already defined in `omega-core::error`. The sandbox will use this variant for all error conditions:

```rust
use omega_core::error::OmegaError;

// Example usage (future):
return Err(OmegaError::Sandbox(
    "command 'rm' is not in the allowed_commands list".to_string()
));
```

### Dependencies

The `Cargo.toml` already declares everything the sandbox will need:

- **`omega-core`** -- config, errors, shared types
- **`tokio`** -- async process execution, timeouts
- **`serde`** -- serialization of execution results
- **`tracing`** -- structured logging
- **`thiserror`** / **`anyhow`** -- error handling

---

## How to implement the sandbox

When Phase 4 development begins, here is the recommended approach:

### Step 1: Define the execution result type

Create a struct that captures everything about a command execution:

```rust
/// Result of a sandboxed command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Standard output from the command.
    pub stdout: String,
    /// Standard error from the command.
    pub stderr: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Wall-clock execution time in milliseconds.
    pub duration_ms: u64,
    /// Whether the output was truncated due to max_output_bytes.
    pub truncated: bool,
}
```

### Step 2: Implement the policy layer

Before any command runs, validate it against the `SandboxConfig`:

```rust
use omega_core::config::SandboxConfig;
use omega_core::error::OmegaError;

pub struct SandboxPolicy {
    config: SandboxConfig,
}

impl SandboxPolicy {
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Check whether the sandbox is enabled at all.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check whether a command is on the allowlist.
    /// If the allowlist is empty, all commands are allowed (or denied -- decide on convention).
    pub fn command_allowed(&self, command: &str) -> bool {
        if self.config.allowed_commands.is_empty() {
            return true; // empty allowlist = no restrictions
        }
        self.config.allowed_commands.iter().any(|c| c == command)
    }

    /// Check whether a path is blocked.
    pub fn path_blocked(&self, path: &str) -> bool {
        self.config.blocked_paths.iter().any(|blocked| path.starts_with(blocked))
    }
}
```

### Step 3: Implement async command execution

Use `tokio::process::Command` to run commands with timeout enforcement:

```rust
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn execute(
    command: &str,
    args: &[&str],
    config: &SandboxConfig,
) -> Result<ExecutionResult, OmegaError> {
    let start = std::time::Instant::now();

    let result = timeout(
        Duration::from_secs(config.max_execution_time_secs),
        Command::new(command)
            .args(args)
            .output(),
    )
    .await
    .map_err(|_| OmegaError::Sandbox(format!(
        "command '{}' timed out after {}s",
        command, config.max_execution_time_secs
    )))?
    .map_err(|e| OmegaError::Sandbox(format!(
        "failed to execute '{}': {}",
        command, e
    )))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    let mut stdout = String::from_utf8_lossy(&result.stdout).to_string();
    let mut stderr = String::from_utf8_lossy(&result.stderr).to_string();
    let mut truncated = false;

    // Enforce output size limit
    if stdout.len() + stderr.len() > config.max_output_bytes {
        let half = config.max_output_bytes / 2;
        stdout.truncate(half);
        stderr.truncate(half);
        truncated = true;
    }

    Ok(ExecutionResult {
        stdout,
        stderr,
        exit_code: result.status.code().unwrap_or(-1),
        duration_ms,
        truncated,
    })
}
```

### Step 4: Build the public Sandbox struct

Combine policy and execution into a single entry point:

```rust
pub struct Sandbox {
    policy: SandboxPolicy,
    config: SandboxConfig,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        let policy = SandboxPolicy::new(config.clone());
        Self { policy, config }
    }

    pub async fn run(&self, command: &str, args: &[&str]) -> Result<ExecutionResult, OmegaError> {
        if !self.policy.is_enabled() {
            return Err(OmegaError::Sandbox("sandbox is disabled".to_string()));
        }

        if !self.policy.command_allowed(command) {
            return Err(OmegaError::Sandbox(format!(
                "command '{}' is not in the allowed_commands list", command
            )));
        }

        // Check args for blocked paths
        for arg in args {
            if self.policy.path_blocked(arg) {
                return Err(OmegaError::Sandbox(format!(
                    "path '{}' is blocked by sandbox policy", arg
                )));
            }
        }

        tracing::info!(command = command, args = ?args, "sandbox: executing command");
        execute(command, args, &self.config).await
    }
}
```

### Step 5: Wire into the gateway

Once the sandbox is implemented, the gateway (`src/gateway.rs`) needs to be updated to route execution requests through it. The sandbox instance would be created from `config.sandbox` and passed to providers or skills that need to run commands.

### Step 6: Write tests

Follow the project convention of unit tests in a `#[cfg(test)] mod tests` block:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use omega_core::config::SandboxConfig;

    #[test]
    fn test_policy_allows_listed_command() {
        let config = SandboxConfig {
            enabled: true,
            allowed_commands: vec!["ls".to_string(), "cat".to_string()],
            blocked_paths: vec![],
            max_execution_time_secs: 30,
            max_output_bytes: 1_048_576,
        };
        let policy = SandboxPolicy::new(config);
        assert!(policy.command_allowed("ls"));
        assert!(!policy.command_allowed("rm"));
    }

    #[test]
    fn test_policy_blocks_sensitive_paths() {
        let config = SandboxConfig {
            enabled: true,
            allowed_commands: vec![],
            blocked_paths: vec!["/etc/shadow".to_string()],
            max_execution_time_secs: 30,
            max_output_bytes: 1_048_576,
        };
        let policy = SandboxPolicy::new(config);
        assert!(policy.path_blocked("/etc/shadow"));
        assert!(!policy.path_blocked("/tmp/safe"));
    }

    #[tokio::test]
    async fn test_sandbox_rejects_disabled() {
        let config = SandboxConfig {
            enabled: false,
            ..Default::default()
        };
        let sandbox = Sandbox::new(config);
        let result = sandbox.run("ls", &[]).await;
        assert!(result.is_err());
    }
}
```

---

## Design decisions to make

When implementation begins, several open questions will need resolution:

| Question | Options | Recommendation |
|----------|---------|----------------|
| What does an empty `allowed_commands` mean? | Allow all commands vs. deny all commands | Allow all (treat the list as opt-in filtering). If you want to block everything, set `enabled = false`. |
| Should the sandbox resolve symlinks before path checking? | Yes / No | Yes. Use `std::fs::canonicalize` to prevent symlink-based bypasses. |
| Should commands run through a shell? | `sh -c "command"` vs. direct exec | Direct exec via `tokio::process::Command`. Shell execution introduces metacharacter injection risks. |
| Should environment variables be filtered? | Pass-through vs. allowlist vs. denylist | Denylist sensitive variables (API keys, tokens). Pass through the rest. |
| Should the sandbox log all executions to the audit log? | Yes / No | Yes. Use `omega-memory`'s audit system to record every command, its result, and the requesting context. |
| Should there be per-command timeout overrides? | Global only vs. per-command | Start with global only. Per-command overrides can be added later if needed. |

---

## How it fits in the architecture

```
User message arrives via channel
    |
    v
Gateway pipeline: auth -> sanitize -> context -> provider
    |
    v
Provider decides a command needs to be run
    |
    v
omega-sandbox: policy check -> execute -> capture output -> return result
    |
    v
Provider incorporates result into response
    |
    v
Gateway: store in memory -> audit log -> send response
```

The sandbox sits between the AI provider (or skill) and the operating system. It is the single chokepoint through which all command execution must pass.

---

## Key project rules that apply

- **No `unwrap()`** -- use `?` and `OmegaError::Sandbox` for all error paths.
- **Tracing, not `println!`** -- use `tracing::{info, warn, error, debug}` for logging.
- **Async everywhere** -- command execution must be async via `tokio::process::Command`.
- **Every public function gets a doc comment.**
- **`cargo clippy --workspace` must pass with zero warnings.**

---

## Quick reference

| You want to... | Where to look |
|----------------|---------------|
| See the sandbox config fields | `omega-core::config::SandboxConfig` |
| See the error variant | `omega-core::error::OmegaError::Sandbox` |
| See example config values | `config.example.toml`, `[sandbox]` section |
| Check project roadmap | `CLAUDE.md`, Phase 4 |
| Understand the gateway pipeline | `src/gateway.rs` |
