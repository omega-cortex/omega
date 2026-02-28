# Idea Brief: OMEGA Brain

## Problem

OMEGA is a personal AI agent infrastructure — NOT a chatbot. It has powerful primitives (projects, skills, scheduling, heartbeats, builds, lessons) that transform it into a domain expert for any business. But today, users must manually:

- Create project directories and write ROLE.md files
- Find and install skills
- Configure schedules and heartbeats
- Understand OMEGA's internal file conventions

This requires technical knowledge of OMEGA's internals. A realtor shouldn't need to know what a ROLE.md is. They should say "I'm a realtor, help me manage my business" and OMEGA should configure itself.

## Solution

A single agent — the **OMEGA Brain** — that understands user business goals and configures OMEGA using existing primitives. One agent, one conversation, done. Not a pipeline of agents. Not a new framework. Just a smart setup agent that composes what already exists.

## How It Works

```
User: "I'm a realtor, help me manage my business"
  |
  v
Brain activates (keyword detection or explicit invocation)
  |
  v
1. Asks 2-4 targeted questions about the domain
   ("What kind of properties? Residential, commercial? What tools do you use?")
  |
  v
2. Proposes a setup using existing primitives:
   - Project with ROLE.md (the intelligence)
   - Heartbeat items (what to monitor)
   - Scheduled actions (recurring tasks)
   - Skill suggestions (if applicable)
  |
  v
3. User approves or modifies the proposal
  |
  v
4. Brain creates everything:
   - Writes ~/.omega/projects/<name>/ROLE.md
   - Writes ~/.omega/projects/<name>/HEARTBEAT.md
   - Emits SCHEDULE_ACTION markers for recurring tasks
   - Emits PROJECT_ACTIVATE: <name>
  |
  v
5. Brain gets out of the way. OMEGA operates normally.
```

## What It Composes (existing primitives only)

| Brain Output | Existing Primitive | File/Marker | Codebase Location |
|---|---|---|---|
| Domain expert instructions | Project ROLE.md | `~/.omega/projects/<name>/ROLE.md` | `omega-skills/src/projects.rs` |
| Monitoring checklist | Project heartbeat | `~/.omega/projects/<name>/HEARTBEAT.md` | `gateway/heartbeat.rs` |
| Recurring tasks | Scheduled actions | `SCHEDULE_ACTION:` marker | `gateway/process_markers.rs` |
| One-time reminders | Scheduled reminders | `SCHEDULE:` marker | `gateway/process_markers.rs` |
| Project activation | Project switch | `PROJECT_ACTIVATE:` marker | `gateway/process_markers.rs` |
| Skill suggestions | Skill installation | Inform user, do not auto-install | `omega-skills/src/skills.rs` |

**No new infrastructure.** No new crates, no new database tables, no new marker types.

## What It Does NOT Do

1. **Does NOT create build pipelines or topologies** — topologies are for building software. The Brain sets up projects for daily operation.
2. **Does NOT install skills autonomously** — suggests skills, user installs. Skills have MCP servers with security implications.
3. **Does NOT run as a pipeline of agents** — one agent, one conversation. No analyst-architect-developer chain.
4. **Does NOT run always-on** — event-triggered. Dormant between activations.
5. **Does NOT design OMEGA's personality** — the Soul section in SYSTEM_PROMPT.md handles that. The Brain designs operational context.
6. **Does NOT act without approval** — always proposes, waits for human confirmation before creating anything.

## Activation Triggers

### Trigger 1: First Contact (MVP)
User describes a new business goal. Brain designs and creates the full setup.
- "I'm a realtor" -> creates realtor project
- "I want to trade stocks" -> creates trading project
- "Help me manage my restaurant" -> creates restaurant project

### Trigger 2: Explicit Restructure
User asks to modify an existing setup.
- "I also need to handle property management" -> updates realtor ROLE.md
- "Add a morning briefing to my trading setup" -> adds schedule
- "Restructure my project" -> reviews and proposes changes

### Trigger 3: Learning Threshold (future)
After N lessons accumulate in a project, the Brain reviews them and suggests ROLE.md adjustments. Runs via heartbeat, not on every message.

## Example Scenarios

### Realtor
**Input:** "I'm a realtor in Lisbon, mostly residential"

**Brain creates:**
```
~/.omega/projects/realtor/
  ROLE.md:
    - Property analysis and comparative market analysis
    - Client communication drafting
    - Showing preparation and property briefs
    - Follow-up tracking via scheduled tasks
    - Local knowledge: Lisbon neighborhoods, price ranges
    - Rules: never fabricate data, verify before claiming

  HEARTBEAT.md:
    - Check for overdue client follow-ups
    - Remind about today's showings
```

**Schedules:** Weekly listings review (Monday 8am), daily deal summary (9am)

### Stock Trader
**Input:** "I want to trade, I already have the ibkr-trader skill"

**Brain behavior:** Reads existing `~/.omega/projects/trader/ROLE.md`, notices it already exists with comprehensive content. Proposes only additions (heartbeat, schedules) rather than rewriting.

### Restaurant Manager
**Input:** "I manage a restaurant, help me organize"

**Brain creates:**
```
~/.omega/projects/restaurant/
  ROLE.md:
    - Inventory tracking and supplier communication
    - Staff scheduling assistance
    - Menu planning and cost analysis
    - Customer review monitoring

  HEARTBEAT.md:
    - Check for low inventory alerts
    - Remind about upcoming health inspections
```

**Schedules:** Daily reservations review (7am), weekly inventory check (Sunday), monthly cost analysis (1st)

## Success Criteria

1. A non-technical user can describe a business goal and have a working OMEGA project in under 5 minutes
2. The ROLE.md produced by the Brain makes OMEGA perform comparably to a manually-crafted expert ROLE.md
3. Zero infrastructure changes — the Brain uses only existing primitives
4. User always approves before anything is created
5. Existing projects/skills/schedules are not disrupted

## MVP Scope

- **Trigger 1 only** (first contact — new business goal)
- Creates: ROLE.md + HEARTBEAT.md + schedules
- Mandatory approval gate before creation
- Read existing projects to avoid duplication
- One agent, one conversation, one call

## Open Questions

1. **Where does the Brain agent live?** As a Claude Code `--agent` invoked by the gateway? As part of the system prompt? As a skill?
2. **How does the gateway detect Brain triggers?** Keyword detection in `keywords.rs`? A classifier call? Explicit `/setup` command?
3. **Learning threshold value?** How many lessons before Trigger 3 fires?
4. **Overwrite safety?** If a project already exists, should the Brain refuse, offer to update, or create alongside?
5. **ROLE.md quality?** The entire value depends on writing excellent ROLE.md files. How do we ensure quality?

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Poor ROLE.md quality | OMEGA becomes a mediocre assistant | Domain research in Brain prompt, examples of great ROLE.md, user can edit after |
| Domain knowledge breadth | Brain can't know every business domain | Lean on Claude's general knowledge + targeted questions |
| Session length | Brain conversation takes too many turns | Cap at 5 questions, propose with available info |
| Keyword ambiguity | "I want to trade" triggers Brain when user just wants to chat | Require explicit confirmation before starting setup |
| Existing project collision | Brain creates a duplicate | Always check existing projects first, offer to update |
