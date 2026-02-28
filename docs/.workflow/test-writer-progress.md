# Test Writer Progress: OMEGA Brain Feature

## Status: COMPLETE

## Summary

75 new tests written across 4 files for the OMEGA Brain feature.
All tests are written BEFORE implementation (TDD red phase).
Tests will fail until the developer implements the production code.

## Modules Tested

### Module 1: Command Registration (`commands/tests.rs`) -- DONE
- 11 tests covering REQ-BRAIN-001 (Must)
- Tests: `/setup` parsing, botname suffix stripping, no false match on `/settings`,
  case sensitivity, unicode/emoji descriptions, help text inclusion
- All tests will FAIL until developer adds `Setup` variant to `Command` enum

### Module 2: Brain Agent Definition (`builds_agents.rs`) -- DONE
- 17 tests covering REQ-BRAIN-002 (Must), REQ-BRAIN-008 (Must), REQ-BRAIN-009 (Should),
  REQ-BRAIN-015 (Should), REQ-BRAIN-021 (Should)
- Tests: BRAIN_AGENT const non-empty, frontmatter validation (name, model, maxTurns,
  permissionMode, tools), body content (SETUP_QUESTIONS, SETUP_PROPOSAL, SETUP_EXECUTE,
  ROLE.md, HEARTBEAT.md, SCHEDULE_ACTION, PROJECT_ACTIVATE markers, non-interactive
  instruction, ROLE.md examples, restricted tools)
- All tests will FAIL until developer creates `topologies/development/agents/omega-brain.md`
  and adds `BRAIN_AGENT` const

### Module 3: Agent Lifecycle Extension (`builds_agents.rs`) -- DONE
- 7 tests covering REQ-BRAIN-003 (Must)
- Tests: `write_single()` creates single agent file, RAII cleanup on drop,
  ref-counting with two guards, intermediate directory creation, only one file created,
  empty content handling, idempotent drop
- All tests will FAIL until developer adds `write_single()` method to `AgentFilesGuard`

### Module 4: Brain Orchestrator (`setup.rs`) -- DONE
- 25 tests covering REQ-BRAIN-004 (Must), REQ-BRAIN-005 (Must), REQ-BRAIN-006 (Must),
  REQ-BRAIN-007 (Must), REQ-BRAIN-008 (Must), REQ-BRAIN-009 (Should),
  REQ-BRAIN-010 (Should), REQ-BRAIN-011 (Should), REQ-BRAIN-012 (Should),
  REQ-BRAIN-018 (Could), REQ-BRAIN-019 (Should), REQ-BRAIN-020 (Must),
  REQ-BRAIN-022 (Must), REQ-BRAIN-023 (Could)
- Tests: context path format, tilde expansion, context file creation/cleanup,
  workspace path validation, parse_setup_round (rounds 1-3, empty, malformed),
  parse_setup_output (questions/proposal/executed/edge cases), pending_setup fact format,
  TTL validation, collision detection context, concurrent session guard,
  description extraction, ROLE.md/HEARTBEAT.md path format, project directory creation,
  SCHEDULE_ACTION marker format, PROJECT_ACTIVATE marker format, audit log prefix
- File created with production stubs + full test module. Developer adds Gateway methods.

### Module 5: Localized Messages (`keywords.rs`) -- DONE
- 14 tests covering REQ-BRAIN-012 (Should), REQ-BRAIN-013 (Should), REQ-BRAIN-014 (Should)
- Tests: SETUP_TTL_SECS value, all setup i18n messages in 8 languages (help, intro,
  followup, proposal, complete, cancelled, expired), confirmation/cancellation keyword
  reuse, modification request handling, pending_setup as system fact
- All tests will FAIL until developer adds the constants and functions to `keywords.rs`

### Module 6: System Fact Keys (`omega-core/config/tests.rs`) -- DONE
- 1 test covering REQ-BRAIN-012 (Should)
- Test: `pending_setup` in `SYSTEM_FACT_KEYS`
- Will FAIL until developer adds "pending_setup" to the array

## Tests That FAIL (TDD Red Phase)

All 75 tests fail initially. The developer must implement:

| Component | What to Implement | Tests Unblocked |
|-----------|-------------------|-----------------|
| `Command::Setup` variant + `/setup` match arm | `commands/mod.rs` | 11 |
| `omega-brain.md` agent file | `topologies/development/agents/omega-brain.md` | 17 |
| `BRAIN_AGENT` const (include_str) | `builds_agents.rs` | 17 |
| `write_single()` method | `builds_agents.rs` | 7 |
| `mod setup;` in gateway/mod.rs | `gateway/mod.rs` | 25 |
| `setup.rs` production code | `gateway/setup.rs` (stubs already present) | 25 |
| Setup i18n functions + SETUP_TTL_SECS | `keywords.rs` | 14 |
| "pending_setup" in SYSTEM_FACT_KEYS | `omega-core/src/config/mod.rs` | 1 |
| `/setup` in help text | `commands/status.rs` | 1 |

