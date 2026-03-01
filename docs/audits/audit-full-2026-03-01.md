# Full Codebase Audit — 2026-03-01

> Read-only audit across all 8 milestones. No code changes made.

## Executive Summary

| Metric | Count |
|--------|-------|
| **Total findings** | **109** |
| P0 (Critical) | 4 |
| P1 (Major) | 15 |
| P2 (Minor) | 30 |
| P3 (Suggestions) | 12 |
| Specs/Docs Drift | 48 |

**Verdict: Requires changes.** 4 critical security findings must be addressed before the next release. Specs/docs drift is severe — most specs describe an earlier version of the codebase and cannot be trusted as references.

### Positive Observations
- Zero `unwrap()` in production code — all confined to tests
- Zero `println!()` outside CLI init functions — tracing used consistently
- All production files under 500 lines
- SQL injection prevention is excellent — all queries use parameterized binding
- Strong error propagation via `OmegaError` — no swallowed errors in core paths
- Good test coverage for serialization, markers, and message splitting
- Sandbox dual-layer design (code + OS level) is architecturally sound

---

## P0: Critical (4 findings)

### P0-001: Auth bypass — empty allowed_users permits all users when auth is enabled
- **Location:** `backend/src/gateway/auth.rs:21-23` (Telegram), `:47` (WhatsApp)
- **Category:** Security
- **Description:** When `auth.enabled = true` and `allowed_users = []`, `check_auth()` returns `None` (allow all). Code comment says "Empty list = allow all (for easy testing)." This contradicts `specs/config-example-toml.md` which says empty list with auth enabled means "no users allowed."
- **Impact:** Any user can interact with the agent when operator believes access is restricted.
- **Fix:** Deny all when `users.is_empty()` and `auth_config.enabled == true`.

### P0-002: Sandbox bypass via relative paths in code-level enforcement
- **Location:** `backend/crates/omega-sandbox/src/lib.rs:62-68` (`is_write_blocked`), `:116-122` (`is_read_blocked`); exploitable via `backend/crates/omega-providers/src/tools.rs:222-255`
- **Category:** Security
- **Description:** Both `is_write_blocked()` and `is_read_blocked()` return `false` for relative paths. The tool executor passes AI-provided paths directly without canonicalization. A path like `../../data/memory.db` bypasses code-level sandbox.
- **Impact:** All HTTP providers (OpenAI, Anthropic, Ollama, OpenRouter, Gemini) can read/write `memory.db` and `config.toml` via relative paths. Claude Code CLI is still protected by OS-level Seatbelt/Landlock.
- **Fix:** Resolve relative paths against workspace `cwd` before checking, or block relative paths entirely.

### P0-003: ToolExecutor.config_path never set — config.toml read protection incomplete
- **Location:** `backend/crates/omega-providers/src/tools.rs:46,65`
- **Category:** Security
- **Description:** `ToolExecutor` has `config_path: Option<PathBuf>` always initialized to `None`. The `with_config_path()` builder documented in specs was never implemented. When config.toml is stored outside `~/.omega/`, `is_read_blocked()` cannot protect it.
- **Impact:** Custom config paths (e.g., `-c /custom/config.toml`) are not protected by tool executor read checks.
- **Fix:** Implement `with_config_path()` and pass the actual config path from the gateway when constructing `ToolExecutor`.

### P0-004: Seatbelt profile does not block writes to config.toml (macOS)
- **Location:** `backend/crates/omega-sandbox/src/seatbelt.rs:26-45`
- **Category:** Security
- **Description:** Seatbelt profile blocks reads to config.toml but NOT writes. Linux Landlock correctly blocks both. Code-level `is_write_blocked()` also does not protect config.toml (only protects `data/` directory).
- **Impact:** On macOS, Claude Code subprocess can overwrite `config.toml`, potentially injecting API keys or altering auth settings.
- **Fix:** Add config.toml to Seatbelt `file-write*` deny block. Add config.toml write protection to `is_write_blocked()`.

