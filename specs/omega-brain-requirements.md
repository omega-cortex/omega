# Requirements: OMEGA Brain

## Scope

### Domains Affected
- **Gateway pipeline** (`backend/src/gateway/pipeline.rs`) -- new `/setup` intercept, `pending_setup` session state
- **Gateway module** (`backend/src/gateway/mod.rs`) -- register new `setup` submodule
- **Commands** (`backend/src/commands/mod.rs`) -- add `Setup` variant to `Command` enum
- **Keywords** (`backend/src/gateway/keywords.rs`) -- setup session messages (8 languages)
- **Agent lifecycle** (`backend/src/gateway/builds_agents.rs`) -- `BRAIN_AGENT` const, `write_single()` method
- **NEW module** (`backend/src/gateway/setup.rs`) -- Brain orchestrator logic
- **NEW agent** (`topologies/development/agents/omega-brain.md`) -- Brain agent definition

### Files NOT Modified (primitives consumed as-is)
- `backend/src/gateway/process_markers.rs` -- existing `PROJECT_ACTIVATE`, `SCHEDULE_ACTION`, `SCHEDULE` markers
- `backend/crates/omega-skills/src/projects.rs` -- existing `load_projects()`, `Project` struct
- `backend/src/gateway/heartbeat.rs` -- existing per-project heartbeat reader
- `backend/src/gateway/builds.rs` -- existing `run_build_phase()` method

## Summary (plain language)

The OMEGA Brain is a setup agent that lets non-technical users describe a business goal (e.g., "I'm a realtor") and have OMEGA configure itself as a domain expert. The user types `/setup` followed by a description. The Brain asks 2-4 clarifying questions, proposes a project setup (ROLE.md, HEARTBEAT.md, schedules), gets approval, then creates everything using OMEGA's existing primitives. No new crates, no new database tables, no new marker types.

## User Stories

- As a **non-technical user**, I want to type `/setup I'm a realtor in Lisbon` so that OMEGA creates a complete project setup (ROLE.md, HEARTBEAT.md, schedules) without me knowing OMEGA's file conventions.
- As a **power user**, I want `/setup` to detect existing projects so that I don't accidentally create duplicates.
- As a **user in any of 8 supported languages**, I want setup session messages to appear in my preferred language so that the experience feels native.
- As a **cautious user**, I want to review and approve the Brain's proposal before anything is created so that I stay in control.
- As a **busy user**, I want the setup to complete in 3 rounds or fewer so that I don't lose interest.

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-BRAIN-001 | `/setup` command registration in `Command::parse()` | Must | `/setup` returns `Some(Command::Setup)`, botname suffix stripped, unknown commands unaffected |
| REQ-BRAIN-002 | Brain agent definition bundled via `include_str!()` | Must | `BRAIN_AGENT` const in `builds_agents.rs`, loaded from `topologies/development/agents/omega-brain.md` |
| REQ-BRAIN-003 | Agent file lifecycle with `write_single()` on `AgentFilesGuard` | Must | Writes single agent `.md` to `.claude/agents/`, RAII cleanup on drop, ref-counted |
| REQ-BRAIN-004 | Brain invocation via `run_build_phase("omega-brain", prompt, model_complex, Some(30))` | Must | Uses existing `run_build_phase()`, Opus model, 30 max_turns, 3-attempt retry |
| REQ-BRAIN-005 | Collision detection via `load_projects()` | Must | Existing projects loaded before invocation, list passed to Brain in prompt context |
| REQ-BRAIN-006 | Brain creates ROLE.md at `~/.omega/projects/<name>/ROLE.md` | Must | Domain-specific instructions, directory created if absent, parseable by `load_projects()` |
| REQ-BRAIN-007 | Brain creates HEARTBEAT.md at `~/.omega/projects/<name>/HEARTBEAT.md` | Must | Domain-specific monitoring items, processed by heartbeat loop |
| REQ-BRAIN-008 | Brain emits `SCHEDULE_ACTION:` markers for recurring tasks | Must | Valid markers processed by `process_markers()`, at least one schedule per setup |
| REQ-BRAIN-009 | Brain emits `PROJECT_ACTIVATE:` marker | Should | Output ends with `PROJECT_ACTIVATE: <name>`, processed by `process_markers()` |
| REQ-BRAIN-010 | Multi-round session with approval gate (3 rounds max) | Should | Up to 3 rounds, state tracked via context file, final round produces proposal |
| REQ-BRAIN-011 | `/setup` intercepted early in `pipeline.rs` (before provider call) | Should | Handled in command dispatch section, same pattern as `/forget` |
| REQ-BRAIN-012 | `pending_setup` fact with 30-minute TTL | Should | State stored as fact, format `<timestamp>\|<sender_id>`, expires after 30 min |
| REQ-BRAIN-013 | Setup confirmation/cancellation keywords (8 languages) | Should | Reuses or parallels build confirm/cancel pattern |
| REQ-BRAIN-014 | Localized setup session messages (8 languages) | Should | Intro, proposal, complete, cancelled, expired messages in EN/ES/PT/FR/DE/IT/NL/RU |
| REQ-BRAIN-015 | Brain agent prompt includes examples of excellent ROLE.md files | Should | 2-3 examples covering different domains |
| REQ-BRAIN-016 | `setup.rs` module registered in `gateway/mod.rs` | Could | `mod setup;` added, contains Brain orchestration logic |
| REQ-BRAIN-017 | Brain suggests relevant skills (inform only, no auto-install) | Could | Informational text, not markers |
| REQ-BRAIN-018 | Audit logging for setup operations | Could | Start, proposal, approval, completion logged with `[SETUP:<project>]` prefix |
| REQ-BRAIN-019 | Brain reads existing ROLE.md files for context | Should | Existing ROLE.md content included in prompt context |
| REQ-BRAIN-020 | Setup session context file cleanup | Must | Context file at `<data_dir>/setup/<sender_id>.md`, deleted on completion/cancel/expiry |
| REQ-BRAIN-021 | Brain agent has restricted tools (Read, Write, Glob, Grep) | Should | No Bash, no Edit in agent frontmatter |
| REQ-BRAIN-022 | Brain agent operates in `~/.omega/` workspace | Must | Agent invoked with workspace `~/.omega/`, files resolve correctly |
| REQ-BRAIN-023 | Guard against concurrent setup sessions per user | Could | Only one active `pending_setup` per sender_id |
| REQ-BRAIN-024 | Trigger 2 (restructure) and Trigger 3 (learning threshold) | Won't | Deferred to future iteration |
| REQ-BRAIN-025 | Skill auto-installation | Won't | Skills have MCP servers with security implications |

