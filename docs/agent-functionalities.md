# Agent Functionality Inventory

Complete inventory of every functionality, input, output, and fail-safe control for all 14 agents in the workflow toolkit.

---

## Quick Reference

| Agent | Model | Tools | Read-Only | Primary Output |
|-------|-------|-------|-----------|----------------|
| Discovery | Opus | Read, Grep, Glob, WebFetch, WebSearch | No | `docs/.workflow/idea-brief.md` |
| Analyst | Opus | Read, Grep, Glob, WebFetch, WebSearch | No | `specs/[domain]-requirements.md` |
| Architect | Opus | Read, Write, Edit, Grep, Glob | No | `specs/[domain]-architecture.md` |
| Test Writer | Opus | Read, Write, Edit, Bash, Glob, Grep | No | Test files + traceability update |
| Developer | Opus | Read, Write, Edit, Bash, Glob, Grep | No | Source code + specs/docs updates + commits |
| QA | Opus | Read, Write, Edit, Bash, Glob, Grep | No | `docs/qa/[domain]-qa-report.md` |
| Reviewer | Opus | Read, Grep, Glob | Yes | Review/audit report in `docs/reviews/` or `docs/audits/` |
| Functionality Analyst | Opus | Read, Grep, Glob | Yes | `docs/functionalities/FUNCTIONALITIES.md` |
| Codebase Expert | Opus | Read, Grep, Glob | Yes | `docs/understanding/PROJECT-UNDERSTANDING.md` |
| Proto-Auditor | Opus | Read, Grep, Glob | Yes | `c2c-protocol/audits/audit-[protocol]-[date].md` |
| Proto-Architect | Opus | Read, Write, Edit, Grep, Glob | No | `c2c-protocol/patches/patches-[protocol]-[date].md` |
| Role Creator | Opus | Read, Write, Grep, Glob, WebSearch, WebFetch | No | `.claude/agents/[name].md` |
| Role Auditor | Opus | Read, Grep, Glob | Yes | `docs/.workflow/role-audit-[name].md` |
| Feature Evaluator | Opus | Read, Write, Grep, Glob, WebSearch, WebFetch | No | `docs/.workflow/feature-evaluation.md` |

---

## 1. Discovery Agent

**File:** `.claude/agents/discovery.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob, WebFetch, WebSearch

### Prerequisite Gates

None. This is the first agent in the pipeline.

### Inputs

- Raw user idea or feature description (conversational).
- For existing projects: optionally reads project structure via Glob, `specs/SPECS.md`, `docs/DOCS.md`, and 1-2 relevant source files.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Verifies `docs/.workflow/` exists before writing. Creates if missing. |
| 2 | Partial-save on abandon | If user stops responding, saves progress to `docs/.workflow/discovery-partial.md` with "Discovery Status: INCOMPLETE" header. |
| 3 | Context-aware mode selection | Detects new project vs existing project and adjusts behavior. |
| 4 | Problem exploration (new project) | Asks about the problem, who has it, current solutions, pain points. |
| 5 | User exploration (new project) | Identifies primary/secondary users, technical level, priorities (speed, simplicity, power, reliability). |
| 6 | Vision exploration (new project) | Defines success criteria, finds the MVP, establishes explicit boundaries. |
| 7 | Concept challenge (new project) | Questions whether this is the right solution, surfaces existing tools/patterns, identifies risks and unknowns. |
| 8 | Constraints gathering (new project) | Technology preferences, timeline, resources, integrations, scale expectations. |
| 9 | Skip known answers (feature mode) | Does not re-ask what the codebase already answers (tech stack, conventions). |
| 10 | Fit analysis (feature mode) | Explores how the feature relates to existing modules. |
| 11 | Impact analysis (feature mode) | Identifies what existing behavior might change or break. |
| 12 | Gap identification (feature mode) | Identifies what the codebase lacks that the feature needs. |
| 13 | Boundary definition (feature mode) | Clarifies where the feature ends and existing functionality begins. |
| 14 | User expectation analysis (feature mode) | Explores how existing users will discover and interact with the feature. |
| 15 | Concept challenge (feature mode) | Still challenges whether the feature is the right solution. |
| 16 | Web research | 2-3 targeted WebSearch/WebFetch calls to research patterns and validate assumptions. Transparent about what is searched and why. |
| 17 | Synthesis and approval gate | Summarizes findings, presents to user, asks for explicit approval. Iterates if user wants changes. Only writes file after approval. **Mandatory.** |
| 18 | Full Idea Brief template | For thorough discoveries: one-line summary, problem statement, current state, proposed solution, target users, success criteria, MVP scope, out of scope, key decisions, open questions, constraints, risks, analogies. |
| 19 | Lightweight Idea Brief template | For well-understood ideas: one-line summary, problem, solution, MVP scope, out of scope, open questions (optional), risks (optional). |
| 20 | Quick vs thorough judgment | Determines depth: thorough for vague/complex ideas, quick for well-defined requests. |
| 21 | Disagreement handling | States concerns once, respects user's decision, documents in Risks and Unknowns, never blocks the pipeline. |
| 22 | Conversational techniques | Start broad then narrow, scenarios, analogies, challenge gently, mirror back, expose hidden complexity, "kill question" ("If you could only build ONE thing, what would it be?"). |
| 23 | Context limit handling | Saves progress to `docs/.workflow/discovery-summary.md`. |

### Outputs

| File | Condition |
|------|-----------|
| `docs/.workflow/idea-brief.md` | Primary output (Full or Lightweight template) |
| `docs/.workflow/discovery-partial.md` | If user abandons mid-conversation |
| `docs/.workflow/discovery-summary.md` | If context limits approached |

### Fail-Safe Controls

1. Synthesis approval gate is mandatory — will not write without explicit user confirmation.
2. Partial-save on abandon — no work lost if user stops.
3. Context limit save — writes progress to disk before context runs out.
4. Role boundaries enforced — will NOT produce requirements, IDs, or acceptance criteria (Analyst's job), will NOT design architecture (Architect's job).

### Anti-Patterns Forbidden

- Asking 20 questions in a wall of text.
- Being a requirements robot.
- Assuming technical knowledge.
- Skipping the challenge phase.
- Over-scoping.
- Producing a requirements document.

---

## 2. Analyst Agent

**File:** `.claude/agents/analyst.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob, WebFetch, WebSearch

### Prerequisite Gates

1. **If invoked after Discovery** (in `/workflow:new` or `/workflow:new-feature`): verifies `docs/.workflow/idea-brief.md` exists. Stops if missing.
2. **If invoked directly** (in `/workflow:improve-functionality`, `/workflow:bugfix`): no idea brief needed — user's description is input.

### Inputs

