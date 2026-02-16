# Technical Specification: Built-in Skills Module

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-skills/src/builtin/mod.rs` |
| **Crate** | `omega-skills` |
| **Module declaration** | `mod builtin;` in `crates/omega-skills/src/lib.rs` (private module) |
| **Status** | **Placeholder / Stub** -- file contains only a module-level doc comment |

## Current Contents

```rust
//! Built-in skills for Omega.
```

That is the entire file. No imports, no structs, no functions, no trait implementations.

---

## Purpose

The `builtin` module is intended to house all first-party skills that ship with Omega. These are skills that are always available without any external plugins, user configuration, or third-party dependencies. They extend Omega's capabilities beyond simple AI conversation into concrete, repeatable actions such as web search, file management, code execution, scheduling, and system information queries.

The module lives inside the `omega-skills` crate, which is the central hub for both built-in and (eventually) user-defined or dynamically loaded skills.

---

## Parent Crate: `omega-skills`

### Crate Metadata (`Cargo.toml`)

| Field | Value |
|-------|-------|
| `name` | `omega-skills` |
| `version` | workspace |
| `edition` | workspace |
| `description` | "Skill and plugin system for Omega" |

### Dependencies

| Dependency | Purpose |
|------------|---------|
| `omega-core` | Access to `OmegaError`, message types, `Context`, traits |
| `tokio` | Async runtime |
| `serde` / `serde_json` | Serialization for skill configuration and I/O |
| `tracing` | Structured logging |
| `thiserror` | Custom error types |
| `anyhow` | Flexible error handling |
| `async-trait` | Async trait support |

### Crate-Level Module Structure (`lib.rs`)

```rust
//! # omega-skills
//!
//! Skill and plugin system for Omega.

mod builtin;
```

The `builtin` module is currently the only submodule. It is declared as `mod builtin;` (private). When the skill system is ready to be consumed by the gateway, this will need to become `pub mod builtin;` or skills will be re-exported through a public registry API.

---

## Expected Trait: `Skill` (Not Yet Defined)

No `Skill` trait exists in the codebase yet. Based on the existing `Provider` and `Channel` trait patterns in `omega-core/src/traits.rs`, a `Skill` trait would likely follow this shape:

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    /// Human-readable skill name.
    fn name(&self) -> &str;

    /// Short description for help output and skill discovery.
    fn description(&self) -> &str;

    /// Whether this skill can handle the given input.
    /// Used by the skill router to match user intent to skills.
    fn can_handle(&self, input: &str) -> bool;

    /// Execute the skill with the given input and return a result.
    async fn execute(&self, input: &str) -> Result<String, OmegaError>;
}
```

This trait would need to be defined in either `omega-core/src/traits.rs` (alongside `Provider` and `Channel`) or in `omega-skills/src/lib.rs` as the crate's own trait.

---

## Expected Data Structures (Not Yet Implemented)

Based on the architectural patterns in the codebase, the following structures would be needed:

### `SkillRegistry`

A central registry that holds all available skills and provides routing logic.

| Field | Type | Description |
|-------|------|-------------|
| `skills` | `Vec<Box<dyn Skill>>` | All registered skill instances |

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new() -> Self` | Create an empty registry |
| `register` | `fn register(&mut self, skill: Box<dyn Skill>)` | Add a skill to the registry |
| `find` | `fn find(&self, input: &str) -> Option<&dyn Skill>` | Find the first skill that can handle the input |
| `list` | `fn list(&self) -> Vec<(&str, &str)>` | List all skill names and descriptions |

### `SkillResult`

A structured result type returned by skill execution.

| Field | Type | Description |
|-------|------|-------------|
| `text` | `String` | The skill's textual output |
| `success` | `bool` | Whether execution succeeded |
| `metadata` | `Option<serde_json::Value>` | Optional structured data |

---

## Planned Built-in Skills

Based on the project roadmap (Phase 4 mentions "skills system") and common patterns in AI agent frameworks, the following built-in skills are candidates:

| Skill | Name String | Description | Dependencies |
|-------|-------------|-------------|--------------|
| **System Info** | `"system-info"` | Report OS, uptime, memory usage, disk space | `sysinfo` crate or `tokio::process::Command` |
| **Web Search** | `"web-search"` | Search the web and return summarized results | HTTP client, search API key |
| **Code Execution** | `"code-exec"` | Run code snippets in a sandboxed environment | `omega-sandbox` crate |
| **File Operations** | `"file-ops"` | Read, write, list files within allowed paths | `tokio::fs`, sandbox constraints |
| **Cron Scheduler** | `"cron"` | Schedule recurring tasks | `tokio-cron-scheduler` or equivalent |
| **Calculator** | `"calc"` | Evaluate mathematical expressions | Pure Rust expression parser |
| **URL Fetch** | `"url-fetch"` | Fetch and summarize web page content | `reqwest`, HTML parser |
| **Reminder** | `"reminder"` | Set time-based reminders | `tokio::time`, memory integration |

---

## Integration Points

### Gateway Integration

The gateway (`src/gateway.rs`) does not currently reference skills. When integrated, skills would be invoked as an additional step in the message processing pipeline, likely between sanitization and provider dispatch:

```
Message -> Auth -> Sanitize -> Skill Router -> (if matched) Skill Execute -> Memory -> Audit -> Send
                                            -> (if not matched) Context -> Provider -> Memory -> Audit -> Send
