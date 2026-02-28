# Architecture: OMEGA Brain

## Scope

Self-configuration agent that transforms a non-technical user's business description into a working OMEGA project setup (ROLE.md, HEARTBEAT.md, scheduled actions, project activation). Triggered by `/setup` command. Covers: command registration, pipeline integration, session state machine, agent definition, file creation, marker emission, localized messages, and cleanup.

## Overview

```
User: "/setup I'm a realtor in Lisbon"
  |
  v
pipeline.rs: Command::Setup intercept
  |
  v
setup.rs: start_setup_session()
  |-- load existing projects via load_projects()
  |-- read existing ROLE.md content for context
  |-- build agent prompt with business description + existing projects
  |-- write Brain agent file via AgentFilesGuard::write_single()
  |-- invoke run_build_phase("omega-brain", prompt, model_complex, Some(30))
  |
  v
Brain agent (Claude Code subprocess in ~/.omega/ workspace)
  |-- Reads existing projects/skills in ~/.omega/
  |-- Round 1: asks 2-4 questions OR produces proposal directly
  |-- Round 2-3: refines based on answers
  |-- Final output: SETUP_PROPOSAL (user-facing preview)
  |
  v
pipeline.rs: pending_setup check
  |-- User sees proposal, replies yes/no/modification
  |-- On "yes": extract SETUP_EXECUTE section, run Brain again
  |-- On "no": cleanup session
  |-- On modification: feed back to Brain for refinement
  |
  v
Brain agent (execution mode)
  |-- Creates ~/.omega/projects/<name>/ROLE.md
  |-- Creates ~/.omega/projects/<name>/HEARTBEAT.md
  |-- Emits SCHEDULE_ACTION: markers
  |-- Emits PROJECT_ACTIVATE: <name>
  |
  v
setup_response.rs: handle_setup_response() -> process_markers()
  |-- Markers processed by existing process_markers()
  |-- Session cleaned up
  |-- Completion message sent to user
```

## Modules

### Module 1: Command Registration (`backend/src/commands/mod.rs`)

- **Responsibility**: Add `Setup` variant to `Command` enum and parse `/setup` command
- **Public interface**: `Command::Setup` variant, matched in `Command::parse()`
- **Dependencies**: None (pure parsing)
- **Implementation order**: 1

#### Changes Required

```rust
// In Command enum, add:
pub enum Command {
    // ... existing variants ...
    Setup,
    Help,
}

// In Command::parse(), add match arm:
"/setup" => Some(Self::Setup),
```

The `/setup` command is intercepted early in `pipeline.rs` (like `/forget`), NOT handled through the normal `commands::handle()` dispatch. The `Setup` variant exists for parsing only -- the handler is in `pipeline.rs`.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| False match on `/settings` | Prefix collision | Unit test | Exact match prevents this | None -- `/settings` is not a valid command |

#### Security Considerations
- No security concerns -- pure string parsing with exact match

#### Performance Budget
- O(1) match lookup, negligible

---

### Module 2: Brain Agent Definition (`topologies/development/agents/omega-brain.md`)

- **Responsibility**: Instruct the Claude Code subprocess to interview the user (via accumulated context), generate ROLE.md, HEARTBEAT.md, and emit markers
- **Public interface**: Agent `.md` file consumed by `run_build_phase()`
- **Dependencies**: Claude Code CLI subprocess, `~/.omega/` filesystem
- **Implementation order**: 2

#### Agent Frontmatter

```yaml
---
name: omega-brain
description: Configures OMEGA as a domain expert by creating projects from user business descriptions
tools: Read, Write, Glob, Grep
model: opus
permissionMode: bypassPermissions
maxTurns: 30
---
```

Key frontmatter decisions:
- **tools: Read, Write, Glob, Grep** -- NO Bash, NO Edit. Brain can read the filesystem to understand existing projects and write new files. No Bash prevents arbitrary command execution. No Edit prevents modifying existing files accidentally (Brain creates new files only).
- **model: opus** -- Brain quality is the highest-leverage factor. Use the complex model.
- **maxTurns: 30** -- Brain needs to read existing projects, analyze, and write multiple files. 30 turns provides headroom without unbounded execution.
- **permissionMode: bypassPermissions** -- Brain is non-interactive (subprocess).

#### Agent Prompt Structure

The agent prompt is divided into these sections:

1. **Identity and Purpose**: You are the OMEGA Brain. You configure OMEGA as a domain expert by creating project setups.

2. **Non-Interactive Instruction**: Do NOT ask the user directly. You receive accumulated context (user description + previous rounds) as your input. Your output is structured text that the gateway parses.

3. **Workspace Context**: You are running in `~/.omega/`. Projects live at `~/.omega/projects/<name>/`. Read existing projects to understand what already exists.

4. **Decision Logic**: Based on the input specificity:
   - If the user's description is specific enough (profession, location/context, concrete needs): skip questions, produce proposal directly
   - If vague: output 2-4 questions (NEVER more than 4)
   - If this is a follow-up round with accumulated answers: produce the proposal

5. **Output Format -- Questioning Mode**:
   ```
   SETUP_QUESTIONS
   <2-4 natural-language questions to understand the domain>
   ```

6. **Output Format -- Proposal Mode**:
   ```
   SETUP_PROPOSAL
   Project: <name>
   Domain: <one-line description>

   What I'll create:
   - Project: ~/.omega/projects/<name>/
   - ROLE.md: <2-3 sentence summary of domain expertise>
   - HEARTBEAT.md: <1-2 monitoring items>
   - Schedules: <list of recurring tasks>

   SETUP_EXECUTE
   <instructions to self for execution mode -- not shown to user>
   ```

7. **Output Format -- Execution Mode** (invoked after approval):
   The Brain receives `EXECUTE_SETUP` in the prompt. It must:
   - Create directory `~/.omega/projects/<name>/`
   - Write `~/.omega/projects/<name>/ROLE.md` with full domain expertise
   - Write `~/.omega/projects/<name>/HEARTBEAT.md` with monitoring checklist
   - Output markers at the end:
     ```
     SCHEDULE_ACTION: <description> | <ISO datetime> | <repeat>
     PROJECT_ACTIVATE: <name>
     ```

8. **ROLE.md Quality Requirements**:
   - MUST contain: domain context, operational rules, knowledge areas, safety constraints
   - MUST be parseable by `load_projects()` (plain markdown or optional YAML frontmatter with `skills: []`)
   - MUST be specific to the user's domain (not generic advice)
   - Structure: Identity section, Core Responsibilities, Operational Rules, Knowledge Areas, Communication Style, Safety/Constraints
   - Length: 80-200 lines (substantial but focused)

