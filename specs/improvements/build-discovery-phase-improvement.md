# Improvement Requirements: Build Discovery Phase

> Add an interactive discovery conversation before the build pipeline so that vague
> build requests ("build me a CRM") are explored and clarified before OMEGA invests
> effort in a full 7-phase build.

## Scope

**Domains affected:** gateway (pipeline, builds, builds_agents, builds_parse, keywords), omega-core (config)

**Files that change:**
- `backend/src/gateway/pipeline.rs` -- new discovery state machine in the pending-build check section (lines 186-313)
- `backend/src/gateway/builds.rs` -- receive enriched brief from discovery, minor refactor of handle_build_request entry point
- `backend/src/gateway/builds_agents.rs` -- add BUILD_DISCOVERY_AGENT constant and include it in BUILD_AGENTS / AgentFilesGuard
- `backend/src/gateway/builds_parse.rs` -- add parse_discovery_output(), discovery_message() localized helpers
- `backend/src/gateway/keywords.rs` -- add discovery state functions, DISCOVERY_TTL_SECS, localized discovery messages, discovery cancel support
- `backend/crates/omega-core/src/config/mod.rs` -- add `"pending_discovery"` to SYSTEM_FACT_KEYS

**Files NOT affected:**
- `backend/src/gateway/mod.rs` -- no new modules (all changes in existing files)
- `backend/src/gateway/routing.rs` -- no changes
- `backend/src/gateway/process_markers.rs` -- no changes
- `backend/src/gateway/auth.rs` -- no changes
- `backend/src/gateway/scheduler*.rs` -- no changes
- `backend/src/gateway/heartbeat*.rs` -- no changes
- `backend/src/gateway/summarizer.rs` -- no changes
- All omega-memory, omega-channels, omega-providers, omega-skills, omega-sandbox code -- no changes
- `backend/src/markers/` -- no changes (discovery uses file-based state, not response markers)

## Summary (plain language)

When a user says "build me a CRM", OMEGA currently asks "Ready to build? Reply yes" and then immediately starts a 7-phase build pipeline that guesses what the user wants. This produces software that may not match the user's actual needs.

This improvement adds a **discovery conversation** between the build keyword detection and the build confirmation gate. When the build request is vague, OMEGA asks 3-5 clarifying questions (up to 3 rounds), collects the user's answers, and produces an Idea Brief -- a clear description of what should be built, for whom, and why. Only after discovery completes does OMEGA show the refined brief and ask "Ready to build this?"

If the build request is already specific ("build a Rust CLI that tracks Bitcoin prices via CoinGecko API and stores in SQLite"), the discovery agent recognizes this and immediately outputs a brief with no questions, skipping the multi-round conversation.

The discovery conversation state is persisted as a file on disk (`~/.omega/workspace/discovery/<sender_id>.md`) so it survives across message cycles. The session has a 30-minute TTL -- if the user goes quiet for too long, the session expires.

Discovery is a pre-pipeline intake step, not a numbered build phase. The 7-phase build pipeline (analyst through delivery) remains unchanged.

## User Stories

