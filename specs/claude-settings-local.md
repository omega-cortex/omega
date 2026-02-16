# Claude Settings Local Specification

## Overview

**File Location:** `.claude/settings.local.json`

**Purpose:** Local Claude Code CLI configuration for the Omega project development environment. This file specifies custom permissions and restrictions for Claude Code operations during development and testing.

**Type:** JSON configuration file

**Scope:** Project-local settings that override default Claude Code CLI behavior within the Omega repository context.

---

## File Structure

The `settings.local.json` file uses a hierarchical JSON structure with the following top-level keys:

```json
{
  "permissions": { ... }
}
```

---

## Settings Reference

### `permissions` Object

Controls which Bash commands are allowed to execute during Claude Code sessions.

#### `permissions.allow` (Array)

An array of strings specifying Bash command patterns that are explicitly allowed to run.

**Type:** `string[]`

**Current Configuration:**

```json
{
  "permissions": {
    "allow": [
      "Bash(cargo build:*)"
    ]
  }
}
```

---

## Current Allowed Commands

### 1. `Bash(cargo build:*)`

**Pattern:** `Bash(cargo build:*)`

**Description:** Allows any variant of the `cargo build` command with any additional flags or arguments.

**Scope:** Covers:
- `cargo build` — standard debug build
- `cargo build --release` — optimized release build
- `cargo build --features=...` — builds with specific features
- `cargo build --example <name>` — builds example binaries
- Any other `cargo build` variant

**Purpose in Omega Context:**

The Omega project is a Rust-based AI agent infrastructure. During development and Claude Code assisted sessions, builds are a core operational step. This permission allows:

1. **Validation during development** — Verify that code compiles as changes are made
2. **Type checking** — Ensure Rust type system correctness across the workspace
3. **Integration testing** — Build and test all six workspace crates together
4. **Binary generation** — Create executable binaries for testing and deployment

**Related to Build Pipeline:**

The Omega `CLAUDE.md` specifies the full build and test checklist:

```bash
cargo check                  # Type check all crates
cargo clippy --workspace     # Zero warnings required
cargo test --workspace       # All tests must pass
cargo fmt                    # Always format before commit
cargo build --release        # Optimized binary
```

The `cargo build:*` permission specifically allows the compilation step, which is essential for verifying code quality before commits.

---

## Security Implications

### Allowed Operations

The current permissions configuration is **permissive but scoped** to build operations:

- **Why allowed:** Building is essential for development workflows and poses no security risk (it only compiles Rust code)
- **What it prevents:** Prevents other destructive or sensitive commands from running without explicit permission

### Design Pattern

This follows Claude Code's security model:

1. **Explicit allowlisting** — Only commands in the `allow` array can execute
2. **Pattern matching** — The wildcard `*` permits any flags/arguments to the specified command
3. **Workspace isolation** — Settings are local to this project and don't affect system-wide Claude Code behavior

### What's NOT Allowed (by default)

- File deletion or modification commands (e.g., `rm`, `rm -rf`)
- System-level operations (e.g., `sudo`, `chown`)
- Package management (e.g., `cargo publish`, without explicit permission)
- Data exfiltration or credential exposure
- Any command not explicitly in the `allow` list

---

## Integration with Development Workflow

### Phase 3 Context

Per `CLAUDE.md`, Omega is in Phase 3 (complete) with Phase 4 next. The settings support:

- **Assisted code generation** — Claude Code can compile changes as they're made
- **Error feedback loops** — Build failures are immediately visible
- **Workspace validation** — All 6 crates (omega-core, omega-providers, omega-channels, omega-memory, omega-skills, omega-sandbox) can be compiled together

### Recommended Usage

When using Claude Code with Omega:

```bash
# Claude Code can run these (permitted):
cargo build
cargo build --release
cargo build --example gateway

# Claude Code cannot run these without additional permissions:
cargo test                    # Testing not currently allowed
cargo fmt                     # Formatting not currently allowed
cargo clippy                  # Linting not currently allowed
cargo publish                 # Publishing not currently allowed
```

---

## Extending Permissions

To allow additional Bash commands in future development phases, add entries to the `permissions.allow` array:

```json
{
  "permissions": {
    "allow": [
      "Bash(cargo build:*)",
      "Bash(cargo test:*)",
      "Bash(cargo fmt:*)",
      "Bash(cargo clippy:*)"
    ]
  }
}
```

**Note:** Permissions should be reviewed and justified before expanding, as broader access increases the scope of operations Claude Code can perform.

---

## Related Files

- **Project Guidelines:** `/Users/isudoajl/ownCloud/Projects/omega/CLAUDE.md`
- **Configuration Files:**
  - `config.toml` (gitignored, application config)
  - `config.example.toml` (template)
- **Source Code:** `/Users/isudoajl/ownCloud/Projects/omega/src/`
- **Workspace Crates:** `/Users/isudoajl/ownCloud/Projects/omega/crates/`

---

## Version History

- **Created:** February 2026
- **Last Updated:** February 16, 2026
- **Status:** Active

---

## Notes

1. This file is **not gitignored** and can be committed to the repository to standardize developer experience across team members working with Claude Code.

2. The `*.local.*` naming convention indicates project-specific settings that are local to this workspace.

3. As Omega evolves through Phase 4 (alternative providers, skills system, sandbox, cron scheduler), additional permissions may be needed to support new development workflows.