## Detailed Acceptance Criteria

### REQ-BRAIN-001: `/setup` command registration
- [ ] Given a message `/setup I'm a realtor`, when `Command::parse()` is called, then it returns `Some(Command::Setup)`
- [ ] Given a message `/setup@omega_bot I'm a realtor`, when `Command::parse()` is called, then it returns `Some(Command::Setup)` (botname suffix stripped)
- [ ] Given a message `/setup` with no description, when `Command::parse()` is called, then it returns `Some(Command::Setup)` (handler deals with empty input)
- [ ] Given a message `/settings`, when `Command::parse()` is called, then it returns `None` (no false match)

### REQ-BRAIN-002: Brain agent definition
- [ ] Given the compiled binary, when `BRAIN_AGENT` const is accessed, then it contains non-empty content starting with `---` (YAML frontmatter)
- [ ] Given the agent frontmatter, when parsed, then it contains: `name: omega-brain`, `model: opus`, `permissionMode: bypassPermissions`, `maxTurns: 30`
- [ ] Given the agent body, when read, then it contains instructions for: questioning strategy, ROLE.md generation, HEARTBEAT.md generation, marker emission, approval gate
- [ ] Given the agent body, when read, then it contains non-interactive instruction (Brain must not ask the user directly)

### REQ-BRAIN-003: `write_single()` agent lifecycle
- [ ] Given a project directory, when `AgentFilesGuard::write_single(dir, "omega-brain", content)` is called, then `<dir>/.claude/agents/omega-brain.md` is created
- [ ] Given an active guard, when it is dropped, then the single agent file and `.claude/agents/` directory are removed
- [ ] Given two concurrent guards for the same directory, when one is dropped, then files persist until the last guard drops (ref-counting)
- [ ] Given a non-existent directory hierarchy, when `write_single()` is called, then it creates all intermediate directories

