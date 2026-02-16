# Built-in Skills -- Developer Documentation

## Current Status

The built-in skills module is a **placeholder**. The file `crates/omega-skills/src/builtin/mod.rs` contains only a module-level doc comment:

```rust
//! Built-in skills for Omega.
```

There is no trait, no struct, no registry, and no skill implementations yet. However, the surrounding infrastructure is already in place:

- The `omega-skills` crate exists in the workspace with its `Cargo.toml` and dependencies.
- The `builtin` module is declared in `crates/omega-skills/src/lib.rs` as `mod builtin;`.
- The crate depends on `omega-core`, `tokio`, `serde`, `tracing`, `async-trait`, `thiserror`, and `anyhow` -- all the building blocks needed for implementation.

This is a Phase 4 task on the Omega roadmap.

---

## What Are Built-in Skills?

Skills are discrete, reusable capabilities that Omega can invoke in response to user requests. Unlike the AI provider (which handles general-purpose conversation), skills perform **specific, concrete actions** -- things like fetching a web page, running a calculation, checking system status, or scheduling a reminder.

Built-in skills are the skills that ship with Omega out of the box. They require no plugins, no external installation, and no special user configuration beyond toggling them on or off. They form the foundation of Omega's ability to take action in the world rather than just talk.

Think of the AI provider as the brain and skills as the hands. The provider decides *what* to do; skills *do* it.

---

## How the Skill System Will Work

### The Skill Trait

Every skill -- built-in or user-defined -- will implement a common `Skill` trait. This trait has not been defined yet, but based on the patterns established by the `Provider` and `Channel` traits in `omega-core/src/traits.rs`, it will look something like this:

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    /// Human-readable skill name (e.g., "calculator", "web-search").
    fn name(&self) -> &str;

    /// Short description for help output and discovery.
    fn description(&self) -> &str;

    /// Check if this skill can handle the given input.
    fn can_handle(&self, input: &str) -> bool;

    /// Execute the skill and return the result as text.
    async fn execute(&self, input: &str) -> Result<String, OmegaError>;
}
```

The key design points:

- **`name()`** and **`description()`** are for discoverability -- listing skills in help output, logging which skill handled a request, etc.
- **`can_handle()`** is for routing -- the skill system checks each registered skill to see if it can handle the current input. This could use keyword matching, regex, or even a lightweight classifier.
- **`execute()`** is the actual work -- take an input string, do something, and return the result.

### The Skill Registry

A `SkillRegistry` will hold all available skills and provide a way to find the right one for a given input:

```rust
pub struct SkillRegistry {
    skills: Vec<Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self { ... }
    pub fn register(&mut self, skill: Box<dyn Skill>) { ... }
    pub fn find(&self, input: &str) -> Option<&dyn Skill> { ... }
    pub fn list(&self) -> Vec<(&str, &str)> { ... }
}
```

The `find()` method iterates through skills in registration order and returns the first one whose `can_handle()` returns `true`. This means registration order determines priority when multiple skills could handle the same input.

### Pipeline Integration

When skills are wired into the gateway, the message processing pipeline will gain a new routing step:

```
User Message
    |
    v
1. Authentication      -- Is this user allowed?
2. Sanitization        -- Neutralize injection patterns
3. Command Dispatch    -- Is this a /command?
4. Skill Router        -- Can any skill handle this?  <-- NEW
   |                    |
   | (skill matched)    | (no skill matched)
   v                    v
5a. Skill Execute      5b. Context Building
   |                        |
   v                        v
6. Memory Storage      6. Provider Call
   |                        |
   v                        v
7. Audit Logging       7. Memory Storage
   |                        |
   v                        v
8. Send Response       8. Audit Logging
                            |
                            v
                       9. Send Response
```

The skill router checks the input against registered skills. If a match is found, the skill handles the request directly without involving the AI provider. If no skill matches, the message flows to the provider as usual.

This design means that skills are **fast paths** -- they bypass the AI provider entirely when they can handle the request on their own. A calculator skill does not need Claude to evaluate `2 + 2`.

---

## How to Add a New Built-in Skill

Here is a step-by-step guide for adding a built-in skill once the trait and registry are defined.

### 1. Create the Skill File

Add a new file under `crates/omega-skills/src/builtin/`. For example, to create a calculator skill:

```
crates/omega-skills/src/builtin/calculator.rs
```

### 2. Implement the Trait

```rust
use async_trait::async_trait;
use omega_core::error::OmegaError;

/// A simple mathematical expression evaluator.
pub struct CalculatorSkill;

#[async_trait]
impl Skill for CalculatorSkill {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions"
    }

    fn can_handle(&self, input: &str) -> bool {
        // Simple heuristic: if the input starts with "calc " or "calculate ",
        // or looks like a pure math expression.
        let lower = input.to_lowercase();
        lower.starts_with("calc ") || lower.starts_with("calculate ")
    }

    async fn execute(&self, input: &str) -> Result<String, OmegaError> {
        let expression = input
            .trim_start_matches("calculate ")
            .trim_start_matches("calc ")
            .trim();

        // Parse and evaluate the expression.
        // (Use a proper expression parser here, not eval.)
        let result = evaluate(expression)
            .map_err(|e| OmegaError::Provider(format!("Calculator error: {e}")))?;

        Ok(format!("{expression} = {result}"))
    }
}
```

### 3. Declare the Submodule

In `crates/omega-skills/src/builtin/mod.rs`, add the module declaration:

```rust
//! Built-in skills for Omega.

mod calculator;

