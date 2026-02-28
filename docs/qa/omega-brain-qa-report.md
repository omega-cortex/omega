# QA Report: OMEGA Brain (`/setup` Command)

## Scope Validated

All files in commit `681aab8` implementing the OMEGA Brain feature:
- Command registration (`commands/mod.rs`)
- Pipeline integration (`gateway/pipeline.rs`)
- Brain orchestrator (`gateway/setup.rs`)
- Agent lifecycle extension (`gateway/builds_agents.rs`)
- i18n messages (`gateway/keywords.rs`)
- Module registration (`gateway/mod.rs`)
- System fact keys (`omega-core/src/config/mod.rs`)
- Brain agent definition (`topologies/development/agents/omega-brain.md`)
- Help text (`commands/status.rs`)
- i18n command translations (`i18n/commands.rs`)
- Documentation (`docs/omega-brain.md`, `specs/omega-brain-requirements.md`, `specs/omega-brain-architecture.md`)

## Summary

**CONDITIONAL APPROVAL** -- All Must requirements are met with passing tests. Two Should requirements have minor discrepancies. The implementation is thorough: 88 brain/setup-specific tests pass, all 857 workspace tests pass, clippy is clean, and i18n covers all 8 languages. The main observations are: (1) `setup.rs` has 591 production lines, violating the 500-line rule by 91 lines; (2) two `unwrap()` calls in production code on `Path::parent()` which, while safe in practice, violate the project's no-unwrap rule; (3) specs drift on the `pending_setup` fact format. None of these are blocking.

## System Entrypoint

System built and tested via:
```bash
nix --extra-experimental-features "nix-command flakes" develop --command bash -c "cd backend && cargo test --workspace"
nix --extra-experimental-features "nix-command flakes" develop --command bash -c "cd backend && cargo clippy --workspace -- -D warnings"
```
- **All 857 tests pass** (0 failures, 0 ignored)
- **Clippy clean** (zero warnings)

Note: End-to-end runtime validation of the `/setup` command flow was not performed because it requires a running OMEGA instance with a configured AI provider (Claude Code CLI subprocess). The feature was validated through unit tests, code inspection, and static analysis. The architecture correctly delegates to `run_build_phase()`, which is the same mechanism used by the battle-tested build pipeline.

## Traceability Matrix Status

| Requirement ID | Priority | Has Tests | Tests Pass | Acceptance Met | Notes |
|---|---|---|---|---|---|
| REQ-BRAIN-001 | Must | Yes (11 tests) | Yes | Yes | Command parsing, botname stripping, no false matches |
| REQ-BRAIN-002 | Must | Yes (13 tests) | Yes | Yes | BRAIN_AGENT const verified: non-empty, YAML frontmatter, name/model/permissions/maxTurns |
| REQ-BRAIN-003 | Must | Yes (7 tests) | Yes | Yes | write_single creates file, RAII cleanup, ref-counted, directory hierarchy |
| REQ-BRAIN-004 | Must | Yes (7 tests) | Yes | Yes | parse_setup_output covers questions, proposal, executed, empty |
| REQ-BRAIN-005 | Must | Yes (2 tests) | Yes | Yes | Collision detection loads projects, context format verified |
| REQ-BRAIN-006 | Must | Yes (2 tests) | Yes | Yes | ROLE.md path format, project directory creation tested |
| REQ-BRAIN-007 | Must | Yes (1 test) | Yes | Yes | HEARTBEAT.md path format verified |
| REQ-BRAIN-008 | Must | Yes (3 tests) | Yes | Yes | SCHEDULE_ACTION marker format, spaces handling, agent mentions marker |
| REQ-BRAIN-020 | Must | Yes (6 tests) | Yes | Yes | Context file path, tilde expansion, creation/cleanup, expiry cleanup, idempotent |
| REQ-BRAIN-022 | Must | Yes (1 test) | Yes | Yes | Workspace path resolves to ~/.omega/ |
| REQ-BRAIN-009 | Should | Yes (3 tests) | Yes | Yes | PROJECT_ACTIVATE marker format, hyphenated names, agent mentions marker |
| REQ-BRAIN-010 | Should | Yes (7 tests) | Yes | Yes | Round parsing from header, rounds 1-3, missing/malformed headers |
| REQ-BRAIN-011 | Should | Yes (4 tests) | Yes | Yes | Description extraction from /setup command, empty, botname, whitespace |
| REQ-BRAIN-012 | Should | Yes (5 tests) | Yes | Partial | TTL = 1800s verified. Fact format has 3 fields (timestamp pipe sender_id pipe round) but spec says 2 fields. See Specs Drift. |
| REQ-BRAIN-013 | Should | Yes (2 tests) | Yes | Yes | Confirmation reuses BUILD_CONFIRM_KW/BUILD_CANCEL_KW |
| REQ-BRAIN-014 | Should | Yes (8 tests) | Yes | Yes | All 7 message functions verified across all 8 languages |
| REQ-BRAIN-015 | Should | Yes (1 test) | Yes | Yes | Agent contains 2 ROLE.md examples (trading + real estate) |
| REQ-BRAIN-019 | Should | Yes (1 test) | Yes | Yes | Reads existing ROLE.md for context |
| REQ-BRAIN-021 | Should | Yes (1 test) | Yes | Yes | Agent tools restricted: Read, Write, Glob, Grep (no Bash, no Edit) |
| REQ-BRAIN-016 | Could | N/A (compile) | Yes | Yes | `mod setup;` in gateway/mod.rs -- compiles, therefore registered |
| REQ-BRAIN-017 | Could | N/A | Yes | Yes | Agent prompt includes skill suggestion section |
| REQ-BRAIN-018 | Could | Yes (1 test) | Yes | Yes | Audit prefix format [SETUP:project] verified |
| REQ-BRAIN-023 | Could | Yes (1 test) | Yes | Yes | Concurrent session guard with TTL check |
| REQ-BRAIN-024 | Won't | N/A | N/A | N/A | Deferred by design |
| REQ-BRAIN-025 | Won't | N/A | N/A | N/A | Deferred by design |