## Files Created/Modified

| File | Action | Test Count |
|------|--------|-----------|
| `backend/src/commands/tests.rs` | MODIFIED | +11 tests |
| `backend/src/gateway/builds_agents.rs` | MODIFIED | +24 tests |
| `backend/src/gateway/setup.rs` | NEW | 25 tests (with production stubs) |
| `backend/src/gateway/keywords.rs` | MODIFIED | +14 tests |
| `backend/crates/omega-core/src/config/tests.rs` | MODIFIED | +1 test |

## Requirement Traceability Summary

| Requirement | Priority | Tests Written | Status |
|------------|----------|--------------|--------|
| REQ-BRAIN-001 | Must | 11 | TDD red -- needs Command::Setup |
| REQ-BRAIN-002 | Must | 14 | TDD red -- needs BRAIN_AGENT const + omega-brain.md |
| REQ-BRAIN-003 | Must | 7 | TDD red -- needs write_single() |
| REQ-BRAIN-004 | Must | 6 | TDD red -- needs parse_setup_output() |
| REQ-BRAIN-005 | Must | 2 | TDD red -- collision context format |
| REQ-BRAIN-006 | Must | 2 | TDD red -- ROLE.md path + directory creation |
| REQ-BRAIN-007 | Must | 1 | TDD red -- HEARTBEAT.md path |
| REQ-BRAIN-008 | Must | 3 | TDD red -- SCHEDULE_ACTION marker format |
| REQ-BRAIN-009 | Should | 3 | TDD red -- PROJECT_ACTIVATE marker format |
| REQ-BRAIN-010 | Should | 7 | TDD red -- parse_setup_round() |
| REQ-BRAIN-011 | Should | 4 | TDD red -- description extraction |
| REQ-BRAIN-012 | Should | 4 | TDD red -- pending_setup fact, TTL, system key |
| REQ-BRAIN-013 | Should | 2 | TDD red -- confirm/cancel keyword reuse |
| REQ-BRAIN-014 | Should | 8 | TDD red -- all i18n functions x 8 languages |
| REQ-BRAIN-015 | Should | 1 | TDD red -- ROLE.md examples in agent |
| REQ-BRAIN-016 | Could | 0 | Compile-time check (no test needed) |
| REQ-BRAIN-017 | Could | 0 | Agent content test (covered by brain agent tests) |
| REQ-BRAIN-018 | Could | 1 | TDD red -- audit prefix format |
| REQ-BRAIN-019 | Should | 1 | TDD red -- read existing ROLE.md |
| REQ-BRAIN-020 | Must | 5 | TDD red -- context file lifecycle |
| REQ-BRAIN-021 | Should | 1 | TDD red -- restricted tools |
| REQ-BRAIN-022 | Must | 1 | TDD red -- workspace path |
| REQ-BRAIN-023 | Could | 1 | TDD red -- concurrent guard logic |
| REQ-BRAIN-024 | Won't | 0 | N/A |
| REQ-BRAIN-025 | Won't | 0 | N/A |

## Specs Gaps Found

1. **setup.rs module registration**: The architecture doc places `setup.rs` as a new module
   under `gateway/`, but `gateway/mod.rs` does not yet have `mod setup;`. The developer
   must add this for the tests to compile. The test file is created with production stubs
   (pure functions like `setup_context_path`, `parse_setup_round`, `parse_setup_output`)
   so that tests can run once the module is registered.

2. **`pending_setup` not in SYSTEM_FACT_KEYS**: The `omega-core/config/mod.rs` file defines
   `SYSTEM_FACT_KEYS` but does not include `"pending_setup"`. The architecture requires
   this for fact injection protection. A test was added to `config/tests.rs` to enforce this.

3. **Gateway methods not unit-testable**: The methods `start_setup_session()`,
   `handle_setup_response()`, and `execute_setup()` are on the `Gateway` struct which
   requires a full runtime context (provider, channels, memory store, etc.). These are
   tested indirectly through the pure functions they call (parse_setup_output,
   parse_setup_round, setup_context_path) and through the acceptance criteria tested
   in each subcomponent. Full integration testing requires the complete Gateway mock
   which is out of scope for unit tests.

4. **Architecture doc shows setup_i18n.rs as separate module**: The requirements reference
   Module 6 as localized messages. The architecture places these functions in `keywords.rs`
   (which already has all other i18n functions), not in a separate `setup_i18n.rs`. Tests
   follow the architecture decision and add to `keywords.rs`.