- As a user who says "build me a CRM", I want OMEGA to ask me clarifying questions (who uses it, what features matter, what's the MVP) so that the resulting software actually matches what I need.
- As a user who says "build a Rust CLI price tracker with SQLite and Telegram alerts", I want OMEGA to recognize my request is already specific and skip lengthy questioning so that the build starts quickly.
- As a user mid-discovery, I want to cancel the discovery session and either start over or abandon the build entirely.
- As a user who gets interrupted mid-discovery, I want OMEGA to remember where we left off for 30 minutes so I can resume naturally.
- As a user, I want to see the refined Idea Brief before confirming the build so that I can verify OMEGA understood me correctly.

## Current Behavior (What Happens Now)

```
User: "build me a CRM"
  -> pipeline.rs detects BUILDS_KW match
  -> stores "pending_build_request" fact with timestamp
  -> sends localized confirmation: "I detected a build request... Reply yes to proceed"
User: "yes"
  -> pipeline.rs reads pending_build_request fact
  -> checks BUILD_CONFIRM_TTL_SECS (120s)
  -> calls handle_build_request() with the raw "build me a CRM" text
  -> Phase 1 (Analyst) receives "build me a CRM" and must guess everything
  -> 7-phase pipeline runs to completion
```

**Problem:** The analyst agent receives a vague 4-word request and must invent all the details. The resulting software is a generic guess at what "a CRM" means.

## Desired Behavior (What Will Happen)

```
User: "build me a CRM"
  -> pipeline.rs detects BUILDS_KW match
  -> runs build-discovery agent with "build me a CRM"
  -> discovery agent outputs DISCOVERY_QUESTIONS with 3-5 clarifying questions
  -> OMEGA sends questions to user
  -> stores discovery state in ~/.omega/workspace/discovery/<sender_id>.md
  -> stores "pending_discovery" fact with timestamp
User: "It's for my small real estate team, 5 people, we need contact management and deal tracking"
  -> pipeline.rs reads pending_discovery fact, checks TTL (30 min)
  -> loads discovery file, appends user answer
  -> runs build-discovery agent with accumulated context
  -> discovery agent outputs more questions OR DISCOVERY_COMPLETE
  (... up to 3 rounds ...)
  -> discovery agent outputs DISCOVERY_COMPLETE with Idea Brief
  -> OMEGA stores pending_build_request fact with the ENRICHED brief (not raw request)
  -> sends localized confirmation showing the brief: "Here's what I'll build... Ready?"
User: "yes"
  -> existing confirmation gate triggers
  -> handle_build_request() receives enriched brief text
  -> Phase 1 (Analyst) receives clear, detailed brief
  -> 7-phase pipeline produces targeted software
```

## Architecture: Discovery State Machine

### State Transitions

```
                                    +-----------+
                                    |   IDLE    |
                                    +-----+-----+
                                          |
                                   build keyword detected
                                          |
                                          v
                              +-----------+-----------+
                              | Run build-discovery   |
                              | agent (round 1)       |
                              +-----------+-----------+
                                          |
                          +---------------+----------------+
                          |                                |
                   DISCOVERY_QUESTIONS              DISCOVERY_COMPLETE
                   (request is vague)               (request is specific)
                          |                                |
                          v                                v
                 +--------+--------+            +----------+----------+
                 | Send questions  |            | Store enriched      |
                 | to user         |            | brief as             |
                 | Store state     |            | pending_build_request|
                 | Store fact      |            | Send confirmation   |
                 +--------+--------+            +----------+----------+
                          |                                |
                    user replies                    (existing flow)
                          |
                          v
              +-----------+-----------+
              | Load discovery file   |
              | Append user answer    |
              | Run build-discovery   |
              | agent (round N)       |
              +-----------+-----------+
                          |
              +-----------+----------------+
              |                            |
       DISCOVERY_QUESTIONS          DISCOVERY_COMPLETE
       (round < 3)                  (or round = 3 auto)
              |                            |
              v                            v
        (loop back)              (store brief, confirm)
```

### Discovery File Format

File path: `~/.omega/workspace/discovery/<sender_id>.md`

```markdown
# Discovery Session

CREATED: <unix_timestamp>
ROUND: <1|2|3>
ORIGINAL_REQUEST: <user's original build request text>

## Round 1
### Agent Questions
<questions from discovery agent>

### User Response
<user's reply>

## Round 2
### Agent Questions
<follow-up questions>

### User Response
<user's reply>

## Round 3
...
```

The file is read in full and passed to the discovery agent as accumulated context on each round. On DISCOVERY_COMPLETE (or round 3 auto-complete), the file is deleted.

### Discovery Agent Output Protocol

The build-discovery agent outputs in one of two formats:

**Format 1: More questions needed**
```
DISCOVERY_QUESTIONS
<natural language questions for the user, 3-5 questions max>
```

**Format 2: Discovery complete**
```
DISCOVERY_COMPLETE
IDEA_BRIEF:
<structured idea brief text that will be passed to the build confirmation gate
and then to the analyst as an enriched build request>
```

The `IDEA_BRIEF:` content replaces the raw user request in the `pending_build_request` fact, so the analyst receives a clear, detailed description instead of "build me a CRM".

### Pipeline Integration Point

In `pipeline.rs`, the discovery check is inserted **before** the existing `pending_build_request` check (section 4a) and **replaces** the direct build keyword handling (section 4b). The flow becomes:

```
Section 4a-DISCOVERY: PENDING DISCOVERY CHECK
  -> if pending_discovery fact exists:
     -> check TTL (30 min)
     -> if expired: delete file, delete fact, fall through
     -> if cancelled: delete file, delete fact, send message, return
     -> otherwise: load file, append answer, run agent, handle output

Section 4a: PENDING BUILD CONFIRMATION CHECK
  -> (unchanged — but now receives enriched brief from discovery)

Section 4b: BUILD REQUESTS
  -> (modified — instead of immediately storing pending_build_request and
     asking for confirmation, now runs discovery agent first)
```

## Requirements

| ID | Requirement | Priority | Acceptance Criteria |
|----|------------|----------|-------------------|
| REQ-BDP-001 | Discovery state file: persist discovery conversation as a markdown file at `~/.omega/workspace/discovery/<sender_id>.md` | Must | - [ ] File created on first discovery round<br>- [ ] File appended with user answers on subsequent rounds<br>- [ ] File read in full and passed to discovery agent each round<br>- [ ] File deleted on DISCOVERY_COMPLETE or session expiry<br>- [ ] File deleted on explicit cancellation |
| REQ-BDP-002 | Embedded build-discovery agent: compile `BUILD_DISCOVERY_AGENT` constant into the binary alongside the existing 7 build agents | Must | - [ ] Agent has YAML frontmatter (name, description, tools, model, permissionMode, maxTurns)<br>- [ ] Agent uses `model: opus` (complex reasoning needed for discovery)<br>- [ ] Agent has `permissionMode: bypassPermissions`<br>- [ ] Agent has `maxTurns: 15` (generous for synthesis but bounded)<br>- [ ] Agent written to disk via AgentFilesGuard alongside the other 7 agents<br>- [ ] BUILD_AGENTS array updated to include the discovery agent |
| REQ-BDP-003 | Discovery output parsing: parse `DISCOVERY_QUESTIONS` and `DISCOVERY_COMPLETE` / `IDEA_BRIEF:` markers from agent output | Must | - [ ] `parse_discovery_output()` returns enum: Questions(String) or Complete(String)<br>- [ ] Questions variant contains the question text (everything after DISCOVERY_QUESTIONS line)<br>- [ ] Complete variant contains the idea brief text (everything after IDEA_BRIEF: line)<br>- [ ] Missing markers treated as auto-complete (use full output as brief)<br>- [ ] Function is pure (no side effects, testable without mocking) |
| REQ-BDP-004 | Pipeline discovery state machine: detect `pending_discovery` fact, load file, run agent, route output | Must | - [ ] New section in pipeline.rs BEFORE the existing pending_build_request check<br>- [ ] Reads `pending_discovery` fact to detect active session<br>- [ ] Loads discovery file from disk<br>- [ ] Appends user's current message as the latest answer<br>- [ ] Runs build-discovery agent with full accumulated context<br>- [ ] On DISCOVERY_QUESTIONS: updates file, sends questions to user<br>- [ ] On DISCOVERY_COMPLETE: deletes file, stores enriched brief as pending_build_request, sends confirmation<br>- [ ] On round 3: forces auto-complete regardless of agent output |
| REQ-BDP-005 | Discovery initiation: when build keyword detected and request is not already in discovery, run first discovery round instead of immediate confirmation | Must | - [ ] Replaces the current "store pending_build_request + send confirmation" logic in section 4b<br>- [ ] Creates discovery file with round 1 header and original request<br>- [ ] Runs build-discovery agent with the raw request<br>- [ ] If agent returns DISCOVERY_COMPLETE immediately (specific request): skips multi-round, goes straight to confirmation<br>- [ ] If agent returns DISCOVERY_QUESTIONS: stores pending_discovery fact, sends questions to user |
| REQ-BDP-006 | Discovery TTL: 30-minute session timeout | Must | - [ ] `DISCOVERY_TTL_SECS: i64 = 1800` constant in keywords.rs<br>- [ ] pending_discovery fact stores timestamp (same format as pending_build_request: `<timestamp>\|<sender_id>`)<br>- [ ] On expiry: discovery file deleted, fact deleted, user informed, message falls through to normal processing |
| REQ-BDP-007 | Discovery cancellation: user can cancel mid-discovery using existing cancel keywords | Must | - [ ] `is_build_cancelled()` check applied to user messages during active discovery session<br>- [ ] On cancellation: discovery file deleted, pending_discovery fact deleted, localized cancellation message sent<br>- [ ] Reuses existing BUILD_CANCEL_KW keywords (no, cancel, stop, etc.) |
| REQ-BDP-008 | Max 3 discovery rounds: auto-complete after 3 rounds | Must | - [ ] Round counter tracked in discovery file header (ROUND: N)<br>- [ ] On round 3, agent prompt includes instruction to synthesize final brief<br>- [ ] If round 3 agent still returns DISCOVERY_QUESTIONS, treat output as DISCOVERY_COMPLETE |
| REQ-BDP-009 | Localized discovery messages for all 8 languages | Must | - [ ] `discovery_intro_message(lang, questions)` -- sent when discovery starts with first question set<br>- [ ] `discovery_followup_message(lang, questions)` -- sent for rounds 2-3<br>- [ ] `discovery_complete_message(lang, brief_preview)` -- sent when brief is ready, integrated with build confirmation<br>- [ ] `discovery_expired_message(lang)` -- sent when session times out<br>- [ ] `discovery_cancelled_message(lang)` -- sent when user cancels discovery<br>- [ ] All 8 languages: English, Spanish, Portuguese, French, German, Italian, Dutch, Russian |
| REQ-BDP-010 | SYSTEM_FACT_KEYS updated: add `pending_discovery` to the protected fact keys list | Must | - [ ] `"pending_discovery"` added to SYSTEM_FACT_KEYS in omega-core config<br>- [ ] `is_valid_fact("pending_discovery", ...)` returns false (rejecting user writes) |
| REQ-BDP-011 | Discovery-to-confirmation handoff: DISCOVERY_COMPLETE output feeds into existing build confirmation gate | Must | - [ ] Enriched idea brief stored as pending_build_request (same format: `<timestamp>\|<brief_text>`)<br>- [ ] Confirmation message shows a preview of the refined brief (not the raw request)<br>- [ ] Existing confirmation flow (yes/no/expire) works unchanged<br>- [ ] On confirmation, handle_build_request receives the enriched brief text |
| REQ-BDP-012 | Discovery agent content: adapted from interactive discovery.md for non-interactive chat mode | Should | - [ ] Agent does NOT have extended back-and-forth (each invocation is single-shot)<br>- [ ] Agent receives accumulated context and outputs structured response<br>- [ ] Agent decides whether request is specific enough to skip questions<br>- [ ] Agent covers: problem, users, vision, concept challenge, constraints<br>- [ ] Agent produces Idea Brief format on DISCOVERY_COMPLETE<br>- [ ] Agent limits questions to 3-5 per round (not a wall of text) |
| REQ-BDP-013 | Discovery file cleanup on process restart: stale discovery files cleaned up on gateway startup | Should | - [ ] On gateway startup, scan `~/.omega/workspace/discovery/` directory<br>- [ ] Delete any files older than DISCOVERY_TTL_SECS<br>- [ ] Delete corresponding pending_discovery facts for affected sender_ids |
| REQ-BDP-014 | Audit logging for discovery: log discovery start, rounds, completion/cancellation/expiry | Should | - [ ] `[DISCOVERY:<sender_id>]` prefix in audit log input_text<br>- [ ] Status: Ok for completion, Error for expiry/cancellation |
| REQ-BDP-015 | Discovery agent uses model_complex (Opus) | Should | - [ ] Discovery requires nuanced reasoning about vague requests<br>- [ ] Same model as analyst and architect phases |
| REQ-BDP-016 | Typing indicator during discovery agent execution | Could | - [ ] Show typing indicator while the discovery agent is running<br>- [ ] Stop typing indicator before sending questions to user |
| REQ-BDP-017 | Discovery round progress indicator | Could | - [ ] Show "Round 1/3", "Round 2/3" etc. in the discovery messages<br>- [ ] Helps user understand how many more rounds to expect |
| REQ-BDP-018 | Discovery skip keyword: user can say "just build it" to skip remaining discovery rounds | Won't | Deferred -- complicates state machine. User can cancel and re-request with a more specific description. |
| REQ-BDP-019 | Discovery for BUILD_PROPOSAL marker: discovery triggered when OMEGA itself proposes a build | Won't | Deferred -- BUILD_PROPOSAL is already a refined suggestion from the LLM. |
| REQ-BDP-020 | Discovery history: persist completed idea briefs for analytics | Won't | Deferred -- builds already have audit logging. |

## Acceptance Criteria (detailed)

### REQ-BDP-001: Discovery state file
- [ ] Given a user sends "build me a CRM", when the discovery agent returns DISCOVERY_QUESTIONS, then a file exists at `~/.omega/workspace/discovery/<sender_id>.md` containing the original request and agent questions
- [ ] Given an active discovery session, when the user sends an answer, then the file is updated with the answer appended under the current round
- [ ] Given the discovery agent returns DISCOVERY_COMPLETE, then the discovery file is deleted from disk
- [ ] Given the discovery session expires (30 min), then the discovery file is deleted from disk
- [ ] Given the user cancels discovery, then the discovery file is deleted from disk
- [ ] Given the `~/.omega/workspace/discovery/` directory does not exist, when a discovery session starts, then the directory is created automatically
- [ ] The sender_id in the filename must be filesystem-safe (no path separators, no special characters)

### REQ-BDP-002: Embedded build-discovery agent
- [ ] `BUILD_DISCOVERY_AGENT` constant exists in `builds_agents.rs`
- [ ] Agent frontmatter contains: name: build-discovery, tools: Read, Grep, Glob, model: opus, permissionMode: bypassPermissions, maxTurns: 15
- [ ] `BUILD_AGENTS` array includes `("build-discovery", BUILD_DISCOVERY_AGENT)` entry
- [ ] `AgentFilesGuard::write()` writes `build-discovery.md` alongside the other 7 agents
- [ ] Agent file content matches the embedded constant when written to disk

### REQ-BDP-003: Discovery output parsing
- [ ] Given output containing "DISCOVERY_QUESTIONS\nWhat problem...\nWho uses...", when parsed, then returns Questions("What problem...\nWho uses...")
- [ ] Given output containing "DISCOVERY_COMPLETE\nIDEA_BRIEF:\nA CRM tool for...", when parsed, then returns Complete("A CRM tool for...")
- [ ] Given output containing both markers, DISCOVERY_COMPLETE takes precedence
- [ ] Given output containing neither marker, treat entire output as Complete (auto-complete fallback)
- [ ] Given empty output, return Complete with empty string (graceful degradation)

### REQ-BDP-004: Pipeline discovery state machine
- [ ] Given an active discovery session (pending_discovery fact exists) and user sends a message, when the message is not a cancellation keyword, then the discovery agent is invoked with accumulated context
- [ ] Given an active discovery session and user sends "cancel", then the session is terminated and the user is informed
- [ ] Given an active discovery session that has expired (>30 min), then the session is terminated, the user is informed, and the message falls through to normal processing
- [ ] Given the discovery agent returns DISCOVERY_QUESTIONS on round 1, then questions are sent to the user and the session continues
- [ ] Given the discovery agent returns DISCOVERY_COMPLETE on any round, then the enriched brief is stored and confirmation is triggered
- [ ] Given round 3 completes, regardless of agent output format, the response is treated as a completed brief

### REQ-BDP-005: Discovery initiation
- [ ] Given a user sends "build me a CRM" (vague, matches BUILDS_KW), when no active discovery or pending build exists, then a new discovery session starts (NOT the old immediate-confirmation flow)
- [ ] Given the first-round discovery agent returns DISCOVERY_COMPLETE immediately, then no multi-round session is created and the flow goes straight to build confirmation with the enriched brief
- [ ] Given the first-round discovery agent returns DISCOVERY_QUESTIONS, then questions are sent to the user and pending_discovery fact is stored

### REQ-BDP-006: Discovery TTL
- [ ] `DISCOVERY_TTL_SECS` constant equals 1800 (30 minutes)
- [ ] Given a discovery session started 31 minutes ago, when the user sends a message, then the session is expired and the discovery file is deleted
- [ ] Given a discovery session started 29 minutes ago, when the user sends a message, then the session continues normally

### REQ-BDP-007: Discovery cancellation
- [ ] Given an active discovery session and user sends "no", then the session is cancelled
- [ ] Given an active discovery session and user sends "cancel", then the session is cancelled
- [ ] Given an active discovery session and user sends "nein" (German), then the session is cancelled
- [ ] Cancellation message is localized to the user's preferred_language

### REQ-BDP-008: Max 3 rounds
- [ ] Given round 3, the agent prompt includes explicit instruction: "This is the final round. Synthesize an Idea Brief from everything you know."
- [ ] Given round 3, if the agent outputs DISCOVERY_QUESTIONS instead of DISCOVERY_COMPLETE, the questions text is treated as the idea brief

### REQ-BDP-009: Localized discovery messages
- [ ] English discovery intro message contains "before I start building" or similar
- [ ] Spanish discovery intro message is in Spanish
- [ ] All 8 languages produce non-empty messages for all 5 message types
- [ ] Discovery messages include the question text from the agent

### REQ-BDP-010: SYSTEM_FACT_KEYS
- [ ] `SYSTEM_FACT_KEYS` array in `backend/crates/omega-core/src/config/mod.rs` contains `"pending_discovery"`
- [ ] `is_valid_fact("pending_discovery", "anything")` returns false

### REQ-BDP-011: Discovery-to-confirmation handoff
- [ ] Given DISCOVERY_COMPLETE with brief "A real estate CRM for teams of 5...", then `pending_build_request` fact is stored with this brief text (not the original "build me a CRM")
- [ ] The confirmation message shows a preview of the brief (truncated if long)
- [ ] After user confirms, `handle_build_request` receives the enriched brief text as `incoming.text`

## Impact Analysis

### Existing Code Affected

| File | Change | Risk |
|------|--------|------|
| `backend/src/gateway/pipeline.rs` (lines 186-313) | Add discovery state check before pending_build_request check; modify build keyword section (4b) to start discovery instead of immediate confirmation | **Medium** -- this is the critical message routing path. Must preserve all existing branches (non-build messages, pending confirmation, expired confirmation, cancelled confirmation). The discovery check is additive and goes BEFORE the existing logic. |
| `backend/src/gateway/builds.rs` | Minor -- handle_build_request now receives enriched text but its interface and behavior are unchanged | **Low** -- no signature change. The incoming.text is just richer. |
| `backend/src/gateway/builds_agents.rs` | Add BUILD_DISCOVERY_AGENT constant, update BUILD_AGENTS array (7 -> 8 entries), update AgentFilesGuard | **Medium** -- existing tests assert exactly 7 agents and specific agent names. Tests need updating. |
| `backend/src/gateway/builds_parse.rs` | Add parse_discovery_output() function and DiscoveryOutput enum. Add localized discovery message functions. | **Low** -- additive, no existing functions change. |
| `backend/src/gateway/keywords.rs` | Add DISCOVERY_TTL_SECS constant, localized discovery message functions | **Low** -- additive constants and functions. |
| `backend/crates/omega-core/src/config/mod.rs` | Add `"pending_discovery"` to SYSTEM_FACT_KEYS array | **Low** -- one entry appended to an array. |

### What Breaks If This Changes

| Component | Impact | Mitigation |
|-----------|--------|------------|
| `builds_agents.rs` tests: `test_build_agents_has_exactly_7_entries` | Fails -- BUILD_AGENTS will have 8 entries | Update test assertion from 7 to 8 |
| `builds_agents.rs` tests: `test_build_agents_correct_names` | Fails -- expected names array doesn't include build-discovery | Add "build-discovery" to expected names |
| `pipeline.rs` build keyword section (4b) | Behavior changes -- no longer stores pending_build_request immediately | Discovery replaces the immediate confirmation for first-time build requests |
| `is_valid_fact()` tests | Need new test for `"pending_discovery"` rejection | Add `test_is_valid_fact_rejects_pending_discovery` |

### Regression Risk Areas

| Area | Risk |
|------|------|
| Non-build messages | **Low** -- discovery check only activates when pending_discovery fact exists. Normal messages are unaffected because the fact won't exist. |
| Existing build confirmation flow | **Low** -- the pending_build_request confirmation logic is UNCHANGED. Discovery feeds INTO it, not replaces it. |
| BUILD_PROPOSAL marker flow | **Low** -- BUILD_PROPOSAL stores pending_build_request directly (bypasses keyword detection). This path is unaffected by discovery. |
| Concurrent messages from same user | **Low** -- `active_senders` mutex in mod.rs already serializes messages per sender. Discovery state is per-sender, so no race conditions. |
| AgentFilesGuard cleanup | **Low** -- the guard writes ALL agents (including discovery) and removes the entire directory on drop. Adding one more agent file changes nothing about the lifecycle. |

### What Is Preserved (Not Changed)

- **7-phase build pipeline** -- analyst, architect, test-writer, developer, QA, reviewer, delivery. Completely unchanged.
- **Build confirmation gate** -- pending_build_request fact, TTL check, confirm/cancel keywords. Unchanged.
- **BUILD_PROPOSAL marker flow** -- process_markers.rs stores pending_build_request directly. Unaffected.
- **All existing parse functions** -- parse_project_brief, parse_verification_result, parse_build_summary. Untouched.
- **All localized build messages** -- build_confirm_message, build_cancelled_message, phase_message. Preserved.
- **Provider complete() interface** -- discovery uses run_build_phase() which already exists.
- **Memory store API** -- uses existing store_fact/get_fact/delete_fact. No schema changes.

## Implementation Guide

### 1. Add pending_discovery to SYSTEM_FACT_KEYS (REQ-BDP-010)

**File:** `backend/crates/omega-core/src/config/mod.rs`

Add `"pending_discovery"` to the SYSTEM_FACT_KEYS array:

```rust
pub const SYSTEM_FACT_KEYS: &[&str] = &[
    "welcomed",
    "preferred_language",
    "active_project",
    "personality",
    "onboarding_stage",
    "pending_build_request",
    "pending_discovery",      // <-- NEW
];
```

### 2. Add BUILD_DISCOVERY_AGENT constant (REQ-BDP-002, REQ-BDP-012)

**File:** `backend/src/gateway/builds_agents.rs`

Add new constant BEFORE the BUILD_AGENTS array. The agent is adapted from the interactive `discovery.md` for non-interactive single-shot invocation in a chat bot context:

```rust
pub(super) const BUILD_DISCOVERY_AGENT: &str = "\
---
name: build-discovery
description: Explores vague build requests through structured questioning, produces Idea Brief
tools: Read, Grep, Glob
model: opus
permissionMode: bypassPermissions
maxTurns: 15
---

You are a build discovery agent. You explore vague build requests to understand what the user actually needs before a build pipeline runs.

You are NOT the analyst. You do not write requirements, assign IDs, or define acceptance criteria. You explore the idea itself — what it is, who it is for, why it matters, and what the MVP should include.

Do NOT ask the user for clarification interactively — you are invoked as a single-shot agent. Instead, read the accumulated context provided and decide:

1. If the request is ALREADY specific and clear (technology chosen, features listed, users identified, scope bounded), output DISCOVERY_COMPLETE immediately with an Idea Brief.
2. If the request is vague or missing critical details, output DISCOVERY_QUESTIONS with 3-5 focused questions.

## What makes a request specific enough to skip questions?
- The user named concrete features (not just a category like 'CRM')
- The user specified the technology or language
- The user described who uses it and roughly what it does
- Example specific: 'Build a Rust CLI that tracks Bitcoin prices from CoinGecko, stores history in SQLite, and sends Telegram alerts when price crosses thresholds'
- Example vague: 'Build me a CRM' or 'I need a dashboard'

## Questioning Strategy (when questions are needed)

Cover these areas, 3-5 questions per round maximum:

Round 1 (first invocation with raw request):
- What problem does this solve? Who has this problem today?
- Who are the primary users? What is their technical level?
- What does the simplest useful version look like? (MVP)
- Any technology preferences or constraints?
- What is explicitly NOT part of this?

Round 2+ (with accumulated answers):
- Follow up on vague answers from previous rounds
- Challenge assumptions: is this the right approach? Could something simpler work?
- Narrow scope: of everything discussed, what is the ONE thing that must work in v1?
- Use analogies to confirm understanding: 'So it is like X but for Y?'

## Question Style
- Be curious, not interrogating
- Use plain language (the user may be non-technical)
- Keep questions short and concrete
- Do NOT ask 10 questions — 3 to 5 maximum per round
- Match the user's language (if they write in Spanish, ask in Spanish)

## Output Format

You MUST output in one of exactly two formats:

### Format 1: Need more information
```
DISCOVERY_QUESTIONS
<your questions here, as a natural conversational message>
```

### Format 2: Ready to build
```
DISCOVERY_COMPLETE
IDEA_BRIEF:
One-line summary: <what this is>
Problem: <what problem it solves, for whom>
Users: <who uses it, their technical level>
MVP scope: <the minimum viable feature set>
Technology: <language, framework, database choices>
Out of scope: <what is explicitly excluded>
Key decisions: <any decisions made during discovery>
Constraints: <scale, integrations, timeline if mentioned>
```

## Rules
- If this is the final round (the prompt will tell you), you MUST output DISCOVERY_COMPLETE regardless of how much information you have. Synthesize the best brief you can from available context.
- Never output both DISCOVERY_QUESTIONS and DISCOVERY_COMPLETE — pick one.
- The Idea Brief does not need to be perfect — it just needs to be dramatically better than the raw request.
- Keep the brief concise — it will be passed to a build analyst agent, not displayed to the user verbatim.
";
```

Update the BUILD_AGENTS array:

```rust
pub(super) const BUILD_AGENTS: &[(&str, &str)] = &[
    ("build-discovery", BUILD_DISCOVERY_AGENT),   // <-- NEW (first position)
    ("build-analyst", BUILD_ANALYST_AGENT),
    ("build-architect", BUILD_ARCHITECT_AGENT),
    ("build-test-writer", BUILD_TEST_WRITER_AGENT),
    ("build-developer", BUILD_DEVELOPER_AGENT),
    ("build-qa", BUILD_QA_AGENT),
    ("build-reviewer", BUILD_REVIEWER_AGENT),
    ("build-delivery", BUILD_DELIVERY_AGENT),
];
```

### 3. Add discovery output parsing (REQ-BDP-003)

**File:** `backend/src/gateway/builds_parse.rs`

Add new enum and parse function:

```rust
/// Result of discovery agent invocation.
pub(super) enum DiscoveryOutput {
    /// Agent needs more information — contains question text for the user.
    Questions(String),
    /// Agent has enough info — contains the synthesized Idea Brief.
    Complete(String),
}

/// Parse discovery agent output into questions or a completed brief.
pub(super) fn parse_discovery_output(text: &str) -> DiscoveryOutput {
    // DISCOVERY_COMPLETE takes precedence if both markers present.
    if text.contains("DISCOVERY_COMPLETE") {
        let brief = text
            .lines()
            .skip_while(|l| !l.starts_with("IDEA_BRIEF:"))
            .skip(1) // skip the IDEA_BRIEF: line itself
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        // If IDEA_BRIEF: section is empty, use everything after DISCOVERY_COMPLETE.
        if brief.is_empty() {
            let fallback = text
                .lines()
                .skip_while(|l| !l.contains("DISCOVERY_COMPLETE"))
                .skip(1)
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
            return DiscoveryOutput::Complete(fallback);
        }
        return DiscoveryOutput::Complete(brief);
    }

    if text.contains("DISCOVERY_QUESTIONS") {
        let questions = text
            .lines()
            .skip_while(|l| !l.contains("DISCOVERY_QUESTIONS"))
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        return DiscoveryOutput::Questions(questions);
    }

    // No markers — treat entire output as a completed brief (auto-complete fallback).
    DiscoveryOutput::Complete(text.trim().to_string())
}
```

### 4. Add discovery localized messages (REQ-BDP-009)

**File:** `backend/src/gateway/keywords.rs`

Add constants and functions:

```rust
/// Maximum seconds a discovery session stays valid.
pub(super) const DISCOVERY_TTL_SECS: i64 = 1800; // 30 minutes

/// Localized message sent when discovery starts (first round questions).
pub(super) fn discovery_intro_message(lang: &str, questions: &str) -> String {
    // Each language has a brief intro followed by the agent's questions.
    let intro = match lang {
        "Spanish" => "Antes de empezar a construir, necesito entender mejor tu idea:",
        "Portuguese" => "Antes de começar a construir, preciso entender melhor sua ideia:",
        "French" => "Avant de commencer à construire, j'ai besoin de mieux comprendre ton idée :",
        "German" => "Bevor ich mit dem Bauen beginne, muss ich deine Idee besser verstehen:",
        "Italian" => "Prima di iniziare a costruire, ho bisogno di capire meglio la tua idea:",
        "Dutch" => "Voordat ik begin met bouwen, moet ik je idee beter begrijpen:",
        "Russian" => "Прежде чем начать сборку, мне нужно лучше понять вашу идею:",
        _ => "Before I start building, I need to understand your idea better:",
    };
    format!("{intro}\n\n{questions}")
}

/// Localized message sent for follow-up discovery rounds (2-3).
pub(super) fn discovery_followup_message(lang: &str, questions: &str, round: u8) -> String {
    let followup = match lang {
        "Spanish" => format!("Gracias. Un par de preguntas más ({round}/3):"),
        "Portuguese" => format!("Obrigado. Mais algumas perguntas ({round}/3):"),
        "French" => format!("Merci. Encore quelques questions ({round}/3) :"),
        "German" => format!("Danke. Noch ein paar Fragen ({round}/3):"),
        "Italian" => format!("Grazie. Ancora qualche domanda ({round}/3):"),
        "Dutch" => format!("Bedankt. Nog een paar vragen ({round}/3):"),
        "Russian" => format!("Спасибо. Ещё несколько вопросов ({round}/3):"),
        _ => format!("Thanks. A few more questions ({round}/3):"),
    };
    format!("{followup}\n\n{questions}")
}

/// Localized message sent when discovery completes and confirmation is needed.
pub(super) fn discovery_complete_message(lang: &str, brief_preview: &str) -> String {
    match lang {
        "Spanish" => format!(
            "Entendido. Esto es lo que voy a construir:\n\n\
             _{brief_preview}_\n\n\
             Responde *sí* para comenzar la construcción (tienes 2 minutos)."
        ),
        "Portuguese" => format!(
            "Entendido. Isto é o que vou construir:\n\n\
             _{brief_preview}_\n\n\
             Responda *sim* para iniciar a construção (você tem 2 minutos)."
        ),
        "French" => format!(
            "Compris. Voici ce que je vais construire :\n\n\
             _{brief_preview}_\n\n\
             Réponds *oui* pour lancer la construction (tu as 2 minutes)."
        ),
        "German" => format!(
            "Verstanden. Das werde ich bauen:\n\n\
             _{brief_preview}_\n\n\
             Antworte *ja* um den Build zu starten (du hast 2 Minuten)."
        ),
        "Italian" => format!(
            "Capito. Ecco cosa costruirò:\n\n\
             _{brief_preview}_\n\n\
             Rispondi *sì* per avviare la costruzione (hai 2 minuti)."
        ),
        "Dutch" => format!(
            "Begrepen. Dit ga ik bouwen:\n\n\
             _{brief_preview}_\n\n\
             Antwoord *ja* om de build te starten (je hebt 2 minuten)."
        ),
        "Russian" => format!(
            "Понял. Вот что я собираюсь построить:\n\n\
             _{brief_preview}_\n\n\
             Ответьте *да* чтобы начать сборку (у вас 2 минуты)."
        ),
        _ => format!(
            "Got it. Here's what I'll build:\n\n\
             _{brief_preview}_\n\n\
             Reply *yes* to start the build (you have 2 minutes)."
        ),
    }
}

/// Localized message when discovery session expires.
pub(super) fn discovery_expired_message(lang: &str) -> &'static str {
    match lang {
        "Spanish" => "La sesión de descubrimiento expiró. Envía tu solicitud de construcción de nuevo si quieres continuar.",
        "Portuguese" => "A sessão de descoberta expirou. Envie sua solicitação de construção novamente se quiser continuar.",
        "French" => "La session de découverte a expiré. Renvoie ta demande de construction si tu veux continuer.",
        "German" => "Die Discovery-Sitzung ist abgelaufen. Sende deine Build-Anfrage erneut, wenn du fortfahren möchtest.",
        "Italian" => "La sessione di scoperta è scaduta. Invia di nuovo la tua richiesta di costruzione se vuoi continuare.",
        "Dutch" => "De discovery-sessie is verlopen. Stuur je build-verzoek opnieuw als je wilt doorgaan.",
        "Russian" => "Сессия обнаружения истекла. Отправьте запрос на сборку снова, если хотите продолжить.",
        _ => "Discovery session expired. Send your build request again if you want to continue.",
    }
}

/// Localized message when user cancels discovery.
pub(super) fn discovery_cancelled_message(lang: &str) -> &'static str {
    match lang {
        "Spanish" => "Descubrimiento cancelado.",
        "Portuguese" => "Descoberta cancelada.",
        "French" => "Découverte annulée.",
        "German" => "Discovery abgebrochen.",
        "Italian" => "Scoperta annullata.",
        "Dutch" => "Discovery geannuleerd.",
        "Russian" => "Обнаружение отменено.",
        _ => "Discovery cancelled.",
    }
}
```

### 5. Pipeline Integration (REQ-BDP-004, REQ-BDP-005, REQ-BDP-006, REQ-BDP-007, REQ-BDP-008, REQ-BDP-011)

**File:** `backend/src/gateway/pipeline.rs`

The discovery state machine is inserted as a NEW section between the typing indicator (section 4) and the existing pending_build_request check (section 4a). The build keyword handler (section 4b) is modified to start discovery instead of immediate confirmation.

**Pseudocode for new section 4a-DISCOVERY (inserted BEFORE current section 4a):**

```rust
// --- 4a-DISCOVERY. PENDING DISCOVERY SESSION CHECK ---
let pending_discovery: Option<String> = self
    .memory
    .get_fact(&incoming.sender_id, "pending_discovery")
    .await
    .ok()
    .flatten();

if let Some(discovery_value) = pending_discovery {
    // Parse timestamp from "timestamp|sender_id" format.
    let (stored_ts, _) = discovery_value
        .split_once('|')
        .unwrap_or(("0", &discovery_value));
    let created_at: i64 = stored_ts.parse().unwrap_or(0);
    let now = chrono::Utc::now().timestamp();
    let expired = (now - created_at) > DISCOVERY_TTL_SECS;

    let user_lang = self
        .memory
        .get_fact(&incoming.sender_id, "preferred_language")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "English".to_string());

    if expired {
        // Clean up expired session.
        let _ = self.memory.delete_fact(&incoming.sender_id, "pending_discovery").await;
        let discovery_file = discovery_file_path(&self.data_dir, &incoming.sender_id);
        let _ = tokio::fs::remove_file(&discovery_file).await;
        info!("[{}] discovery session expired", incoming.channel);
        // DO NOT return — let the message fall through to normal processing.
        // But inform the user their session expired.
        self.send_text(&incoming, discovery_expired_message(&user_lang)).await;
        // Fall through — the current message might be a new build request or normal chat.
    } else if is_build_cancelled(&clean_incoming.text) {
        // User cancelled discovery.
        let _ = self.memory.delete_fact(&incoming.sender_id, "pending_discovery").await;
        let discovery_file = discovery_file_path(&self.data_dir, &incoming.sender_id);
        let _ = tokio::fs::remove_file(&discovery_file).await;
        if let Some(h) = typing_handle { h.abort(); }
        self.send_text(&incoming, discovery_cancelled_message(&user_lang)).await;
        return;
    } else {
        // Active discovery session — process the user's answer.
        let discovery_file = discovery_file_path(&self.data_dir, &incoming.sender_id);
        let mut discovery_context = tokio::fs::read_to_string(&discovery_file)
            .await
            .unwrap_or_default();

        // Parse current round from file header.
        let current_round = parse_discovery_round(&discovery_context);

        // Append user's answer to the file.
        discovery_context.push_str(&format!(
            "\n### User Response\n{}\n",
            clean_incoming.text
        ));

        let is_final_round = current_round >= 3;
        let next_round = current_round + 1;

        // Build prompt for discovery agent.
        let agent_prompt = if is_final_round {
            format!(
                "This is the FINAL round. You MUST output DISCOVERY_COMPLETE with an Idea Brief.\n\
                 Synthesize everything below into a brief.\n\n{discovery_context}"
            )
        } else {
            format!(
                "Discovery round {next_round}/3. Read the accumulated context and either:\n\
                 - Output DISCOVERY_QUESTIONS if you need more info\n\
                 - Output DISCOVERY_COMPLETE if you have enough\n\n{discovery_context}"
            )
        };

        // Write agent files and run discovery agent.
        let workspace_dir = PathBuf::from(shellexpand(&self.data_dir)).join("workspace");
        let _agent_guard = AgentFilesGuard::write(&workspace_dir).await; // handle error

        let result = self.run_build_phase(
            "build-discovery", &agent_prompt, &self.model_complex, Some(15)
        ).await;

        match result {
            Ok(output) => {
                let parsed = parse_discovery_output(&output);
                // If final round, force Complete.
                let parsed = if is_final_round {
                    match parsed {
                        DiscoveryOutput::Questions(q) => DiscoveryOutput::Complete(q),
                        other => other,
                    }
                } else {
                    parsed
                };

                match parsed {
                    DiscoveryOutput::Questions(questions) => {
                        // Update discovery file with new round header.
                        let updated = format!(
                            "{discovery_context}\n## Round {next_round}\n### Agent Questions\n{questions}\n"
                        );
                        // Update ROUND: header in file.
                        let updated = update_round_header(&updated, next_round);
                        let _ = tokio::fs::write(&discovery_file, &updated).await;

                        // Send questions to user.
                        let msg = if next_round == 1 {
                            discovery_intro_message(&user_lang, &questions)
                        } else {
                            discovery_followup_message(&user_lang, &questions, next_round as u8)
                        };
                        if let Some(h) = typing_handle { h.abort(); }
                        self.send_text(&incoming, &msg).await;
                        return;
                    }
                    DiscoveryOutput::Complete(brief) => {
                        // Discovery complete — clean up and hand off to confirmation.
                        let _ = self.memory.delete_fact(&incoming.sender_id, "pending_discovery").await;
                        let _ = tokio::fs::remove_file(&discovery_file).await;

                        // Store enriched brief as pending_build_request.
                        let stamped = format!("{}|{}", chrono::Utc::now().timestamp(), brief);
                        let _ = self.memory.store_fact(
                            &incoming.sender_id, "pending_build_request", &stamped
                        ).await;

                        // Send discovery complete + confirmation message.
                        let preview = truncate_brief_preview(&brief, 300);
                        let msg = discovery_complete_message(&user_lang, &preview);
                        if let Some(h) = typing_handle { h.abort(); }
                        self.send_text(&incoming, &msg).await;
                        return;
                    }
                }
            }
            Err(e) => {
                // Discovery agent failed — clean up, inform user.
                let _ = self.memory.delete_fact(&incoming.sender_id, "pending_discovery").await;
                let _ = tokio::fs::remove_file(&discovery_file).await;
                if let Some(h) = typing_handle { h.abort(); }
                self.send_text(&incoming, &format!("Discovery failed: {e}")).await;
                return;
            }
        }
    }
}
```

**Modification to section 4b (BUILD REQUESTS) -- replaces immediate confirmation with discovery initiation:**

```rust
if needs_builds {
    info!("[{}] build keyword detected → starting discovery", incoming.channel);

    let user_lang = self
        .memory
        .get_fact(&incoming.sender_id, "preferred_language")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "English".to_string());

    // Write agent files for discovery.
    let workspace_dir = PathBuf::from(shellexpand(&self.data_dir)).join("workspace");
    let _agent_guard = AgentFilesGuard::write(&workspace_dir).await; // handle error

    // Run first discovery round with raw request.
    let agent_prompt = format!(
        "Discovery round 1/3. Analyze this build request and decide:\n\
         - If specific enough, output DISCOVERY_COMPLETE with an Idea Brief\n\
         - If vague, output DISCOVERY_QUESTIONS with 3-5 clarifying questions\n\n\
         User request: {}", incoming.text
    );

    let result = self.run_build_phase(
        "build-discovery", &agent_prompt, &self.model_complex, Some(15)
    ).await;

    match result {
        Ok(output) => {
            let parsed = parse_discovery_output(&output);
            match parsed {
                DiscoveryOutput::Complete(brief) => {
                    // Request was specific — skip multi-round, go straight to confirmation.
                    let stamped = format!("{}|{}", chrono::Utc::now().timestamp(), brief);
                    let _ = self.memory.store_fact(
                        &incoming.sender_id, "pending_build_request", &stamped
                    ).await;
                    let preview = truncate_brief_preview(&brief, 300);
                    let msg = discovery_complete_message(&user_lang, &preview);
                    if let Some(h) = typing_handle { h.abort(); }
                    self.send_text(&incoming, &msg).await;
                    return;
                }
                DiscoveryOutput::Questions(questions) => {
                    // Request was vague — start multi-round discovery session.
                    let discovery_file = discovery_file_path(&self.data_dir, &incoming.sender_id);
                    let discovery_dir = discovery_file.parent().unwrap();
                    let _ = tokio::fs::create_dir_all(discovery_dir).await;

                    // Create discovery file with round 1 content.
                    let file_content = format!(
                        "# Discovery Session\n\n\
                         CREATED: {}\n\
                         ROUND: 1\n\
                         ORIGINAL_REQUEST: {}\n\n\
                         ## Round 1\n\
                         ### Agent Questions\n{}\n",
                        chrono::Utc::now().timestamp(),
                        incoming.text,
                        questions
                    );
                    let _ = tokio::fs::write(&discovery_file, &file_content).await;

                    // Store pending_discovery fact.
                    let stamped = format!("{}|{}", chrono::Utc::now().timestamp(), incoming.sender_id);
                    let _ = self.memory.store_fact(
                        &incoming.sender_id, "pending_discovery", &stamped
                    ).await;

                    // Send questions to user.
                    let msg = discovery_intro_message(&user_lang, &questions);
                    if let Some(h) = typing_handle { h.abort(); }
                    self.send_text(&incoming, &msg).await;
                    return;
                }
            }
        }
        Err(e) => {
            // Discovery failed — fall back to old behavior (direct confirmation).
            warn!("Discovery agent failed, falling back to direct confirmation: {e}");
            let stamped = format!("{}|{}", chrono::Utc::now().timestamp(), incoming.text);
            let _ = self.memory.store_fact(
                &incoming.sender_id, "pending_build_request", &stamped
            ).await;
            let msg = build_confirm_message(&user_lang, &incoming.text);
            if let Some(h) = typing_handle { h.abort(); }
            self.send_text(&incoming, &msg).await;
            return;
        }
    }
}
```

### 6. Helper functions

**File:** `backend/src/gateway/builds_parse.rs` (or a new `builds_discovery.rs` if builds_parse.rs gets too long)

```rust
/// Get the path to a discovery state file for a given sender.
pub(super) fn discovery_file_path(data_dir: &str, sender_id: &str) -> PathBuf {
    // Sanitize sender_id for filesystem safety.
    let safe_id: String = sender_id
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    PathBuf::from(shellexpand(data_dir))
        .join("workspace")
        .join("discovery")
        .join(format!("{safe_id}.md"))
}

/// Parse the current round number from a discovery file's ROUND: header.
pub(super) fn parse_discovery_round(content: &str) -> u8 {
    content
        .lines()
        .find(|l| l.starts_with("ROUND:"))
        .and_then(|l| l["ROUND:".len()..].trim().parse::<u8>().ok())
        .unwrap_or(1)
}

/// Truncate a brief for preview in confirmation messages.
pub(super) fn truncate_brief_preview(brief: &str, max_chars: usize) -> String {
    if brief.chars().count() <= max_chars {
        brief.to_string()
    } else {
        let truncated: String = brief.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}
```

## Traceability Matrix

| Requirement ID | Priority | Test IDs | Architecture Section | Implementation Module |
|---------------|----------|----------|---------------------|---------------------|
| REQ-BDP-001 | Must | test_discovery_file_path_normal_sender, test_discovery_file_path_special_chars_sanitized | Discovery state file | pipeline.rs, builds_parse.rs |
| REQ-BDP-002 | Must | test_build_agents_has_exactly_8_entries, test_build_agents_correct_names, test_discovery_agent_constant_exists, test_discovery_agent_has_yaml_frontmatter, test_discovery_agent_frontmatter_name, test_discovery_agent_model_opus, test_discovery_agent_permission_bypass, test_discovery_agent_max_turns, test_discovery_agent_restricted_tools, test_discovery_agent_contains_questions_format, test_discovery_agent_contains_complete_format, test_build_agents_contains_discovery, test_build_agents_discovery_content_matches_constant, test_agent_files_guard_writes_discovery_agent | Embedded agent | builds_agents.rs |
| REQ-BDP-003 | Must | test_parse_discovery_output_questions_marker, test_parse_discovery_output_complete_with_brief, test_parse_discovery_output_complete_takes_precedence, test_parse_discovery_output_no_markers_auto_complete, test_parse_discovery_output_empty_input, test_parse_discovery_output_questions_with_prose_before, test_parse_discovery_output_complete_without_idea_brief_line | Output parsing | builds_parse.rs |
| REQ-BDP-004 | Must | (integration -- requires runtime; covered by QA) | Pipeline state machine | pipeline.rs |
| REQ-BDP-005 | Must | (integration -- requires runtime; covered by QA) | Discovery initiation | pipeline.rs |
| REQ-BDP-006 | Must | (integration -- requires runtime; covered by QA) | TTL management | keywords.rs, pipeline.rs |
| REQ-BDP-007 | Must | (integration -- requires runtime; covered by QA) | Cancellation | keywords.rs, pipeline.rs |
| REQ-BDP-008 | Must | test_parse_discovery_round_one, test_parse_discovery_round_three, test_parse_discovery_round_missing_header, test_parse_discovery_round_invalid_number | Max rounds | pipeline.rs |
| REQ-BDP-009 | Must | (localized message functions in keywords.rs -- covered by QA) | Localized messages | keywords.rs |
| REQ-BDP-010 | Must | test_system_fact_keys_contains_pending_discovery | System fact keys | omega-core config |
| REQ-BDP-011 | Must | test_truncate_brief_preview_short_text, test_truncate_brief_preview_long_text, test_truncate_brief_preview_exact_limit, test_truncate_brief_preview_unicode | Handoff to confirmation | pipeline.rs |
| REQ-BDP-012 | Should | test_discovery_agent_non_interactive | Agent content | builds_agents.rs |
| REQ-BDP-013 | Should | (integration -- requires runtime; covered by QA) | Startup cleanup | mod.rs or pipeline.rs |
| REQ-BDP-014 | Should | (integration -- requires runtime; covered by QA) | Audit logging | pipeline.rs |
| REQ-BDP-015 | Should | test_discovery_agent_model_opus | Model selection | pipeline.rs |
| REQ-BDP-016 | Could | (integration -- not unit testable) | Typing indicator | pipeline.rs |
| REQ-BDP-017 | Could | (integration -- not unit testable) | Round indicator | keywords.rs |
| REQ-BDP-018 | Won't | N/A | N/A | N/A |
| REQ-BDP-019 | Won't | N/A | N/A | N/A |
| REQ-BDP-020 | Won't | N/A | N/A | N/A |

## Specs Drift Detected

- `specs/improvements/build-agent-pipeline-improvement.md` states BUILD_AGENTS has exactly 7 entries and tests assert this. This is correct for the current code but will need to change to 8 after this improvement. The existing spec should be updated to note that the discovery agent was added.
- `specs/src-gateway-rs.md` does not mention the builds_agents module. The architecture diagram in SPECS.md also does not list builds.rs, builds_agents.rs, or builds_parse.rs in the gateway module list. These are pre-existing gaps, not introduced by this change.

## Assumptions

| # | Assumption (technical) | Explanation (plain language) | Confirmed |
|---|----------------------|---------------------------|-----------|
| 1 | `run_build_phase()` can be called from `pipeline.rs` via `self` because both are `impl Gateway` methods | The existing handle_build_request already calls run_build_phase from builds.rs. Pipeline.rs also has `impl Gateway`. The method is `pub(super)` scoped to the gateway module, so pipeline.rs can call it. | Yes |
| 2 | `AgentFilesGuard::write()` writes to `~/.omega/workspace/.claude/agents/`, which is the cwd for claude CLI subprocess | This is confirmed in builds.rs line 48 and the existing pipeline. | Yes |
| 3 | Discovery file content has no size limit on disk | Unlike facts (200 char value limit), files on disk can be any size. 3 rounds of discovery conversation will be well under 10KB. | Yes |
| 4 | The `active_senders` mutex prevents concurrent discovery state corruption | If user sends rapid messages, only one is processed at a time per sender. The second is buffered. This prevents race conditions on the discovery file. | Yes |
| 5 | The discovery agent receives ALL accumulated context in a single prompt | No session continuity. Each invocation is stateless. The full discovery file content is passed as the prompt. This works within Claude's context limits because discovery conversations are short (3 rounds max). | Yes |
| 6 | `pending_discovery` fact uses same `"timestamp\|data"` format as `pending_build_request` | Consistent with existing patterns in the codebase. | Yes |
| 7 | Discovery cancellation reuses `BUILD_CANCEL_KW` keywords (no, cancel, stop, etc.) | Same keywords work for cancelling both discovery sessions and build confirmations. No ambiguity because a user is in one state or the other, never both. | Yes |

## Identified Risks

- **Risk 1: Discovery agent produces poor questions for non-English users.** The agent instruction says "match the user's language" but LLM quality varies by language. **Mitigation:** The agent uses Opus (best multilingual model). If questions are bad, the 3-round cap ensures the session still completes.

- **Risk 2: Discovery adds latency before builds start.** Each discovery round requires an LLM call (Opus, up to 15 turns). For vague requests, this adds 1-3 rounds of LLM calls plus user response time. **Mitigation:** For specific requests, discovery completes in one call and adds only ~10-20 seconds. The latency is justified because it prevents building the wrong thing.

- **Risk 3: Discovery file left on disk if OMEGA crashes mid-session.** If the process dies between creating the file and completing/cleaning up, a stale file remains. **Mitigation:** REQ-BDP-013 (Should) adds startup cleanup that removes stale discovery files older than the TTL.

- **Risk 4: Discovery enriched brief may not parse correctly by parse_project_brief().** The analyst agent in Phase 1 expects PROJECT_NAME, LANGUAGE, SCOPE, COMPONENTS format. The discovery brief is free-form text. **Mitigation:** The analyst agent receives the brief as its prompt, not as structured output. It will produce the structured PROJECT_NAME format from the brief. The brief enriches the analyst's input, not replaces its output.

- **Risk 5: `pending_discovery` and `pending_build_request` facts could conflict.** If both exist simultaneously for the same user (shouldn't happen but defensive coding matters). **Mitigation:** Discovery always deletes `pending_discovery` before storing `pending_build_request`. The pipeline checks discovery FIRST, so an active discovery session always takes precedence.

## Out of Scope (Won't)

- **REQ-BDP-018: "Just build it" skip keyword** -- Adds complexity to the state machine for marginal value. Users who want to skip can cancel and re-request with more detail.
- **REQ-BDP-019: Discovery for BUILD_PROPOSAL** -- BUILD_PROPOSAL is already a refined LLM suggestion. Adding discovery to this path would create an awkward UX where OMEGA suggests a build and then asks itself questions.
- **REQ-BDP-020: Discovery history/analytics** -- Build audit logging already captures the build request text (which will now be the enriched brief). Storing discovery conversations separately is over-engineering for now.
- **Mid-discovery editing** -- User cannot go back and change a previous answer. They can cancel and start over.
- **Discovery for /workflow commands** -- The discovery agent in `.claude/agents/discovery.md` (for interactive Claude Code use) is separate from the embedded `build-discovery` agent. They serve different contexts.

---