```

### Command Integration

The bot command system (`src/commands.rs`) could expose skills via a `/skill` command or by mapping specific commands to skills (e.g., `/calc 2+2` routes to the calculator skill).

### Memory Integration

Skills that produce structured data (e.g., facts, reminders) should store results through `omega-memory` for persistence and future context building.

### Sandbox Integration

Skills that execute code or access the filesystem should route through `omega-sandbox` for security enforcement.

---

## Configuration (Not Yet Defined)

No `[skills]` section exists in `config.example.toml` yet. A future configuration structure might look like:

```toml
[skills]
enabled = true

[skills.builtin]
system_info = true
web_search = true
code_exec = true
file_ops = true
cron = false
calculator = true
url_fetch = true
reminder = true

[skills.builtin.web_search]
api_key = ""
engine = "google"

[skills.builtin.code_exec]
sandbox = true
timeout_secs = 30
```

A corresponding config struct would be needed in `omega-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub builtin: BuiltinSkillsConfig,
}
```

---

## Module Visibility

The `builtin` module is currently declared as `mod builtin;` (private) in `lib.rs`. This mirrors the pattern used by placeholder modules throughout the codebase (e.g., `mod whatsapp;`, `mod anthropic;`). When implementation begins, the visibility strategy depends on the chosen architecture:

- **If skills are accessed via a public registry:** Keep `builtin` private and expose a `pub fn register_builtins(registry: &mut SkillRegistry)` function.
- **If skills are accessed individually:** Make the module `pub mod builtin;` so consumers can construct specific skills.

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Module declaration in `lib.rs` | Done | `mod builtin;` (private) |
| Module doc comment | Done | `//! Built-in skills for Omega.` |
| `Skill` trait definition | Not started | Needs to be defined in `omega-core` or `omega-skills` |
| `SkillRegistry` struct | Not started | -- |
| `SkillResult` type | Not started | -- |
| `SkillsConfig` in `omega-core` | Not started | No `[skills]` section in config |
| Config example entry | Not started | No entry in `config.example.toml` |
| Individual built-in skills | Not started | -- |
| Gateway integration | Not started | No skill routing in the pipeline |
| Command integration | Not started | No `/skill` command |
| Tests | Not started | -- |

---

## Dependencies Required

Based on the `Cargo.toml` already defined for `omega-skills`, the crate has the foundational dependencies it needs. Additional dependencies may be required for specific skills:

| Dependency | Purpose | Currently in Cargo.toml |
|------------|---------|------------------------|
| `omega-core` | Core types and traits | Yes |
| `tokio` | Async runtime | Yes |
| `serde` / `serde_json` | Serialization | Yes |
| `tracing` | Logging | Yes |
| `async-trait` | Async trait support | Yes |
| `thiserror` | Error types | Yes |
| `anyhow` | Error handling | Yes |
| `reqwest` | HTTP requests (web search, URL fetch) | No |
| `sysinfo` | System information queries | No |

---

## Relationship to `omega-sandbox`

The `omega-sandbox` crate (also a placeholder) is intended to provide secure command execution. Built-in skills that need to run shell commands or execute user-provided code should delegate to `omega-sandbox` rather than spawning processes directly. This separation ensures that security constraints (allowed commands, blocked paths, execution timeouts) are enforced uniformly regardless of which skill triggers the execution.

---

## File Organization Convention

When built-in skills are implemented, each skill should live in its own file under the `builtin/` directory:

```
crates/omega-skills/src/
  lib.rs                    # Crate root, declares `mod builtin`
  builtin/
    mod.rs                  # Module root, declares submodules, registers all built-in skills
    system_info.rs          # System information skill
    calculator.rs           # Calculator skill
    web_search.rs           # Web search skill
    code_exec.rs            # Code execution skill (uses omega-sandbox)
    file_ops.rs             # File operations skill (uses omega-sandbox)
    cron.rs                 # Cron scheduler skill
    url_fetch.rs            # URL fetching skill
    reminder.rs             # Reminder skill
```

The `mod.rs` file would declare each submodule and provide a convenience function to register all built-in skills with the skill registry.
