# Feature Evaluation: Integrate Role Creator Agent into Brain Setup Pipeline

## Feature Description
Delegate ROLE.md creation during `/setup` from the Brain agent to a dedicated "Role Creator" agent, using a second sequential `run_build_phase()` call after Brain finishes. Brain would handle questioning, proposal, confirmation, HEARTBEAT.md, schedule markers, and PROJECT_ACTIVATE, but would NOT write ROLE.md. A new `omega-role-creator` agent (adapted from `.claude/agents/role-creator.md`) would then receive the accumulated context and write the ROLE.md using a "14-item anatomy checklist and domain expertise."

## Evaluation Summary

| Dimension | Score (1-5) | Assessment |
|-----------|-------------|------------|
| D1: Necessity | 2 | No evidence that current Brain ROLE.md quality is inadequate. The Brain agent already has detailed quality requirements, 2 examples, and 80-200 line targets at `topologies/development/agents/omega-brain.md` lines 90-177 |
| D2: Impact | 2 | Marginal quality improvement at best. ROLE.md quality is subjective and already addressed in the Brain prompt. No user complaint or measured deficiency cited |
| D3: Complexity Cost | 3 | Uses proven `write_single()` + `run_build_phase()` pattern, but adds a second Opus subprocess call (~30 turns), a new agent definition, a new `include_str!()` const, and context-passing plumbing between two sequential phases |
| D4: Alternatives | 2 | The simplest alternative -- improving the Brain agent's ROLE.md instructions directly -- delivers the same outcome with zero code changes. Just edit `omega-brain.md` |
| D5: Alignment | 3 | The project's first principle is "less is more." Adding a second agent invocation for a task the Brain already handles contradicts this. However, the pattern itself (multi-agent pipeline) is aligned with the build system architecture |
| D6: Risk | 3 | Context loss between Brain and Role Creator is the primary risk. Brain accumulates multi-round context; passing it faithfully to a second agent is non-trivial. Also doubles the Opus API cost per setup and doubles latency for the user |
| D7: Timing | 4 | No conflicting in-progress work. The Brain feature is recently shipped and stable. Prerequisites (write_single, run_build_phase) exist |

**Feature Viability Score: 2.6 / 5.0**

Calculation: ((2 + 2 + 3) x 2 + (3 + 2 + 3 + 4)) / 10 = (14 + 12) / 10 = 2.6

## Verdict: CONDITIONAL

The feature has a working technical approach but solves a problem that has not been demonstrated to exist. The existing Role Creator agent (`.claude/agents/role-creator.md`) is designed to create **agent definitions** (`.claude/agents/*.md` files with YAML frontmatter, personality sections, process phases, anti-patterns, etc.), not **project ROLE.md files** (`~/.omega/projects/<name>/ROLE.md` with domain expertise, operational rules, knowledge areas). These are fundamentally different artifacts. The "14-item anatomy checklist" (identity, personality, prerequisite gate, directory safety, source of truth, context management, process, output format, rules, anti-patterns, failure handling, integration, scope handling, context limits) is designed for AI agent behavior specifications, not for domain expertise documents. Adapting it would require substantial rewriting, at which point you are essentially writing a new agent from scratch, not reusing the existing role-creator.

## Detailed Analysis

### What Problem Does This Solve?
The proposal implies that Brain-generated ROLE.md files are not high enough quality and that a dedicated specialist agent would produce better results. However, no specific quality deficiency is cited. The Brain agent at `topologies/development/agents/omega-brain.md` already contains:
- Detailed ROLE.md quality requirements (lines 90-98): domain-specific, structured, actionable, parseable, 80-200 lines
- Two complete example ROLE.md files (lines 100-177): a trading agent and a Lisbon real estate agent
- Structured sections to include: Identity, Core Responsibilities, Operational Rules, Knowledge Areas, Communication Style, Safety/Constraints (line 95)

The architecture spec at `specs/omega-brain-architecture.md` already acknowledges that "Brain produces shallow ROLE.md" is a known risk but notes "No automated detection (quality is subjective)" and provides the mitigation "User can edit ROLE.md manually."

### What Already Exists?
1. **Brain agent** (`topologies/development/agents/omega-brain.md`) -- already writes ROLE.md with quality guidelines and examples
2. **Role Creator** (`.claude/agents/role-creator.md`) -- creates AI **agent definitions**, NOT project ROLE.md files. Its 14-item checklist is for agent behavior specs (prerequisite gates, anti-patterns, failure handling), not domain expertise documents
3. **OMEGA Topology Architect** (`.claude/agents/omega-topology-architect.md`) -- a dev-time agent that designs OMEGA configurations including ROLE.md, but is not integrated into the runtime binary
4. **`AgentFilesGuard::write_single()`** (line 133 of `builds_agents.rs`) -- proven pattern for single-agent lifecycle
5. **`run_build_phase()`** (line 432 of `builds.rs`) -- proven subprocess invocation with retry

