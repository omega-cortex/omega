# Feature Evaluation: OMEGA Brain

## Feature Description

A single agent -- the OMEGA Brain -- that understands user business goals and configures OMEGA by composing existing primitives. When a user says "I'm a realtor, help me manage my business," the Brain asks 2-4 targeted questions, proposes a setup, and upon approval creates:

- `~/.omega/projects/<name>/ROLE.md` (domain expertise)
- `~/.omega/projects/<name>/HEARTBEAT.md` (monitoring checklist)
- Scheduled actions via `SCHEDULE_ACTION:` markers
- Project activation via `PROJECT_ACTIVATE:` marker

MVP scope: Trigger 1 only (new business goal -- create project setup). One agent, one conversation, one call. No new crates, no new database tables, no new marker types.

Source: `docs/.workflow/idea-brief.md`

## Evaluation Summary

| Dimension | Score (1-5) | Assessment |
|-----------|-------------|------------|
| D1: Necessity | 4 | Today OMEGA's project primitives require technical knowledge of file conventions. A non-technical user cannot self-onboard. The system prompt already tells OMEGA to "suggest creating a project" but OMEGA cannot actually create one -- it can only describe the manual steps. |
| D2: Impact | 4 | Transforms OMEGA from "AI chatbot that knows things" to "AI that configures itself for your domain." A 5-minute onboarding that produces a working domain expert is the kind of demo that makes Anthropic fall in love. Multiplier effect: every Brain-created project generates ongoing heartbeat, scheduling, and learning value. |
| D3: Complexity Cost | 4 | The Brain is a single agent call (like build-discovery) using `run_build_phase()`. No new crates, no new DB tables, no new markers. Changes needed: (1) Brain agent .md file, (2) keyword detection or trigger in `keywords.rs`/`pipeline.rs`, (3) file-writing logic for ROLE.md and HEARTBEAT.md (the agent does this via Claude Code tools). Estimated: ~200-400 new lines across 2-3 files. |
| D4: Alternatives | 3 | OMEGA already tells users to create projects manually, and Claude Code can technically write files when asked. But the current path requires the user to know OMEGA's file conventions, and there is no structured conversation flow. No external tool solves "configure this specific agent infrastructure." A simpler alternative exists: enhanced system prompt instructions that teach OMEGA to write ROLE.md directly -- but this lacks the structured question/approval flow that ensures quality. |
| D5: Alignment | 5 | This is the most natural next step for "personal AI agent infrastructure." The project's first principle is "less is more" -- and the Brain adds value by composing existing primitives rather than creating new ones. The mission says "Anthropic falls in love with our Agent" -- a self-configuring agent that asks you about your business and sets itself up is exactly the kind of simplicity-through-intelligence that showcases Omega. |
| D6: Risk | 4 | Low breakage risk: the Brain writes new files to `~/.omega/projects/` and emits existing markers. Existing projects, skills, and schedules are unaffected. The main risk is ROLE.md quality -- if the Brain writes mediocre instructions, OMEGA becomes a mediocre domain assistant. Secondary risk: keyword trigger ambiguity ("I want to trade" might trigger Brain when user just wants to chat about trading). Both are mitigatable: quality through prompt engineering, ambiguity through explicit confirmation gate. |
| D7: Timing | 4 | Prerequisites are met: projects system is stable (`omega-skills/src/projects.rs`), heartbeats support per-project files (`gateway/heartbeat.rs`), scheduled actions work (`gateway/process_markers.rs`), the agent execution pattern is proven (`run_build_phase()`). The topology extraction just landed (commit `86ad3c7`), meaning the agent-running infrastructure is at its most mature. No in-progress work conflicts. |

**Feature Viability Score: 4.0 / 5.0**

```
FVS = ((D1:4 + D2:4 + D5:5) x 2 + (D3:4 + D4:3 + D6:4 + D7:4)) / 10
    = (26 + 15) / 10
    = 4.1
```

**Corrected Feature Viability Score: 4.1 / 5.0**

## Verdict: GO

The OMEGA Brain is a high-value, low-complexity feature that composes existing primitives into a dramatically better user experience. It does not fight the architecture -- it leverages it. The MVP scope (Trigger 1 only) is tight enough to deliver real value without scope creep, and the implementation pattern is proven (single agent call via `run_build_phase()`). This is the rare feature where Necessity, Impact, and Alignment all score high while Complexity remains low.

## Detailed Analysis

### What Problem Does This Solve?

OMEGA has powerful primitives for domain expertise: projects with ROLE.md, per-project heartbeats, scheduled actions, reward-based learning. But today, using them requires knowing OMEGA's file conventions. The system prompt in `pipeline.rs` (lines 858-865) already instructs OMEGA to "suggest creating a project (~/.omega/projects/<name>/ROLE.md)" -- but "suggest" means telling the user to manually create directories and write markdown files. A realtor should not need to know what a ROLE.md is.

The gap is real and current: OMEGA already has the intelligence (Claude) and the infrastructure (projects, heartbeats, schedules) but lacks the bridge between a non-technical user's intent and the technical setup. The Brain IS that bridge.

### What Already Exists?

The codebase has every building block the Brain needs:

1. **Project system** (`/Users/isudoajl/ownCloud/Projects/omega/backend/crates/omega-skills/src/projects.rs`): `load_projects()` scans `~/.omega/projects/*/ROLE.md`, parses TOML/YAML frontmatter for skills, returns `Project` structs. The Brain writes files that this loader already knows how to read.