### REQ-BRAIN-004: Brain invocation
- [ ] Given a setup request, when the Brain is invoked, then `run_build_phase("omega-brain", prompt, model_complex, Some(30))` is called
- [ ] Given a provider failure, when `run_build_phase` retries 3 times and all fail, then the setup fails gracefully with an error message to the user
- [ ] Given a successful Brain invocation, when the output is received, then it is parsed for ROLE.md content, HEARTBEAT.md content, and markers

### REQ-BRAIN-005: Collision detection
- [ ] Given 3 existing projects (realtor, trader, restaurant), when user types `/setup I'm a realtor`, then the Brain receives context listing existing projects including "realtor"
- [ ] Given the collision context, when Brain detects a matching project name, then it proposes updating the existing project instead of creating a duplicate
- [ ] Given no existing projects, when user types `/setup I'm a chef`, then Brain proceeds with normal creation flow

### REQ-BRAIN-006: ROLE.md creation
- [ ] Given a realtor domain, when Brain creates ROLE.md, then the file exists at `~/.omega/projects/realtor/ROLE.md`
- [ ] Given the ROLE.md content, when read, then it contains: domain context, operational rules, relevant knowledge areas, and constraints
- [ ] Given the ROLE.md content, when loaded by `load_projects()`, then it is parsed successfully as a valid project

### REQ-BRAIN-007: HEARTBEAT.md creation
- [ ] Given a realtor domain, when Brain creates HEARTBEAT.md, then the file exists at `~/.omega/projects/realtor/HEARTBEAT.md`
- [ ] Given the HEARTBEAT.md content, when read, then it contains domain-specific monitoring items
- [ ] Given the HEARTBEAT.md file, when the heartbeat loop runs, then it processes the project-specific heartbeat items

### REQ-BRAIN-008: Schedule markers
- [ ] Given a realtor setup, when Brain completes, then output contains at least one `SCHEDULE_ACTION:` marker
- [ ] Given the marker content, when processed by `process_markers()`, then scheduled tasks are created in `scheduled_tasks` table
- [ ] Given scheduled tasks, when their due time arrives, then they execute via the existing action scheduler

### REQ-BRAIN-009: Project activation marker
- [ ] Given a successful setup, when Brain output is processed, then `PROJECT_ACTIVATE: <name>` marker is present
- [ ] Given the marker, when `process_markers()` runs, then `active_project` fact is set for the user
- [ ] Given the activated project, when user sends subsequent messages, then OMEGA operates with the new project context

### REQ-BRAIN-010: Multi-round session
- [ ] Given a vague setup request (`/setup help me with my business`), when Brain runs round 1, then it returns 2-4 clarifying questions
- [ ] Given the user's answers, when Brain runs round 2, then it either asks follow-up questions or produces a proposal
- [ ] Given round 3, when Brain runs, then it MUST produce a final proposal (no more questions allowed)
- [ ] Given a specific request (`/setup I'm a realtor in Lisbon, residential properties`), when Brain runs round 1, then it may skip questions and produce a proposal directly
- [ ] Given any round, when Brain output is received, then the round counter is incremented in the session context file

### REQ-BRAIN-011: Pipeline intercept
- [ ] Given a `/setup` command, when the pipeline processes it, then it is handled in the command dispatch section (step 3)
- [ ] Given the intercept, when `/setup <description>` is parsed, then the text after `/setup` is extracted as the business description
- [ ] Given the intercept, when handling begins, then no provider call is made for the original message
- [ ] Given a `/setup` with no text, when intercepted, then user receives a help message explaining usage

### REQ-BRAIN-012: Session state
- [ ] Given a new setup session, when state is stored, then `pending_setup` fact contains `<timestamp>|<sender_id>`
- [ ] Given a session older than 30 minutes, when the user sends a message, then the session is expired and user is notified
- [ ] Given a valid session, when user responds, then the response is routed to the setup handler (not normal pipeline)
- [ ] Given session completion or cancellation, when cleanup runs, then `pending_setup` fact is deleted

### REQ-BRAIN-013: Confirmation keywords
- [ ] Given a setup proposal, when user replies with "yes" (or equivalent in any of 8 languages), then creation proceeds
- [ ] Given a setup proposal, when user replies with "no" (or equivalent), then setup is cancelled
- [ ] Given a non-confirmation reply during approval phase, when received, then it is treated as a modification request (passed back to Brain)

