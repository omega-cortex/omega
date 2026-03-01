# Bugfix Analysis: Config Path Resolution

**Date:** 2026-03-01
**Severity:** High — OMEGA reads config from source repo directory instead of `~/.omega/config.toml`

---

## Bug Description

OMEGA reads `config.toml` from the source code directory instead of from `~/.omega/config.toml`. The LaunchAgent plist bakes in an absolute path to `backend/config.toml` in the source repo.

## Root Cause (3 reinforcing causes)

### Cause 1: CLI default is a relative path
`main.rs:41` — `default_value = "config.toml"` resolves relative to CWD, not `~/.omega/`.

### Cause 2: init.rs generates config in CWD
`init.rs:114,271` — `let config_path = "config.toml"` writes to CWD instead of `~/.omega/config.toml`.

### Cause 3: service.rs canonicalizes the CWD path and bakes it into the plist
`service.rs:144` — `Path::new(config_path).canonicalize()` resolves `config.toml` to the source directory absolute path, which gets written into the LaunchAgent plist.

### Evidence
Current plist at `~/Library/LaunchAgents/com.omega-cortex.omega.plist`:
```xml
<string>/Users/isudoajl/ownCloud/Projects/omega/backend/config.toml</string>
```

### Missing: No tilde expansion in config::load()
`config/mod.rs:323` — `Path::new(path)` does not expand `~`, so even if the default were `~/.omega/config.toml`, it would fail the `.exists()` check.

## Requirements

| ID | Requirement | Priority |
|----|------------|----------|
| REQ-CFG-001 | Change CLI default config path from `"config.toml"` to `"~/.omega/config.toml"` | Must |
| REQ-CFG-002 | `omega init` writes config to `~/.omega/config.toml` instead of CWD | Must |
| REQ-CFG-003 | `omega service install` uses `~/.omega/config.toml` as canonical path | Must |
| REQ-CFG-005 | `config::load()` resolves `~` in path before file existence check | Must |

## Acceptance Criteria

### REQ-CFG-001
- [ ] `Cli.config` default value is `"~/.omega/config.toml"`
- [ ] Running `omega start` without `-c` reads from `~/.omega/config.toml`
- [ ] Explicit `-c /other/path.toml` still works

### REQ-CFG-002
- [ ] Interactive init writes to `~/.omega/config.toml`
- [ ] Non-interactive init writes to `~/.omega/config.toml`
- [ ] Existing `~/.omega/config.toml` is not overwritten

### REQ-CFG-003
- [ ] Generated plist points to expanded `~/.omega/config.toml`
- [ ] WorkingDirectory in plist is `~/.omega` (not source directory)
- [ ] Service works after source directory is deleted

### REQ-CFG-005
- [ ] `load("~/.omega/config.toml")` correctly expands tilde
- [ ] Falls back to defaults if file does not exist

## Impact Analysis

| File | Change | Risk |
|------|--------|------|
| `backend/src/main.rs:41` | Default value change | Low |
| `backend/src/init.rs:114,128,271,284` | Config write path | Low |
| `backend/src/service.rs:133-155,322-340` | Config path resolution | Medium |
| `backend/crates/omega-core/src/config/mod.rs:322-323` | Add shellexpand call | Low |

## Migration Note

After deploying the fix, users MUST:
1. Copy their real `config.toml` to `~/.omega/config.toml`
2. Run `omega service install` to regenerate the plist