---

## P1: Major (15 findings)

### P1-001: API server runs without authentication by default
- **Location:** `backend/src/api.rs:264-268`
- **Category:** Security
- Enabling `api.enabled = true` without setting `api_key` exposes an open HTTP API on port 3000 (webhook injection, WhatsApp pairing).
- **Fix:** Log a warning at startup. Consider requiring `api_key` when API is enabled.

### P1-002: constant_time_eq leaks token length
- **Location:** `backend/src/api.rs:51-58`
- **Category:** Security
- Returns `false` immediately when `a.len() != b.len()`, leaking token length via timing side-channel.
- **Fix:** Hash both values with SHA-256 before comparing, or pad to fixed length.

### P1-003: Case-sensitive role tag matching in sanitizer is bypassable
- **Location:** `backend/crates/omega-core/src/sanitize.rs:30-49`
- **Category:** Security
- Covers `[System]` and `[SYSTEM]` but not `[system]`, `[sYSTEM]`, etc.
- **Fix:** Perform case-insensitive matching.

### P1-004: Agent name path traversal unvalidated
- **Location:** `backend/crates/omega-providers/src/claude_code/tests.rs:348-367`
- **Category:** Security
- Test explicitly passes `"../../../etc/passwd"` as agent name and asserts it passes through unchanged. No validation anywhere.
- **Fix:** Reject agent names containing `/`, `\`, or `..`.

### P1-005: String-level prefix matching for OS directories causes false positives
- **Location:** `backend/crates/omega-sandbox/src/lib.rs:98-101`
- **Category:** Bug
- Uses `path_str.starts_with(prefix)` (byte comparison) instead of `Path::starts_with()` (component-aware). `/binaries/test` incorrectly matches `/bin`.
- **Fix:** Use `resolved.starts_with(prefix)` (component-aware Path method).

### P1-006: Per-message filesystem I/O for project loading
- **Location:** `backend/src/gateway/pipeline.rs:136`
- **Category:** Performance
- `load_projects()` called on every incoming message, reading `~/.omega/projects/` directory.
- **Fix:** Cache projects in Gateway struct, refresh periodically.

### P1-007: O(n²) message cloning in agentic loops (all HTTP providers)
- **Location:** `openai.rs:194`, `ollama.rs:284`, `anthropic.rs:289`, `gemini.rs:319`
- **Category:** Performance
- Entire `messages` vector + `tools` vector cloned every loop iteration. O(n²) total allocations.
- **Fix:** Use `Cow` or serialize once and reuse.

### P1-008: Missing tests for security-critical auth module
- **Location:** `backend/src/gateway/auth.rs`
- **Category:** Missing Test
- Zero test coverage for the authorization gatekeeper. Edge cases untested.

### P1-009: Landlock restrictions skip non-existent paths (TOCTOU)
- **Location:** `backend/crates/omega-sandbox/src/landlock_sandbox.rs:96-104`
- **Category:** Security
- Checks `data_data.exists()` before adding Landlock rules. On first run, paths may not exist yet.
- **Fix:** Ensure dirs are created before Landlock rules are applied.

### P1-010: selfcheck.rs has no timeout on HTTP request
- **Location:** `backend/src/selfcheck.rs:113`
- **Category:** Performance
- `reqwest::Client::new()` with no timeout. Startup can hang if Telegram API unreachable.
- **Fix:** Add 10-second timeout.

### P1-011: Repository URL inconsistency
- **Location:** `backend/Cargo.toml:9` vs `CLAUDE.md:88`
- **Category:** Tech Debt
- Cargo.toml says `github.com/omega-cortex/omega`, CLAUDE.md says `github.com/omgagi/omega`.

### P1-012: Blocking I/O (subprocess) inside async runtime
- **Location:** `backend/crates/omega-skills/src/parse.rs:65-73`
- **Category:** Performance
- `which_exists()` spawns blocking subprocess for each tool check, running inside tokio runtime.
- **Fix:** Use pure-Rust PATH search or `tokio::task::spawn_blocking()`.

### P1-013: Telegram send_text swallows non-Markdown errors
- **Location:** `backend/crates/omega-channels/src/telegram/send.rs:29-54`
- **Category:** Tech Debt
- Non-Markdown-related send errors return `Ok(())` — caller thinks message was delivered.
- **Fix:** Return `Err` for non-Markdown failures.

### P1-014: WhatsApp Mutex held during retry_send (up to 3.5s)
- **Location:** `backend/crates/omega-channels/src/whatsapp/channel.rs:70-100`
- **Category:** Performance
- Holds Mutex lock on `self.client` for entire retry duration, blocking all other operations.
- **Fix:** Clone the `Arc<Client>` and release the lock immediately.

### P1-015: WhatsApp SQLite session store missing WAL mode
- **Location:** `backend/crates/omega-channels/src/whatsapp_store/mod.rs:22`
- **Category:** Performance
- Main omega-memory store correctly sets WAL mode, but WhatsApp session store does not.
- **Fix:** Add `.journal_mode(SqliteJournalMode::Wal)` to WhatsApp store connection.

---

## P2: Minor (30 findings)

### Security (3)
| ID | Location | Description |
|----|----------|-------------|
| P2-SEC-001 | `sanitize.rs:53-68` | Override phrase detection bypassable via Unicode homoglyphs, double spaces |
| P2-SEC-002 | `omega-skills/skills.rs:163-208` | No path traversal validation in skill/project loading (symlink following) |
| P2-SEC-003 | `omega-skills/skills.rs:186-194` | MCP frontmatter command field trusted without validation |

### Performance (5)
| ID | Location | Description |
|----|----------|-------------|
| P2-PERF-001 | `whatsapp_store/app_sync_store.rs:82-96` | Individual INSERTs without transaction in `put_mutation_macs` |
| P2-PERF-002 | `whatsapp_store/protocol_store.rs:27-37` | Individual INSERTs without transaction in `add_skdm_recipients` |
| P2-PERF-003 | `whatsapp/events.rs:127` | New `reqwest::Client` created per voice message (should reuse) |
| P2-PERF-004 | `omega-memory/store/context.rs:42-49` | DESC + reverse in Rust instead of ASC subquery |
| P2-PERF-005 | `omega-memory/store/context.rs:60-101` | 7 sequential DB queries that could be parallelized with `tokio::join!` |

### Tech Debt (12)
| ID | Location | Description |
|----|----------|-------------|
| P2-DEBT-001 | `config/mod.rs:202-254` | `migrate_layout` uses blocking I/O in async context |
| P2-DEBT-002 | `config/prompts.rs:140-227` | `install_bundled_prompts` and `Prompts::load` use blocking I/O |
| P2-DEBT-003 | `config/mod.rs:321-351` | `config::load` uses blocking `std::fs::read_to_string` |
| P2-DEBT-004 | `telegram/send.rs:88-91` | `send_photo_bytes` swallows errors, returns `Ok(())` |
| P2-DEBT-005 | `telegram/send.rs + whatsapp/send.rs` | `split_message` function duplicated identically |
| P2-DEBT-006 | `telegram/polling.rs:285` | `download_telegram_file` has no file size limit |
| P2-DEBT-007 | `memory/store/tasks.rs:145-146` | `format!` for SQL offset (safe but fragile pattern) |
| P2-DEBT-008 | `memory/store/tasks.rs:87-121` | `get_due_tasks` returns 8-element tuple (use named struct) |
| P2-DEBT-009 | `memory/store/context_helpers.rs:71` | `build_system_prompt` has 9 parameters |
| P2-DEBT-010 | `gateway/mod.rs:78-93` | `Gateway::new` takes 14 parameters |
| P2-DEBT-011 | `init.rs:197-308` | 19 `println!` calls in non-interactive mode |
| P2-DEBT-012 | `builds_i18n.rs + builds_loop.rs` | 4 TODO comments for phase-2 work |

### Dead Code (2)
| ID | Location | Description |
|----|----------|-------------|
| P2-DEAD-001 | `gateway/routing.rs` | `classify_and_route()` and `execute_steps()` are dead code |
| P2-DEAD-002 | `openai.rs:134`, `claude_code/mod.rs:42` | `#[allow(dead_code)]` on unused fields |