### REQ-BRAIN-014: Localized messages
- [ ] Given a Spanish-speaking user, when setup starts, then intro message is in Spanish
- [ ] Given each of the 8 languages, when any setup message is generated, then it is properly translated
- [ ] Given the proposal message, when displayed, then it includes a preview of what will be created and instructions to confirm

### REQ-BRAIN-015: ROLE.md quality via examples
- [ ] Given the Brain agent definition, when read, then it contains at least 2 example ROLE.md files
- [ ] Given the examples, when reviewed, then they demonstrate: clear domain context, specific operational rules, relevant knowledge areas, safety constraints
- [ ] Given the examples, when Brain generates a new ROLE.md, then the output follows the demonstrated structure

### REQ-BRAIN-020: Context file cleanup
- [ ] Given a completed setup, when cleanup runs, then `<data_dir>/setup/<sender_id>.md` is deleted
- [ ] Given a cancelled setup, when cleanup runs, then the context file is deleted
- [ ] Given an expired setup, when the next message arrives, then the context file is deleted

### REQ-BRAIN-022: Workspace
- [ ] Given Brain invocation, when `run_build_phase` is called, then the working directory is `~/.omega/`
- [ ] Given file writes by the Brain, when paths are used, then they resolve correctly relative to `~/.omega/`

## Impact Analysis

### Existing Code Affected

| File | Change | Risk |
|------|--------|------|
| `backend/src/commands/mod.rs` | Add `Setup` variant to `Command` enum + match arm in `parse()` | Low -- additive, no existing arms affected |
| `backend/src/gateway/pipeline.rs` | Add `pending_setup` check block (like `pending_discovery`) + `/setup` intercept | Medium -- pipeline.rs is the critical path |
| `backend/src/gateway/builds_agents.rs` | Add `BRAIN_AGENT` const + `write_single()` method | Low -- additive, existing methods unchanged |
| `backend/src/gateway/keywords.rs` | Add setup localized messages (5-6 functions) | Low -- additive |
| `backend/src/gateway/mod.rs` | Add `mod setup;` declaration | Low -- one line |

### Regression Risk Areas

| Area | Why It Might Break | Mitigation |
|------|-------------------|------------|
| Build discovery session | Both use `pending_*` facts and same TTL pattern; simultaneous triggers could cause state confusion | Mutually exclusive checks (setup checked first, discovery checked second) |
| Marker processing | Brain emits markers; wrong format = silent failure | Include exact marker format in Brain agent prompt + integration test |
| Heartbeat loader | Brain creates HEARTBEAT.md; wrong format = silently skipped | Include exact format in Brain prompt |
| `load_projects()` | Brain creates ROLE.md; bad structure = empty instructions | Include exact ROLE.md format requirements in Brain prompt |

## Traceability Matrix

