# Feature Evaluation: Pluggable Agent Topology System

## Note on Idea Brief

The existing `docs/.workflow/idea-brief.md` describes the **Inbound Webhook** feature, not the Pluggable Agent Topology System. This evaluation is based on the feature description provided in the command arguments.

## Feature Description

Replace the hardcoded 7-phase sequential agent pipeline in `builds.rs` with a pluggable topology system where:
1. The gateway endpoint connecting to Claude Code has an adaptable "cap" that accommodates different execution patterns (pipeline, orchestration, DAG, parallel, etc.)
2. The current development pipeline (analyst->architect->test-writer->developer->QA->reviewer->delivery) is preserved as a default built-in topology
3. Users can create custom topologies for their own needs (e.g., trading system with Market Data Agent -> Analysis Agent -> Strategy Agent -> Execution Agent, etc.)
4. An Architecture Assistant agent helps users design custom agent topologies on the fly

## Evaluation Summary

| Dimension | Score (1-5) | Assessment |
|-----------|-------------|------------|
| D1: Necessity | 3 | The current fixed pipeline works and serves Omega's only build use case. No users are blocked. But the vision of Omega as "personal AI agent infrastructure" implies multi-topology support eventually. |
| D2: Impact | 4 | Transforms Omega from a single-purpose build tool into a general-purpose agent orchestration platform. This directly serves the "Anthropic falls in love" mission. |
| D3: Complexity Cost | 1 | Massive cross-cutting change: refactors 4 tightly coupled gateway files (~1,400 lines), requires new topology definition format, parser, validator, runtime executor with DAG/parallel support, security model for user-defined agents, and an Architecture Assistant agent. Ongoing maintenance burden is high. |
| D4: Alternatives | 3 | Simpler config-driven sequential phase lists could deliver 70% of the value. LangGraph, CrewAI, and Microsoft Agent Framework already solve general-purpose orchestration. But none integrate with Omega's channel+provider architecture natively. |
| D5: Alignment | 4 | Strongly aligned with "personal AI agent infrastructure" and the Lego-block philosophy. However, "less is more" and "the best engine part is the one you can remove" caution against premature abstraction. |
| D6: Risk | 2 | High risk of destabilizing the working build pipeline during refactor. User-defined agent execution creates a new security surface (arbitrary prompts, tool access, sandboxing). The feature's scope is ambiguous enough that rework is likely. |
| D7: Timing | 2 | The build pipeline was just strengthened (safety controls, QA loops, reviewer loops, discovery phase) in the last 3 commits. The foundation has not yet proven itself in production. Abstracting now is premature. |

**Feature Viability Score: 2.8 / 5.0**

```
FVS = (D1:3 + D2:4 + D5:4) x 2 + (D3:1 + D4:3 + D6:2 + D7:2) = 22 + 8 = 30 / 10 = 3.0
```

Wait -- recalculating properly:
```
FVS = ((3 + 4 + 4) x 2 + (1 + 3 + 2 + 2)) / 10
    = (22 + 8) / 10
    = 3.0
```

**Feature Viability Score: 3.0 / 5.0**

## Verdict: CONDITIONAL

**Override applied: D3 (Complexity) scores 1 -- verdict capped at CONDITIONAL regardless of FVS.**

The feature has genuine strategic value for Omega's identity as "personal AI agent infrastructure." However, the complexity cost as described is extreme, the timing is premature (the pipeline it would abstract was literally built in the last week), and the scope is too ambitious for a single feature. This needs to be decomposed and sequenced.

## Detailed Analysis

### What Problem Does This Solve?

The current build pipeline in `gateway/builds.rs` is a hardcoded 7-phase sequential chain. If a user wants a different agent topology (e.g., trading, research, content creation), they cannot create one -- they can only use the built-in development pipeline triggered by build keywords. This limits Omega to being a build tool rather than a general agent orchestration platform.