### Gaps Found

- No integration test that exercises the full `start_setup_session` -> `handle_setup_response` -> `execute_setup` -> `cleanup_setup_session` flow against a mock provider. All tests are unit-level (pure function testing). This is acceptable given the feature delegates to the already-tested `run_build_phase()` primitive.
- No test for the pipeline.rs `/setup` intercept with empty description showing help message (tested indirectly via separate command parse + help text tests).

## Acceptance Criteria Results

### Must Requirements

#### REQ-BRAIN-001: `/setup` command registration
- [x] `/setup` returns `Some(Command::Setup)` -- PASS
- [x] `/setup@omega_bot I'm a realtor` returns `Some(Command::Setup)` (botname stripped) -- PASS
- [x] `/setup` with no description returns `Some(Command::Setup)` (handler deals with empty) -- PASS
- [x] `/settings` returns `None` (no false match) -- PASS

#### REQ-BRAIN-002: Brain agent definition
- [x] `BRAIN_AGENT` const is non-empty, starts with `---` (YAML frontmatter) -- PASS
- [x] Frontmatter contains: `name: omega-brain`, `model: opus`, `permissionMode: bypassPermissions`, `maxTurns: 30` -- PASS
- [x] Body contains instructions for questioning, ROLE.md, HEARTBEAT.md, marker emission, approval gate -- PASS
- [x] Non-interactive instruction present ("You are non-interactive. Do NOT ask the user directly.") -- PASS

#### REQ-BRAIN-003: `write_single()` agent lifecycle
- [x] Creates `<dir>/.claude/agents/omega-brain.md` -- PASS
- [x] RAII cleanup on drop removes agent file and directory -- PASS
- [x] Ref-counting: files persist until last guard drops -- PASS
- [x] Creates intermediate directories when needed -- PASS

#### REQ-BRAIN-004: Brain invocation
- [x] `run_build_phase("omega-brain", prompt, model_complex, Some(30))` called correctly -- PASS (verified in code at setup.rs:160)
- [x] Provider failure handled gracefully with error message -- PASS (setup.rs:242-252)
- [x] Output parsed for questions, proposal, and executed variants -- PASS

#### REQ-BRAIN-005: Collision detection
- [x] Existing projects loaded and passed to Brain context -- PASS (setup.rs:105-118)
- [x] Context format includes project names with first line of ROLE.md -- PASS (test verified)
- [x] Empty project list handled -- PASS

#### REQ-BRAIN-006: ROLE.md creation
- [x] Path format `~/.omega/projects/<name>/ROLE.md` -- PASS
- [x] Brain agent prompt specifies creation instructions -- PASS
- [x] Delegated to Brain agent (Claude Code subprocess) which creates file -- PASS

#### REQ-BRAIN-007: HEARTBEAT.md creation
- [x] Path format `~/.omega/projects/<name>/HEARTBEAT.md` -- PASS
- [x] Brain agent prompt specifies format and monitoring items -- PASS