| Requirement ID | Priority | Test IDs | Architecture Section | Implementation Module |
|---------------|----------|----------|---------------------|---------------------|
| REQ-BRAIN-001 | Must | test_parse_setup_command, test_parse_setup_command_with_description, test_parse_setup_command_with_long_description, test_parse_setup_command_with_botname_suffix, test_parse_setup_command_with_botname_no_text, test_parse_settings_does_not_match_setup, test_parse_setup_case_sensitive, test_parse_setup_command_unicode_description, test_parse_setup_command_emoji_description, test_parse_setup_registered_in_command_enum, test_help_includes_setup | Module 1: Command Registration | `commands/mod.rs` |
| REQ-BRAIN-002 | Must | test_brain_agent_is_non_empty, test_brain_agent_starts_with_yaml_frontmatter, test_brain_agent_has_closing_frontmatter, test_brain_agent_frontmatter_name, test_brain_agent_frontmatter_model, test_brain_agent_frontmatter_permission_mode, test_brain_agent_frontmatter_max_turns, test_brain_agent_non_interactive, test_brain_agent_contains_setup_questions_format, test_brain_agent_contains_setup_proposal_format, test_brain_agent_contains_setup_execute_format, test_brain_agent_mentions_role_md, test_brain_agent_mentions_heartbeat_md | Module 3: Agent Lifecycle Extension (BRAIN_AGENT const) | `gateway/builds_agents.rs` |
| REQ-BRAIN-003 | Must | test_write_single_creates_agent_file, test_write_single_cleanup_on_drop, test_write_single_ref_counting, test_write_single_creates_directory_hierarchy, test_write_single_creates_only_one_file, test_write_single_with_empty_content, test_write_single_drop_idempotent | Module 3: Agent Lifecycle Extension (write_single) | `gateway/builds_agents.rs` |
| REQ-BRAIN-004 | Must | test_parse_setup_output_questions, test_parse_setup_output_proposal, test_parse_setup_output_executed, test_parse_setup_output_empty, test_parse_setup_output_questions_takes_priority, test_parse_setup_output_questions_empty, test_parse_setup_output_proposal_minimal | Module 4: Brain Orchestrator (start_setup_session, execute_setup) | `gateway/setup.rs` |
| REQ-BRAIN-005 | Must | test_collision_detection_context_format, test_collision_detection_no_projects | Module 4: Brain Orchestrator (load_projects context) | `gateway/setup.rs` |
| REQ-BRAIN-006 | Must | test_role_md_path_format, test_project_directory_creation | Module 2: Brain Agent Definition (ROLE.md creation) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-007 | Must | test_heartbeat_md_path_format | Module 2: Brain Agent Definition (HEARTBEAT.md creation) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-008 | Must | test_brain_agent_mentions_schedule_action_marker, test_schedule_action_marker_format, test_schedule_action_marker_with_spaces | Module 2: Brain Agent Definition (SCHEDULE_ACTION markers) | `topologies/.../omega-brain.md` + `process_markers.rs` |
| REQ-BRAIN-009 | Should | test_brain_agent_mentions_project_activate_marker, test_project_activate_marker_format, test_project_activate_marker_hyphenated_name | Module 2: Brain Agent Definition (PROJECT_ACTIVATE marker) | `topologies/.../omega-brain.md` + `process_markers.rs` |
| REQ-BRAIN-010 | Should | test_parse_setup_round_extracts_round, test_parse_setup_round_round_2, test_parse_setup_round_round_3, test_parse_setup_round_no_header, test_parse_setup_round_empty_content, test_parse_setup_round_malformed_value, test_parse_setup_round_not_first_line | Session State Machine (3-round limit) | `gateway/setup.rs` |
| REQ-BRAIN-011 | Should | test_extract_description_from_setup_command, test_extract_description_empty, test_extract_description_with_botname, test_extract_description_whitespace_only | Module 5: Pipeline Integration (Point 1 -- /setup intercept) | `gateway/pipeline.rs` |
| REQ-BRAIN-012 | Should | test_setup_ttl_secs_value, test_pending_setup_fact_format, test_pending_setup_ttl_value, test_pending_setup_within_ttl, test_pending_setup_is_system_fact, test_system_fact_keys_contains_pending_setup | Module 4 + Module 5: Pipeline Integration (Point 2 -- pending_setup check) | `gateway/setup.rs` + `gateway/pipeline.rs` + `omega-core/config/tests.rs` |
| REQ-BRAIN-013 | Should | test_setup_confirmation_reuses_build_keywords, test_setup_modification_is_neither_confirm_nor_cancel | Design Decisions (reuse BUILD_CONFIRM_KW / BUILD_CANCEL_KW) | `gateway/keywords.rs` |
| REQ-BRAIN-014 | Should | test_setup_help_message_all_languages, test_setup_help_message_default_english, test_setup_intro_message_all_languages, test_setup_followup_message_all_languages, test_setup_proposal_message_all_languages, test_setup_complete_message_all_languages, test_setup_cancelled_message_all_languages, test_setup_expired_message_all_languages | Module 6: Localized Messages | `gateway/keywords.rs` |
| REQ-BRAIN-015 | Should | test_brain_agent_contains_role_md_examples | Module 2: Brain Agent Definition (examples section) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-016 | Could | N/A (compile-time check) | Module 7: Gateway Module Registration | `gateway/mod.rs` |
| REQ-BRAIN-017 | Could | N/A (covered by Brain agent content tests) | Module 2: Brain Agent Definition (skill suggestions) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-018 | Could | test_audit_setup_prefix_format | Module 4: Brain Orchestrator (audit_setup) | `gateway/setup.rs` |
| REQ-BRAIN-019 | Should | test_read_existing_role_for_context | Module 4: Brain Orchestrator (read existing ROLE.md) | `gateway/setup.rs` |
| REQ-BRAIN-020 | Must | test_setup_context_path_format, test_setup_context_path_tilde_expansion, test_setup_context_path_special_sender_id, test_setup_context_file_creation_and_cleanup, test_setup_context_file_deleted_on_expiry, test_setup_cleanup_idempotent | Module 4: Brain Orchestrator (cleanup_setup_session) | `gateway/setup.rs` |
| REQ-BRAIN-021 | Should | test_brain_agent_restricted_tools | Module 2: Brain Agent Definition (tools restriction -- no Bash, no Edit) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-022 | Must | test_workspace_path_is_omega_root | Module 4: Brain Orchestrator (workspace path = ~/.omega/) | `gateway/setup.rs` |
| REQ-BRAIN-023 | Could | test_concurrent_session_guard_logic | Module 4: Brain Orchestrator (concurrent session guard) | `gateway/setup.rs` |
| REQ-BRAIN-024 | Won't | N/A | Out of scope | N/A |
| REQ-BRAIN-025 | Won't | N/A | Out of scope | N/A |