### Missing Tests (6)
| ID | Location | Description |
|----|----------|-------------|
| P2-TEST-001 | `omega-core/error.rs` | No tests for OmegaError conversions |
| P2-TEST-002 | `omega-core/message.rs` | No tests for IncomingMessage/OutgoingMessage |
| P2-TEST-003 | `provider_builder.rs` | Provider factory has no tests |
| P2-TEST-004 | `gateway/pipeline.rs` | Main message pipeline has no direct tests |
| P2-TEST-005 | `gateway/process_markers.rs` | Marker processing orchestration untested |
| P2-TEST-006 | `omega-memory/audit.rs` | `AuditLogger::log()` has no test for actual DB write |

### Compliance (2)
| ID | Location | Description |
|----|----------|-------------|
| P2-COMP-001 | `.gitignore:6` | Stale personal plist entry `com.ilozada.omega.plist` |
| P2-COMP-002 | `landlock_sandbox.rs:53-59` | `unsafe` block not listed in CLAUDE.md exemptions |

---

## P3: Suggestions (12 findings)

| ID | Location | Description |
|----|----------|-------------|
| P3-001 | `gateway/mod.rs:194-280` | Wrap frequently-cloned data in `Arc` (~42 clones in `run()`) |
| P3-002 | Multiple files | Add spec files for 6 undocumented modules |
| P3-003 | `specs/src-gateway-rs.md` | Update gateway spec: 19 files → 23, wrong line counts |
| P3-004 | `specs/config-example-toml.md` | Update WhatsApp fields, max_turns default, add Gemini |
| P3-005 | `parse.rs:65-73` | Replace `which` subprocess with pure-Rust PATH search |
| P3-006 | `skills.rs:344-345` | `match_skill_triggers` clones could use references |
| P3-007 | `parse.rs:6-13` | `expand_tilde` doesn't handle bare `~` without trailing `/` |
| P3-008 | `README.md:204` | License mismatch with `Cargo.toml` (MIT vs MIT OR Apache-2.0) |
| P3-009 | HTTP providers | Hardcoded 120-second timeout, not configurable |
| P3-010 | `claude_code/provider.rs:87` | Double parsing of stdout JSON |
| P3-011 | `whatsapp_store/` (5 files) | Zero test coverage for Signal protocol session persistence |
| P3-012 | `mcp_client.rs` | No integration test for connect/call/shutdown lifecycle |