9. **HEARTBEAT.md Quality Requirements**:
   - Markdown checklist format, compatible with heartbeat loop
   - 2-5 domain-specific monitoring items
   - Each item should be actionable (check X, remind about Y)
   - Example format:
     ```markdown
     # <Project Name> Heartbeat Checklist

     ## Monitoring Items
     - Check for <domain-specific condition>
     - Remind about <recurring domain obligation>
     ```

10. **SCHEDULE_ACTION Marker Format**:
    ```
    SCHEDULE_ACTION: <description> | <ISO 8601 datetime> | <repeat: daily|weekly|monthly|none>
    ```
    - Description: action task instruction (will be executed by OMEGA's action scheduler)
    - Datetime: next occurrence in ISO 8601 (e.g., `2026-03-01T08:00:00`)
    - Repeat: `daily`, `weekly`, `monthly`, or `none`
    - Emit 1-3 schedules per setup (not zero, not more than 5)

11. **Collision Handling**: If the prompt context lists an existing project with the same name:
    - Propose updating/extending the existing project
    - Do NOT create a duplicate directory
    - In execution mode: read the existing ROLE.md, merge new content, write back

12. **Skill Suggestions**: After the proposal, optionally suggest relevant skills from `~/.omega/skills/` if they exist. Informational only -- do NOT emit install markers.

13. **Examples of Excellent ROLE.md Files**:
    Include 2 examples in the agent prompt:
    - The trader ROLE.md (abbreviated, as reference for structure and quality)
    - A synthetic realtor example (demonstrating domain-specific knowledge)

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Brain produces garbage | Bad prompt, model hallucination | Output parsing fails (no SETUP_PROPOSAL/SETUP_QUESTIONS) | Return error message, clean up session | User retries |
| Brain creates files in wrong location | Path confusion | File existence check after execution | Clean up orphaned files | Temporary disk waste |
| Brain emits malformed markers | Wrong format | `parse_schedule_action_line()` returns None | Markers silently skipped, user notified | Missing schedules |
| Brain produces shallow ROLE.md | Insufficient domain knowledge | No automated detection (quality is subjective) | User can edit ROLE.md manually | Mediocre domain expertise |

#### Security Considerations
- **No Bash tool**: Brain cannot execute arbitrary commands
- **No Edit tool**: Brain cannot modify existing files (only create new ones via Write)
- **Workspace boundary**: Agent runs in `~/.omega/`, which is the correct scope
- **Sandbox still active**: OS-level sandbox blocks writes to protected paths
- **Prompt injection**: User's business description is sanitized before reaching the Brain (sanitize step happens earlier in pipeline)

---

### Module 3: Agent Lifecycle Extension (`backend/src/gateway/builds_agents.rs`)

- **Responsibility**: Add `BRAIN_AGENT` constant and `write_single()` method to `AgentFilesGuard`
- **Public interface**: `pub(super) const BRAIN_AGENT`, `pub(super) async fn write_single()`
- **Dependencies**: `topologies/development/agents/omega-brain.md` file
- **Implementation order**: 3

#### Changes Required

```rust
// New constant (at end of constants block, before BUILD_AGENTS):
pub(super) const BRAIN_AGENT: &str =
    include_str!("../../../topologies/development/agents/omega-brain.md");

// New method on AgentFilesGuard:
impl AgentFilesGuard {
    /// Write a single agent file to `<project_dir>/.claude/agents/`.
    ///
    /// Used by the Brain setup flow which only needs one agent,
    /// not the full build topology. Same RAII cleanup behavior.
    pub(super) async fn write_single(
        project_dir: &Path,
        agent_name: &str,
        content: &str,
    ) -> std::io::Result<Self> {
        let agents_dir = project_dir.join(".claude").join("agents");
        tokio::fs::create_dir_all(&agents_dir).await?;
        let path = agents_dir.join(format!("{agent_name}.md"));
        tokio::fs::write(&path, content).await?;
        let mut counts = GUARD_REFCOUNTS.lock().unwrap();
        *counts.entry(agents_dir.clone()).or_insert(0) += 1;
        Ok(Self { agents_dir })
    }
}
```

#### Line Count Impact
- Production code: ~159 lines currently. Adding ~20 lines = ~179 lines. Well within 500-line limit.
- Test code: ~624 lines currently. Adding ~60 lines of Brain tests = ~684 lines total. File total ~863 lines, but production code is under 200.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| `include_str!()` file missing | Agent file not in repo | Compile-time error | Developer adds the file | Build fails -- caught in CI |
| Filesystem write fails | Permissions, disk full | `std::io::Error` propagated | Caller handles error, sends message to user | Setup fails gracefully |
| Ref count leak | Guard not dropped (task cancelled) | Memory leak (minor) | Orphaned files cleaned on next run | Stale agent file on disk |

---

### Module 4: Brain Orchestrator (`backend/src/gateway/setup.rs` + `setup_response.rs`)

- **Responsibility**: Session lifecycle management -- starting, continuing, completing, and cleaning up Brain setup sessions
- **Public interface**: `pub(super)` methods on `Gateway` split across two files:
  - `setup.rs`: `start_setup_session()`, `execute_setup()`, `cleanup_setup_session()`, `audit_setup()`
  - `setup_response.rs`: `handle_setup_response()` (dispatches to `handle_setup_confirmation()` and `handle_setup_questioning()`)
- **Dependencies**: `builds_agents.rs` (write_single), `builds.rs` (run_build_phase), `keywords.rs` (localized messages, confirm/cancel checks), `omega-skills/projects.rs` (load_projects), `process_markers.rs` (marker processing)
- **Implementation order**: 5

#### Struct Definitions

No new structs needed. Session state is stored via:
1. **Fact**: `pending_setup` with value `<timestamp>|<sender_id>|<round>` -- tracks active session
2. **Context file**: `<data_dir>/setup/<sender_id>.md` -- accumulates conversation context across rounds

This follows the exact same pattern as `pending_discovery` + discovery context file.

#### Method Signatures

```rust
//! Brain setup session — `/setup` command orchestrator.

use std::path::PathBuf;
use omega_core::config::shellexpand;
use omega_core::message::IncomingMessage;
use omega_memory::audit::{AuditEntry, AuditStatus};
use tracing::{info, warn};

use super::builds_agents::{AgentFilesGuard, BRAIN_AGENT};
use super::keywords::*;
use super::Gateway;

/// Path to the setup context file for a given sender.
pub(super) fn setup_context_path(data_dir: &str, sender_id: &str) -> PathBuf {
    PathBuf::from(shellexpand(data_dir))
        .join("setup")
        .join(format!("{sender_id}.md"))
}

/// Parse the current round number from a setup context file's header.
/// Returns 0 if no ROUND: header is found.
pub(super) fn parse_setup_round(content: &str) -> u8 {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("ROUND:") {
            return rest.trim().parse().unwrap_or(0);
        }
    }
    0
}

/// Parse Brain output to determine if it's questions or a proposal.
pub(super) enum SetupOutput {
    /// Brain needs more information -- contains question text.
    Questions(String),
    /// Brain is ready -- contains the full proposal text.
    Proposal(String),
    /// Brain is in execution mode -- contains markers and status.
    Executed(String),
}

/// Parse raw Brain output into a SetupOutput variant.
pub(super) fn parse_setup_output(output: &str) -> SetupOutput {
    if output.contains("SETUP_QUESTIONS") {
        let questions = output
            .split("SETUP_QUESTIONS")
            .nth(1)
            .unwrap_or("")
            .trim()
            .to_string();
        SetupOutput::Questions(questions)
    } else if output.contains("SETUP_PROPOSAL") {
        SetupOutput::Proposal(output.to_string())
    } else {
        // Execution mode output (contains created files and markers).
        SetupOutput::Executed(output.to_string())
    }
}

impl Gateway {
    /// Start a new setup session from a `/setup <description>` command.
    ///
    /// Steps:
    /// 1. Check for concurrent session (guard REQ-BRAIN-023)
    /// 2. Load existing projects for collision detection (REQ-BRAIN-005)
    /// 3. Read existing ROLE.md files for context (REQ-BRAIN-019)
    /// 4. Build Brain prompt with context
    /// 5. Write single agent file (REQ-BRAIN-003)
    /// 6. Invoke run_build_phase (REQ-BRAIN-004)
    /// 7. Parse output: questions -> store session; proposal -> show to user
    pub(super) async fn start_setup_session(
        &self,
        incoming: &IncomingMessage,
        description: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        // ... implementation ...
    }

    /// Handle a follow-up message during an active setup session.
    ///
    /// Routes to: cancel, confirm (execute), modification (new round), or expiry.
    pub(super) async fn handle_setup_response(
        &self,
        incoming: &IncomingMessage,
        setup_value: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        // ... implementation ...
    }

    /// Execute the approved setup: run Brain in execution mode.
    ///
    /// Brain creates ROLE.md + HEARTBEAT.md files, emits markers.
    /// Markers are processed by existing process_markers().
    pub(super) async fn execute_setup(
        &self,
        incoming: &IncomingMessage,
        proposal_context: &str,
    ) -> Result<String, String> {
        // ... implementation ...
    }

    /// Clean up session state (fact + context file).
    pub(super) async fn cleanup_setup_session(&self, sender_id: &str) {
        let _ = self.memory.delete_fact(sender_id, "pending_setup").await;
        let ctx_file = setup_context_path(&self.data_dir, sender_id);
        let _ = tokio::fs::remove_file(&ctx_file).await;
    }

    /// Log an audit entry for a setup operation.
    pub(super) async fn audit_setup(
        &self,
        incoming: &IncomingMessage,
        project: &str,
        status: &str,
        detail: &str,
    ) {
        let _ = self.audit.log(&AuditEntry {
            channel: incoming.channel.clone(),
            sender_id: incoming.sender_id.clone(),
            sender_name: incoming.sender_name.clone(),
            input_text: format!("[SETUP:{project}] {}", incoming.text),
            output_text: Some(format!("[{status}] {detail}")),
            provider_used: Some(self.provider.name().to_string()),
            model: None,
            processing_ms: None,
            status: if status == "complete" || status == "started" {
                AuditStatus::Ok
            } else {
                AuditStatus::Error
            },
            denial_reason: None,
        }).await;
    }
}
```

#### Detailed Flow: `start_setup_session()`

```
1. Check for existing pending_setup fact for this sender_id
   - If exists and not expired: send "You have an active setup session" message, return
   - If exists and expired: clean up old session, continue

2. Load existing projects: let projects = omega_skills::load_projects(&self.data_dir);

3. Build existing project context:
   - For each project: read its ROLE.md (first 50 lines) and list in context
   - Format: "Existing projects:\n- trader: [first line of ROLE.md]\n- restaurant: [first line]"

4. Build Brain prompt:
   "Setup round 1. The user wants to configure OMEGA for their domain.
    User description: <description>

    Existing projects:
    <project list with summaries>

    Analyze the description. If specific enough, output SETUP_PROPOSAL.
    If you need more information, output SETUP_QUESTIONS (2-4 questions max)."

5. Write agent file:
   let omega_dir = PathBuf::from(shellexpand(&self.data_dir));
   let _agent_guard = AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT).await?;

6. Invoke Brain:
   let result = self.run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30)).await;

7. Parse output:
   match parse_setup_output(&output) {
       SetupOutput::Questions(questions) => {
           // Create context file with round 1 content
           // Store pending_setup fact
           // Send questions to user
       }
       SetupOutput::Proposal(proposal) => {
           // Extract user-facing preview (before SETUP_EXECUTE)
           // Store pending_setup fact with round=proposal
           // Store full proposal in context file
           // Send proposal + confirmation prompt to user
       }
       SetupOutput::Executed(_) => {
           // Should not happen in questioning mode -- treat as error
       }
   }
```

#### Detailed Flow: `handle_setup_response()`

```
1. Parse pending_setup value: "<timestamp>|<sender_id>|<round>"

2. Check TTL (30 minutes):
   - If expired: cleanup, send expired message, fall through to normal pipeline

3. Check for cancellation:
   - If is_build_cancelled(msg): cleanup, send cancelled message, return

4. Parse current phase from context file:
   - If context contains "SETUP_PROPOSAL" -> we're in confirmation phase
   - Otherwise -> we're in questioning phase

5a. CONFIRMATION PHASE:
   - If is_build_confirmed(msg): execute_setup(), process markers, cleanup, send success
   - Else: treat as modification request -> append to context, run Brain again

5b. QUESTIONING PHASE:
   - Append user answer to context file
   - Increment round counter
   - If round >= 3: force final proposal (add "FINAL ROUND" to prompt)
   - Run Brain with accumulated context
   - Parse output: Questions -> update file, send questions; Proposal -> send proposal
```

#### Detailed Flow: `execute_setup()`

```
1. Build execution prompt:
   "EXECUTE_SETUP. Create all files and emit markers.
    <full accumulated context with approved proposal>"

2. Write agent file:
   let _agent_guard = AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT).await?;

3. Invoke Brain:
   let result = self.run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30)).await;

4. Verify files were created:
   - Check ~/.omega/projects/<name>/ROLE.md exists
   - Check ~/.omega/projects/<name>/HEARTBEAT.md exists (warning if missing, not fatal)

5. Return output text (contains markers for process_markers())
```

#### Line Count Estimate
- `setup.rs`: ~250-350 lines production code. Well within 500-line limit.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Provider timeout | Claude Code subprocess hangs | `run_build_phase` 3-retry pattern | After 3 failures, error message + cleanup | User retries |
| Session timeout | User takes >30 min to respond | TTL check on next message | Session expired, cleanup, notify user | User re-invokes /setup |
| Context file missing | Filesystem error, race condition | `read_to_string` returns error | Treat as expired session, cleanup | User retries |
| Partial creation | Brain writes ROLE.md, crashes before HEARTBEAT.md | File existence checks after execution | ROLE.md exists (project is functional), HEARTBEAT.md missing (logged as warning) | Project works but no heartbeat |
| Concurrent sessions | User sends /setup while session active | Check pending_setup fact first | Reject with "session already active" message | No data corruption |
| Marker processing fails | Malformed markers from Brain | parse_schedule_action_line returns None | Markers silently skipped (existing behavior) | Missing schedules, user can add manually |

#### Security Considerations
- **User input sanitization**: Already handled by pipeline step 2 before reaching setup
- **Path traversal**: Project names derived from Brain output -- validated by Brain prompt rules (alphanumeric + hyphen only, no dots, no slashes)
- **Fact injection**: `pending_setup` uses the system key pattern -- not writable by the AI provider
- **File overwrite protection**: Brain prompt instructs "merge, don't overwrite" for existing projects; OS sandbox provides additional protection

#### Performance Budget
- **Latency**: 15-90 seconds for Brain invocation (Claude Code subprocess with Opus model). Acceptable for a setup flow.
- **Memory**: Single agent file (~10KB) written to disk, context file (~5KB). Negligible.
- **Disk**: Project files ~20KB total (ROLE.md + HEARTBEAT.md). Negligible.
- **Concurrency**: Only one active setup session per user. No global lock needed.

---

### Module 5: Pipeline Integration (`backend/src/gateway/pipeline.rs`)

- **Responsibility**: Intercept `/setup` command and route `pending_setup` session messages
- **Public interface**: No new public interface -- modifies `handle_message()` flow
- **Dependencies**: `setup.rs` (start_setup_session), `setup_response.rs` (handle_setup_response), `commands/mod.rs` (Command::Setup)
- **Implementation order**: 6

#### Integration Points

There are exactly 2 integration points in `pipeline.rs`:

**Point 1: `/setup` command intercept (in step 3, command dispatch)**

Insert AFTER the `/forget` early return, BEFORE the normal command dispatch:

```rust
// --- 3. COMMAND DISPATCH ---
let projects = omega_skills::load_projects(&self.data_dir);
if let Some(cmd) = commands::Command::parse(&clean_incoming.text) {
    if matches!(cmd, commands::Command::Forget) {
        // ... existing /forget handling ...
        return;
    }

    // NEW: /setup intercept -- delegate to Brain orchestrator
    if matches!(cmd, commands::Command::Setup) {
        let description = clean_incoming
            .text
            .strip_prefix("/setup")
            .or_else(|| {
                // Handle /setup@botname
                clean_incoming.text.split_whitespace()
                    .next()
                    .and_then(|first| {
                        if first.starts_with("/setup") {
                            Some(&clean_incoming.text[first.len()..])
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or("")
            .trim();

        if description.is_empty() {
            let user_lang = self.memory
                .get_fact(&incoming.sender_id, "preferred_language")
                .await.ok().flatten()
                .unwrap_or_else(|| "English".to_string());
            if let Some(h) = typing_handle {
                h.abort();
            }
            self.send_text(&incoming, setup_help_message(&user_lang)).await;
            return;
        }

        self.start_setup_session(&incoming, description, typing_handle).await;
        return;
    }

    // ... existing command dispatch continues ...
}
```

**Point 2: `pending_setup` session check (in step 4a, BEFORE pending_discovery)**

Insert a new block BETWEEN "--- 4. TYPING INDICATOR ---" and "--- 4a-DISCOVERY. PENDING DISCOVERY SESSION CHECK ---":

```rust
// --- 4a-SETUP. PENDING SETUP SESSION CHECK ---
let pending_setup: Option<String> = self
    .memory
    .get_fact(&incoming.sender_id, "pending_setup")
    .await
    .ok()
    .flatten();

if pending_setup.is_some() {
    self.handle_setup_response(
        &incoming,
        &pending_setup.unwrap(),
        typing_handle,
    ).await;
    return;
}

// --- 4a-DISCOVERY. PENDING DISCOVERY SESSION CHECK ---
// ... existing discovery check ...
```

**Ordering rationale**: Setup check comes BEFORE discovery check. If a user has both `pending_setup` and `pending_discovery` (which should be impossible via guards, but defense-in-depth), setup takes priority because it's the more focused interaction.

#### Line Count Impact
- Adding ~35 lines to `pipeline.rs`. File is already at ~942 lines (no tests), which exceeds the 500-line rule. However, the requirement document explicitly locates this integration in `pipeline.rs`, and moving the entire pipeline to submodules is out of scope for this feature. The 35-line addition is minimal and follows existing patterns exactly.

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Stale pending_setup fact | Process crash during session | TTL check (30 min) | Next message triggers expiry cleanup | User sees "expired" message once |
| Both pending_setup and pending_discovery exist | Bug or race condition | Setup checked first | Setup takes priority, discovery cleaned up on next interaction | Minor UX confusion |

---

### Module 6: Localized Messages (`backend/src/gateway/keywords.rs`)

- **Responsibility**: Setup session TTL constant and all user-facing localized messages for the setup flow
- **Public interface**: Constants and functions used by `setup.rs` and `pipeline.rs`
- **Dependencies**: None
- **Implementation order**: 4

#### Constants and Functions to Add

```rust
/// Maximum seconds a setup session stays valid.
pub(super) const SETUP_TTL_SECS: i64 = 1800; // 30 minutes

/// Localized help message when /setup is invoked with no description.
pub(super) fn setup_help_message(lang: &str) -> &'static str {
    match lang {
        "Spanish" => "Usa /setup seguido de una descripci\u{f3}n de tu negocio.\n\nEjemplo: /setup Soy agente inmobiliario en Lisboa",
        "Portuguese" => "Use /setup seguido de uma descri\u{e7}\u{e3}o do seu neg\u{f3}cio.\n\nExemplo: /setup Sou corretor de im\u{f3}veis em Lisboa",
        "French" => "Utilise /setup suivi d'une description de ton activit\u{e9}.\n\nExemple : /setup Je suis agent immobilier \u{e0} Lisbonne",
        "German" => "Verwende /setup gefolgt von einer Beschreibung deines Gesch\u{e4}fts.\n\nBeispiel: /setup Ich bin Immobilienmakler in Lissabon",
        "Italian" => "Usa /setup seguito da una descrizione della tua attivit\u{e0}.\n\nEsempio: /setup Sono un agente immobiliare a Lisbona",
        "Dutch" => "Gebruik /setup gevolgd door een beschrijving van je bedrijf.\n\nVoorbeeld: /setup Ik ben makelaar in Lissabon",
        "Russian" => "\u{418}\u{441}\u{43f}\u{43e}\u{43b}\u{44c}\u{437}\u{443}\u{439}\u{442}\u{435} /setup \u{441} \u{43e}\u{43f}\u{438}\u{441}\u{430}\u{43d}\u{438}\u{435}\u{43c} \u{432}\u{430}\u{448}\u{435}\u{433}\u{43e} \u{431}\u{438}\u{437}\u{43d}\u{435}\u{441}\u{430}.\n\n\u{41f}\u{440}\u{438}\u{43c}\u{435}\u{440}: /setup \u{42f} \u{440}\u{438}\u{44d}\u{43b}\u{442}\u{43e}\u{440} \u{432} \u{41b}\u{438}\u{441}\u{441}\u{430}\u{431}\u{43e}\u{43d}\u{435}",
        _ => "Use /setup followed by a description of your business.\n\nExample: /setup I'm a realtor in Lisbon",
    }
}

/// Localized intro message when Brain asks questions (round 1).
pub(super) fn setup_intro_message(lang: &str, questions: &str) -> String {
    let intro = match lang {
        "Spanish" => "Para configurar OMEGA como tu experto, necesito entender mejor tu negocio:",
        "Portuguese" => "Para configurar OMEGA como seu especialista, preciso entender melhor seu neg\u{f3}cio:",
        "French" => "Pour configurer OMEGA comme ton expert, j'ai besoin de mieux comprendre ton activit\u{e9} :",
        "German" => "Um OMEGA als deinen Experten einzurichten, muss ich dein Gesch\u{e4}ft besser verstehen:",
        "Italian" => "Per configurare OMEGA come tuo esperto, ho bisogno di capire meglio la tua attivit\u{e0}:",
        "Dutch" => "Om OMEGA als je expert in te richten, moet ik je bedrijf beter begrijpen:",
        "Russian" => "\u{427}\u{442}\u{43e}\u{431}\u{44b} \u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{438}\u{442}\u{44c} OMEGA \u{43a}\u{430}\u{43a} \u{432}\u{430}\u{448}\u{435}\u{433}\u{43e} \u{44d}\u{43a}\u{441}\u{43f}\u{435}\u{440}\u{442}\u{430}, \u{43c}\u{43d}\u{435} \u{43d}\u{443}\u{436}\u{43d}\u{43e} \u{43b}\u{443}\u{447}\u{448}\u{435} \u{43f}\u{43e}\u{43d}\u{44f}\u{442}\u{44c} \u{432}\u{430}\u{448} \u{431}\u{438}\u{437}\u{43d}\u{435}\u{441}:",
        _ => "To configure OMEGA as your domain expert, I need to understand your business better:",
    };
    format!("{intro}\n\n{questions}")
}

/// Localized follow-up message for setup rounds 2-3.
pub(super) fn setup_followup_message(lang: &str, questions: &str, round: u8) -> String {
    let followup = match lang {
        "Spanish" => format!("Gracias. Unas preguntas m\u{e1}s ({round}/3):"),
        "Portuguese" => format!("Obrigado. Mais algumas perguntas ({round}/3):"),
        "French" => format!("Merci. Encore quelques questions ({round}/3) :"),
        "German" => format!("Danke. Noch ein paar Fragen ({round}/3):"),
        "Italian" => format!("Grazie. Ancora qualche domanda ({round}/3):"),
        "Dutch" => format!("Bedankt. Nog een paar vragen ({round}/3):"),
        "Russian" => format!("\u{421}\u{43f}\u{430}\u{441}\u{438}\u{431}\u{43e}. \u{415}\u{449}\u{451} \u{43d}\u{435}\u{441}\u{43a}\u{43e}\u{43b}\u{44c}\u{43a}\u{43e} \u{432}\u{43e}\u{43f}\u{440}\u{43e}\u{441}\u{43e}\u{432} ({round}/3):"),
        _ => format!("Thanks. A few more questions ({round}/3):"),
    };
    format!("{followup}\n\n{questions}")
}

/// Localized proposal message with confirmation prompt.
pub(super) fn setup_proposal_message(lang: &str, proposal_preview: &str) -> String {
    match lang {
        "Spanish" => format!(
            "Esto es lo que configurar\u{e9}:\n\n\
             {proposal_preview}\n\n\
             Responde *s\u{ed}* para crear todo, *no* para cancelar, o escribe cambios."
        ),
        "Portuguese" => format!(
            "Isto \u{e9} o que vou configurar:\n\n\
             {proposal_preview}\n\n\
             Responda *sim* para criar tudo, *n\u{e3}o* para cancelar, ou escreva altera\u{e7}\u{f5}es."
        ),
        "French" => format!(
            "Voici ce que je vais configurer :\n\n\
             {proposal_preview}\n\n\
             R\u{e9}ponds *oui* pour tout cr\u{e9}er, *non* pour annuler, ou \u{e9}cris des modifications."
        ),
        "German" => format!(
            "Das werde ich einrichten:\n\n\
             {proposal_preview}\n\n\
             Antworte *ja* um alles zu erstellen, *nein* zum Abbrechen, oder schreibe \u{c4}nderungen."
        ),
        "Italian" => format!(
            "Ecco cosa configurer\u{f2}:\n\n\
             {proposal_preview}\n\n\
             Rispondi *s\u{ec}* per creare tutto, *no* per annullare, o scrivi modifiche."
        ),
        "Dutch" => format!(
            "Dit ga ik instellen:\n\n\
             {proposal_preview}\n\n\
             Antwoord *ja* om alles aan te maken, *nee* om te annuleren, of schrijf wijzigingen."
        ),
        "Russian" => format!(
            "\u{412}\u{43e}\u{442} \u{447}\u{442}\u{43e} \u{44f} \u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{44e}:\n\n\
             {proposal_preview}\n\n\
             \u{41e}\u{442}\u{432}\u{435}\u{442}\u{44c}\u{442}\u{435} *\u{434}\u{430}* \u{447}\u{442}\u{43e}\u{431}\u{44b} \u{441}\u{43e}\u{437}\u{434}\u{430}\u{442}\u{44c} \u{432}\u{441}\u{451}, *\u{43d}\u{435}\u{442}* \u{434}\u{43b}\u{44f} \u{43e}\u{442}\u{43c}\u{435}\u{43d}\u{44b}, \u{438}\u{43b}\u{438} \u{43d}\u{430}\u{43f}\u{438}\u{448}\u{438}\u{442}\u{435} \u{438}\u{437}\u{43c}\u{435}\u{43d}\u{435}\u{43d}\u{438}\u{44f}."
        ),
        _ => format!(
            "Here's what I'll set up:\n\n\
             {proposal_preview}\n\n\
             Reply *yes* to create everything, *no* to cancel, or describe changes."
        ),
    }
}

/// Localized setup completion message.
pub(super) fn setup_complete_message(lang: &str, project_name: &str) -> String {
    match lang {
        "Spanish" => format!("Proyecto *{project_name}* configurado y activado. OMEGA ahora es tu experto en este dominio."),
        "Portuguese" => format!("Projeto *{project_name}* configurado e ativado. OMEGA agora \u{e9} seu especialista neste dom\u{ed}nio."),
        "French" => format!("Projet *{project_name}* configur\u{e9} et activ\u{e9}. OMEGA est maintenant ton expert dans ce domaine."),
        "German" => format!("Projekt *{project_name}* eingerichtet und aktiviert. OMEGA ist jetzt dein Experte in diesem Bereich."),
        "Italian" => format!("Progetto *{project_name}* configurato e attivato. OMEGA ora \u{e8} il tuo esperto in questo dominio."),
        "Dutch" => format!("Project *{project_name}* ingesteld en geactiveerd. OMEGA is nu je expert in dit domein."),
        "Russian" => format!("\u{41f}\u{440}\u{43e}\u{435}\u{43a}\u{442} *{project_name}* \u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{435}\u{43d} \u{438} \u{430}\u{43a}\u{442}\u{438}\u{432}\u{438}\u{440}\u{43e}\u{432}\u{430}\u{43d}. OMEGA \u{442}\u{435}\u{43f}\u{435}\u{440}\u{44c} \u{432}\u{430}\u{448} \u{44d}\u{43a}\u{441}\u{43f}\u{435}\u{440}\u{442} \u{432} \u{44d}\u{442}\u{43e}\u{439} \u{43e}\u{431}\u{43b}\u{430}\u{441}\u{442}\u{438}."),
        _ => format!("Project *{project_name}* configured and activated. OMEGA is now your domain expert."),
    }
}

/// Localized setup cancelled message.
pub(super) fn setup_cancelled_message(lang: &str) -> &'static str {
    match lang {
        "Spanish" => "Configuraci\u{f3}n cancelada.",
        "Portuguese" => "Configura\u{e7}\u{e3}o cancelada.",
        "French" => "Configuration annul\u{e9}e.",
        "German" => "Einrichtung abgebrochen.",
        "Italian" => "Configurazione annullata.",
        "Dutch" => "Instelling geannuleerd.",
        "Russian" => "\u{41d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{430} \u{43e}\u{442}\u{43c}\u{435}\u{43d}\u{435}\u{43d}\u{430}.",
        _ => "Setup cancelled.",
    }
}

/// Localized setup expired message.
pub(super) fn setup_expired_message(lang: &str) -> &'static str {
    match lang {
        "Spanish" => "La sesi\u{f3}n de configuraci\u{f3}n expir\u{f3}. Usa /setup para empezar de nuevo.",
        "Portuguese" => "A sess\u{e3}o de configura\u{e7}\u{e3}o expirou. Use /setup para come\u{e7}ar novamente.",
        "French" => "La session de configuration a expir\u{e9}. Utilise /setup pour recommencer.",
        "German" => "Die Einrichtungssitzung ist abgelaufen. Verwende /setup um neu zu starten.",
        "Italian" => "La sessione di configurazione \u{e8} scaduta. Usa /setup per ricominciare.",
        "Dutch" => "De instellingssessie is verlopen. Gebruik /setup om opnieuw te beginnen.",
        "Russian" => "\u{421}\u{435}\u{441}\u{441}\u{438}\u{44f} \u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438} \u{438}\u{441}\u{442}\u{435}\u{43a}\u{43b}\u{430}. \u{418}\u{441}\u{43f}\u{43e}\u{43b}\u{44c}\u{437}\u{443}\u{439}\u{442}\u{435} /setup \u{447}\u{442}\u{43e}\u{431}\u{44b} \u{43d}\u{430}\u{447}\u{430}\u{442}\u{44c} \u{437}\u{430}\u{43d}\u{43e}\u{432}\u{43e}.",
        _ => "Setup session expired. Use /setup to start again.",
    }
}
```

#### Line Count Impact
- Adding ~120 lines of functions to `keywords.rs`. Production code goes from ~597 to ~717 lines. This exceeds 500 lines.
- **Recommendation**: Extract ALL setup keyword functions into a new file `backend/src/gateway/setup_i18n.rs` (following the `builds_i18n.rs` pattern). This keeps both files under limit and follows existing conventions.

Alternative structure:
```
backend/src/gateway/setup_i18n.rs  (~130 lines) -- all setup localized messages
backend/src/gateway/keywords.rs   (~600 lines) -- only SETUP_TTL_SECS constant added
```

#### Failure Modes
| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Missing language | User has unsupported language | Default English fallback | Always works | Non-localized message |

---

### Module 7: Gateway Module Registration (`backend/src/gateway/mod.rs`)

- **Responsibility**: Register new submodules
- **Public interface**: `mod setup;` and `mod setup_i18n;` declarations
- **Dependencies**: None
- **Implementation order**: 4 (with setup_i18n.rs)

#### Changes Required

```rust
// In gateway/mod.rs, add after existing mod declarations:
mod setup;
mod setup_i18n;
```

---

## Session State Machine

```
                    /setup <desc>
                        |
                        v
                  +-----------+
                  | IDLE      |
                  +-----------+
                        |
           Brain returns questions?  ----yes----> QUESTIONING
                        |                              |
                       no                     user answers
                        |                              |
                        v                    Brain returns questions?
                  +-----------+                  |           |
                  | PROPOSAL  |<---yes-----------+          no
                  +-----------+                              |
                   /    |     \                              v
                  /     |      \                      +-----+-----+
           "yes"    "no"  modification              | PROPOSAL  |
              |       |        |                     +-----------+
              v       v        v
         EXECUTING  CANCELLED  back to QUESTIONING/PROPOSAL
              |
              v
         COMPLETED (cleanup)
```

### State Encoding

State is encoded in two storage locations:

1. **Fact `pending_setup`**: `<timestamp>|<sender_id>`
   - Presence = session active
   - Timestamp = session start (for TTL check)
   - Absence = no active session

2. **Context file** (`<data_dir>/setup/<sender_id>.md`): Accumulated conversation
   - Contains `ROUND: <n>` header for round tracking
   - Contains `PHASE: questioning` or `PHASE: proposal` for state
   - Contains accumulated Q&A and proposal text

### Round Limits

| Round | Brain Behavior | Gateway Behavior |
|-------|---------------|-----------------|
| 1 | Questions OR proposal | Store state, send to user |
| 2 | Questions OR proposal | Store state, send to user |
| 3 | MUST produce proposal (forced) | If Brain outputs questions, treat as proposal |
| Approval | N/A | Confirm, cancel, or modification |
| Execute | Creates files, emits markers | Process markers, cleanup |

### TTL Enforcement

- Session TTL: 30 minutes (`SETUP_TTL_SECS = 1800`)
- Checked on every message during active session
- On expiry: cleanup session, send expired message, fall through to normal pipeline
- Same pattern as `DISCOVERY_TTL_SECS`

---

## Failure Modes (system-level)

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| Provider (Claude Code) unavailable | setup.rs, pipeline.rs | `run_build_phase` fails after 3 retries | Error message to user, cleanup session | User retries later |
| Provider returns garbage | setup.rs | `parse_setup_output` can't find markers | Default to treating output as proposal | User sees raw Brain output, can cancel |
| Disk full | setup.rs, Brain agent | `tokio::fs::write` returns error | Error message, cleanup session | No files created |
| Context file corrupted | setup.rs | Parse failure on round/phase | Treat as round 1 (restart session) | User may need to re-answer questions |
| ROLE.md written but HEARTBEAT.md not | Brain agent | Post-execution file existence check | Log warning, proceed (project still works) | No heartbeat for this project |
| Markers malformed | process_markers.rs | `parse_schedule_action_line` returns None | Silently skipped (existing behavior) | Missing schedules |
| Concurrent /setup and build discovery | pipeline.rs | `pending_setup` checked before `pending_discovery` | Setup takes priority | Unlikely scenario, no data loss |
| Process crash during execution | All | Orphaned `pending_setup` fact | TTL expiry on restart (30 min) | Files may be partially created |

---

## Security Model

### Trust Boundaries

- **User input -> Brain prompt**: User's business description is sanitized by the existing sanitizer (step 2 in pipeline). The description is embedded in a structured prompt template, reducing injection risk.
- **Brain output -> filesystem**: Brain writes files via Claude Code's Write tool. OS-level sandbox (Seatbelt/Landlock) prevents writes outside `~/.omega/`. The Brain has no Bash tool.
- **Brain output -> markers**: Markers are parsed by existing strict parsers (`parse_schedule_action_line`, `extract_project_activate`). Malformed markers are silently dropped.

### Data Classification

| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| Business description | Internal | Context file (temp) | Deleted after session |
| ROLE.md content | Internal | `~/.omega/projects/` | User-readable, bot-readable |
| HEARTBEAT.md content | Internal | `~/.omega/projects/` | Heartbeat loop reads |
| Session state (pending_setup) | Internal | SQLite facts table | Per-user, TTL-bounded |

### Attack Surface

- **Prompt injection via /setup description**: Risk: User crafts description to make Brain write malicious ROLE.md. Mitigation: Sanitization in step 2, Brain prompt focuses on domain expertise, sandbox limits filesystem access.
- **Brain escaping workspace**: Risk: Brain writes files outside `~/.omega/`. Mitigation: No Bash tool, OS sandbox, Write tool scope limited to workspace.
- **Session hijacking**: Risk: Another user's message routed to wrong session. Mitigation: `pending_setup` keyed by `sender_id`, unique per user.

---

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| Claude Code CLI | Brain runs, creates files | 3-retry failure, error message | User retries /setup |
| Filesystem (~/.omega/) | Files created successfully | Error message, session cleanup | User retries after fixing disk |
| SQLite (facts table) | Session state stored/retrieved | Fact operations fail silently | Session lost, user retries |

---

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| `/setup` command parse | <1ms | <1ms | Negligible | String matching |
| Brain invocation (per round) | 20s | 90s | ~50MB (subprocess) | Claude Code Opus model |
| File creation | <10ms | <50ms | Negligible | Small files |
| Marker processing | <5ms | <10ms | Negligible | Existing path |
| Session cleanup | <10ms | <50ms | Negligible | Delete fact + file |
| Total setup (happy path) | 30s | 120s | ~50MB peak | 1-2 Brain invocations |
| Total setup (multi-round) | 60s | 300s | ~50MB peak | 3 rounds + execution |

---

## Data Flow

```
/setup <description>
    |
    v
[pipeline.rs] Command::Setup intercept
    |
    +-- extract description text
    |
    v
[setup.rs] start_setup_session()
    |
    +-- load_projects() -> Vec<Project>
    +-- read existing ROLE.md content (first 50 lines each)
    +-- build prompt: description + existing projects
    |
    v
[builds_agents.rs] AgentFilesGuard::write_single()
    |
    +-- writes omega-brain.md to ~/.omega/.claude/agents/
    |
    v
[builds.rs] run_build_phase("omega-brain", prompt, model_complex, 30)
    |
    +-- Claude Code subprocess in ~/.omega/ workspace
    +-- Brain reads existing projects, decides questions vs proposal
    |
    v
[setup.rs] parse_setup_output()
    |
    +-- Questions? -> store context file, store pending_setup fact, send to user
    +-- Proposal? -> store context file, store pending_setup fact, send preview to user
    |
    v
[pipeline.rs] next message -> pending_setup check
    |
    +-- cancelled? -> cleanup_setup_session()
    +-- expired? -> cleanup_setup_session()
    +-- confirmed? -> execute_setup()
    +-- modification? -> append to context, run Brain again
    |
    v
[setup.rs] execute_setup()
    |
    +-- Brain creates ROLE.md and HEARTBEAT.md
    +-- Brain emits SCHEDULE_ACTION: and PROJECT_ACTIVATE: markers
    |
    v
[process_markers.rs] process_markers()
    |
    +-- SCHEDULE_ACTION: -> scheduled_tasks table
    +-- PROJECT_ACTIVATE: -> active_project fact
    |
    v
[setup.rs] cleanup_setup_session()
    +-- delete pending_setup fact
    +-- delete context file
    +-- send completion message
```

---

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| `/setup` command (explicit) | Keyword detection ("I'm a realtor") | Avoids ambiguity. Keywords like "I'm a realtor" could trigger Brain when user just wants to chat about real estate. Explicit command = explicit intent. |
| Separate `setup.rs` module | Inline in `pipeline.rs` | `pipeline.rs` is already 942 lines. Adding ~300 lines would make it unmaintainable. `setup.rs` follows the `builds.rs` delegation pattern. |
| `write_single()` method | Reuse `write_from_topology()` | Brain is one agent, not a topology. Loading a full topology for one agent is wasteful and adds unnecessary complexity. `write_single()` is simpler and more explicit. |
| Context file for session state | Database table | Requirements say "no new database tables." Context file matches the discovery pattern exactly. File is easier to debug (can cat it), and is cleaned up via RAII-like pattern. |
| 3-round limit | Unlimited rounds | Requirements specify max 3 rounds (REQ-BRAIN-010). Prevents runaway sessions and user fatigue. |
| Same confirm/cancel keywords as builds | Separate setup-specific keywords | BUILD_CONFIRM_KW and BUILD_CANCEL_KW already cover all 8 languages. Reusing them avoids duplication and provides consistent UX. Only context differs (checked during pending_setup vs pending_build). |
| Brain creates files directly (Write tool) | Gateway creates files from Brain output | Brain as Claude Code subprocess has Write tool. Letting it create files directly is simpler than parsing structured output and writing from Rust. The OS sandbox ensures safety. |
| `setup_i18n.rs` for localized messages | Add to `keywords.rs` | `keywords.rs` is already at ~600 production lines. Adding 120 more exceeds 500-line limit. Following `builds_i18n.rs` pattern keeps files modular. |
| Brain workspace is `~/.omega/` | `~/.omega/workspace/` | Projects live at `~/.omega/projects/`. Brain needs to write directly there. Using the build workspace would require moving files after creation. |
| Reuse `is_build_confirmed()` / `is_build_cancelled()` | New `is_setup_confirmed()` functions | Same confirmation words apply. The functions check exact match on short words like "yes", "si", "da" -- context-free and safe to reuse. |

---

## External Dependencies

No new external dependencies. All functionality uses existing crates:
- `tokio` -- async filesystem operations
- `chrono` -- timestamps for TTL
- `tracing` -- logging
- `omega-skills` -- `load_projects()`, `Project` struct
- `omega-memory` -- `Store` (facts), `AuditLogger`
- `omega-core` -- `shellexpand`, `IncomingMessage`, `Context`

---

## File Summary

| File | Action | Lines Added (est.) | Risk |
|------|--------|-------------------|------|
| `backend/src/commands/mod.rs` | Modify | +3 | Low |
| `backend/src/gateway/mod.rs` | Modify | +2 | Low |
| `backend/src/gateway/pipeline.rs` | Modify | +35 | Medium |
| `backend/src/gateway/builds_agents.rs` | Modify | +20 prod, +60 test | Low |
| `backend/src/gateway/setup.rs` | **New** | ~300 | Medium |
| `backend/src/gateway/setup_i18n.rs` | **New** | ~130 | Low |
| `topologies/development/agents/omega-brain.md` | **New** | ~250 | High (quality-critical) |

Total new/modified: ~800 lines across 7 files. No file exceeds 500 production lines.

---

## Requirement Traceability

| Requirement ID | Architecture Section | Module(s) |
|---------------|---------------------|-----------|
| REQ-BRAIN-001 | Module 1: Command Registration | `commands/mod.rs` |
| REQ-BRAIN-002 | Module 3: Agent Lifecycle Extension | `gateway/builds_agents.rs` |
| REQ-BRAIN-003 | Module 3: Agent Lifecycle Extension (write_single) | `gateway/builds_agents.rs` |
| REQ-BRAIN-004 | Module 4: Brain Orchestrator (start_setup_session, execute_setup) | `gateway/setup.rs` |
| REQ-BRAIN-005 | Module 4: Brain Orchestrator (load_projects context) | `gateway/setup.rs` |
| REQ-BRAIN-006 | Module 2: Brain Agent Definition (ROLE.md creation) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-007 | Module 2: Brain Agent Definition (HEARTBEAT.md creation) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-008 | Module 2: Brain Agent Definition (SCHEDULE_ACTION markers) | `topologies/.../omega-brain.md` + `process_markers.rs` |
| REQ-BRAIN-009 | Module 2: Brain Agent Definition (PROJECT_ACTIVATE marker) | `topologies/.../omega-brain.md` + `process_markers.rs` |
| REQ-BRAIN-010 | Session State Machine (3-round limit) | `gateway/setup.rs` |
| REQ-BRAIN-011 | Module 5: Pipeline Integration (Point 1) | `gateway/pipeline.rs` |
| REQ-BRAIN-012 | Module 4: Brain Orchestrator (pending_setup fact), Module 5: Pipeline Integration (Point 2) | `gateway/setup.rs` + `gateway/pipeline.rs` |
| REQ-BRAIN-013 | Module 6: Localized Messages + Design Decision (reuse BUILD_CONFIRM_KW) | `gateway/keywords.rs` |
| REQ-BRAIN-014 | Module 6: Localized Messages | `gateway/setup_i18n.rs` |
| REQ-BRAIN-015 | Module 2: Brain Agent Definition (examples section) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-016 | Module 7: Gateway Module Registration | `gateway/mod.rs` |
| REQ-BRAIN-017 | Module 2: Brain Agent Definition (skill suggestions) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-018 | Module 4: Brain Orchestrator (audit_setup) | `gateway/setup.rs` |
| REQ-BRAIN-019 | Module 4: Brain Orchestrator (read existing ROLE.md) | `gateway/setup.rs` |
| REQ-BRAIN-020 | Module 4: Brain Orchestrator (cleanup_setup_session) | `gateway/setup.rs` |
| REQ-BRAIN-021 | Module 2: Brain Agent Definition (tools restriction) | `topologies/.../omega-brain.md` |
| REQ-BRAIN-022 | Module 4: Brain Orchestrator (workspace path) | `gateway/setup.rs` |
| REQ-BRAIN-023 | Module 4: Brain Orchestrator (concurrent session guard) | `gateway/setup.rs` |
| REQ-BRAIN-024 | Out of scope | N/A |
| REQ-BRAIN-025 | Out of scope | N/A |

---

## Implementation Order

```
Phase 1: Plumbing (no functional change)
  1. commands/mod.rs -- add Setup variant + parse arm
  2. topologies/.../omega-brain.md -- create agent definition
  3. builds_agents.rs -- add BRAIN_AGENT const + write_single()
  4. gateway/mod.rs -- add mod setup; mod setup_i18n;
  4. setup_i18n.rs -- all localized messages

Phase 2: Core Flow
  5. setup.rs -- full orchestrator (start, handle, execute, cleanup)
  6. pipeline.rs -- /setup intercept + pending_setup check

Phase 3: Testing
  7. Unit tests for command parsing, output parsing, round parsing
  8. Unit tests for write_single lifecycle
  9. Integration test via scheduler injection pattern
```
