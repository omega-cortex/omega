# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About This Repository

This is a **multi-agent workflow toolkit** for Claude Code — not an application. It consists of agent definitions (`.claude/agents/*.md`), slash commands (`.claude/commands/*.md`), a setup script, and the CLAUDE.md rules file. All of these are designed to be **copied into target projects** to enable structured TDD workflows.

### Development

There is no build system, test suite, or runtime. To test changes:
1. Edit agent/command files in this repo
2. Copy them to a target project: `bash scripts/setup.sh` (run from the target project directory)
3. Run the workflow commands in the target project via Claude Code

The setup script (`scripts/setup.sh`) copies agents, commands, and CLAUDE.md into the current directory. It creates `specs/` and `docs/` scaffolding if missing and never overwrites existing files (except agents and commands which are always overwritten).

### Architecture

**Agents** (`.claude/agents/`) — subagent definitions with YAML frontmatter (`name`, `description`, `tools`, `model`):
- `analyst.md` (claude-opus-4-6) — full BA: requirements with acceptance criteria, MoSCoW priorities, traceability matrix, impact analysis. Outputs `specs/[domain]-requirements.md`
- `architect.md` (claude-opus-4-6) — designs architecture with failure modes, security, performance budgets. Maintains specs/ and docs/. Outputs `specs/[domain]-architecture.md`
- `test-writer.md` (claude-opus-4-6) — writes failing tests before code (TDD red phase), priority-driven (Must first), references requirement IDs for traceability
- `developer.md` (claude-opus-4-6) — implements minimum code to pass tests, commits per module
- `qa.md` (claude-opus-4-6) — end-to-end validation, acceptance criteria verification, traceability matrix completion, exploratory testing
- `reviewer.md` (claude-opus-4-6, read-only) — audits for bugs/security/performance/drift, outputs review reports
- `functionality-analyst.md` (claude-opus-4-6, read-only) — maps what the codebase does, outputs structured functionality inventory

**Commands** (`.claude/commands/`) — slash command orchestrators that chain agents in sequence:
- `workflow-new.md` — full chain (all 6 agents) for greenfield projects
- `workflow-feature.md` — full chain for existing projects (context-aware)
- `workflow-improve.md` — no architect; analyst → test-writer → developer → QA → reviewer
- `workflow-bugfix.md` — reduced chain with bug reproduction test + QA validation
- `workflow-audit.md` — reviewer only (read-only analysis)
- `workflow-docs.md` — architect only (documentation generation)
- `workflow-sync.md` — architect only (drift detection and fix)
- `workflow-functionalities.md` — functionality-analyst only (codebase functionality inventory)

All commands accept `--scope="area"` to limit context window usage. Agent model assignments are set in the YAML frontmatter.

---

# Workflow Rules (copied to target projects)

Everything below this line defines the workflow behavior when this CLAUDE.md is installed in a target project.

---

# Claude Code Quality Workflow

## Philosophy
This project uses a multi-agent workflow designed to produce the highest quality code possible.
Each agent has a specific role and the code passes through multiple validation layers before being considered complete.

