# Functionalities: omega-sandbox

## Overview

OS-level system protection for the Omega agent. Three layers of defense: code-level path checking, OS-level sandboxing (Seatbelt on macOS, Landlock on Linux), and prompt-level instructions.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | protected_command() | Factory | `backend/crates/omega-sandbox/src/lib.rs:44` | Builds a Command with OS-level protection. Always active, dispatches to platform-specific implementation | seatbelt / landlock |
| 2 | is_write_blocked() | Guard | `backend/crates/omega-sandbox/src/lib.rs:62` | Checks if path is write-protected: data dir, config.toml, OS system dirs. Resolves symlinks. Blocks relative paths | -- |
| 3 | is_read_blocked() | Guard | `backend/crates/omega-sandbox/src/lib.rs:123` | Checks if path is read-protected: data dir, config.toml, external config. Resolves symlinks. Blocks relative paths | -- |
| 4 | Seatbelt (macOS) | Platform | `backend/crates/omega-sandbox/src/seatbelt.rs` | Apple Seatbelt sandbox via sandbox-exec: denies R/W to data dir, config; denies writes to system dirs | -- |
| 5 | Landlock (Linux) | Platform | `backend/crates/omega-sandbox/src/landlock_sandbox.rs` | Linux Landlock LSM via pre_exec hook (kernel 5.13+): broad read-only on /, full access to $HOME, restricted data/config | -- |

## Internal Dependencies

- Claude Code provider uses protected_command() for subprocess creation
- HTTP provider tool executors use is_write_blocked() and is_read_blocked() for code-level enforcement
- CLAUDE.md maintenance uses protected_command() for subprocess creation

## Dead Code / Unused

- None detected.