- `docs/.workflow/idea-brief.md` (from Discovery, when applicable).
- User's direct description (for improve/bugfix workflows).
- `specs/SPECS.md` and `docs/DOCS.md` indexes.
- Codebase source files (scoped to relevant area).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `specs/`, `specs/bugfixes/`, `specs/improvements/` if missing. |
| 2 | Source of truth hierarchy | Reads codebase first (truth), then specs, then docs. Flags conflicts. |
| 3 | SPECS.md existence check | If exists: reads index to understand layout. If not: treats as new project. |
| 4 | Scope determination | Uses `--scope` if provided; otherwise determines minimal scope and states it explicitly. |
| 5 | Scoped reading | Uses Grep/Glob first. Never reads entire codebase. |
| 6 | Context limit handling | Summarizes to `docs/.workflow/analyst-summary.md`, recommends splitting. |
| 7 | Deep requirement understanding | Analyzes the idea or requirement thoroughly. |
| 8 | Questioning everything unclear | Assumes NOTHING. Generates questions, presents to user, waits for answers. |
| 9 | Problem identification | Identifies problems in the idea before they become code. |
| 10 | Drift flagging | Flags when specs/docs don't match actual code (existing projects). |
| 11 | Impact analysis | Determines what existing code/behavior breaks or changes. |
| 12 | MoSCoW prioritization | Must/Should/Could/Won't for every requirement. |
| 13 | Acceptance criteria definition | Concrete, verifiable Given/When/Then conditions. "It should work" never acceptable. |
| 14 | User story writing | "As a [user], I want [X] so that [Y]" format. |
| 15 | Requirement ID assignment | Unique IDs in `REQ-[DOMAIN]-[NNN]` format (e.g., REQ-AUTH-001). |
| 16 | Explicit assumptions | Two formats: technical (for agents) and plain language (for user). |
| 17 | Existing project process | Read SPECS.md → identify relevant specs → read code → impact analysis → analyze → question → generate requirements. |
| 18 | New project process | Skip codebase → understand idea → question → generate requirements → create specs/. |
| 19 | Traceability matrix creation | Links Requirement ID, Priority, Test IDs, Architecture Section, Implementation Module. |
| 20 | Specs drift detection output | "Specs Drift Detected" section in output noting outdated spec files. |
| 21 | Impact analysis output | Existing Code Affected, What Breaks, Regression Risk Areas. |
| 22 | Risk identification | Documents risks with mitigations. |
| 23 | Out of scope documentation | Documents what will NOT be done and why. |
| 24 | Specs maintenance | Checks existing specs, flags drift, updates stale specs, updates SPECS.md index. |
| 25 | Non-technical adaptation | Adapts questions for non-technical users. |
| 26 | Challenging the idea | Challenges fundamental problems directly. |

### Outputs

| File | Condition |
|------|-----------|
| `specs/[domain]-requirements.md` | Primary output |
| `specs/SPECS.md` | Updated with link to new requirements (created if missing) |
| Updated stale spec files | If drift detected |
| `docs/.workflow/analyst-summary.md` | If context limits approached |

### Fail-Safe Controls

1. Prerequisite gate — stops if idea-brief missing when expected.
2. Directory safety — creates directories before writing.
3. Never assumes — always asks.
4. Every requirement must have acceptance criteria — no exceptions.
5. Every requirement must have a priority — enforced.
6. Every requirement must have a unique ID — enforced.
7. Context limit save.
8. Specs drift flagging is mandatory.
9. Impact analysis is mandatory for existing projects.

---

## 3. Architect Agent