---

## Specs/Docs Drift (48 findings)

### Severely Outdated Specs
These specs describe a fundamentally different version of the codebase:

| Spec | Issue |
|------|-------|
| `specs/core-lib.md` | Missing majority of public API (shellexpand, SYSTEM_FACT_KEYS, migrate_layout, Prompts, HeartbeatConfig, SchedulerConfig, ApiConfig, GeminiConfig, ContextNeeds, etc.) |
| `specs/core-config.md` | Wrong defaults: `max_turns` 10→25, `allowed_tools` has values→empty, `name` "Omega"→"OMEGA Ω". Missing `max_tokens` on AnthropicConfig |
| `specs/memory-lib.md` | Lists 3 migrations → 13 exist. Single `store.rs` → 10-file directory module. Missing 24 public methods. Missing migration 008 spec |
| `specs/cargo-toml-root.md` | Says omega-skills and omega-sandbox are "planned". Missing Gemini, tracing-appender, axum, cliclack |
| `specs/claude-md.md` | Gateway described as single file. WhatsApp and skills/sandbox as "planned". Missing Gemini |
| `specs/readme-md.md` | 6 commands → 16. Wrong Rust version. Missing trading/markers/background loops |
| `specs/config-example-toml.md` | Wrong WhatsApp fields. Wrong max_turns. Missing Gemini provider entirely |
| `specs/channels-telegram.md` | Single file → 5-file directory module. 2 tests → 10. Missing send_photo/as_any |
| `specs/channels-lib.md` | Missing entire WhatsApp public API surface |
| `specs/providers-tools.md` | Documents `with_config_path()` that was never implemented |
| `specs/providers-claude-code.md` | Wrong default max_turns (10→25), misleading default model |
| `docs/skills-cargo-toml.md` | Massively outdated — describes 8 deps (actual: 4), a `Skill` trait (doesn't exist), a `builtin` module (doesn't exist) |