### Complexity Assessment
**What needs to change:**
- New file: `topologies/development/agents/omega-role-creator.md` -- a new agent definition adapted from either `role-creator.md` or `omega-topology-architect.md` (not a simple copy; requires substantial rewriting for the ROLE.md domain)
- Modified: `builds_agents.rs` -- add `ROLE_CREATOR_AGENT` const via `include_str!()`
- Modified: `setup.rs` `execute_setup()` -- add second `write_single()` + `run_build_phase()` call after Brain finishes
- Modified: `setup_response.rs` `handle_setup_confirmation()` -- adjust execution flow to split Brain output from Role Creator output
- Modified: `omega-brain.md` -- remove ROLE.md writing responsibility, adjust execution mode instructions

**Maintenance cost:** Low-medium. One additional agent to maintain in `topologies/development/agents/`. One additional `include_str!()` const. The sequential two-phase pattern in `execute_setup()` adds complexity to an already multi-round session state machine. When the Brain prompt changes, the Role Creator context expectations may also need updating.

**Runtime cost:** Each `/setup` completion currently makes 1 Opus subprocess call for execution. This would double to 2 Opus calls. At ~30 max_turns each, this roughly doubles the token cost and latency of setup execution. For a personal AI agent used by one person, this may be acceptable; for scale, it would not be.

### Risk Assessment
1. **Context loss between phases** -- Brain accumulates multi-round context (user description, Q&A, proposal, confirmation). Passing this faithfully to a second agent requires careful prompt construction. Information that Brain understood implicitly may not transfer
2. **Coordination complexity** -- Brain currently writes ROLE.md, HEARTBEAT.md, and emits markers atomically. Splitting ROLE.md to a second agent means Brain must still create the project directory but not write ROLE.md. If the Role Creator fails, you have a project with HEARTBEAT.md and schedules but no ROLE.md -- a broken state
3. **Doubled latency** -- User waits for two sequential Opus subprocess calls instead of one. The `/setup` flow already involves multiple rounds of Brain invocations
4. **Agent mismatch** -- The existing `role-creator.md` creates agent definitions, not ROLE.md files. A new agent must be written from scratch for this purpose

## Conditions
- [ ] **Demonstrate the quality gap**: Provide 3+ concrete examples of Brain-generated ROLE.md files that are inadequate, with specific deficiencies identified. Without evidence of a problem, this is a solution in search of one
- [ ] **Write a purpose-built agent, not adapt role-creator.md**: The existing role-creator's 14-item checklist is irrelevant to project ROLE.md files. A new `omega-role-creator.md` must be designed specifically for domain expertise documents
- [ ] **Handle partial failure**: Define what happens when Brain succeeds but Role Creator fails. The project directory will exist with HEARTBEAT.md, schedules, and PROJECT_ACTIVATE but no ROLE.md
- [ ] **Evaluate simpler alternative first**: Try improving the Brain agent's ROLE.md instructions (add more examples, add a structural checklist specific to domain expertise documents) and measure whether quality improves before committing to a second agent

## Alternatives Considered
- **Improve Brain's ROLE.md instructions directly** (RECOMMENDED): Edit `topologies/development/agents/omega-brain.md` to include better structural guidance, more examples, and a domain-expertise-specific quality checklist. Zero code changes. Zero runtime cost increase. Delivers 80%+ of the value. If ROLE.md quality is genuinely the concern, this should be tried first
- **Use the Topology Architect as inspiration**: The `omega-topology-architect.md` agent already has strong ROLE.md guidance ("ROLE.md is the centerpiece -- spend the most effort on writing a high-quality ROLE.md that genuinely captures domain expertise, not a generic placeholder"). Its ROLE.md writing approach could be ported into the Brain agent's prompt
- **Post-setup ROLE.md refinement**: Instead of splitting the pipeline, add a separate `/refine` command that runs the topology architect or a dedicated agent to improve an existing ROLE.md after initial setup. This decouples the concern and lets users opt into quality improvement without adding latency to every setup

## Recommendation
Do NOT build this feature in its current form. The proposal conflates two different artifact types (agent definitions vs. project ROLE.md files) and introduces runtime complexity (second Opus call, context passing, partial failure handling) without evidence that the current approach is insufficient.

**Instead, try this first**: Edit `topologies/development/agents/omega-brain.md` to strengthen the ROLE.md generation section. Borrow the ROLE.md writing guidance from `omega-topology-architect.md`. Add a 6-item domain expertise checklist (identity, responsibilities, operational rules, knowledge areas, communication style, constraints) directly to the Brain prompt. Test with 5 different `/setup` domains. If ROLE.md quality is still inadequate after prompt improvements, revisit the two-agent approach with concrete evidence.

## User Decision
[Awaiting user response: PROCEED / ABORT / MODIFY]
