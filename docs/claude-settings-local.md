# Claude Code Settings for Omega Development

## Quick Summary

The Omega project uses a local Claude Code settings file (`.claude/settings.local.json`) to control which Bash commands can execute during Claude Code assisted development sessions. Currently, only `cargo build` commands are permitted.

---

## What are Claude Code Settings?

Claude Code is Anthropic's CLI tool that provides AI assistance for code tasks. Like most tools that execute commands, it includes **security boundaries** that control what operations Claude can perform on your system.

The `.claude/settings.local.json` file defines these boundaries at the project level, allowing developers to:

1. **Enable trusted operations** — Permit commands necessary for development (like building code)
2. **Prevent accidental damage** — Block dangerous operations by default
3. **Maintain security** — Require explicit permission before expanding capabilities

---

## Current Configuration: `cargo build:*`

### What This Allows

The current setting permits all variants of Cargo's build command:

```bash
cargo build                    # Compile in debug mode
cargo build --release          # Compile optimized binary
cargo build --example gateway  # Compile specific examples
cargo build --features foo     # Build with specific Cargo features
```

### Why This Matters for Omega

Omega is a Rust project with a multi-crate workspace:

| Crate | Role |
|-------|------|
| `omega-core` | Types, traits, configuration, error handling |
| `omega-providers` | AI backends (Claude, OpenAI, Ollama, etc.) |
| `omega-channels` | Messaging integrations (Telegram, WhatsApp) |
| `omega-memory` | SQLite storage and conversation history |
| `omega-skills` | Plugin/extension system (planned) |
| `omega-sandbox` | Secure execution environment (planned) |

Building is essential because:

1. **Compilation validates code** — Rust's type system catches many bugs at compile time
2. **Cross-crate integration** — A single build command compiles all 6 crates together
3. **Rapid feedback** — Claude Code can iterate, compile, and show errors in real-time
4. **Deployment prep** — Creating the optimized binary for deployment

### Development Workflow

When you ask Claude Code to help with Omega development, it can now:

```
User Request
    ↓
Claude Code analyzes and modifies code
    ↓
Claude Code runs: cargo build --release  (ALLOWED)
    ↓
Compilation succeeds or fails
    ↓
Claude Code sees errors and iterates
    ↓
Shows you the working code
```

---

## What's Currently NOT Allowed

For security, other commands require explicit permission. This includes:

| Command Type | Examples | Why Restricted |
|--------------|----------|-----------------|
| Testing | `cargo test`, `cargo test --lib` | Could modify system state |
| Formatting | `cargo fmt` | Could alter files without review |
| Linting | `cargo clippy` | File modification and code analysis |
| Publishing | `cargo publish` | Uploads to public registries |
| File operations | `rm`, `cp`, `mv` | Direct filesystem access |
| System commands | `sudo`, `chown`, `chmod` | Elevated privileges |

These are blocked by default because they're more dangerous and should happen under explicit human control.

---

## How This Relates to the Build Pipeline

The `CLAUDE.md` project guide specifies a mandatory check before every commit:

```bash
cargo clippy --workspace && cargo test --workspace && cargo fmt --check
```

Currently, Claude Code can help with:

- ✓ **`cargo build`** — Compiling code (currently allowed)

But cannot directly run:

- ✗ `cargo clippy` — Linting (requires explicit permission)
- ✗ `cargo test` — Testing (requires explicit permission)
- ✗ `cargo fmt` — Formatting (requires explicit permission)

**Why this design?** The build permission allows rapid iteration and feedback, while other steps (testing, linting, formatting) remain under human control to ensure quality gates are met.

---

## Future Extensions

As Omega evolves, additional permissions might be useful. For example:

```json
{
  "permissions": {
    "allow": [
      "Bash(cargo build:*)",
      "Bash(cargo test:*)",           // Phase 4: Add testing
      "Bash(cargo clippy:*)",         // Phase 4: Add linting
      "Bash(cargo fmt:*)",            // Phase 4: Add formatting
      "Bash(cargo run:*)"             // Phase 4: Add execution
    ]
  }
}
```