However, this is a **forward-looking** problem, not a **current** problem. Today, the build pipeline is Omega's only multi-agent use case, and it works. No user is blocked. The problem being solved is "Omega should be more than a build tool" -- which is a vision, not a deficiency.

### What Already Exists?

The codebase has significant relevant infrastructure:

1. **`gateway/builds.rs`** (500 lines): Hardcoded 7-phase orchestrator with `run_build_phase()` that already provides a generic phase runner (agent name + prompt + model + max_turns). This is the natural extraction point.

2. **`gateway/builds_agents.rs`** (~1,800 lines): 8 agent definitions compiled into the binary as `const` strings. Uses an RAII `AgentFilesGuard` that writes `.claude/agents/*.md` files to disk and cleans up on drop. The guard already handles concurrent builds via a reference counter.

3. **`gateway/builds_loop.rs`** (345 lines): QA loop (3 iterations) and reviewer loop (2 iterations) with chain state persistence. These are hardcoded retry patterns.

4. **`gateway/builds_parse.rs`** (~1,100 lines): All parsing for build outputs -- `ProjectBrief`, `VerificationResult`, `ReviewResult`, `BuildSummary`, `ChainState`, plus localized message templates for 8 languages.

5. **`Context.agent_name`** in `omega-core/src/context.rs`: Already supports arbitrary agent names via `--agent` flag. This is provider-agnostic.

6. **Skills system** (`omega-skills/src/skills.rs`): File-based skill definitions at `~/.omega/skills/*/SKILL.md` with frontmatter parsing (TOML/YAML), trigger matching, and MCP server activation. This is the closest existing pattern for user-defined extensibility.

7. **Projects system** (`omega-skills/src/projects.rs`): User-defined project contexts at `~/.omega/projects/*/ROLE.md`. Scoped sessions, scoped learning.

The `run_build_phase()` method in `builds.rs` is already essentially a generic "run agent with prompt" function. The missing piece is not execution -- it is topology definition, sequencing logic, and user-facing creation tools.

### Complexity Assessment

This feature as described requires changes to:

- **`gateway/builds.rs`**: Complete refactor from hardcoded sequence to topology-driven execution. This file is at the 500-line limit already.
- **`gateway/builds_agents.rs`**: Agent definitions would need to become external/configurable rather than compiled constants.
- **`gateway/builds_loop.rs`**: Retry loops need to become topology-defined (which phases have loops, with what caps).
- **`gateway/builds_parse.rs`**: Parsing becomes topology-dependent (different topologies produce different outputs).
- **New: Topology definition format**: TOML/YAML schema for defining phase sequences, DAGs, parallel groups, retry policies, validation gates.
- **New: Topology loader**: Similar to `load_skills()` but for `~/.omega/topologies/*/TOPOLOGY.md` (or similar).
- **New: Topology runtime**: DAG executor with parallel support, conditional branching, error propagation.
- **New: Architecture Assistant agent**: An AI agent that helps design topologies -- this is itself a significant feature.
- **New: Security model**: User-defined agents with arbitrary prompts and tool access need sandboxing and validation.
- **`gateway/pipeline.rs`**: The build keyword detection and confirmation flow would need to route to different topologies.
- **`omega-core/src/config.rs`**: New configuration for topology defaults and settings.

Estimated scope: 2,000-3,500 new/modified lines across 10+ files in 3+ crates. This is comparable to the entire build pipeline feature that was built over multiple iterations.

**Ongoing maintenance burden**: Every new topology pattern (DAG, parallel, conditional) adds execution paths that must be tested, debugged, and maintained. User-defined topologies create a support surface where Omega must handle arbitrary configurations gracefully.

### Risk Assessment

1. **Pipeline destabilization**: The build pipeline was built and refined across 5+ commits with safety controls, discovery, QA loops, and audit findings fixes. Refactoring it now risks regressing all of that work. The `builds_loop.rs` QA and reviewer retry logic is tightly coupled to the current phase names.

