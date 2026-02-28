# Requirements: Topology Extraction (Phase 1)

> Extract the hardcoded 7-phase build pipeline into a config-driven topology format.
> This is a pure refactoring — zero behavioral change. The existing pipeline becomes
> the default "development" topology loaded from TOML + external agent .md files.

## Scope

**Domains affected:** gateway (builds, builds_agents, builds_loop, builds_parse), file system layout (~/.omega/topologies/)

**Files directly modified:**
- `backend/src/gateway/builds.rs` — Replace hardcoded phase sequence with topology-driven orchestrator loop
- `backend/src/gateway/builds_agents.rs` — Remove embedded const strings; replace with topology loader that reads .md files from disk; keep AgentFilesGuard with new source
- `backend/src/gateway/builds_loop.rs` — Parameterize retry counts from topology config instead of hardcoded 3/2
- `backend/src/gateway/builds_parse.rs` — Update phase_message() to accept phase name (string) instead of phase number (u8); keep all parsers intact
- `backend/src/gateway/mod.rs` — Register new builds_topology submodule

**New files:**
- `backend/src/gateway/builds_topology.rs` — NEW: Topology data structures, TOML deserialization, loader, bundled default deployment
- `topologies/development/TOPOLOGY.toml` — Bundled topology config (source, compiled into binary)
- `topologies/development/agents/build-analyst.md` — Agent definition (source, compiled into binary)
- `topologies/development/agents/build-architect.md` — Agent definition
- `topologies/development/agents/build-test-writer.md` — Agent definition
- `topologies/development/agents/build-developer.md` — Agent definition
- `topologies/development/agents/build-qa.md` — Agent definition
- `topologies/development/agents/build-reviewer.md` — Agent definition
- `topologies/development/agents/build-delivery.md` — Agent definition
- `topologies/development/agents/build-discovery.md` — Agent definition

**Runtime deployment path:** `~/.omega/topologies/development/`

**Files NOT affected:**
- `backend/src/gateway/pipeline.rs` — No changes; still calls handle_build_request() with same signature
- `backend/src/gateway/routing.rs`, `keywords.rs`, `process_markers.rs` — No changes
- All omega-core, omega-providers, omega-channels, omega-memory, omega-skills, omega-sandbox crates — No changes

## Confirmed Decisions

| # | Decision | Confirmed |
|---|----------|-----------|
| 1 | Topology directory: `~/.omega/topologies/<name>/` | Yes |
| 2 | Lazy deployment: deploy bundled default on first build request, not process startup | Yes |
| 3 | Never overwrite: existing user-customized files are preserved | Yes |
| 4 | toml crate (0.8, workspace dep) used for TOPOLOGY.toml parsing | Yes |
| 5 | Agent .md files use same YAML frontmatter format as today | Yes |
| 6 | Only "development" topology in Phase 1, no selection UI | Yes |

## Summary (plain language)

Today, OMEGA's build pipeline is a 7-step assembly line where each step's instructions, retry limits, and validation rules are written directly in Rust code. The agent instructions (~400 lines of markdown each, 8 agents total) are compiled into the binary as string constants. Changing anything — adding a step, tweaking a retry count, editing an agent's prompt — requires modifying Rust source, recompiling, and redeploying.

This change moves the pipeline definition into a TOML configuration file and external agent .md files on disk. The Rust code becomes a generic "topology engine" that reads the config and executes whatever pipeline it describes. Agent content moves from embedded const strings to standalone .md files that can be edited independently.

The first (and only) topology shipped is "development" — it produces identical behavior to today's hardcoded pipeline. The topology files are bundled in the binary (via include_str!) and auto-deployed to ~/.omega/topologies/development/ on first use.

## User Stories