pub use calculator::CalculatorSkill;
```

### 4. Register the Skill

In whatever initialization code builds the `SkillRegistry` (likely in the gateway or a dedicated setup function):

```rust
use omega_skills::builtin::CalculatorSkill;

let mut registry = SkillRegistry::new();
registry.register(Box::new(CalculatorSkill));
```

Or, provide a convenience function in `builtin/mod.rs`:

```rust
/// Register all built-in skills with the given registry.
pub fn register_all(registry: &mut SkillRegistry) {
    registry.register(Box::new(calculator::CalculatorSkill));
    // Add more built-in skills here as they are implemented.
}
```

### 5. Write Tests

Add tests in the skill file or in a separate test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let skill = CalculatorSkill;
        assert!(skill.can_handle("calc 2 + 2"));
        assert!(skill.can_handle("calculate 10 * 5"));
        assert!(!skill.can_handle("what is the weather?"));
    }

    #[tokio::test]
    async fn test_execute() {
        let skill = CalculatorSkill;
        let result = skill.execute("calc 2 + 2").await.unwrap();
        assert!(result.contains("4"));
    }
}
```

Run tests with:

```bash
cargo test --workspace
```

### 6. Add Configuration (Optional)

If the skill needs configuration (e.g., an API key for a web search skill), add a config struct to `omega-core/src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchSkillConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_search_engine")]
    pub engine: String,
}
```

And a corresponding section in `config.example.toml`:

```toml
[skills.builtin.web_search]
enabled = true
api_key = ""
engine = "google"
```

---

## Design Conventions

When implementing built-in skills, follow the same conventions enforced throughout the Omega codebase:

- **No `unwrap()`** -- Use `?` and return `Result<..., OmegaError>`. Skills should never panic.
- **Tracing, not `println!`** -- Use `tracing::debug!`, `tracing::info!`, `tracing::warn!` for all logging.
- **Async everywhere** -- Skill execution is async. Use `tokio` for any I/O.
- **Doc comments on every public item** -- Every public struct, function, and method gets a `///` doc comment.
- **Security through `omega-sandbox`** -- Skills that execute shell commands or access the filesystem should route through the sandbox crate, not spawn processes directly.

---

## Planned Built-in Skills

These are the skills that are candidates for initial implementation:

### System Info

Report operating system, uptime, memory usage, and disk space. Useful for remote monitoring of the machine Omega runs on.

### Calculator

Evaluate mathematical expressions without involving the AI provider. Fast, deterministic, and offline.

### Web Search

Search the web and return summarized results. Requires an API key for the search engine (Google, Bing, etc.).

### Code Execution

Run code snippets in a sandboxed environment. Delegates to `omega-sandbox` for security. Supports common languages (Python, JavaScript, shell).

### File Operations

Read, write, and list files within allowed paths. All operations are constrained by sandbox rules to prevent unauthorized file access.

### Cron Scheduler

Schedule recurring tasks. For example, "remind me every Monday at 9am to check server logs." Stores schedules in `omega-memory`.

### URL Fetch

Fetch a web page and return a text summary. Useful for reading articles, documentation, or API responses.

### Reminder

Set one-off time-based reminders. "Remind me in 30 minutes to check the build." Stores reminders in `omega-memory` and triggers a notification when the time arrives.

---

## File Organization

When implementation begins, the `builtin/` directory will grow to contain one file per skill:

```
crates/omega-skills/src/
  lib.rs                    # Crate root
  builtin/
    mod.rs                  # Module root, registers all built-in skills
    system_info.rs
    calculator.rs
    web_search.rs
    code_exec.rs
    file_ops.rs
    cron.rs
    url_fetch.rs
    reminder.rs
```

Each skill is self-contained in its own file. The `mod.rs` file declares all submodules and provides a `register_all()` function for bulk registration.

---

## Error Handling

Skills should use the existing `OmegaError` type from `omega-core`. The most relevant variant is `OmegaError::Provider(String)`, though a dedicated `OmegaError::Skill(String)` variant may be added when the skill system is formalized. Common failure modes:

- **Invalid input** -- The skill was matched but the input is malformed (e.g., "calc abc").
- **External service failure** -- A web search or URL fetch fails due to network issues.
- **Sandbox rejection** -- The sandbox blocks a command or file access attempt.
- **Timeout** -- A long-running skill exceeds its execution time limit.

All errors should include descriptive messages that help with debugging, and should be logged via `tracing::warn!` or `tracing::error!` before being returned.

---

## Relationship to Other Crates

| Crate | Relationship to `omega-skills` |
|-------|-------------------------------|
| `omega-core` | Provides `OmegaError`, message types, and will host the `Skill` trait (or it will be defined in `omega-skills`) |
| `omega-sandbox` | Provides secure execution for skills that run commands or access files |
| `omega-memory` | Provides persistence for skills that store data (reminders, cron schedules, cached results) |
| `omega-providers` | Orthogonal -- skills bypass the provider when they can handle a request directly |
| `omega-channels` | Orthogonal -- channels deliver messages; skills process them |

---

## Reference

- Skills crate root: `crates/omega-skills/src/lib.rs`
- Built-in module: `crates/omega-skills/src/builtin/mod.rs`
- Core traits (Provider, Channel): `crates/omega-core/src/traits.rs`
- Error types: `crates/omega-core/src/error.rs`
- Gateway pipeline: `src/gateway.rs`
- Config system: `crates/omega-core/src/config.rs`
- Example config: `config.example.toml`
- Sandbox crate (planned): `crates/omega-sandbox/`