### Missing Specs/Docs
- Missing specs: `provider_builder.rs`, `pair.rs`, `prompt_builder.rs`, `shared_markers.rs`, `pipeline_builds.rs`, `keywords_data.rs`, `memory-migration-008.md`
- Missing docs: `src-i18n-rs.md`, `src-task-confirmation-rs.md`, `src-init-wizard-rs.md`, `src-markers-rs.md`, `src-api-rs.md`, `memory-migration-006.md`, `memory-migration-008.md`, `memory-migration-012.md`

### Minor Drift
- `docs/sandbox-cargo-toml.md` omits `is_read_blocked` (says 2 exports, actual 3)
- `specs/skills-lib.md` missing `skills` field on `Project` struct
- `specs/skills-lib.md` lists 3 bundled skills (actual: 5)
- `docs/skills-lib.md` says projects have "no frontmatter" but code parses TOML/YAML frontmatter
- `specs/memory-lib.md` wrong return types for `find_idle_conversations`, `close_current_conversation`, `build_context`, `store_exchange`, `get_or_create_conversation`, `build_system_prompt`
- `specs/providers-lib.md` wrong `AnthropicProvider::from_config` signature
- `specs/src-gateway-rs.md` says 19 files (actual: 23+)
- `specs/src-main-rs.md` missing Pair/Service commands, provider_builder module
- `docs/DOCS.md` shows 10 commands (actual: 17)

---

## Recommendations

### Immediate (P0)
1. Fix auth bypass in `auth.rs` — deny all when `allowed_users` empty + auth enabled
2. Fix sandbox relative path bypass — resolve paths before checking
3. Implement `with_config_path()` on `ToolExecutor`
4. Add `config.toml` to Seatbelt write-deny + `is_write_blocked()`

### Short-term (P1)
5. Add auth module tests before any auth changes
6. Add startup warning when API runs without authentication
7. Fix constant_time_eq token length leak
8. Fix string-level prefix matching in sandbox OS directory check
9. Cache project loading in Gateway
10. Add timeout to selfcheck HTTP requests
11. Fix Telegram silent send failures

### Medium-term (P2-P3)
12. Bulk spec/docs update pass — current specs are unreliable
13. Add missing test coverage (auth, pipeline, provider_builder, process_markers)
14. Extract duplicated `split_message` to shared utility
15. Enable WAL mode for WhatsApp SQLite store
16. Remove dead code in routing.rs

---

## Audit Methodology

4 parallel reviewer agents, each covering a subset of milestones:
- Agent 1: Milestones 1-2 (Root + Binary) — 32 findings
- Agent 2: Milestones 3-4 (Core + Providers) — 29 findings
- Agent 3: Milestones 5-6 (Channels + Memory) — 32 findings
- Agent 4: Milestones 7-8 (Skills + Sandbox + Prompts) — 16 findings

Cross-referenced findings were deduplicated (e.g., relative path bypass flagged by both Agent 2 and Agent 4).