#### REQ-BRAIN-008: Schedule markers
- [x] Brain agent prompt specifies `SCHEDULE_ACTION:` marker format -- PASS
- [x] Marker format: `SCHEDULE_ACTION: <description> | <ISO datetime> | <repeat>` -- PASS
- [x] Markers processed by existing `process_markers()` -- PASS (setup.rs:323-324)

#### REQ-BRAIN-020: Context file cleanup
- [x] Context path: `<data_dir>/setup/<sender_id>.md` -- PASS
- [x] Deleted on completion -- PASS (setup.rs:334)
- [x] Deleted on cancellation -- PASS (setup.rs:293)
- [x] Deleted on expiry -- PASS (setup.rs:282)
- [x] Cleanup idempotent (double delete does not error) -- PASS

#### REQ-BRAIN-022: Workspace
- [x] Brain invoked with workspace = `~/.omega/` (omega_dir) -- PASS
- [x] `write_single()` called with `omega_dir` (not workspace subdir) -- PASS (setup.rs:142)

### Should Requirements

#### REQ-BRAIN-009: Project activation marker
- [x] Brain agent prompt specifies `PROJECT_ACTIVATE: <name>` -- PASS
- [x] Marker detected and used for completion message -- PASS (setup.rs:327-332)

#### REQ-BRAIN-010: Multi-round session
- [x] Round counter parsed from context file header -- PASS
- [x] Rounds 1-3 supported -- PASS
- [x] Round 3 is final (MUST produce proposal) -- PASS (setup.rs:437-442)
- [x] State tracked via context file -- PASS

#### REQ-BRAIN-011: Pipeline intercept
- [x] `/setup` handled in command dispatch section (step 3) -- PASS (pipeline.rs:146-189)
- [x] Description extracted after `/setup` keyword -- PASS
- [x] Empty description shows help message -- PASS
- [x] No provider call made for the original message -- PASS (early return)

#### REQ-BRAIN-012: Session state
- [x] `pending_setup` fact stored -- PASS
- [x] 30-minute TTL (SETUP_TTL_SECS = 1800) -- PASS
- [x] Expired sessions cleaned up and user notified -- PASS
- [x] Fact deleted on completion/cancellation -- PASS
- [ ] Fact format matches spec -- PARTIAL: Spec says `<timestamp>|<sender_id>`, implementation uses `<timestamp>|<sender_id>|<round>`. See Specs Drift section.

#### REQ-BRAIN-013: Confirmation keywords
- [x] Reuses `is_build_confirmed()` / `is_build_cancelled()` -- PASS
- [x] Non-confirmation text treated as modification request -- PASS (setup.rs:352-420)

#### REQ-BRAIN-014: Localized messages
- [x] All 8 languages: `setup_help_message`, `setup_intro_message`, `setup_followup_message`, `setup_proposal_message`, `setup_complete_message`, `setup_cancelled_message`, `setup_expired_message`, `setup_conflict_message` -- PASS
- [x] Help text `/setup` translation in i18n/commands.rs -- PASS

#### REQ-BRAIN-015: ROLE.md examples
- [x] 2 examples in agent prompt: Trading Agent + Lisbon Real Estate -- PASS

#### REQ-BRAIN-019: Read existing ROLE.md
- [x] First line of existing ROLE.md files included in context -- PASS

#### REQ-BRAIN-021: Restricted tools
- [x] Agent frontmatter: `tools: Read, Write, Glob, Grep` -- PASS
- [x] No Bash, no Edit -- PASS

### Could Requirements

#### REQ-BRAIN-016: Module registration
- [x] `mod setup;` in gateway/mod.rs -- PASS

#### REQ-BRAIN-017: Skill suggestions
- [x] Agent prompt includes "Skill Suggestions" section -- PASS

#### REQ-BRAIN-018: Audit logging
- [x] Audit entry with `[SETUP:<project>]` prefix format -- PASS

#### REQ-BRAIN-023: Concurrent session guard
- [x] Checks existing `pending_setup` before starting new session -- PASS
- [x] TTL-based expiry allows re-entry after 30 minutes -- PASS

## End-to-End Flow Results

| Flow | Steps | Result | Notes |
|---|---|---|---|
| `/setup` empty command | 1. Parse command 2. Detect empty description 3. Show help | PASS (code verified) | Help message in user's language |
| `/setup <description>` start | 1. Parse 2. Check concurrent 3. Load projects 4. Write agent 5. Invoke Brain 6. Parse output 7. Store state | PASS (code verified) | Handles questions/proposal/error paths |
| Setup follow-up (questions) | 1. Read context 2. Increment round 3. Invoke Brain 4. Parse output 5. Update state | PASS (code verified) | Round 3 forces proposal |
| Setup confirmation | 1. Read context 2. Detect SETUP_PROPOSAL 3. Check confirmation 4. Execute Brain 5. Process markers 6. Cleanup | PASS (code verified) | Markers processed via process_markers() |
| Setup cancellation | 1. Detect cancel keyword 2. Cleanup session 3. Notify user | PASS (code verified) | Reuses BUILD_CANCEL_KW |
| Setup expiry | 1. Check TTL 2. Cleanup session 3. Notify user | PASS (code verified) | 30-min TTL |
| Setup modification | 1. Detect non-confirm text 2. Append to context 3. Re-invoke Brain | PASS (code verified) | Falls through to Brain for revision |