2. **Per-project heartbeats** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/heartbeat.rs` lines 267-275): The heartbeat loop already checks for `~/.omega/projects/<name>/HEARTBEAT.md` and runs project-specific heartbeats. The Brain writes HEARTBEAT.md files that this system already consumes.

3. **Marker processing** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/process_markers.rs`): `SCHEDULE_ACTION:`, `SCHEDULE:`, and `PROJECT_ACTIVATE:` markers are already fully implemented. The Brain's agent output includes these markers, and the existing `process_markers()` handles them.

4. **Agent execution pattern** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/builds.rs` lines 432-458): `run_build_phase()` takes an agent name, prompt, model, and max_turns -- exactly what the Brain needs. The build-discovery agent already demonstrates a single-call agent that asks questions and produces structured output.

5. **Keyword detection** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/keywords.rs`): The `kw_match()` function and keyword constant pattern (with 8-language support) is the established way to trigger special flows.

6. **Confirmation gate pattern** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/pipeline.rs` lines 421-484): The build confirmation flow (store pending request with timestamp, check TTL, confirm/cancel) is directly reusable for Brain approval.

7. **System prompt hint** (`/Users/isudoajl/ownCloud/Projects/omega/backend/src/gateway/pipeline.rs` lines 851-865): OMEGA already tells users it can help with projects. The Brain makes this promise real.

### Complexity Assessment

The Brain follows the exact same pattern as build-discovery:

1. **New agent file** (~100-200 lines): A `brain-setup.md` agent definition, either bundled via `include_str!()` or placed in a new topology directory. The agent's prompt teaches it to ask domain questions, generate ROLE.md content, generate HEARTBEAT.md content, and emit schedule markers. This is the single most important piece -- the quality of this prompt determines everything.

2. **Keyword detection** (~20-30 lines in `keywords.rs`): A `BRAIN_KW` constant with triggers like "set me up", "configure omega", "I'm a [profession]", "help me manage my [business]" in 8 languages, plus a `brain_confirm_message()` function.

3. **Pipeline integration** (~100-150 lines in `pipeline.rs`): Following the build keyword pattern: detect Brain trigger -> run Brain agent -> if agent proposes setup -> show proposal to user -> on confirmation, Brain writes files and emits markers. The Brain agent itself writes ROLE.md and HEARTBEAT.md via Claude Code's Write tool; the markers are processed by existing `process_markers()`.

4. **No new crates, no new database tables, no new marker types.** The Brain composes what exists.

**Estimated total: ~200-400 new lines across 2-3 files.** For context, the build-discovery feature added roughly 300 lines across similar files.

**Ongoing maintenance burden: Low.** The Brain is a single agent with a prompt file. Improving Brain quality means editing the prompt, not modifying Rust code. The only Rust maintenance is the keyword list and the pipeline trigger path -- both are thin and well-tested patterns.

### Risk Assessment

1. **ROLE.md quality** (medium risk): The entire value of the Brain depends on writing excellent ROLE.md files. If the Brain produces generic, shallow instructions, the user's OMEGA becomes a mediocre assistant. Mitigation: the Brain's agent prompt can include examples of excellent ROLE.md files, and the user can edit after creation.

2. **Trigger ambiguity** (low risk): "I'm a realtor" might trigger the Brain when the user just wants to chat about real estate. Mitigation: require explicit confirmation (same pattern as builds), or use an explicit command like `/setup`.

3. **Existing project collision** (low risk): The Brain might try to create a project that already exists. Mitigation: read existing projects first (the idea brief already specifies this), offer to update rather than overwrite.

4. **No breakage of existing functionality**: The Brain writes to `~/.omega/projects/` (new directories only) and emits existing markers. It cannot break existing projects, skills, schedules, or the build pipeline. The pipeline integration adds a new code path but does not modify existing ones.

## Conditions

None -- feature approved for pipeline entry.

One advisory note for the Analyst: the open questions from the idea brief (where the Brain agent lives, how triggers work, overwrite safety) should be resolved during requirements. The evaluation finds these are design decisions, not blockers.

## Alternatives Considered

- **Enhanced system prompt only**: Teach OMEGA via system prompt to create project files when users describe domains. Zero code change, but no structured question flow, no approval gate, no guaranteed file format. The AI might write a ROLE.md in the wrong location, with the wrong structure, or skip the heartbeat entirely. Verdict: too fragile for a core onboarding experience.

- **Interactive CLI wizard**: A `omega setup` command that walks through project creation interactively in the terminal. Works for technical users but not for Telegram/WhatsApp users -- OMEGA's primary channels. Verdict: wrong interface.

- **Skill-based approach**: Create a "brain" skill with an MCP server that writes files. This works architecturally but adds unnecessary indirection -- a skill is for external tool integration, not for OMEGA self-configuration. Verdict: pattern mismatch.

- **Manual project creation (status quo)**: Users create `~/.omega/projects/<name>/ROLE.md` by hand. Works for the developer who built OMEGA, does not work for anyone else. Verdict: not scalable to the "Anthropic buys us" mission.

## Recommendation

Build it. The OMEGA Brain is that rare feature where the value is obvious, the complexity is genuinely low (it composes existing primitives), and the alignment with the project's mission is near-perfect.

Key guidance for downstream agents:
- The Brain agent prompt is the highest-leverage artifact. Spend disproportionate effort on it.
- Follow the build-discovery pattern exactly: keyword detection -> agent call -> structured output -> confirmation gate -> execution.
- MVP is Trigger 1 only (new project creation). Do not scope creep into Trigger 2 (restructure) or Trigger 3 (learning threshold).
- The 8-language requirement applies to Brain keywords and confirmation messages.
- Test with at least 3 domain scenarios: a profession (realtor), a hobby (fitness), and an existing-project case (trading, where the project already exists).

## User Decision

[Awaiting user response: PROCEED / ABORT / MODIFY]
