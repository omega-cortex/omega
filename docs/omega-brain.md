# OMEGA Brain

> Self-configuration agent that transforms business descriptions into fully configured OMEGA projects.

## Overview

The OMEGA Brain lets non-technical users describe their business goal and have OMEGA configure itself as a domain expert. Users type `/setup` followed by a description, and the Brain handles everything: asking clarifying questions, proposing a project setup, and creating all necessary files.

## Usage

```
/setup I'm a realtor in Lisbon specializing in residential properties
```

The Brain will:
1. Analyze the description and ask 2-4 clarifying questions (if needed)
2. Propose a project setup (ROLE.md, HEARTBEAT.md, scheduled tasks)
3. Wait for user approval before creating anything
4. Execute the setup, creating all files and activating the project

### Empty Command

```
/setup
```

Shows help text explaining the command usage.

## Session Flow

### Multi-Round Questioning (up to 3 rounds)

1. **Round 1**: Brain receives the user's description. If specific enough, it produces a proposal directly. If vague, it asks 2-4 clarifying questions.
2. **Round 2**: Brain receives the user's answers. May ask follow-up questions or produce a proposal.
3. **Round 3 (final)**: Brain MUST produce a proposal. No more questions allowed.

### Approval Gate

After the Brain produces a proposal, the user sees a preview and must confirm:
- **Confirm** (yes/si/sim/oui/ja/da): Creates all files and activates the project
- **Cancel** (no/cancel/cancelar/annuler/abbrechen/annullare/annuleren): Cancels setup
- **Any other text**: Treated as a modification request, sent back to the Brain for revision

### Session State

- State tracked via `pending_setup` fact in memory
- Format: `<timestamp>|<sender_id>|<round_or_phase>`
- Context file stored at `<data_dir>/setup/<sender_id>.md`
- 30-minute TTL: sessions expire after 30 minutes of inactivity
- One active session per user (concurrent sessions rejected)

## What Gets Created

### ROLE.md

Located at `~/.omega/projects/<name>/ROLE.md`. Contains domain-specific instructions that tell OMEGA how to behave as a domain expert. Includes operational rules, knowledge areas, personality adjustments, and safety constraints.

### HEARTBEAT.md

Located at `~/.omega/projects/<name>/HEARTBEAT.md`. Contains domain-specific monitoring items that the heartbeat loop checks periodically (e.g., "check new listings in Lisbon" for a realtor).

### Scheduled Tasks

The Brain emits `SCHEDULE_ACTION:` markers that create recurring tasks (e.g., daily market checks, weekly reports).

### Project Activation

The Brain emits a `PROJECT_ACTIVATE:` marker that activates the new project for the user.

## Architecture

### Files

| File | Purpose |
|------|---------|
| `backend/src/gateway/setup.rs` | Brain orchestrator: session lifecycle, state machine, provider invocation |
| `backend/src/gateway/pipeline.rs` | `/setup` command intercept, `pending_setup` session check |
| `backend/src/gateway/keywords.rs` | Setup i18n messages (8 languages), `SETUP_TTL_SECS` constant |
| `backend/src/gateway/builds_agents.rs` | `BRAIN_AGENT` const, `write_single()` method |
| `backend/src/commands/mod.rs` | `Command::Setup` variant registration |
| `topologies/development/agents/omega-brain.md` | Brain agent definition (YAML frontmatter + prompt) |
| `backend/crates/omega-core/src/config/mod.rs` | `pending_setup` in `SYSTEM_FACT_KEYS` |

### Brain Agent

The Brain agent (`omega-brain`) is bundled via `include_str!()` from `topologies/development/agents/omega-brain.md`. It runs as a Claude Code subprocess with:
- **Model**: Opus (complex reasoning)
- **Max turns**: 30
- **Tools**: Read, Write, Glob, Grep (no Bash, no Edit)
- **Permission mode**: bypassPermissions

### Collision Detection

Before starting, the Brain receives a list of existing projects (loaded via `load_projects()`). If a matching project exists, it proposes updating instead of duplicating.

## Supported Languages

All user-facing messages support 8 languages: English, Spanish, Portuguese, French, German, Italian, Dutch, Russian. Language is determined by the user's `preferred_language` fact.

## Limitations

- Only Trigger 1 (explicit `/setup` command) is implemented. Triggers 2 (restructure) and 3 (learning threshold) are deferred.
- Skills are suggested but not auto-installed (security implications with MCP servers).
- The Brain does not create build topologies.