## Assumptions

| # | Assumption | Confirmed |
|---|-----------|-----------|
| 1 | `/setup` is the trigger, not keyword detection | Yes (user decision) |
| 2 | Multi-round session follows discovery pattern | Yes (user decision) |
| 3 | Existing projects detected via `load_projects()` | Yes (user decision) |
| 4 | Brain uses `model_complex` (Opus) | Yes (inferred from idea brief) |
| 5 | Brain agent lives at `topologies/development/agents/omega-brain.md` | Yes (pattern match) |
| 6 | Brain bundled via `include_str!()` like build agents | Yes (pattern match) |
| 7 | No new database tables needed | Yes (idea brief) |
| 8 | No new marker types needed | Yes (idea brief) |
| 9 | MVP scope is Trigger 1 only | Yes (idea brief) |
| 10 | 8-language support mandatory for setup messages | Yes (project rule) |

## Priority Summary

| Priority | Count | IDs |
|----------|-------|-----|
| Must | 9 | REQ-BRAIN-001, 002, 003, 004, 005, 006, 007, 008, 020, 022 |
| Should | 8 | REQ-BRAIN-009, 010, 011, 012, 013, 014, 015, 019, 021 |
| Could | 4 | REQ-BRAIN-016, 017, 018, 023 |
| Won't | 2 | REQ-BRAIN-024, 025 |

## Implementation Order (recommended)

1. **Phase 1 -- Plumbing** (Must): REQ-BRAIN-001 (command), REQ-BRAIN-002 (agent const), REQ-BRAIN-003 (write_single), REQ-BRAIN-016 (module)
2. **Phase 2 -- Core Flow** (Must): REQ-BRAIN-004 (invocation), REQ-BRAIN-011 (intercept), REQ-BRAIN-022 (workspace), REQ-BRAIN-005 (collision)
3. **Phase 3 -- Session** (Should): REQ-BRAIN-010 (multi-round), REQ-BRAIN-012 (state), REQ-BRAIN-013 (confirmation), REQ-BRAIN-020 (cleanup)
4. **Phase 4 -- Output** (Must): REQ-BRAIN-006 (ROLE.md), REQ-BRAIN-007 (HEARTBEAT.md), REQ-BRAIN-008 (schedules), REQ-BRAIN-009 (activate)
5. **Phase 5 -- Polish** (Should/Could): REQ-BRAIN-014 (i18n), REQ-BRAIN-015 (examples), REQ-BRAIN-017 (skills), REQ-BRAIN-018 (audit), REQ-BRAIN-019 (context), REQ-BRAIN-021 (tools), REQ-BRAIN-023 (concurrency)

## Out of Scope (Won't)

- **Trigger 2: Explicit Restructure** -- deferred (REQ-BRAIN-024)
- **Trigger 3: Learning Threshold** -- deferred (REQ-BRAIN-024)
- **Skill auto-installation** -- security implications (REQ-BRAIN-025)
- **Build pipeline integration** -- Brain does NOT create topologies
- **OMEGA personality** -- handled by Soul in SYSTEM_PROMPT.md