## Source of Truth Hierarchy
1. **Codebase** — the ultimate source of truth. Always trust code over documentation.
2. **specs/** — technical specifications per domain. `specs/SPECS.md` is the master index linking to per-domain spec files (e.g. `specs/auth.md`, `specs/memory-store.md`).
3. **docs/** — user-facing and developer documentation. `docs/DOCS.md` is the master index linking to topic guides.

When specs or docs conflict with the codebase, the codebase wins. Agents must flag the discrepancy and update specs/docs accordingly.

## Main Workflow

```
Idea → Analyst (BA: requirements, acceptance criteria, MoSCoW priorities, traceability)
     → Architect (design with failure modes, security, performance budgets)
     → Test Writer (TDD by priority: Must first, then Should, then Could)
     → Developer (implements module by module)
     → Compiler (automatic validation)
     → QA (end-to-end validation, acceptance criteria verification, exploratory testing)
     → Reviewer (audits code, verifies specs/docs accuracy)
     → Git (automatic versioning)
```

## Traceability Chain
Every requirement flows through the entire pipeline via unique IDs:
```
Analyst assigns REQ-XXX-001 → Architect maps to module → Test Writer writes TEST-XXX-001 → Developer implements → QA verifies acceptance criteria → Reviewer audits completeness
```

## Global Rules

1. **NEVER write code without tests first** (strict TDD)
2. **NEVER assume** — if something is unclear, the analyst must ask
3. **Module by module** — do not implement everything at once
4. **Document before coding** — architecture is defined first
5. **Every assumption must be explicit** — technical + human-readable summary
6. **Codebase is king** — when in doubt, read the actual code
7. **Keep specs/ and docs/ in sync** — every code change must update relevant specs and docs
8. **Every requirement has acceptance criteria** — "it should work" is not acceptable
9. **Every requirement has a priority** — Must/Should/Could/Won't (MoSCoW)
10. **Every requirement is traceable** — from ID through tests to implementation

## Context Window Management

### Critical Rules
- **NEVER read the entire codebase at once** — always scope to the relevant area
- **Read indexes first** — start with `specs/SPECS.md` or `docs/DOCS.md` to identify which files matter
- **Scope narrowing** — all commands accept an optional scope parameter to limit the area of work
- **Chunking** — for large operations (audit, sync, docs), work one milestone/domain at a time

### Agent Scoping Strategy
1. Read the master index (`specs/SPECS.md`) to understand the project layout
2. Identify which domains/milestones are relevant to the task
3. Read ONLY the relevant spec files and code files
4. If you feel context getting heavy, stop and summarize what you've learned so far before continuing

### Scope Parameter
All workflow commands accept an optional scope to limit context usage:
```
/workflow:feature "add retry logic" --scope="omega-providers"
/workflow:audit --scope="milestone 3: omega-core"
/workflow:sync --scope="omega-memory"
/workflow:bugfix "scheduler crash" --scope="backend/src/gateway/scheduler.rs"
```

When no scope is provided, the analyst determines the minimal scope needed based on the task description.

### When Approaching Context Limits
If an agent notices it's consuming too much context:
1. **Summarize** what has been learned so far into a temporary file at `docs/.workflow/[agent]-[task]-summary.md`
2. **Delegate** remaining work by spawning a continuation subagent that reads the summary
3. **Never silently degrade** — if you can't do a thorough job, say so and suggest splitting the task

## Project Layout

```
root-project/
├── backend/              ← Backend source code (Rust or preferred language)
├── frontend/             ← Frontend source code (if applicable)
├── specs/                ← Technical specifications (at project root)
├── docs/                 ← Documentation (at project root)
├── CLAUDE.md             ← Workflow rules
└── .claude/              ← Agents and commands
```

Code lives in `backend/` (and optionally `frontend/`). Specs and docs remain at the project root.
Agents must be aware of this structure when scoping reads and writes.

## Documentation Structure

### specs/ (technical specifications)
```
specs/
├── SPECS.md              ← Master index (links to all domain specs)
├── core-config.md        ← Per-domain spec files
├── core-context.md
├── memory-store.md
├── channels-telegram.md
└── ...
```
- One spec file per domain/module/file
- SPECS.md must be updated when new specs are added
- Specs describe WHAT the code does technically

### docs/ (documentation)
```
docs/
├── DOCS.md               ← Master index (links to all doc files)
├── quickstart.md
├── architecture.md
├── configuration.md
└── ...
```
- Topic-oriented guides and references
- DOCS.md must be updated when new docs are added
- Docs describe HOW to use/understand the system

## Usage Modes

### New project from scratch
```
/workflow:new "description of the idea"
```
Full chain: analyst → architect → test-writer → developer → QA → reviewer

### Add feature to existing project
```
/workflow:feature "description of the feature" [--scope="area"]
```
Full chain: analyst → architect → test-writer → developer → QA → reviewer. The analyst reads the codebase + specs first.

### Improve existing code
```
/workflow:improve "description of the improvement" [--scope="area"]
```
Reduced chain (no architect): analyst → test-writer (regression) → developer (refactor) → QA → reviewer

### Fix a bug
```
/workflow:bugfix "description of the bug" [--scope="file or module"]
```
Reduced chain: analyst → test-writer (reproduces the bug) → developer → QA → reviewer

### Audit existing code
```
/workflow:audit [--scope="milestone or module"]
```
Reviewer only: looks for vulnerabilities, technical debt, performance issues, and specs/docs drift.

### Document existing project
```
/workflow:docs [--scope="milestone or module"]
```
Architect only: reads the codebase, generates/updates specs/ and docs/.

### Sync specs and docs with codebase
```
/workflow:sync [--scope="milestone or module"]
```
Architect only: reads the codebase, compares against specs/ and docs/, flags drift, updates outdated files.

### Map codebase functionalities
```
/workflow:functionalities [--scope="module or area"]
```
Functionality-analyst only: reads the codebase and produces a structured inventory of all functionalities (endpoints, services, models, handlers, etc.).

## Conventions
- Preferred language: Rust (or whatever the user defines)
- Tests: alongside code or in `backend/tests/` (or `frontend/tests/`) folder
- Commits: conventional (feat:, fix:, docs:, refactor:, test:)
- Branches: feature/, bugfix/, hotfix/
