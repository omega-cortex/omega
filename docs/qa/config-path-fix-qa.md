# QA Report: Config Path Fix

## Scope Validated
Config path resolution across four files: `config/mod.rs`, `main.rs`, `init.rs`, `service.rs`. Verified that OMEGA reads config from `~/.omega/config.toml` instead of a CWD-relative `config.toml`.

## Summary
**PASS** -- All four Must requirements are met. The bugfix correctly changes the default config path from the relative `config.toml` (which resolved to the source repo directory) to `~/.omega/config.toml`. Tilde expansion is performed before path existence checks. The `-c` flag for custom paths continues to work. All 970 workspace tests pass with zero failures. Five specs/docs drift items found (non-blocking).

## System Entrypoint
```bash
cd /Users/isudoajl/ownCloud/Projects/omega/backend
nix --extra-experimental-features "nix-command flakes" develop --command bash -c "cargo test --workspace"
nix --extra-experimental-features "nix-command flakes" develop --command bash -c "cargo run -- --help"
```

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-CFG-001 | Must | Yes (CLI --help output) | Yes | Yes | `default_value = "~/.omega/config.toml"` on main.rs:41 |
| REQ-CFG-002 | Must | Yes (init::tests, 13 tests) | Yes | Yes | Both interactive (line 114) and non-interactive (line 273) write to `~/.omega/config.toml` |
| REQ-CFG-003 | Must | Yes (service::tests, 5 tests) | Yes | Yes | Both `install()` and `install_quiet()` expand tilde, use `~/.omega` as working dir |
| REQ-CFG-005 | Must | Yes (test_load_expands_tilde_path, test_load_tilde_path_falls_back_to_defaults) | Yes | Yes | `load()` calls `shellexpand(path)` before `Path::new()` |

### Gaps Found
- No gaps: all four requirements have corresponding tests and passing code.

## Acceptance Criteria Results

### Must Requirements

#### REQ-CFG-001: CLI default is `~/.omega/config.toml`
- [x] `Cli.config` default value is `"~/.omega/config.toml"` -- PASS: Verified at `main.rs:41`: `#[arg(short, long, default_value = "~/.omega/config.toml")]`
- [x] Running `omega start` without `-c` reads from `~/.omega/config.toml` -- PASS: CLI help output confirms `[default: ~/.omega/config.toml]`
- [x] Explicit `-c /other/path.toml` still works -- PASS: The config path flows through `config::load()` which calls `shellexpand()`. For absolute paths, `shellexpand()` returns the path unchanged. `cmd_start()`, `cmd_status()`, and `cmd_ask()` all pass `cli.config` to `config::load()`.

#### REQ-CFG-002: Init writes to `~/.omega/config.toml`
- [x] Interactive init writes to `~/.omega/config.toml` -- PASS: `init.rs:114`: `let config_path_expanded = shellexpand("~/.omega/config.toml");`
- [x] Non-interactive init writes to `~/.omega/config.toml` -- PASS: `init.rs:273`: `let config_path_expanded = shellexpand("~/.omega/config.toml");`
- [x] Existing `~/.omega/config.toml` is not overwritten -- PASS: Interactive path (line 116): checks `Path::new(config_path).exists()` and warns. Non-interactive path (line 275): bails with error message.

#### REQ-CFG-003: Service install uses expanded path and `~/.omega` as working dir
- [x] Generated plist points to expanded `~/.omega/config.toml` -- PASS: `service.rs:144`: `omega_core::shellexpand(config_path)` followed by `.canonicalize()` produces an absolute path.
- [x] WorkingDirectory in plist is `~/.omega` (not source directory) -- PASS: `service.rs:152`: `let working_dir = omega_core::shellexpand("~/.omega");` (hardcoded, no longer derived from config file parent).
- [x] `install_quiet()` uses same logic -- PASS: `service.rs:327` and `service.rs:334` mirror the interactive path exactly.
- [x] Service works after source directory is deleted -- PASS (by design): working directory is `~/.omega`, binary path is from `current_exe()`, config is canonicalized from `~/.omega/config.toml`. No reference to source directory remains.