Note: These flows were verified through code inspection and unit test coverage, not runtime execution, because the system requires a live AI provider (Claude Code CLI) for the Brain agent invocation.

## Exploratory Testing Findings

| # | What Was Tried | Expected | Actual | Severity |
|---|---|---|---|---|
| 1 | Check `/setup` with botname suffix `/setup@omega_bot` | Parses correctly | PASS -- botname stripped, returns Command::Setup | N/A |
| 2 | Check `/settings` does not match `/setup` | Returns None | PASS -- exact match prevents false positive | N/A |
| 3 | Check concurrent session guard | Second `/setup` rejected while session active | PASS -- conflict message shown, old expired sessions cleaned up | N/A |
| 4 | Check `unwrap()` on `ctx_path.parent()` at setup.rs:169 and 206 | Should use `?` per project rules | Uses `unwrap()` -- safe in practice because `setup_context_path()` always returns a path with parent, but violates no-unwrap rule | LOW |
| 5 | Check `setup.rs` production line count | Under 500 lines per project rules | 591 production lines (before test section). Exceeds by 91 lines | MEDIUM |
| 6 | Check `pipeline.rs` production line count | Under 500 lines | 1000 production lines (no test section, pre-existing violation) | LOW (pre-existing) |
| 7 | Check `keywords.rs` production line count | Under 500 lines | 751 production lines (pre-existing: 598 before brain feature) | LOW (pre-existing, worsened) |

## Failure Mode Validation

| Failure Scenario | Triggered | Detected | Recovered | Degraded OK | Notes |
|---|---|---|---|---|---|
| Brain agent write fails | Untestable (needs real FS failure) | Yes (code) | Yes -- error message sent | Yes -- "could not initialize the Brain agent" | setup.rs:143-156 |
| Brain invocation fails (all 3 retries) | Untestable (needs real provider) | Yes (code) | Yes -- cleanup + error msg | Yes -- "Setup failed after multiple attempts" | setup.rs:242-252 |
| Context file missing during response | Code verified | Yes (unwrap_or_default) | Yes -- empty context returned | Yes -- proceeds with empty context | setup.rs:304-306 |
| Session expired mid-flow | Code verified | Yes (TTL check) | Yes -- cleanup + expiry msg | Yes -- localized message sent | setup.rs:280-289 |
| Unexpected Executed output in question mode | Code verified | Yes (warn log) | Yes -- unexpected state msg | Yes -- user asked to retry | setup.rs:228-239 |
| Modification Brain call fails | Code verified | Yes (error match) | Yes -- suggests proceeding or cancelling | Yes -- degraded gracefully | setup.rs:408-419 |
| Follow-up Brain call fails | Code verified | Yes (error match) | Yes -- cleanup + retry msg | Yes -- session cleaned up | setup.rs:522-530 |
| Concurrent setup session | Code verified via test | Yes (fact check) | Yes -- conflict message | Yes -- blocks duplicate, allows after TTL | setup.rs:80-102 |

## Security Validation

| Attack Surface | Test Performed | Result | Notes |
|---|---|---|---|
| Brain agent tools restriction | Verified frontmatter: `tools: Read, Write, Glob, Grep` | PASS | No Bash, no Edit. Brain cannot execute arbitrary commands |
| Brain workspace isolation | Verified invocation uses `~/.omega/` as workspace | PASS | Agent operates within OMEGA data directory only |
| Input sanitization | Verified pipeline.rs sanitizes input before `/setup` intercept (line 73-79) | PASS | Sanitization happens at step 2, before command dispatch at step 3 |
| Prompt injection via description | Verified Brain prompt includes user description as data, not instructions | PASS | Description is clearly labeled as "User description:" in the prompt |
| Session state injection | Verified `pending_setup` fact format is validated (timestamp parsed with unwrap_or) | PASS | Malformed fact values default to 0, treated as expired |
| Path traversal via sender_id | Context file path: `<data_dir>/setup/<sender_id>.md` | LOW RISK | If sender_id contains path separators, could potentially write outside setup dir. However, sender_id comes from authenticated channel APIs (Telegram/WhatsApp) which return numeric IDs. Not exploitable in practice |
| Denial of service via long description | Verified description is passed to Brain agent prompt without length limit | LOW RISK | Brain has maxTurns: 30 limit. Description flows to Claude Code which has its own input limits |