- As OMEGA (the running process), I want to load my build pipeline definition from ~/.omega/topologies/development/TOPOLOGY.toml so that the pipeline structure can be modified without recompiling the binary.
- As OMEGA, I want to load agent instructions from ~/.omega/topologies/development/agents/*.md files so that agent prompts can be edited, tested, and versioned independently of the binary.
- As an Omega developer, I want the default "development" topology to be bundled in the binary and auto-deployed to ~/.omega/topologies/development/ on first run, so that OMEGA works out of the box without manual setup.
- As an Omega developer, I want the topology engine to produce identical build behavior to the current hardcoded pipeline, so that this refactoring introduces zero regressions.

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-TOP-001 | Define TOPOLOGY.toml schema with serde structs | Must | Topology, Phase, ModelTier, PhaseType, RetryConfig, ValidationRule structs defined; all derive serde::Deserialize; invalid TOML returns Err, not panic |
| REQ-TOP-002 | Bundle default "development" topology in binary | Must | 8 agent .md files + TOPOLOGY.toml compiled via include_str!(); auto-deployed to ~/.omega/topologies/development/ if directory missing; does NOT overwrite existing files |
| REQ-TOP-003 | Load topology from disk at build-request time | Must | Reads and parses TOPOLOGY.toml from topology directory; reads agent .md files referenced by topology; falls back to bundled default on missing directory; reports clear error on corrupt TOML |
| REQ-TOP-004 | Dynamic phase execution loop in orchestrator | Must | builds.rs iterates over topology.phases instead of hardcoded if-let chains; dispatches to correct behavior based on phase_type; carries orchestrator state (brief, project_dir) across phases |
| REQ-TOP-005 | Agent files loaded from topology directory | Must | AgentFilesGuard reads .md content from topology loader (not const strings); writes to ~/.omega/workspace/.claude/agents/ same as today; RAII cleanup on drop unchanged |
| REQ-TOP-006 | Parameterized retry counts from topology | Must | QA max retries from topology (default 3); reviewer max retries from topology (default 2); fix agent name from topology (default "build-developer") |
| REQ-TOP-007 | Parameterized pre/post-phase validation | Must | Validation rules read from topology config per phase; current rules preserved identically; two validation types: file_exists and file_patterns |
| REQ-TOP-008 | Phase message localization by name | Should | phase_message() accepts phase name string instead of phase number u8; maps phase names to i18n strings for all 8 languages; generic fallback for unknown phase names |
| REQ-TOP-009 | Behavioral parity with hardcoded pipeline | Must | Same phase sequence, same models, same max_turns, same validations; same user-facing messages; same audit log entries; same chain state on failure |
| REQ-TOP-010 | Existing parse functions unchanged | Must | parse_project_brief(), parse_verification_result(), parse_review_result(), parse_build_summary() identical; all existing tests pass |
| REQ-TOP-011 | 500-line file limit compliance | Must | No .rs file exceeds 500 lines (excluding tests); builds_agents.rs shrinks significantly |
| REQ-TOP-012 | Discovery agent included in topology | Should | build-discovery.md loaded from topology agents/ directory; discovery flow in pipeline.rs continues working |
| REQ-TOP-013 | Topology name validation | Should | Alphanumeric + hyphens + underscores, max 64 chars; rejects path traversal, shell metacharacters |
| REQ-TOP-014 | Graceful error on missing agent files | Should | Clear error message naming the missing file; does not silently skip phases or use empty content |
| REQ-TOP-015 | Chain state includes topology name | Could | ChainState struct gains topology_name field; chain-state.md output shows which topology was used |
| REQ-TOP-016 | User-facing topology selection | Won't | Deferred to Phase 2 |
| REQ-TOP-017 | DAG/parallel execution | Won't | Deferred to future phase |
| REQ-TOP-018 | Architecture assistant agent | Won't | Deferred to future phase |
| REQ-TOP-019 | Custom topology creation UI | Won't | Deferred to future phase |

## Phase Type Dispatch

The current pipeline has phases with **bespoke orchestrator logic**. These are captured as `phase_type` values:

| PhaseType | Behavior | Used By |
|-----------|----------|---------|
| `Standard` | Run agent, check for error, proceed | architect, test-writer |
| `ParseBrief` | Run agent, parse output via parse_project_brief(), create project dir, extract name/scope | analyst |
| `CorrectiveLoop` | Run agent, parse result, on fail re-invoke fix_agent, retry up to max iterations | qa, reviewer |
| `ParseSummary` | Run agent, parse output via parse_build_summary(), format final message | delivery |

## Impact Analysis

### Existing Code Affected
- `builds.rs` (500 lines): Major rewrite of handle_build_request() from 7 explicit phases to topology-driven loop — Risk: high
- `builds_agents.rs` (1,272 lines): Major rewrite. Remove ~400 lines of const string agents. AgentFilesGuard::write() changes source from const strings to topology-loaded content — Risk: high
- `builds_loop.rs` (345 lines): Moderate change. run_qa_loop() and run_review_loop() accept retry count and fix_agent as parameters — Risk: medium
- `builds_parse.rs` (1,388 lines): Minor change. phase_message() signature changes. All parsers unchanged — Risk: low

### Regression Risk Areas
- Build pipeline behavior: topology engine must produce identical results
- Discovery flow: pipeline.rs uses AgentFilesGuard and discovery parse functions
- i18n: 8 languages x 7 phases = 56 phase messages must remain correct
- RAII cleanup: agent file lifecycle must remain correct under concurrent builds

## Proposed TOPOLOGY.toml Format

```toml
[topology]
name = "development"
description = "Default 7-phase TDD build pipeline"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"
model_tier = "complex"
max_turns = 25
phase_type = "parse-brief"

[[phases]]
name = "architect"
agent = "build-architect"
model_tier = "complex"
post_validation = ["specs/architecture.md"]

[[phases]]
name = "test-writer"
agent = "build-test-writer"
model_tier = "complex"

[phases.pre_validation]
type = "file_exists"
paths = ["specs/architecture.md"]

[[phases]]
name = "developer"
agent = "build-developer"
model_tier = "complex"

[phases.pre_validation]
type = "file_patterns"
patterns = ["test", "spec", "_test."]

[[phases]]
name = "qa"
agent = "build-qa"
model_tier = "complex"
phase_type = "corrective-loop"

[phases.pre_validation]
type = "file_patterns"
patterns = [".rs", ".py", ".js", ".ts", ".go", ".java", ".rb", ".c", ".cpp"]

[phases.retry]
max = 3
fix_agent = "build-developer"

[[phases]]
name = "reviewer"
agent = "build-reviewer"
model_tier = "complex"
phase_type = "corrective-loop"

[phases.retry]
max = 2
fix_agent = "build-developer"

[[phases]]
name = "delivery"
agent = "build-delivery"
model_tier = "complex"
phase_type = "parse-summary"
```

## Traceability Matrix

| Requirement ID | Priority | Test IDs | Architecture Section | Implementation Module |
|---------------|----------|----------|---------------------|---------------------|
| REQ-TOP-001 | Must | test_topology_deserialize_minimal_valid_toml, test_topology_deserialize_defaults_applied, test_topology_deserialize_all_phase_types, test_topology_deserialize_model_tiers, test_topology_deserialize_retry_config, test_topology_deserialize_validation_file_exists, test_topology_deserialize_validation_file_patterns, test_topology_deserialize_post_validation, test_topology_deserialize_max_turns, test_topology_deserialize_invalid_toml_returns_err, test_topology_deserialize_missing_required_field, test_topology_deserialize_wrong_type_returns_err, test_topology_deserialize_unknown_phase_type_returns_err, test_topology_deserialize_empty_phases, test_topology_deserialize_ignores_unknown_fields, test_topology_deserialize_full_development_topology, test_topology_deserialize_empty_string, test_topology_deserialize_special_chars_in_name, test_topology_deserialize_many_phases, test_topology_deserialize_retry_without_corrective_loop, test_loaded_topology_resolve_model | Module 1: builds_topology.rs, Structs section | builds_topology.rs |
| REQ-TOP-002 | Must | test_deploy_bundled_topology_creates_directory_structure, test_deploy_bundled_topology_preserves_existing_files, test_deploy_bundled_topology_preserves_existing_agent_files, test_deploy_bundled_topology_idempotent, test_bundled_topology_toml_is_non_empty, test_bundled_topology_toml_is_valid, test_bundled_agents_count, test_bundled_agents_all_non_empty | Module 1: Bundled Defaults + Loader (deploy_bundled_topology) | builds_topology.rs, topologies/development/ |
| REQ-TOP-003 | Must | test_load_topology_reads_valid_topology, test_load_topology_loads_all_referenced_agents, test_load_topology_loads_fix_agent, test_load_topology_corrupt_toml_returns_error, test_load_topology_missing_toml_returns_error, test_load_topology_development_fallback_deploys_bundled, test_load_topology_unknown_name_returns_not_found, test_loaded_topology_agent_content_found, test_loaded_topology_all_agents | Module 1: Loader (load_topology) | builds_topology.rs |
| REQ-TOP-004 | Must | (integration test — deferred to developer, tested via behavioral parity) | Module 2: builds.rs, OrchestratorState + Dispatch loop | builds.rs |
| REQ-TOP-005 | Must | (integration test — deferred to developer, tested via existing AgentFilesGuard tests) | Module 3: builds_agents.rs, write_from_topology() | builds_agents.rs |
| REQ-TOP-006 | Must | (integration test — deferred to developer, tested via existing run_qa_loop/run_review_loop parity) | Module 4: builds_loop.rs, run_corrective_loop() | builds_loop.rs |
| REQ-TOP-007 | Must | test_run_validation_file_exists_passes, test_run_validation_file_exists_fails, test_run_validation_file_exists_multiple_paths, test_run_validation_file_patterns_passes, test_run_validation_file_patterns_fails, test_run_validation_file_patterns_recursive, test_run_validation_parity_test_writer, test_run_validation_parity_developer, test_run_validation_file_exists_empty_paths, test_run_validation_file_patterns_empty_patterns | Module 4: builds_loop.rs, run_validation() | builds_loop.rs |
| REQ-TOP-008 | Should | test_phase_message_by_name_analyst_english, test_phase_message_by_name_all_phases_english, test_phase_message_by_name_all_languages_all_phases, test_phase_message_by_name_spanish, test_phase_message_by_name_russian, test_phase_message_by_name_unknown_phase_fallback, test_phase_message_by_name_unknown_language, test_phase_message_by_name_empty_phase_name, test_phase_message_by_name_parity_with_phase_message | Module 5: builds_parse.rs, phase_message_by_name() | builds_parse.rs |
| REQ-TOP-009 | Must | (behavioral parity verified through existing test suite — all 503 tests must continue passing) | Design Decisions, Dispatch loop, Data Flow | builds.rs |
| REQ-TOP-010 | Must | (existing parse function tests remain passing — 503 total) | Module 5: "Keep everything else unchanged" | builds_parse.rs, builds_loop.rs |
| REQ-TOP-011 | Must | (verified manually via line count check) | Module 3: Line count impact (~60 prod lines after rewrite) | all gateway/builds_*.rs files |
| REQ-TOP-012 | Should | test_bundled_agents_includes_discovery | Design Decisions: "Discovery agent included in topology agents" | builds_topology.rs (BUNDLED_AGENTS) |
| REQ-TOP-013 | Should | test_validate_topology_name_accepts_valid_names, test_validate_topology_name_rejects_empty, test_validate_topology_name_rejects_too_long, test_validate_topology_name_accepts_64_chars, test_validate_topology_name_rejects_path_traversal_dots, test_validate_topology_name_rejects_forward_slash, test_validate_topology_name_rejects_backslash, test_validate_topology_name_rejects_shell_metacharacters, test_validate_topology_name_rejects_unicode, test_validate_topology_name_rejects_dots | Module 1: validate_topology_name() | builds_topology.rs |
| REQ-TOP-014 | Should | test_load_topology_missing_agent_file_names_file, test_load_topology_missing_fix_agent_file_errors, test_loaded_topology_agent_content_missing | Module 1: Loader error handling (agent file not found) | builds_topology.rs |
| REQ-TOP-015 | Could | test_chain_state_includes_topology_name, test_chain_state_topology_name_optional | Module 4: save_chain_state() extension | builds_loop.rs |

## Specs Drift Detected
- `specs/src-gateway-rs.md` line 28: Reports builds_agents.rs as "~444 prod lines" — actual is ~480 (minor)
- `docs/architecture.md` line 508: States "Agent definitions: Embedded constants (builds_agents.rs)" — must be updated after this change

## Identified Risks

1. **Behavioral parity** (high): Topology engine must produce byte-identical user messages and audit logs. Mitigation: behavioral parity tests.
2. **File I/O at build time** (medium): Loading from disk adds latency. Mitigation: cache loaded topology in memory; fall back to bundled default.
3. **500-line limit** (medium): builds_topology.rs must stay under 500 lines. Mitigation: use include_str!() for bundled content.
4. **Agent content drift** (low): User edits agent .md incorrectly. Mitigation: validate frontmatter at load time.
5. **Test breakage** (high): 30+ tests in builds_agents.rs assert on const array. Mitigation: rewrite tests.