Each addition should be:

1. **Justified** — Why is this command safe for Claude Code to run?
2. **Scoped** — Use wildcards (`*`) to allow variations, not unlimited access
3. **Reviewed** — Team consensus before expanding permissions

---

## Security Design Philosophy

Claude Code's permission system follows **least privilege** principles:

- **Default: deny** — Commands are blocked unless explicitly allowed
- **Explicit allowlist** — Only permitted operations can execute
- **Pattern-based** — Wildcards allow flexibility within a command family
- **Local scope** — Project-specific settings don't affect other projects

This prevents scenarios like:

```bash
# BAD: Would be allowed without protection
rm -rf ~/.omega
sudo chown root:root /

# SAFE: Only these variants are possible
cargo build
cargo build --release
```

---

## Configuration Location and Format

**File Path:** `.claude/settings.local.json`

**Format:** JSON with hierarchical structure

```json
{
  "permissions": {
    "allow": [
      "command pattern 1",
      "command pattern 2"
    ]
  }
}
```

**Naming Convention:**

- `.claude/` — Directory for Claude Code project settings
- `settings.local.json` — Local (project-specific) settings file
- `.local` suffix indicates non-global configuration

---

## Practical Example

When you use Claude Code with Omega:

### Example 1: Request a Bug Fix

```
You: "The gateway doesn't compile. Can you fix it?"

Claude Code:
  1. Analyzes the error
  2. Modifies src/gateway.rs
  3. Runs: cargo build --release (ALLOWED)
  4. Shows you the compilation result
  5. If there are errors, iterates again
```

### Example 2: Request a New Feature

```
You: "Add a self-check command to omega-core."

Claude Code:
  1. Creates new code in crates/omega-core/
  2. Runs: cargo build (ALLOWED)
  3. Tests the new code manually OR
  4. Shows you the code for you to test (TESTING NOT ALLOWED)
  5. Waits for your feedback
```

### Example 3: Request Tests

```
You: "Write tests for the memory module."

Claude Code:
  1. Creates tests in crates/omega-memory/src/lib.rs
  2. Asks: "Should I run cargo test to verify?" OR
  3. You run tests yourself (because TESTING NOT ALLOWED)
  4. Shows results or asks for your test output
```

---

## Integration with CI/CD

This local setting is distinct from:

- **CI/CD pipelines** (GitHub Actions, etc.) — Which have their own permissions
- **Global Claude Code settings** (if any) — Which would apply everywhere
- **System permissions** — Which control the Omega binary itself

The `.claude/settings.local.json` only affects Claude Code's behavior when working on this project.

---

## Next Steps

If you want to extend Claude Code's capabilities for Omega:

1. **Identify the need** — What command would improve your workflow?
2. **Assess the risk** — Is it safe for automated execution?
3. **Add to allow list** — Update `.claude/settings.local.json`
4. **Test thoroughly** — Verify Claude Code behavior with the new permission
5. **Document the change** — Update this file to explain why

Example for enabling testing:

```json
{
  "permissions": {
    "allow": [
      "Bash(cargo build:*)",
      "Bash(cargo test:*)"
    ]
  }
}
```

---

## See Also

- **Project Guidelines:** `CLAUDE.md` — Omega's architecture, design rules, and build checklist
- **Specifications:** `specs/claude-settings-local.md` — Detailed technical specification
- **Config Files:** `config.toml`, `config.example.toml` — Omega's application configuration

---

## Summary

The Claude Code settings file is a security boundary that:

1. **Permits compilation** — `cargo build` can run during Claude Code sessions
2. **Prevents misuse** — Dangerous operations are blocked by default
3. **Enables iteration** — Rapid feedback loops for assisted development
4. **Maintains control** — Humans remain in the decision-making loop for risky operations

This balanced approach lets Claude Code be helpful without compromising project security.