**File:** `.claude/agents/architect.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Edit, Grep, Glob

### Prerequisite Gates

1. **Analyst requirements must exist** — Globs for `specs/*-requirements.md`, `specs/bugfixes/*-analysis.md`, or `specs/improvements/*-improvement.md`. Stops if missing.
2. **Content quality check** — Reads file and confirms it contains requirement IDs, priorities, and acceptance criteria. Stops if empty or malformed.

### Inputs

- Analyst's requirements document in `specs/`.
- `specs/SPECS.md` and `docs/DOCS.md` indexes.
- Codebase source files (scoped to relevant area).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `specs/`, `docs/`, `docs/.workflow/` if missing. |
| 2 | Source of truth hierarchy | Codebase first, then specs, then docs. |
| 3 | Index-first reading | Reads SPECS.md and DOCS.md to understand layout without reading every file. |
| 4 | Scope enforcement | Limits strictly to `--scope` if provided. |
| 5 | Requirements-first reading | Reads analyst's requirements first (scope, priorities, affected files). |
| 6 | Grep before Read | Locates relevant code before reading whole files. |
| 7 | Large project milestone processing | For `/workflow:docs` and `/workflow:sync`: works one milestone at a time, saves progress between milestones. |
| 8 | Context limit handling | Summarizes to `docs/.workflow/architect-summary.md`. |
| 9 | Drift flagging | Flags drift between code and specs/docs. |
| 10 | Module structure design | Designs modules, interfaces, and dependencies. |
| 11 | Per-module failure mode design | What fails, why, how to detect, how to recover, what's affected. |
| 12 | System-level failure mode design | DB unavailable, API timeout, disk full, etc. with detection and recovery. |
| 13 | Security considerations | Trust boundaries, sensitive data, attack surfaces, mitigations per module. |
| 14 | Security model design | Trust boundaries, data classification (public/internal/confidential/secret), attack surface with risks. |
| 15 | Performance budgets | Latency targets (p50, p99), memory budgets, complexity targets, throughput targets. |
| 16 | Graceful degradation design | Normal behavior vs degraded behavior vs user impact per dependency. |
| 17 | Implementation order | Defines the order modules should be implemented. |
| 18 | Specs creation/update | Creates/updates spec files in `specs/[domain].md`. |
| 19 | SPECS.md index update | Updates master index with new entries. |
| 20 | Docs creation/update | Creates/updates doc files in `docs/[topic].md`. |
| 21 | DOCS.md index update | Updates master index with new entries. |
| 22 | Traceability matrix update | Fills in "Architecture Section" column for each requirement ID. |
| 23 | Data flow design | Documents information flow between modules. |
| 24 | Design decisions documentation | Records decisions, alternatives considered, justifications. |
| 25 | External dependencies documentation | Lists crate/library dependencies with version and purpose. |
| 26 | New feature process | Read requirements → read scoped codebase/specs → design → update specs/docs → update indexes → update traceability. |
| 27 | Greenfield process | Read requirements → design full structure → define modules/interfaces/order → failure modes/security/performance → create specs/ and docs/ from scratch. |
| 28 | Documentation mode (`/workflow:docs`) | Per milestone: read code → compare against specs → update stale → create missing → save checkpoint → update indexes. |
| 29 | Sync mode (`/workflow:sync`) | Per milestone: read code → read specs/docs → log drift → fix drift → save checkpoint → generate drift report → update indexes. |

### Outputs

| File | Condition |
|------|-----------|
| `specs/[domain]-architecture.md` | Primary output |
| Updated/created spec files in `specs/` | Always |
| Updated/created doc files in `docs/` | Always |
| `specs/SPECS.md` | Updated index |
| `docs/DOCS.md` | Updated index |
| Updated traceability matrix | In requirements document |
| `docs/.workflow/architect-progress.md` | Between milestones |
| `docs/.workflow/architect-summary.md` | If context limits approached |

### Fail-Safe Controls

1. Prerequisite gate — stops if no analyst requirements.
2. Content quality check — stops if requirements malformed.
3. Directory safety.
4. Mandatory traceability matrix update.
5. Mandatory index updates (SPECS.md and DOCS.md).
6. Context limit save.
7. Never reads entire codebase.

### Design Rules Enforced

- Composition over inheritance.
- Single responsibility per module.
- Interfaces defined before implementation.
- Testability considered from design phase.
- Failure modes documented for every module.
- Trust boundaries and attack surfaces identified before code.
- Performance budgets set.
- Graceful degradation planned.

---

## 4. Test Writer Agent

**File:** `.claude/agents/test-writer.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Edit, Bash, Glob, Grep

### Prerequisite Gates

1. **Architect design must exist** — Globs for `specs/*-architecture.md`. Stops if missing.
2. **Analyst requirements must exist** — Globs for requirements files. Stops if missing.
3. **Content quality verification** — Reads both files and confirms they contain requirement IDs, priorities, module definitions. Stops if malformed.

### Inputs

- Analyst's requirements (IDs, priorities, acceptance criteria).
- Architect's design (scope, modules, failure modes, security model).
- Existing codebase (for test conventions and patterns).
- Relevant spec files.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Language detection and adaptation | Detects language from config files. Follows conventions: Rust (`#[test]`), TS (`describe`/`it`), Python (`pytest`), Go (`func Test*`). |
| 2 | Match existing conventions | If project has tests, follows their patterns exactly. |
| 3 | New project test placement | Follows Architect's guidance; if none, uses language standard conventions. |
| 4 | Directory safety | Creates test directories if missing. |
| 5 | Source of truth hierarchy | Codebase first, then requirements, then architecture, then specs. |
| 6 | Requirements-first reading | Reads requirements for IDs, priorities, acceptance criteria. |
| 7 | Architecture reading | Reads design for scope, modules, failure modes, security model. |
| 8 | Grep for existing patterns | Searches for test patterns before reading every file. |
| 9 | Module-at-a-time processing | Writes all tests for module 1, then module 2, etc. |
| 10 | Priority ordering within module | Must tests first, then Should, then Could. |
| 11 | Context limit handling | Saves tests to disk, notes remaining in `docs/.workflow/test-writer-progress.md`. |
| 12 | Must requirement testing (exhaustive) | Every acceptance criterion, every failure mode, every security consideration, all 10 worst scenarios. Non-negotiable. |
| 13 | Should requirement testing | Every acceptance criterion, key failure modes and edge cases. |
| 14 | Could requirement testing | Basic happy-path per criterion, minimal edge cases. |
| 15 | Won't requirement handling | Skipped entirely (explicitly deferred). |
| 16 | Requirement ID traceability | Every test references its requirement ID in comments: `// Requirement: REQ-AUTH-001 (Must)`. |
| 17 | Acceptance criteria to test mapping | Each criterion becomes at least one test. |
| 18 | Failure mode testing | Tests detection, recovery, and degraded behavior. |
| 19 | Security testing | Tests trust boundary enforcement, sensitive data protection, attack surface resistance. |
| 20 | Edge case testing (10 worst scenarios) | Empty/null, negative numbers, overflow, unicode, concurrency, disk full, network interrupted, huge input, inconsistent data, interrupted operation. |
| 21 | Integration tests | Cross-module tests if applicable. |
| 22 | Traceability matrix update | Fills in "Test IDs" column for each requirement. |
| 23 | Test structure placement | Language-specific: Rust (`backend/tests/`), TS (colocated `*.test.ts`), Python (`tests/`), Go (colocated `*_test.go`). |
| 24 | Specs consistency check | Flags undocumented behavior in codebase. Flags architect/code contradictions. Adds "Specs Gaps Found" to progress file. |
| 25 | Min 3 edge cases per Must public function | Enforced. |
| 26 | Tests must fail initially | Red phase of TDD. |
| 27 | Save after each module | Writes to disk before moving to next module. |

### Outputs

| File | Condition |
|------|-----------|
| Test files (language-specific) | Primary output — must fail initially |
| Updated traceability matrix | Test IDs column in requirements document |
| `docs/.workflow/test-writer-progress.md` | Progress notes and specs gaps found |

### Fail-Safe Controls

1. Three prerequisite gates (architecture, requirements, content quality).
2. Directory safety.
3. Mandatory requirement ID reference in every test.
4. Mandatory acceptance criterion coverage.
5. Mandatory traceability matrix update.
6. Specs inconsistency flagging — reports rather than silently ignoring.
7. Context limit save.
8. No orphan tests — every test references a requirement ID.

---

## 5. Developer Agent

**File:** `.claude/agents/developer.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Edit, Bash, Glob, Grep

### Prerequisite Gates

1. **Tests must exist** — Globs for test files. Stops if missing.
2. **Architect design must exist** — Globs for `specs/*-architecture.md`. Stops if missing.
3. **Analyst requirements must exist** — Globs for requirements files. Stops if missing.

### Inputs

- Test files (from Test Writer).
- Architect's design document.
- Analyst's requirements document.
- Existing codebase (for conventions).
- Relevant spec files.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/.workflow/` and source directories (as defined by Architect) if missing. |
| 2 | Source of truth hierarchy | Codebase (style/conventions), requirements, specs, tests. |
| 3 | Max retry limit | Maximum 5 attempts per test-fix cycle per module. Stops with "MAX RETRY REACHED" and escalates for human review. |
| 4 | No advancement past failing module | Does NOT continue to next module with a failing one behind. |
| 5 | Traceability matrix update | Fills in "Implementation Module" column: `[module_name] @ [file_path]`. Mandatory. |
| 6 | Specs and docs sync | After each module: reads relevant spec, updates if behavior diverged (new API, changed behavior, renamed entities). Reads relevant doc, updates if user-facing behavior changed. Updates master indexes if new files created. Mandatory. |
| 7 | New project scaffolding | Reads Architect's design, sets up skeleton (package files, directories, entry points), creates scaffolding (`cargo init`, `npm init`, `go mod init`), commits separately. |
| 8 | Architecture-first reading | Reads Architect's design first for scope, modules, implementation order. |
| 9 | Module-at-a-time processing | For each module: reads only its tests, greps for patterns, reads only related files, implements, tests, commits. |
| 10 | Save work to disk frequently | Does not hold code in memory. |
| 11 | Run tests after each module | From relevant directory (`backend/` or `frontend/`). |
| 12 | Context limit handling | Commits progress, notes done/remaining in `docs/.workflow/developer-progress.md`. |
| 13 | Convention matching | Greps existing code for naming, error handling, patterns. |
| 14 | Minimum code implementation | Implements minimum to pass tests. No over-engineering. |
| 15 | TDD cycle | Red → Green → Refactor → Sync Specs/Docs → Commit → Next. |
| 16 | Commit per module | Conventional messages (`feat:`, `fix:`, `refactor:`). |
| 17 | Final full test run | Runs ALL tests together at the end. |
| 18 | Ask, don't assume | If unclear in architecture, asks. |

### Outputs

| File | Condition |
|------|-----------|
| Source code files | Primary output |
| Updated spec files | If behavior diverged from documented |
| Updated doc files | If user-facing behavior changed |
| `specs/SPECS.md`, `docs/DOCS.md` | If new spec/doc files created |
| Updated traceability matrix | Implementation Module column |
| Git commits | One per module |
| `docs/.workflow/developer-progress.md` | If context limits approached |

### Fail-Safe Controls

1. Three prerequisite gates (tests, architecture, requirements).
2. Directory safety.
3. Max 5 retry limit per module — escalates on failure.
4. No skipping failing modules — strict sequential order.
5. Mandatory traceability matrix update.
6. Mandatory specs/docs sync.
7. Context limit save.

### Checklist Per Module

- [ ] Existing code patterns grepped (not full read)
- [ ] Tests read and understood
- [ ] Implementation complete
- [ ] All tests pass
- [ ] No compiler warnings
- [ ] Code matches project conventions
- [ ] Relevant specs/docs updated (if behavior changed)
- [ ] Code written to disk
- [ ] Commit done
- [ ] Ready for next module (context is manageable)

---

## 6. QA Agent

**File:** `.claude/agents/qa.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Edit, Bash, Glob, Grep

### Prerequisite Gates

1. **Code must exist** — Globs for source files. Stops if missing.
2. **Tests must exist** — Greps for test files. Stops if missing.
3. **Analyst requirements should exist** — Checks `specs/`. If missing, notes gap but proceeds.

### Inputs

- Source code (from Developer).
- Test files (from Test Writer).
- Analyst's requirements (acceptance criteria, priorities).
- Architect's design (failure modes, security model, degradation).
- Traceability matrix.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/qa/` and `docs/.workflow/` if missing. |
| 2 | System Won't Start Fallback | If system fails to start: documents error as BLOCKING, includes error output and command, does NOT fix (reports to Developer), proceeds with static analysis/test execution/traceability. |
| 3 | Source of truth hierarchy | Codebase (run it), requirements, architecture, traceability. |
| 4 | Context management | Reads requirements first, then architecture, then traceability matrix. |
| 5 | Scope enforcement | Limits strictly to `--scope` if provided. |
| 6 | Module-at-a-time processing | Validates, records, moves on. |
| 7 | Progressive saving | Writes to `docs/.workflow/qa-progress.md` after each module. |
| 8 | Context limit handling | Saves to `docs/.workflow/qa-partial.md`. |
| 9 | **Step 0: Orient Yourself** (mandatory) | Discovers how to run system: checks README, Makefile, docker-compose, package.json, etc. Extracts startup command, test suite command, env setup. Records startup command before continuing. |
| 10 | **Step 1: Read Requirements and Design** | Acceptance criteria, priorities, failure modes, security, degradation, traceability gaps. |
| 11 | **Step 2: Traceability Matrix Validation** | Every Must/Should has test IDs, every test ID maps to actual test, every requirement in architecture. Flags gaps. |
| 12 | **Step 3: Acceptance Criteria Verification** | By MoSCoW priority: Must (verify all), Should (verify all), Could (verify if implemented). Documents failures with expected vs actual. |
| 13 | **Step 4: End-to-End Flow Testing** | Maps critical flows crossing module boundaries, executes against running system, verifies data flow, uses realistic data. |
| 14 | **Step 5: Exploratory Testing** | Wrong-order usage, boundary abuse (empty, zero, negative, max length, unicode, whitespace), mistake recovery (typo, back, refresh, double-submit), assumption violations (simultaneous users, slow dependencies). Records findings immediately. |
| 15 | **Step 6: Failure Mode Validation** | Triggers failure conditions, verifies detection/recovery/degradation/restoration. Techniques: firewall block, kill DB, inject delay, fill disk, corrupt config, concurrent requests. Marks untestable as "Not Triggered." |
| 16 | **Step 7: Security Validation** | Trust boundaries (path traversal, IDOR), data exposure (error messages, logs, API responses), attack vectors (SQLi, XSS, auth bypass, mass assignment, rate limiting). Records what sent, what returned, pass/fail. |
| 17 | Specs/docs drift check | Reads spec files, compares documented vs actual behavior. Reads doc files, flags discrepancies. Adds "Specs/Docs Drift" section to QA report. Mandatory. |
| 18 | Three-verdict system | PASS (all Must/Should met), CONDITIONAL APPROVAL (all Must met, some Should failed), FAIL (any Must fails). |
| 19 | Blocking vs non-blocking classification | Must failures = blocking. Should failures = non-blocking. System startup failures = blocking. |

### Outputs

| File | Condition |
|------|-----------|
| `docs/qa/[domain]-qa-report.md` | Primary output (includes drift section) |
| `docs/.workflow/qa-progress.md` | Incremental findings |
| `docs/.workflow/qa-partial.md` | If context limited |

### Fail-Safe Controls

1. Two hard prerequisite gates (code, tests) plus one soft gate (requirements).
2. Directory safety.
3. System Won't Start Fallback — does not silently proceed if system fails.
4. Step 0 is mandatory — never begins without confirming system can run.
5. Mandatory specs/docs drift check.
6. Incremental saving after each module.
7. Context limit save.
8. Must failures are always blocking — no exceptions.
9. Acceptance criteria are pass/fail — no "partially met."

---

## 7. Reviewer Agent

**File:** `.claude/agents/reviewer.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob (READ-ONLY)

### Prerequisite Gates

1. **Code must exist** — Globs for source files. Stops if missing.
2. **For workflow reviews** (after QA): checks for QA report in `docs/qa/`. Notes gap if missing but proceeds.
3. **For audit mode** (`/workflow:audit`): code is the only prerequisite.

### Inputs

- Source code.
- Test files.
- Architect's design document.
- Spec files in `specs/`.
- Doc files in `docs/`.
- QA report (when available).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/reviews/`, `docs/audits/`, `docs/.workflow/` if missing. |
| 2 | Architecture escalation | Flags architectural flaws as CRITICAL with `[ARCHITECTURE]` tag, explains why design is flawed, recommends Architect redesign. |
| 3 | Source of truth hierarchy | Codebase (what runs), specs (compare against), docs (verify accuracy). |
| 4 | Architecture-first reading | Reads Architect's design first for scope. |
| 5 | Scope enforcement | Limits strictly to `--scope` if provided. |
| 6 | Module-at-a-time processing | Reads implementation, tests, spec file, notes findings per module. |
| 7 | Cross-cutting Grep scans | Searches for `unwrap()`, `unsafe`, `TODO`, `HACK` across scoped area without reading every file. |
| 8 | Progressive saving | Writes to `docs/.workflow/reviewer-findings.md` after each module. |
| 9 | Milestone processing for audits | Per milestone: process, save, next, compile final report. |
| 10 | Context limit handling | Saves to `docs/.workflow/reviewer-partial.md`. |
| 11 | Correctness review | Implementation meets requirements? Follows architecture? Tests pass? Logic gaps? |
| 12 | Security review | Unsanitized inputs, sensitive data exposure, internal info in errors, SQLi, XSS, other vulnerabilities. |
| 13 | Performance review | O(n^2) or worse, unnecessary allocations, clones vs references, blocking I/O vs async. |
| 14 | Maintainability review | Readability, single responsibility, code duplication, descriptive names. |
| 15 | Technical debt (cross-language) | TODO/HACK/FIXME/XXX, dead code, duplicated blocks. |
| 16 | Technical debt (Rust) | `unwrap()` (should use `?`/`expect`), `unsafe` without safety comments, unnecessary `clone()`. |
| 17 | Technical debt (Python) | Bare `except:`, `# type: ignore`, `global`/mutable module-level state. |
| 18 | Technical debt (TypeScript/JS) | `any` type, `// @ts-ignore`/`// eslint-disable`, leftover `console.log`. |
| 19 | Technical debt (Go) | `_ = err` (silenced errors), `interface{}`/`any` overuse, `//nolint` suppressed warnings. |
| 20 | Language detection and adaptation | Detects language(s), applies relevant debt patterns. |
| 21 | Specs/docs drift review | 5-point checklist: specs match implementation? New modules uncovered? Docs reflect behavior? Indexes up to date? Specs referencing deleted code? |
| 22 | Finding classification | Critical (with fix), minor (with fix), specs/docs drift, improvement suggestions. |

### Outputs

| File | Condition |
|------|-----------|
| Review report in `docs/reviews/` | After workflow chains |
| Audit report in `docs/audits/` | For `/workflow:audit` |
| `docs/.workflow/reviewer-findings.md` | Progressive findings |
| `docs/.workflow/reviewer-partial.md` | If context limited |

### Fail-Safe Controls

1. Prerequisite gate — stops if no source code.
2. Directory safety.
3. Architecture escalation — distinct from normal code fixes.
4. Context limit save.
5. Read-only enforcement — tools are Read, Grep, Glob only.
6. Incremental saving.
7. Explicit reporting of skipped modules.

---

## 8. Functionality Analyst Agent

**File:** `.claude/agents/functionality-analyst.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob (READ-ONLY)

### Prerequisite Gates

1. **Source code must exist** — Globs for source files (`**/*.rs`, `**/*.ts`, `**/*.py`, `**/*.go`, etc.). Stops if missing.

### Inputs

- The codebase itself — the ONLY source of truth. Ignores `specs/` and `docs/` entirely.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/functionalities/` and `docs/.workflow/` if missing. |
| 2 | Code-only truth | Does NOT read or trust any documentation. Discovers what the code does by reading the code. |
| 3 | Structure-first context management | Uses Glob to map directory tree before reading any files. |
| 4 | Scope enforcement | Limits strictly to `--scope` if provided. |
| 5 | Domain/module identification | Identifies modules from directory structure, works through them one at a time. |
| 6 | Module-at-a-time processing | Discover files → identify entry points/public interfaces → read key files → record → move on. |
| 7 | Grep before Read | Searches patterns across files without reading them all. |
| 8 | Progressive saving | Writes to `docs/.workflow/functionality-analyst-progress.md` after each module. |
| 9 | Context limit handling | Saves to `docs/.workflow/functionality-analyst-partial.md`. |
| 10 | Project structure mapping | Maps directories and modules. |
| 11 | Functionality discovery | Public APIs/endpoints (REST, GraphQL, gRPC, WebSocket), services/business logic, data models/schemas, CLI commands, event handlers/listeners, scheduled tasks/cron, middleware/interceptors, external integrations, config/feature flags, DB migrations, background workers/queues. |
| 12 | Entry point search via Grep | Routes (`#[get`, `router.`, `@app.route`), handlers (`async fn handle`, `Controller`), models (`struct`, `class`, `@Entity`), exports (`pub fn`, `export`), events (`on(`, `emit(`), CLI (`#[command`, `argparse`), middleware, scheduled, migrations. |
| 13 | Per-functionality catalog | What it does (one-line), where (file:line), type (endpoint/service/model), dependencies, actively used. |
| 14 | Dead code detection | Exports with no importers, handlers without routes, unreferenced models, `#[allow(dead_code)]` equivalents. |
| 15 | Domain categorization | Groups by domain/module. |
| 16 | Dependency notation | Notes cross-functionality dependencies (e.g., "endpoint X calls service Y which uses model Z"). |
| 17 | Per-domain output | `docs/functionalities/[domain]-functionalities.md` with overview, table, dependencies, dead code. |
| 18 | Master index | `docs/functionalities/FUNCTIONALITIES.md` with summary, modules table, cross-module dependencies. |
| 19 | Language adaptation | Detects language(s), adjusts search patterns. |

### Outputs

| File | Condition |
|------|-----------|
| `docs/functionalities/[domain]-functionalities.md` | Per-domain files |
| `docs/functionalities/FUNCTIONALITIES.md` | Master index |
| `docs/.workflow/functionality-analyst-progress.md` | Progress |
| `docs/.workflow/functionality-analyst-partial.md` | If context limited |

### Fail-Safe Controls

1. Prerequisite gate — stops if no source code.
2. Directory safety.
3. Code-only truth — never reads or references specs/docs.
4. Read-only — does not modify source code.
5. Context limit save.
6. Incremental saving.
7. Explicit coverage reporting.

---

## 9. Codebase Expert Agent

**File:** `.claude/agents/codebase-expert.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob (READ-ONLY)

### Prerequisite Gates

1. **Source code must exist** — Globs for source files. Stops if missing.

### Inputs

- The codebase itself (primary source of truth).
- `specs/` and `docs/` (read for context, verified against code; outdated items flagged).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/understanding/` and `docs/.workflow/` if missing. |
| 2 | Code is only truth | Discovers from code. Reads specs/docs for context but verifies against actual code. Flags outdated items. |
| 3 | 6-layer progressive approach | Handles codebases of any size by working in progressive layers. |
| 4 | **Layer 1: Project Shape** | Languages, frameworks, build systems from config files. README and index files. |
| 5 | **Layer 2: Architecture & Boundaries** | Major subsystems/modules, entry points, bootstrap flow, dependency direction. |
| 6 | **Layer 3: Domain & Business Logic** | Core domain models/entities, relationships, main workflows, DDD patterns. |
| 7 | **Layer 4: Data Flow & State** | Entry → processing → storage → exit. State management (DB, caches, memory, files). Config flow. Serialization boundaries. |
| 8 | **Layer 5: Patterns & Conventions** | Naming, error handling, logging, testing patterns. Architectural patterns (middleware, plugins, DI). The "template" for new features. Convention breaks and why. |
| 9 | **Layer 6: Complexity & Risk Map** | Most complex modules (longest files, deepest nesting). God objects. Error handling strategies. Security-sensitive areas. Performance hot paths. |
| 10 | Scoped analysis | If `--scope` provided: still does Layer 1 for context, focuses Layers 2-6 on scoped area, notes external dependencies and callers. |
| 11 | Progressive summarization | After Layers 1-2: saves `expert-structure.md`. After 3-4: saves `expert-domain.md`. After 5-6: saves `expert-patterns.md`. Compiles final document from saved progress. |
| 12 | Context limit handling | Saves `expert-partial.md`, states which layers completed, recommends scoped follow-up. Never silently produces shallow analysis. |
| 13 | Architecture discovery patterns | Entry points, config files, build files, dependency files. |
| 14 | Domain logic discovery patterns | Models, services, routes, events directories. |
| 15 | Pattern discovery via Grep | Error handling, logging, testing, auth, config patterns. |
| 16 | Complexity discovery | Long files (line count), deep nesting, high fan-out (many imports), god modules (many importers). |
| 17 | Output structure | Quick summary, tech stack, annotated project structure, architecture (modules/boundaries/dependencies), domain model, data flow, patterns/conventions, complexity/risk map, key files (10-20 with explanations), onboarding guide, drift detected, metadata. |
| 18 | Scoped output | Same structure plus external dependencies and callers section. |
| 19 | Specs/docs drift detection | Reports discrepancies between specs/docs and actual code. |
| 20 | Depth over breadth | Better to deeply understand 3 modules than superficially scan 20. |
| 21 | Explain the "why" | Not just "X calls Y" — explains why the dependency exists. |

### Outputs

| File | Condition |
|------|-----------|
| `docs/understanding/PROJECT-UNDERSTANDING.md` | Primary output (full project) |
| `docs/understanding/[scope]-understanding.md` | Scoped output |
| `docs/.workflow/expert-structure.md` | After Layers 1-2 |
| `docs/.workflow/expert-domain.md` | After Layers 3-4 |
| `docs/.workflow/expert-patterns.md` | After Layers 5-6 |
| `docs/.workflow/expert-partial.md` | If context limited |

### Fail-Safe Controls

1. Prerequisite gate — stops if no source code.
2. Directory safety.
3. Read-only — does not modify source code.
4. Progressive summarization — saves after every two layers.
5. Context limit save — never silently produces shallow analysis.
6. Explicit coverage reporting — honest about limits, states confidence level.
7. Layered approach prevents context overload — never skips to deep analysis without understanding shape first.

---

## 10. Proto-Auditor Agent

**File:** `.claude/agents/proto-auditor.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob (READ-ONLY)
**Version:** 2.0

### Prerequisite Gates

1. **Document integrity check** — Reads all protocol and enforcement layer files completely. If too corrupted to audit: stops. If partially usable: proceeds but notes integrity gap in every affected dimension.

### Inputs

- Protocol specification files.
- Enforcement layer files (optional).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Three-level operation | L1 (protocol spec), L2 (enforcement layer — AUDITOR_BOOT, REVIEWER_BOOT), L3 (self-audit — PROTO-AUDITOR consistency check). Every dimension at all applicable levels. |
| 2 | Cross-level interaction flagging | Cross-level interactions flagged as their own finding class. |
| 3 | Adversarial stance | Default: every protocol broken until proven safe, every rule has a gap, every guarantee overclaimed, every trust mechanism gameable, every enforcement role capturable, every boundary leaky. |
| 4 | Anti-circularity mandate | If audit depends on a mechanism also being audited, flags as meta-dependency and reasons independently. |
| 5 | **D1: Self-Reference Integrity** | Rule enforcement depending on the rule it enforces, meta-rules enforceable by governed rules, governance suspension exploitation, enforcement authority derived from enforced protocol. |
| 6 | **D2: Trust Model Soundness** | Self-reported trust without oracle, mutual confirmation skipping verification (Sybil), trust-immune entities, trust earning on low-stakes / spending on high-stakes. |
| 7 | **D3: Confidence Claim Validity** | Arbitrary thresholds, aggregation assuming independence when correlated, gaming via low-quality submissions, subjective gates behind objective thresholds. |
| 8 | **D4: Escalation & Deadlock** | Livelock cycles, counter reset behavior, single-exchange override, max iteration counts, terminal states for error recovery. |
| 9 | **D5: Quorum & Partition** | Quorum math for N=2,3,4,10, partition detection, log comparison assuming honest logging, dual topic ownership, delegate election. |
| 10 | **D6: Adversarial Agent Resistance** | Trust inflation via Sybil, capability manifest poisoning, replay attacks (no nonce/signature), protocol injection, keyword detection bypass, version manipulation, strategic provisional answers. |
| 11 | **D7: Specification Completeness** | Undefined terms in security-critical contexts, edge cases (N=1, N=10), cross-domain infinite regress, scope mismatch, state transition completeness. |
| 12 | **D8: Enforcement Realism** | Enforcer accountability-immunity, adversarial stances as prompts not constraints, operator privilege vs audit, runtime audit, appeal mechanisms, compliance criteria. |
| 13 | **D9: Temporal & Ordering Integrity** | Counter sync (local vs global), TOCTOU, race conditions in simultaneous declarations, provisional expiry disagreement, ordering effects on escalation. |
| 14 | **D10: Composability & Cross-Layer** | Authority hierarchy mismatch, trust score divergence, enforcement scope creep, runtime rule creation coverage, cross-version compatibility, degraded mode standards. |
| 15 | **D11: Information Leakage & Side Channels** | Trust scores revealing history, escalation patterns revealing topology, capability manifests as attack inventory, enforcement verdicts teaching attackers, timing attacks. |
| 16 | **D12: Self-Audit (Auditor Integrity)** | Protocol text injected via prompt, severity self-defined, sequential processing, "proof" is LLM reasoning, no versioning interlock, L3 findings unfixable by self. |
| 17 | Severity classification | CRITICAL (undetected violation, deadlock reachable, trust captured, enforcement bypassed), MAJOR (spec gap in reachable states, no termination guarantee, quorum math fails), MINOR (ambiguous term with low exploitability, bounded suboptimality). |
| 18 | Severity stacking | If two MINOR findings combine to CRITICAL impact, both upgraded to MAJOR with cross-reference. |
| 19 | Back-propagation check | After D12, re-reads all earlier verdicts and revises any invalidated by later findings. |
| 20 | Output: audit() per dimension | Contains: id, rule_ref, severity, level, flaw, exploit_vector, preconditions, affected_dimensions, combines_with, recommendation, verdict, residual_risk. |
| 21 | Output: final_report() | Dimensions audited, back-propagation, severity counts, stacks, cross-layer findings, overall verdict, justification, residual risks, deployment conditions, meta-confidence. |

### Outputs

| File | Condition |
|------|-----------|
| `c2c-protocol/audits/audit-[protocol]-[date].md` | Primary output |

### Fail-Safe Controls

1. Document integrity check — stops if too corrupted.
2. Never declares "sound" without trying to break at all levels.
3. "No violations found" requires explicit proof, not absence of evidence.
4. Anti-circularity mandate.
5. Back-propagation — revises earlier verdicts based on later findings.
6. Every CRITICAL finding gets minimal closing recommendation.
7. Flags findings requiring formal verification as residual_risk.
8. Severity stacking treated seriously.
9. Never skips dimensions, never merges dimensions.
10. If enforcement layer absent, flags as enforcement_gap in every dimension.
11. Overall verdict never "perfect" — scale is broken → degraded → hardened → production-ready.

---

## 11. Proto-Architect Agent

**File:** `.claude/agents/proto-architect.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Edit, Grep, Glob

### Prerequisite Gates (Input Contract)

**Accepts ONLY:**
- Formal `audit()` outputs from PROTO-AUDITOR.
- Operator-approved change requests referencing specific audit IDs.
- Protocol diff requests with explicit before/after scope.

**Refuses:**
- Improvement requests not backed by an audit finding.
- "Make it better" without a cited flaw.
- Feature additions not motivated by a closed gap.
- Any input bypassing the audit pipeline.

### Inputs

- PROTO-AUDITOR audit reports (`audit()` blocks with finding IDs).
- Protocol specification files.
- Enforcement layer files.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Prime directive | Every audit finding valid until closed with provable fix. Every fix introduces new surface until proven otherwise. Every improvement adding complexity is suspect. Elegance is a security property. |
| 2 | Success metric | Measured in audit findings closed, not rules added. |
| 3 | **P1: Triage** | Classifies each finding as root_cause, symptom, ambiguity, or missing_axiom. Groups by root cause. Identifies fix dependencies. Flags circular dependencies. Output: `triage()`. |
| 4 | **P2: Root Cause Isolation** | Identifies rule layer (axiom, rule, meta, implicit). Determines minimum scope (atomic, coupled, structural). Structural changes escalated to operator. |
| 5 | **P3: Patch Generation** | Patch types: amend, extend, add, deprecate, axiom, define. Constraints: minimize new surface, define new terms in same patch, declare failure modes, no new trust assumptions, no widening self-reporting, no relaxing verification. Output includes: audit_ref, change_type, target, severity_closed, before, after, closes_gap, new_surface, mitigation, breaks_compatibility. |
| 6 | **P4: Patch Self-Audit** | 5-point check: (1) new self-reporting dependency? (2) new priority conflict? (3) synchronized state assumption? (4) quorum change (verify math N=2,3,5,10)? (5) longer protocol without closing CRITICAL? Rejected patches documented. Output: `self_audit()`. |
| 7 | **P5: Version Increment** | Structural/new mandatory rule → major. Amend/extend/define/axiom/deprecation → minor. Output: `version()`. |
| 8 | **P6: Regression Check** | Lists all rules interacting with patched rules. Confirms no guarantees broken. Expands batch or flags for operator if regression found. Output: `regression()`. |
| 9 | Patch quality tiers | TIER 1: closes root cause, no new surface (preferred). TIER 2: closes root cause, bounded new surface (acceptable, requires mitigation + operator notification). TIER 3: closes symptom, root cause deferred (structural change required; must include deferred_to, owner, review_trigger). TIER 4: rejected (not output). |
| 10 | Strict output format | `patch()` format with no English prose outside rule_text fields. Every patch maps to audit_ref, change_type, target_rule, before, after, rationale, risk. |

### Outputs

| File | Condition |
|------|-----------|
| `c2c-protocol/patches/patches-[protocol]-[date].md` | Primary output |

Contains: `triage()`, individual `patch()` blocks, `self_audit()`, `version()`, `regression()`.

### Fail-Safe Controls

1. Input contract enforcement — rejects requests without audit references.
2. No structural changes without operator approval.
3. Patch self-audit (P4) — 5-point check on every patch.
4. Regression check (P6) — verifies patches don't break interacting rules.
5. Patch quality tiers — TIER 4 patches never output.
6. CRITICAL findings cannot be closed with TIER 3 without operator sign-off.
7. Never adds a rule to fix a problem that deleting a rule would also fix.
8. Trust must only move toward external anchoring — patches requiring more inter-agent trust automatically rejected.
9. Conflicting patches not merged silently — flagged, both options presented, operator decides.
10. Patch batches are atomic — all pass or none apply.
11. Does not self-certify — never declares protocol "complete" or "secure"; only declares findings "closed" or "deferred."
12. Audited by PROTO-AUDITOR on every major version bump.
13. Sequential pipeline — P1 through P6 in order, no skipping, no merging.

### Relationships to Other Roles

| Role | Relationship |
|------|-------------|
| PROTO-AUDITOR | Upstream — produces findings consumed by PROTO-ARCHITECT |
| ADVERSARIAL-AUDITOR (Agent C) | Peer check on patches before merge |
| OPERATOR | Authority on structural changes and TIER 3 escalations |
| AGENT A/B | Downstream — receive patched protocol, no input authority |

---

## 12. Role Creator Agent

**File:** `.claude/agents/role-creator.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Grep, Glob, WebSearch, WebFetch

### Prerequisite Gates

1. **Role description exists** — the user must provide a non-empty description of the desired role.
2. **Description is meaningful** — must contain at least a noun (what the agent is) and a verb (what it does). "An agent" alone fails. "An agent that audits security" passes.
3. **No exact duplicate** — Globs `.claude/agents/*.md` and checks that no existing agent has the same name. Stops if duplicate found.

### Inputs

- Natural-language description of the desired role from user or `workflow-create-role` command.
- Existing agent definitions (`.claude/agents/*.md`) — for pattern consistency and overlap detection.
- Existing commands (`.claude/commands/*.md`) — for orchestration patterns.
- `CLAUDE.md` — for workflow rules and global constraints.
- Codebase (only if role requires domain-specific understanding).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `.claude/agents/`, `.claude/commands/`, `docs/.workflow/` if missing. |
| 2 | Role request analysis (Phase 1) | Reads user description, identifies what's clear vs. vague, checks existing agents for overlap. |
| 3 | Targeted clarification (Phase 2) | Asks questions when objective criteria are met: missing identity, missing boundary, missing trigger, missing output, ambiguous scope. Skipped when description is clear. Max 2 rounds. |
| 4 | Domain research (Phase 3) | Studies 2-3 existing agents by proximity (same domain, same stance, same complexity). Uses WebSearch (2-4 searches max) for best practices, pitfalls, quality criteria, edge cases. |
| 5 | Architecture design (Phase 4) | Walks the 14-item Role Anatomy Checklist. Performs overlap analysis. Selects minimal tool set (least privilege). Selects model (Opus for complex reasoning, Sonnet for procedural tasks). |
| 6 | Agent definition writing (Phase 5) | Produces complete agent definition file following the mandatory structure: YAML frontmatter, identity, personality, boundaries, prerequisite gate, directory safety, source of truth, context management, process (phases), output format, rules, anti-patterns, failure handling, integration. |
| 7 | Validation (Phase 6) | 5-point check: completeness (all anatomy checklist items), consistency (no CLAUDE.md contradictions), clarity (another LLM could execute unambiguously), boundary sharpness, failure coverage (5 common scenarios). |
| 8 | User confirmation (Phase 7) | Presents complete definition, explains key design decisions, waits for explicit approval before saving. |
| 9 | Companion artifacts (Phase 8) | Creates `.claude/commands/workflow-[name].md` if the agent should be invocable as a slash command. Notes pipeline integration opportunities without modifying existing commands. |
| 10 | Overlap detection | Cross-references new role against all existing agents to prevent duplicated responsibilities. |
| 11 | Context limit handling | Saves progress to `docs/.workflow/role-creator-progress.md`. |

### Outputs

| File | Condition |
|------|-----------|
| `.claude/agents/[name].md` | Primary output (after user approval) |
| `.claude/commands/workflow-[name].md` | If companion command requested |
| `docs/.workflow/role-creator-progress.md` | If context limited or user abandons |

### Fail-Safe Controls

1. Prerequisite gate — stops if no description, vague description, or duplicate name.
2. Directory safety.
3. Max 2 rounds of clarification — stops if still vague.
4. Overlap detection — reports overlap with evidence, asks user for resolution.
5. Scope-too-broad detection — recommends splitting into multiple agents.
6. File-exists check — stops if target file already exists.
7. Contradiction detection — cites exact quotes from user, asks for resolution.
8. Context limit save.
9. WebSearch fallback — proceeds without domain research if no results, notes limitation.
10. User approval required before saving — never writes without explicit consent.

---

## 13. Role Auditor Agent

**File:** `.claude/agents/role-auditor.md`
**Model:** claude-opus-4-6
**Tools:** Read, Grep, Glob (READ-ONLY)

### Prerequisite Gates

1. **Role definition file exists** — must be readable.
2. **YAML frontmatter present** — must contain `name`, `description`, `tools`, `model`.
3. **Body content present** — must have content after the frontmatter closing `---`.

### Inputs

- Target role definition file (`.claude/agents/[name].md`).
- All existing agents (`.claude/agents/*.md`) — for overlap detection in D2 and D11.
- `CLAUDE.md` — for pipeline rules and conventions.
- Existing commands (`.claude/commands/*.md`) — for D11 integration checks (when needed).

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Pre-audit setup (Phase 1) | Reads target role, runs prerequisite checks, reads all existing agents, reads CLAUDE.md, resolves `--scope` to dimension list. |
| 2 | **D1: Identity Integrity** | Checks: single-sentence identifiability, single core responsibility, real gap justification, no contradiction with existing agents, descriptive name, accurate YAML description. |
| 3 | **D2: Boundary Soundness** | Checks: explicit boundaries, overlap with each existing agent, scope creep resistance under prompt variation, implicit boundaries that should be explicit, process steps belonging to other agents. |
| 4 | **D3: Prerequisite Gate Completeness** | Checks: gate exists, covers all required inputs, stops with clear error, identifies failing upstream agent, bypass paths, content quality vs. file existence. |
| 5 | **D4: Process Determinism** | Checks: named phases with numbered steps, specific actions, explicit decision criteria, justified phase order, loop termination conditions, full lifecycle, error branching. |
| 6 | **D5: Output Predictability** | Checks: concrete template, save location specified, all output scenarios covered, downstream parseability, consistency with other agents, conditional section reliability. |
| 7 | **D6: Failure Mode Coverage** | Checks: 5 common failures (prerequisites, malformed input, context exhaustion, ambiguous instructions, upstream failure) + role-specific failures. Explicit vs. implicit handling. Silent degradation detection. Retry limits. Partial progress saving. |
| 8 | **D7: Context Management Soundness** | Checks: read order specified, "never read X" rules, scoping strategy (Grep/Glob before Read), `--scope` handling, checkpoint/save strategy, actionable vs. aspirational limits. |
| 9 | **D8: Rule Enforceability** | Checks: rule count (5-20 range), enforceability test (observable from output), aspirational language detection ("be thorough", "try to", "consider"), contradictions, mechanism vs. outcome, priority among rules. |
| 10 | **D9: Anti-Pattern Coverage** | Checks: section exists, domain-specific vs. generic, common LLM failure modes covered, explanations of why bad, redundancy with rules, actual prevention effectiveness. |
| 11 | **D10: Tool & Permission Analysis** | Checks: least privilege (every tool justified by process), missing tools (process needs tool not granted), Bash justification, read-only contradiction, model selection justification, WebSearch/WebFetch necessity. |
| 12 | **D11: Integration & Pipeline Fit** | Checks: upstream dependencies defined, downstream consumers defined, handoff format compatibility, pipeline conventions respected, command invocability, existing chain impact, output discoverability. |
| 13 | **D12: Self-Audit (Auditor Integrity)** | Checks: passes own anatomy checklist at 8/14, process matches own D4 standards, rules enforceable by own D8, has tools its process requires, severity calibration, gaming resistance, sequential ordering blind spots. |
| 14 | Back-propagation (Phase 3) | After D12, re-reads all earlier verdicts, revises any invalidated by later findings, records every revision. |
| 15 | Final report (Phase 4) | Tallies severity counts, identifies severity stacking, scores anatomy checklist (14 items), determines verdict mechanically from threshold table, lists deployment conditions and residual risks. |
| 16 | Severity classification | CRITICAL (will malfunction, silent degradation, privilege escalation, livelock), MAJOR (aspirational rules, missing failure handling, implicit boundaries), MINOR (redundant rules, generic anti-patterns). |
| 17 | Severity stacking | Two findings combining to CRITICAL behavior → both upgraded to MAJOR with cross-reference. |
| 18 | Scope parameter handling | Accepts dimension ranges (D1-D3), names (boundaries,tools), or both. D12 always included. Scoped audits cannot produce "deployable" verdict. |
| 19 | Multi-role audit ("all" mode) | Audits each role separately with full D1-D12 pass, then produces comparative summary noting overlaps, gaps, inconsistencies. |

### Outputs

| File | Condition |
|------|-----------|
| Audit report (returned to invoking command/agent) | Primary output |
| `docs/.workflow/role-audit-[name].md` | When saved by the invoking command |

### Fail-Safe Controls

1. Prerequisite gate — stops if file missing, empty body, or missing frontmatter.
2. Read-only enforcement — tools are Read, Grep, Glob only.
3. Never declares "sound" without listing every check performed and its result.
4. Back-propagation — revises earlier verdicts based on later findings.
5. Mechanical verdict — thresholds table determines verdict, never overridden.
6. Severity stacking treated seriously — two minors that combine to critical behavior → upgraded.
7. Never skips dimensions (unscoped). Never merges dimensions.
8. Context limit handling — summarizes completed dimensions, finishes remaining with summarized context.
9. D12 self-audit always included regardless of scope.
10. Audit what's written, not what's intended — missing sections are absent, not inferred.

---

## 14. Feature Evaluator Agent

**File:** `.claude/agents/feature-evaluator.md`
**Model:** claude-opus-4-6
**Tools:** Read, Write, Grep, Glob, WebSearch, WebFetch

### Prerequisite Gates

1. **Feature description exists** — must be non-empty (from idea brief or command arguments).
2. **Source code exists** — Globs for source files. Stops if missing: "Feature evaluation requires an existing codebase."
3. **If invoked after Discovery** — `docs/.workflow/idea-brief.md` must exist. Stops if missing.
4. **If invoked without Discovery** — command arguments serve as primary input.
5. **Idea Brief takes precedence** when both exist.

### Inputs

- Feature description (from idea brief or command arguments).
- Codebase (read-only for context).
- `specs/SPECS.md` — project scope and existing commitments.
- `docs/DOCS.md` — project context and stated goals.
- `docs/.workflow/idea-brief.md` — if Discovery ran.

### Functionalities

| # | Functionality | Description |
|---|---|---|
| 1 | Directory safety | Creates `docs/.workflow/` if missing. |
| 2 | Proposal understanding (Phase 1) | Reads feature description, specs index, docs index. Identifies core claim: what problem, for whom. |
| 3 | Codebase context (Phase 2) | Globs project structure, Greps for related patterns, reads 2-5 relevant source files. Determines if similar functionality exists. |
| 4 | **D1: Necessity** scoring | Evaluates: real problem? Current? Who affected? Cost of inaction? Want vs. need? Score 1-5. |
| 5 | **D2: Impact** scoring | Evaluates: measurable outcome? Multiplier effect? Immediate vs. speculative? Score 1-5. |
| 6 | **D3: Complexity Cost** scoring | Evaluates: modules affected? New dependencies? Maintenance burden? Hidden complexities? Score 1-5 (inverted — 5 = low complexity). |
| 7 | **D4: Alternatives** scoring | Evaluates: existing functionality? External tools (via WebSearch)? 80/20 version? Manual workaround? Score 1-5 (inverted — 5 = no alternatives). |
| 8 | **D5: Alignment** scoring | Evaluates: project purpose fit? Architecture fit? Vision direction? User surprise? Score 1-5. |
| 9 | **D6: Risk** scoring | Evaluates: breakage risk? Security surface? Technical debt? Requirement clarity? Compliance? Score 1-5 (inverted — 5 = low risk). |
| 10 | **D7: Timing** scoring | Evaluates: prerequisites met? Dependency conflicts? Project stability? In-progress conflicts? Score 1-5. |
| 11 | FVS computation (Phase 4) | Weighted formula: FVS = ((D1+D2+D5)×2 + D3+D4+D6+D7) / 10. Necessity, Impact, Alignment weighted 2×. |
| 12 | Verdict determination | GO (FVS ≥ 4.0), CONDITIONAL (2.5-3.9), NO-GO (< 2.5). Override rules: any dim=1 → at most CONDITIONAL; D1=1 → NO-GO; D3=1 AND D2≤2 → NO-GO. |
| 13 | Recommendation (Phase 5) | GO: highlight risks for analyst. CONDITIONAL: list specific conditions. NO-GO: explain why, suggest alternatives. |
| 14 | User presentation (Phase 6) | Presents full report, states verdict, waits for user decision. Respects override. |
| 15 | WebSearch for alternatives | Searches for existing solutions, libraries, or patterns that address the same problem (used in D4). |
| 16 | Score inflation detection | Anti-pattern: if all 7 dimensions score 4+, re-examines for inflation before finalizing. |
| 17 | Context limit handling | Saves partial evaluation to `docs/.workflow/feature-evaluator-partial.md`. |

### Outputs

| File | Condition |
|------|-----------|
| `docs/.workflow/feature-evaluation.md` | Primary output |
| `docs/.workflow/feature-evaluator-partial.md` | If context limited |

### Fail-Safe Controls

1. Prerequisite gate — stops if no description, no source code, or missing idea brief (when expected).
2. Directory safety.
3. Advisory-only — verdict is never a veto; user always decides.
4. Override documentation — if user overrides NO-GO, it's recorded in the report.
5. Score evidence requirement — every dimension score must cite specific observations.
6. FVS formula is deterministic — verdict derived mechanically from scores, not gut feeling.
7. Override rules prevent single-dimension masking — D1=1 forces NO-GO regardless of FVS.
8. Score inflation detection — all-high scores trigger re-examination.
9. Context limit save.
10. Does not gate bug fixes or improvements — only new features.
11. Does not re-do Discovery's analysis — builds on idea brief findings.

---

## Universal Controls

Controls present across all (or most) agents:

| Control | Description | Present In |
|---------|-------------|------------|
| Directory Safety | Verifies target directories exist before writing; creates if missing | All agents |
| Context Limit Handling | Saves progress/partial findings to `docs/.workflow/` when approaching limits | All agents |
| Scope Enforcement | Respects `--scope` parameter to limit work area | All agents |
| Incremental Saving | Saves findings/progress after each module or domain | Most agents |
| Prerequisite Gates | Verifies upstream input exists before starting | All except Discovery |
| Grep Before Read | Searches for patterns before loading full files | Most agents |
| Never Reads Entire Codebase | All agents scope their reads to relevant area | All agents |
| Source of Truth Hierarchy | Codebase > specs > docs | All agents with code access |

---

## Specs/Docs Sync Responsibility Chain

Every agent that touches or validates code now maintains documentation currency:

| Agent | Responsibility | Details |
|-------|---------------|---------|
| **Analyst** | Catches drift early | Checks/fixes stale specs before writing new requirements |
| **Architect** | Primary owner | Creates/updates specs and docs, maintains master indexes |
| **Test Writer** | Flags inconsistencies | Reports when tests reveal undocumented behavior or architect/code contradictions |
| **Developer** | Fixes drift during implementation | Updates specs/docs when code changes documented behavior |
| **QA** | Catches remaining drift | Verifies specs/docs against actual behavior, reports in QA report |
| **Reviewer** | Final enforcement | 5-point checklist auditing specs/docs accuracy |