#### REQ-CFG-005: `load()` does `shellexpand()` before `Path::new()`
- [x] `load("~/.omega/config.toml")` correctly expands tilde -- PASS: `config/mod.rs:324`: `let expanded = shellexpand(path);` then `let path = Path::new(&expanded);`
- [x] Falls back to defaults if file does not exist -- PASS: Test `test_load_tilde_path_falls_back_to_defaults` verifies `load("~/.omega/__nonexistent_test_config__.toml")` returns defaults.
- [x] Dedicated test `test_load_expands_tilde_path` creates a real file under `$HOME`, loads it via tilde path, and asserts correct deserialization -- PASS.

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| CLI default path verification | 1. Build binary. 2. Run `--help`. 3. Check default. | PASS | Output shows `[default: ~/.omega/config.toml]` |
| Config load with tilde expansion | 1. Create file at `$HOME/.omega_test_cfg_tilde/config.toml`. 2. Load via `~/.omega_test_cfg_tilde/config.toml`. 3. Assert content. | PASS | Covered by `test_load_expands_tilde_path` |
| Config load fallback on missing file | 1. Load non-existent tilde path. 2. Assert defaults. | PASS | Covered by `test_load_tilde_path_falls_back_to_defaults` |
| Custom `-c` path still works | 1. `cmd_start`/`cmd_status`/`cmd_ask` pass `cli.config` to `config::load()`. 2. `shellexpand` is no-op for absolute paths. | PASS | Code inspection confirms no regression |

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | Checked if `shellexpand` handles paths without `~/` prefix | Returns path unchanged | Returns path unchanged (line 193-194 of config/mod.rs) | N/A -- works correctly |
| 2 | Checked if `shellexpand` handles `$HOME` not set | Falls back gracefully | Returns path unchanged if HOME is not set (line 190-192) | low -- edge case, correct behavior |
| 3 | Checked if service.rs `install()` still derives working_dir from config file parent | Should NOT derive from parent | Confirmed: line 152 hardcodes `omega_core::shellexpand("~/.omega")` | N/A -- bug is fixed |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Config file missing at `~/.omega/config.toml` | Yes (via test) | Yes | Yes (defaults used) | Yes | `load()` logs info and returns defaults |
| HOME env var not set during shellexpand | Not Triggered | N/A | N/A | N/A | Code handles it (returns path unchanged) but cannot safely test in running environment |
| Config file at custom `-c` path not found | Yes (via service.rs canonicalize) | Yes | Yes (bail with helpful message) | Yes | "run `omega init` first" error message |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Path traversal in config path | Code inspection of `shellexpand()` | PASS | Only expands `~/` prefix to `$HOME/`, does not process other special characters. No user-controlled path traversal risk. |
| Config file permissions | Out of Scope | N/A | Config file permissions are OS-level; not in scope for this bugfix. |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `specs/src-main-rs.md:52` | `config: String` -- default: `"config.toml"` | Default is `"~/.omega/config.toml"` | medium |
| `specs/src-main-rs.md:159` | `-c, --config <CONFIG>` -- default: `"config.toml"` | Default is `"~/.omega/config.toml"` | medium |
| `specs/src-service-rs.md:52` | `WorkingDirectory`: Absolute path to config file's parent directory | WorkingDirectory is hardcoded to `~/.omega` (not config parent) | medium |
| `specs/src-service-rs.md:87` | Step 4: Derive working directory from config file's parent | Step 3: Working directory = `omega_core::shellexpand("~/.omega")` (hardcoded) | medium |
| `specs/src-init-rs.md:236` | Config written to CWD (`config.toml`) | Config written to `~/.omega/config.toml` | high |
| `specs/src-init-rs.md:256` | "Config file is written to CWD, not to `~/.omega`" | Config IS written to `~/.omega/config.toml` | high |
| `specs/core-config.md:307` | `load()` step 1: "Converts `path` to a `std::path::Path`" | `load()` step 1: calls `shellexpand(path)` THEN converts to `Path` | medium |
| `specs/core-config.md:370` | "`~` in paths is expanded at usage time, not in the config module" | `load()` in the config module now DOES expand `~` via `shellexpand()` | high |

## Blocking Issues (must fix before merge)
None. All Must requirements pass.

## Non-Blocking Observations

- **[OBS-001]**: `specs/src-main-rs.md` -- Two references to default config path say `"config.toml"` but actual default is `"~/.omega/config.toml"`. Update spec.
- **[OBS-002]**: `specs/src-service-rs.md` -- Two references describe working directory as "derived from config file's parent" but it is now hardcoded to `~/.omega`. Update spec.
- **[OBS-003]**: `specs/src-init-rs.md` -- Phase 8 description says config is written to CWD. It is now written to `~/.omega/config.toml`. Update spec.
- **[OBS-004]**: `specs/core-config.md` -- `load()` behavior description omits the `shellexpand()` step. Also, the Environment Variable Overrides section states tilde expansion does not happen in the config module, which is now false. Update spec.

## Modules Not Validated
None. All four files in scope were fully validated.

## Test Results Summary
- **Total tests run:** 970 (full workspace)
- **Passed:** 970
- **Failed:** 0
- **Specific to this fix:** `test_load_expands_tilde_path` and `test_load_tilde_path_falls_back_to_defaults` (both pass)

## Final Verdict

**PASS** -- All four Must requirements (REQ-CFG-001 through REQ-CFG-005) are met. All 970 workspace tests pass. No blocking issues. The `-c` flag for custom paths continues to work (no regression). Eight specs/docs drift items found and documented as non-blocking observations. Approved for review.