2. **Security surface**: User-defined agents mean user-defined prompts with tool access. The current sandbox (`omega-sandbox`) blocks writes to OS dirs and `memory.db`, but user-defined agents could still access files in `~/.omega/workspace/`, exfiltrate data via tools, or consume excessive API credits.

3. **Scope creep**: "Pluggable topology" + "DAG support" + "parallel execution" + "Architecture Assistant agent" is at least 4 distinct features bundled as one. Each alone is non-trivial.

4. **Premature abstraction**: The current pipeline is the ONLY multi-agent topology in Omega. Abstracting from 1 example risks building the wrong abstraction. The classic software engineering mistake is generalizing before you have 3 concrete cases.

## Conditions

The following conditions must be met before this feature should proceed:

- [ ] **Decompose into phases**: Split this into at least 3 sequential features: (1) Extract topology abstraction from existing pipeline, (2) Add config-driven sequential topologies, (3) Add DAG/parallel support + Architecture Assistant. Each should be independently valuable.
- [ ] **Let the current pipeline bake**: The build pipeline should be used in production for at least 2-4 weeks before abstracting it. Real usage will reveal which parts of the design are stable and which need to flex.
- [ ] **Define 3 concrete topology examples**: Before building the abstraction, fully spec out 3 real topologies (development pipeline, trading system, and one more). This prevents premature/wrong abstraction.
- [ ] **Scope Phase 1 tightly**: Phase 1 should ONLY extract the existing pipeline into a config-driven format. No new topology types. No Architecture Assistant. No DAG execution. Just: "the same 7-phase pipeline, but defined in a TOPOLOGY file instead of hardcoded in Rust."
- [ ] **Security model for user-defined agents**: Before allowing user-defined agents, define the sandboxing and resource limits. This is a prerequisite, not an afterthought.

## Alternatives Considered

- **Config-driven sequential phase lists**: Define topologies as ordered lists of `(agent_name, prompt_template, retry_policy)` in TOML files at `~/.omega/topologies/*/TOPOLOGY.toml`. This delivers the "custom topologies" value without DAG complexity. Estimated effort: ~500-800 lines. **This is the recommended Phase 1.**

- **Use existing Skills+Projects pattern**: Instead of a topology system, each "topology" could be a Project with a ROLE.md that teaches the AI to self-orchestrate (call tools in sequence, manage state). This is zero code change but relies on the AI's ability to follow complex multi-step instructions reliably. Fragile but free.

- **External orchestration framework**: Use LangGraph, CrewAI, or a similar framework for topology execution, with Omega providing the channel/provider infrastructure. This avoids reinventing orchestration but adds a Python dependency and breaks the "monolithic Rust" architecture.

- **Do nothing**: Keep the hardcoded pipeline. Add new hardcoded pipelines when needed (one for trading, one for research, etc.). Simple, proven, but does not scale past 3-4 topologies.

## Recommendation

**Do not build this feature as described.** The scope is too large, the timing is premature, and bundling 4 features into one creates unnecessary risk.

Instead, pursue a phased approach:

1. **Now**: Let the current build pipeline prove itself in production for 2-4 weeks.
2. **Phase 1** (after bake period): Extract the existing pipeline into a config-driven sequential topology format. Keep the same 7 phases, same agents, same retry logic -- just externalize the definition from Rust constants into a `TOPOLOGY.toml` file.
3. **Phase 2** (after Phase 1 works): Allow users to create custom sequential topologies via the skills-like pattern (`~/.omega/topologies/*/TOPOLOGY.toml`).
4. **Phase 3** (if needed): Add DAG/parallel execution and the Architecture Assistant agent.

Each phase is independently valuable, independently testable, and can be evaluated on its own merits.

## User Decision

[Awaiting user response: PROCEED / ABORT / MODIFY]