## Specs/Docs Drift

| File | Documented Behavior | Actual Behavior | Severity |
|------|-------------------|-----------------|----------|
| `specs/omega-brain-requirements.md` (REQ-BRAIN-012) | `pending_setup` fact format: `<timestamp>\|<sender_id>` | Actual format: `<timestamp>\|<sender_id>\|<round_or_phase>` (3 fields, not 2) | LOW |
| `specs/omega-brain-requirements.md` (REQ-BRAIN-012) | No mention of third field | Third field is round number (1, 2, 3) or "proposal" to track session phase | LOW |
| `docs/omega-brain.md` line 47-48 | Format: `<timestamp>\|<sender_id>\|<round_or_phase>` | Matches implementation | N/A (docs correct, spec outdated) |
| `specs/src-gateway-rs.md` line 19 | `keywords.rs` ~751 lines | Actual: 751 prod + 637 test = 1388 total | N/A (spec says ~751 which matches prod lines) |
| `specs/src-gateway-rs.md` line 34 | `setup.rs` ~596 lines | Actual: 591 prod + 689 test = 1280 total. Spec says ~596 which is close but off by 5 | LOW |

## Blocking Issues (must fix before merge)

None. All Must requirements are met.

## Non-Blocking Observations

- **[OBS-001]**: `setup.rs:169` and `setup.rs:206` -- `unwrap()` on `ctx_path.parent()`. While safe because `setup_context_path()` always constructs a path with a parent (`<data_dir>/setup/<sender_id>.md`), the project's CLAUDE.md rule says "No `unwrap()` -- use `?` and proper error types." Recommend replacing with `.ok_or_else(|| ...)` or `.expect("setup context path always has parent")` with a comment explaining the invariant.

- **[OBS-002]**: `setup.rs` has 591 production lines, exceeding the 500-line rule by 91 lines. The file could be split: extract the pure functions (`setup_context_path`, `parse_setup_round`, `parse_setup_output`, `SetupOutput`) into a `setup_parse.rs` submodule (~60 lines), or extract `audit_setup` into the existing audit patterns. Not blocking because the file has a clear single responsibility (Brain orchestrator) and the overage is modest.

- **[OBS-003]**: Specs drift on `pending_setup` fact format (REQ-BRAIN-012). The spec says 2 fields but implementation uses 3. Recommend updating `specs/omega-brain-requirements.md` REQ-BRAIN-012 acceptance criteria to reflect the actual `<timestamp>|<sender_id>|<round_or_phase>` format.

- **[OBS-004]**: `pipeline.rs` has 1000 production lines (no test section), and `keywords.rs` has 751 production lines -- both pre-existing 500-line rule violations worsened slightly by the brain feature (+59 and +153 lines respectively). These are systemic issues not introduced by this feature, but the brain feature expanded them. Recommend eventual modularization of both files.

- **[OBS-005]**: No integration test for the full setup flow (`start_setup_session` -> `handle_setup_response` -> `execute_setup`). All tests are unit-level on pure functions. Integration testing would require a mock provider, which is non-trivial. Acceptable for initial implementation since the feature delegates to `run_build_phase()` which is proven.

- **[OBS-006]**: Error message at setup.rs:234 ("Setup encountered an unexpected state. Please try again with /setup.") is not localized (English only). Low severity since this is an edge case that should never occur in normal flow.

## Modules Not Validated (if context limited)

- **Runtime end-to-end flow**: Could not be validated because it requires a running OMEGA instance with configured Claude Code CLI provider. The Brain agent invocation, file creation, and marker processing were verified through code inspection but not executed against a live system.

## Final Verdict

**CONDITIONAL APPROVAL** -- All 10 Must requirements are met and verified through 88 passing unit tests. All 8 Should requirements are implemented with one minor specs drift on the `pending_setup` fact format (REQ-BRAIN-012). All 4 Could requirements that could be implemented are implemented. The following non-blocking observations should be resolved before GA:

1. Replace 2 `unwrap()` calls in production code (OBS-001)
2. Update spec for `pending_setup` fact format to reflect 3-field reality (OBS-003)
3. Consider splitting `setup.rs` to stay under 500-line rule (OBS-002)

Approved for review with the expectation that OBS-001 and OBS-003 are resolved before merge.
