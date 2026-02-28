# Architecture: Topology Extraction (Phase 1)

## Scope

Gateway builds subsystem: `builds.rs`, `builds_agents.rs`, `builds_loop.rs`, `builds_parse.rs`, `builds_i18n.rs`, plus new `builds_topology.rs`. File system layout at `topologies/development/` (source) and `~/.omega/topologies/development/` (runtime).

## Overview

This refactoring extracts the hardcoded 7-phase build pipeline into a TOML-defined topology with external agent `.md` files. The Rust orchestrator becomes a generic topology executor that reads phase definitions, agent content, retry rules, and validation rules from configuration rather than code.

```
topologies/development/
  TOPOLOGY.toml              <-- Phase sequence, models, retries, validation
  agents/
    build-analyst.md          <-- Agent instructions (moved from const strings)
    build-architect.md
    build-test-writer.md
    build-developer.md
    build-qa.md
    build-reviewer.md
    build-delivery.md
    build-discovery.md

    [binary bundles all via include_str!()]
              |
              v
~/.omega/topologies/development/   <-- Deployed on first build request
  TOPOLOGY.toml
  agents/*.md
```

```
Execution flow (unchanged behavior):

  handle_build_request()
         |
         v
  load_topology("development")  <-- NEW: replaces hardcoded sequence
         |
         v
  for phase in topology.phases:
    match phase.phase_type:
      ParseBrief   -> run agent, parse_project_brief(), create dir
      Standard     -> run agent, check error
      CorrectiveLoop -> run_corrective_loop(retry, fix_agent)
      ParseSummary -> run agent, parse_build_summary(), format msg
```

## Modules

### Module 1: `builds_topology.rs` (NEW)

- **Responsibility**: Define topology schema structs, load topology from disk, bundle and deploy defaults
- **Public interface**:
  - `Topology` struct (pub(super))
  - `Phase` struct (pub(super))
  - `PhaseType` enum (pub(super))
  - `ModelTier` enum (pub(super))
  - `RetryConfig` struct (pub(super))
  - `ValidationConfig` struct (pub(super))
  - `ValidationType` enum (pub(super))
  - `LoadedTopology` struct (pub(super)) -- topology + agent content map
  - `load_topology(data_dir: &str, name: &str) -> Result<LoadedTopology, String>` (pub(super))
  - `deploy_bundled_topology(data_dir: &str) -> Result<(), String>` (pub(super))
  - `validate_topology_name(name: &str) -> Result<(), String>` (pub(super))
- **Dependencies**: `toml` (workspace dep), `serde::Deserialize`, `std::collections::HashMap`, `tracing`
- **Implementation order**: 1 (must exist before builds.rs refactoring)

#### Structs

```rust
/// Root topology document.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct Topology {
    pub topology: TopologyMeta,
    pub phases: Vec<Phase>,
}

/// Topology metadata header.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct TopologyMeta {
    pub name: String,
    pub description: String,
    pub version: u32,
}

/// A single phase in the pipeline.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct Phase {
    pub name: String,
    pub agent: String,
    #[serde(default = "default_model_tier")]
    pub model_tier: ModelTier,
    #[serde(default)]
    pub max_turns: Option<u32>,
    #[serde(default = "default_phase_type")]
    pub phase_type: PhaseType,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
    #[serde(default)]
    pub pre_validation: Option<ValidationConfig>,
    #[serde(default)]
    pub post_validation: Option<Vec<String>>,
}

/// Which model tier to use for a phase.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum ModelTier {
    Fast,
    #[default]
    Complex,
}

/// Phase execution behavior. Dispatches to existing Rust functions.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum PhaseType {
    #[default]
    Standard,
    ParseBrief,
    CorrectiveLoop,
    ParseSummary,
}

/// Retry configuration for corrective loop phases.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RetryConfig {
    pub max: u32,
    pub fix_agent: String,
}

/// Pre-phase validation rules.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct ValidationConfig {
    #[serde(rename = "type")]
    pub validation_type: ValidationType,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// The two validation strategies that exist today.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum ValidationType {
    FileExists,
    FilePatterns,
}
```

#### `LoadedTopology` -- the runtime object

```rust
/// A fully resolved topology: parsed TOML + all agent .md contents loaded.
pub(super) struct LoadedTopology {
    pub topology: Topology,
    /// Map of agent name -> agent .md file content.
    pub agents: HashMap<String, String>,
}

impl LoadedTopology {
    /// Get agent content by name. Returns Err if the agent is referenced but not loaded.
    pub fn agent_content(&self, name: &str) -> Result<&str, String> {
        self.agents
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| format!("agent '{name}' referenced in topology but .md file not found"))
    }

    /// Resolve the model string for a phase based on its ModelTier.
    pub fn resolve_model<'a>(&self, phase: &Phase, model_fast: &'a str, model_complex: &'a str) -> &'a str {
        match phase.model_tier {
            ModelTier::Fast => model_fast,
            ModelTier::Complex => model_complex,
        }
    }

    /// Collect all (agent_name, agent_content) pairs for AgentFilesGuard.
    pub fn all_agents(&self) -> Vec<(&str, &str)> {
        self.agents.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }
}
```

#### Bundled Defaults

```rust
/// Bundled topology files -- compiled into the binary via include_str!().
const BUNDLED_TOPOLOGY_TOML: &str = include_str!("../../../topologies/development/TOPOLOGY.toml");

const BUNDLED_AGENTS: &[(&str, &str)] = &[
    ("build-analyst", include_str!("../../../topologies/development/agents/build-analyst.md")),
    ("build-architect", include_str!("../../../topologies/development/agents/build-architect.md")),
    ("build-test-writer", include_str!("../../../topologies/development/agents/build-test-writer.md")),
    ("build-developer", include_str!("../../../topologies/development/agents/build-developer.md")),
    ("build-qa", include_str!("../../../topologies/development/agents/build-qa.md")),
    ("build-reviewer", include_str!("../../../topologies/development/agents/build-reviewer.md")),
    ("build-delivery", include_str!("../../../topologies/development/agents/build-delivery.md")),
    ("build-discovery", include_str!("../../../topologies/development/agents/build-discovery.md")),
];
```

#### Loader

```rust
/// Deploy bundled "development" topology to ~/.omega/topologies/development/.
/// Never overwrites existing files (preserves user customizations).
/// Follows the same pattern as omega-skills::install_bundled_skills().
pub(super) fn deploy_bundled_topology(data_dir: &str) -> Result<(), String> {
    let base = PathBuf::from(shellexpand(data_dir))
        .join("topologies")
        .join("development");

    // Create directories.
    let agents_dir = base.join("agents");
    std::fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("failed to create topology dir: {e}"))?;

    // Deploy TOPOLOGY.toml if not present.
    let toml_path = base.join("TOPOLOGY.toml");
    if !toml_path.exists() {
        std::fs::write(&toml_path, BUNDLED_TOPOLOGY_TOML)
            .map_err(|e| format!("failed to write TOPOLOGY.toml: {e}"))?;
        info!("topologies: deployed bundled TOPOLOGY.toml");
    }

    // Deploy agent .md files if not present.
    for (name, content) in BUNDLED_AGENTS {
        let path = agents_dir.join(format!("{name}.md"));
        if !path.exists() {
            std::fs::write(&path, content)
                .map_err(|e| format!("failed to write {name}.md: {e}"))?;
            info!("topologies: deployed bundled agent {name}");
        }
    }
    Ok(())
}

/// Load a topology by name from ~/.omega/topologies/<name>/.
/// Falls back to bundled default if the directory doesn't exist.
pub(super) fn load_topology(data_dir: &str, name: &str) -> Result<LoadedTopology, String> {
    validate_topology_name(name)?;

    let base = PathBuf::from(shellexpand(data_dir))
        .join("topologies")
        .join(name);

    // If directory doesn't exist, deploy bundled default first.
    if !base.exists() && name == "development" {
        deploy_bundled_topology(data_dir)?;
    } else if !base.exists() {
        return Err(format!("topology '{name}' not found at {}", base.display()));
    }

    // Parse TOPOLOGY.toml.
    let toml_path = base.join("TOPOLOGY.toml");
    let toml_content = std::fs::read_to_string(&toml_path)
        .map_err(|e| format!("failed to read TOPOLOGY.toml: {e}"))?;
    let topology: Topology = toml::from_str(&toml_content)
        .map_err(|e| format!("failed to parse TOPOLOGY.toml: {e}"))?;

    // Load all referenced agent .md files.
    let mut agents = HashMap::new();
    let agents_dir = base.join("agents");

    // Collect unique agent names from phases (including fix_agent references).
    let mut required_agents: Vec<&str> = topology.phases.iter().map(|p| p.agent.as_str()).collect();
    for phase in &topology.phases {
        if let Some(retry) = &phase.retry {
            required_agents.push(&retry.fix_agent);
        }
    }
    required_agents.sort_unstable();
    required_agents.dedup();

    for agent_name in required_agents {
        let agent_path = agents_dir.join(format!("{agent_name}.md"));
        let content = std::fs::read_to_string(&agent_path)
            .map_err(|e| format!("agent '{agent_name}' referenced in topology but file not found: {e}"))?;
        agents.insert(agent_name.to_string(), content);
    }

    Ok(LoadedTopology { topology, agents })
}

/// Validate a topology name: alphanumeric + hyphens + underscores, max 64 chars.
/// Rejects path traversal, shell metacharacters, empty names.
pub(super) fn validate_topology_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("topology name cannot be empty".to_string());
    }
    if name.len() > 64 {
        return Err(format!("topology name too long ({} chars, max 64)", name.len()));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(format!("topology name '{name}' contains path traversal characters"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("topology name '{name}' contains invalid characters (only alphanumeric, hyphens, underscores allowed)"));
    }
    Ok(())
}
```

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| TOPOLOGY.toml missing | User deleted file | `read_to_string` returns Err | Re-deploy from bundled default (development only) | Build delayed by re-deploy, no data loss |
| TOPOLOGY.toml corrupt | User edit error | `toml::from_str` returns Err | Return descriptive parse error to user | Build blocked, user must fix TOML |
| Agent .md file missing | Deleted or renamed | `read_to_string` returns Err | Return error naming the specific missing file | Build blocked, user informed |
| Path traversal in name | Malicious input | `validate_topology_name` rejects | Return error, do not touch filesystem | No I/O performed |
| Disk full during deploy | System resource | `fs::write` returns Err | Return error, fall back to bundled in-memory | Build can still proceed from memory (see fallback) |

#### Security Considerations

- **Trust boundary**: Topology name comes from code (hardcoded "development" in Phase 1), not user input. Still validated defensively for Phase 2 readiness.
- **Sensitive data**: Agent .md files contain prompt engineering but no secrets. No credentials in TOPOLOGY.toml.
- **Attack surface**: Path traversal via topology name. Mitigated by strict alphanumeric validation.
- **Mitigations**: `validate_topology_name()` rejects `..`, `/`, `\`, shell metacharacters. Max 64 chars prevents buffer-related issues.

#### Performance Budget

- **Latency target**: < 5ms for topology load from disk (8 small files, ~400 lines each)
- **Memory budget**: ~50KB for loaded topology (TOML parsed + 8 agent strings)
- **Complexity target**: O(n) where n = number of phases (linear scan)

### Module 2: `builds.rs` (MAJOR REWRITE)

- **Responsibility**: Topology-driven build orchestrator. Replaces hardcoded phase sequence with a loop over `topology.phases`.
- **Public interface**: `handle_build_request(&self, incoming, typing_handle)` (signature unchanged -- loads topology internally)
- **Dependencies**: `builds_topology::LoadedTopology`, `builds_agents::AgentFilesGuard`, `builds_loop`, `builds_parse`
- **Implementation order**: 3 (after topology + agents)

#### Refactoring Plan

The current `handle_build_request()` is a 420-line function with 7 explicit phase blocks. It becomes a ~200-line function that:

1. Loads the topology via `load_topology(data_dir, "development")`
2. Writes agent files via `AgentFilesGuard::write_from_topology(&workspace_dir, &loaded_topology)`
3. Iterates over `loaded_topology.topology.phases`
4. Dispatches each phase based on `phase.phase_type`

**Orchestrator state carried between phases:**

```rust
/// State accumulated during orchestration, passed between phases.
struct OrchestratorState {
    /// Raw text output from the analyst (ParseBrief) phase.
    brief_text: Option<String>,
    /// Parsed project brief (name, scope, language, etc.).
    brief: Option<ProjectBrief>,
    /// Project directory path (created after brief is parsed).
    project_dir: Option<PathBuf>,
    /// Project directory as string (for prompt interpolation).
    project_dir_str: Option<String>,
    /// Phases completed so far (names, for chain state).
    completed_phases: Vec<String>,
}
```

**Dispatch loop (pseudocode):**

```rust
let loaded = load_topology(&self.data_dir, "development")?;
let _guard = AgentFilesGuard::write_from_topology(&workspace_dir, &loaded).await?;

let mut state = OrchestratorState::default();

for (idx, phase) in loaded.topology.phases.iter().enumerate() {
    let model = loaded.resolve_model(phase, &self.model_fast, &self.model_complex);

    // Send localized phase message.
    self.send_text(incoming, &phase_message_by_name(&user_lang, &phase.name)).await;

    // Run pre-validation if configured.
    if let Some(ref validation) = phase.pre_validation {
        if let Some(project_dir) = &state.project_dir {
            if let Some(err) = self.run_validation(project_dir, validation) {
                // Save chain state and return.
                ...
            }
        }
    }

    // Dispatch based on phase type.
    match phase.phase_type {
        PhaseType::ParseBrief => {
            // Run agent, parse brief, create project dir.
            self.execute_parse_brief(incoming, phase, model, &mut state).await?;
        }
        PhaseType::Standard => {
            // Run agent, check for error.
            self.execute_standard(incoming, phase, model, &state).await?;
        }
        PhaseType::CorrectiveLoop => {
            // Run corrective loop with topology-defined retry config.
            let retry = phase.retry.as_ref()
                .ok_or("corrective-loop phase requires retry config")?;
            self.run_corrective_loop(
                incoming, phase, model, retry, &state, &user_lang,
            ).await?;
        }
        PhaseType::ParseSummary => {
            // Run agent, parse summary, send final message.
            self.execute_parse_summary(incoming, phase, model, &state).await?;
        }
    }

    // Run post-validation if configured.
    if let Some(ref paths) = phase.post_validation {
        if let Some(project_dir) = &state.project_dir {
            for path in paths {
                if !project_dir.join(path).exists() {
                    // Report failure, save chain state.
                    ...
                }
            }
        }
    }

    state.completed_phases.push(phase.name.clone());
}
```

**Key design decisions:**

1. **Prompt interpolation**: Phase prompts still constructed in Rust (they reference `project_dir_str`, `brief_text`). Moving prompts to TOML is deferred -- they depend on runtime state that a template engine would need. The orchestrator constructs prompts per phase type, same as today.

2. **Error handling pattern**: Each dispatch method returns `Result<(), EarlyReturn>` where `EarlyReturn` signals "send error to user and stop." The loop caller handles typing_handle abort and chain state saving.

3. **Post-validation for architect phase**: The current hardcoded `arch_file.exists()` check maps to `post_validation = ["specs/architecture.md"]` in TOPOLOGY.toml. This runs after the phase, not before the next one.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Topology load fails | Missing/corrupt files | `load_topology` returns Err | Send error message to user | Build blocked |
| Unknown phase_type | TOML has unrecognized type | serde deserialization error at load time | Report TOML parse error | Build blocked at load |
| Phase agent call fails | Provider timeout/error | `run_build_phase` returns Err | 3 retries with 2s delay (existing) | Same as today |
| Corrective loop without retry config | TOML misconfigured | Runtime check in dispatch | Return config error to user | Build blocked |

#### Performance Budget

- **Latency target**: No additional latency vs. current (topology loaded once per build, <5ms)
- **Memory budget**: OrchestratorState ~1KB (strings for brief and project dir)

### Module 3: `builds_agents.rs` (MAJOR REWRITE)

- **Responsibility**: Agent file RAII lifecycle. Reads agent content from `LoadedTopology` instead of const strings.
- **Public interface**: `AgentFilesGuard::write_from_topology(project_dir, loaded_topology)` (replaces `write()`)
- **Dependencies**: `builds_topology::LoadedTopology`
- **Implementation order**: 2 (after topology, before builds.rs)

#### Refactoring Plan

**Remove:**
- All 8 `const BUILD_*_AGENT: &str` constants (~880 lines of embedded markdown)
- `const BUILD_AGENTS: &[(&str, &str)]` array

**Keep:**
- `GUARD_REFCOUNTS` static (unchanged)
- `AgentFilesGuard` struct (unchanged)
- `impl Drop for AgentFilesGuard` (unchanged)

**Add:**
- `AgentFilesGuard::write_from_topology(project_dir: &Path, topology: &LoadedTopology) -> io::Result<Self>`

```rust
impl AgentFilesGuard {
    /// Write agent files from a loaded topology to `<project_dir>/.claude/agents/`.
    ///
    /// Replaces the old `write()` that used const strings. Same RAII behavior:
    /// increments ref count, files cleaned up on last guard drop.
    pub async fn write_from_topology(
        project_dir: &Path,
        topology: &LoadedTopology,
    ) -> std::io::Result<Self> {
        let agents_dir = project_dir.join(".claude").join("agents");
        tokio::fs::create_dir_all(&agents_dir).await?;
        for (name, content) in &topology.agents {
            let path = agents_dir.join(format!("{name}.md"));
            tokio::fs::write(&path, content).await?;
        }
        let mut counts = GUARD_REFCOUNTS.lock().unwrap();
        *counts.entry(agents_dir.clone()).or_insert(0) += 1;
        Ok(Self { agents_dir })
    }
}
```

**Line count impact:** Current file is ~1,272 lines (480 prod + 792 test). After removing 880 lines of const strings, prod code drops to ~60 lines. Tests rewritten to work with topology loader.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Agents dir creation fails | Permissions/disk | `create_dir_all` returns Err | Propagated to caller, build blocked | Same as today |
| Agent file write fails | Disk full | `write` returns Err | Propagated to caller, build blocked | Same as today |
| Concurrent guard cleanup race | Two builds finish simultaneously | Mutex on GUARD_REFCOUNTS | Last guard wins, files cleaned up correctly | No change from today |

### Module 4: `builds_loop.rs` (MODERATE CHANGE)

- **Responsibility**: Corrective loops (QA, reviewer) and phase validation
- **Public interface**: `run_corrective_loop()` (new, replaces `run_qa_loop` + `run_review_loop`), `run_validation()` (new), existing `validate_phase_output()` kept for compatibility during transition
- **Dependencies**: `builds_topology::{Phase, RetryConfig, ValidationConfig, ValidationType}`, `builds_parse`
- **Implementation order**: 4 (after builds.rs restructure)

#### Refactoring Plan

**Replace** `run_qa_loop()` and `run_review_loop()` with a single parameterized function:

```rust
/// Run a corrective loop: verify -> fix -> re-verify, up to retry.max iterations.
///
/// Parameters come from the topology's RetryConfig for the phase.
/// The verify_agent is the phase's own agent. The fix_agent comes from retry config.
pub(super) async fn run_corrective_loop(
    &self,
    incoming: &IncomingMessage,
    project_dir_str: &str,
    user_lang: &str,
    phase: &Phase,
    retry: &RetryConfig,
    model: &str,
) -> Result<(), String> {
    let is_qa = phase.name == "qa";  // Determines which parser to use

    for attempt in 1..=retry.max {
        let verification = match self
            .run_build_phase(&phase.agent, &prompt, model, phase.max_turns)
            .await
        {
            Ok(text) => {
                if is_qa {
                    match parse_verification_result(&text) {
                        VerificationResult::Pass => Ok(()),
                        VerificationResult::Fail(r) => Err(r),
                    }
                } else {
                    match parse_review_result(&text) {
                        ReviewResult::Pass => Ok(()),
                        ReviewResult::Fail(r) => Err(r),
                    }
                }
            }
            Err(e) => Err(e),
        };

        match verification {
            Ok(()) => {
                // Send pass message.
                ...
                return Ok(());
            }
            Err(reason) => {
                if attempt < retry.max {
                    // Send retry message, invoke fix agent.
                    self.run_build_phase(
                        &retry.fix_agent, &fix_prompt, model, None
                    ).await?;
                } else {
                    return Err(reason);
                }
            }
        }
    }
    Err("loop terminated without resolution".to_string())
}
```

**Add** a topology-driven validation function:

```rust
/// Run pre-phase validation from topology config.
/// Returns Some(error_message) on failure, None on success.
pub(super) fn run_validation(
    project_dir: &Path,
    config: &ValidationConfig,
) -> Option<String> {
    match config.validation_type {
        ValidationType::FileExists => {
            for path in &config.paths {
                if !project_dir.join(path).exists() {
                    return Some(format!(
                        "Pre-validation failed: required file '{path}' not found"
                    ));
                }
            }
            None
        }
        ValidationType::FilePatterns => {
            let has_match = Self::has_files_matching(
                project_dir,
                &config.patterns.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                0,
            );
            if !has_match {
                return Some(format!(
                    "Pre-validation failed: no files matching patterns {:?}",
                    config.patterns
                ));
            }
            None
        }
    }
}
```

**Keep unchanged:** `has_files_matching()`, `save_chain_state()`, `MAX_SCAN_DEPTH`, all existing tests for these functions.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Fix agent call fails in loop | Provider error | `run_build_phase` returns Err | Returns Err, pipeline stops | Same as today |
| Validation checks wrong path | TOML config error | Files appear missing when they exist | User fixes TOML validation config | Build blocked on that phase |

### Module 5: `builds_parse.rs` (MINOR CHANGE)

- **Responsibility**: Parse functions, data structures, i18n messages
- **Public interface**: `phase_message_by_name()` (new), all existing functions unchanged
- **Dependencies**: None (pure functions)
- **Implementation order**: 5 (last, smallest change)

#### Refactoring Plan

**Add** `phase_message_by_name()` alongside existing `phase_message()`:

```rust
/// Localized phase progress message, keyed by phase name (string).
///
/// Maps topology phase names to the same i18n strings as the old
/// phase_message(u8) function. Unknown phase names use the generic fallback.
pub(super) fn phase_message_by_name(lang: &str, phase_name: &str) -> String {
    // Map phase name to the legacy phase number for reuse.
    let phase_num = match phase_name {
        "analyst" => 1,
        "architect" => 2,
        "test-writer" => 3,
        "developer" => 4,
        "qa" => 5,
        "reviewer" => 6,
        "delivery" => 7,
        _ => 0,
    };

    if phase_num > 0 {
        // Delegate to existing function for known phases.
        phase_message(lang, phase_num, phase_name)
    } else {
        // Generic fallback for unknown/custom phase names.
        let action = phase_name.replace('-', " ");
        format!("\u{2699}\u{fe0f} {action}...")
    }
}
```

**Keep everything else unchanged:** `parse_project_brief()`, `parse_verification_result()`, `parse_review_result()`, `parse_build_summary()`, `phase_message()` (kept for backward compatibility and test stability), all i18n helper functions, `ChainState` struct, all existing tests.

**Rationale for keeping `phase_message(u8)`:** The new function delegates to it. This avoids duplicating 56 i18n strings (8 languages x 7 phases) and keeps all existing tests passing.

#### Failure Modes

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| Unknown phase name | Custom topology with novel names | Falls through to generic fallback | Shows "{phase_name}..." instead of localized text | Cosmetic only |

## File Layout: `topologies/development/`

Source directory (committed to repo, compiled into binary):

```
topologies/
  development/
    TOPOLOGY.toml                    # Pipeline definition
    agents/
      build-analyst.md               # Extracted from BUILD_ANALYST_AGENT const
      build-architect.md             # Extracted from BUILD_ARCHITECT_AGENT const
      build-test-writer.md           # Extracted from BUILD_TEST_WRITER_AGENT const
      build-developer.md             # Extracted from BUILD_DEVELOPER_AGENT const
      build-qa.md                    # Extracted from BUILD_QA_AGENT const
      build-reviewer.md              # Extracted from BUILD_REVIEWER_AGENT const
      build-delivery.md              # Extracted from BUILD_DELIVERY_AGENT const
      build-discovery.md             # Extracted from BUILD_DISCOVERY_AGENT const
```

The `include_str!()` path from `builds_topology.rs` to `topologies/` traverses: `backend/src/gateway/builds_topology.rs` -> `../../../topologies/development/TOPOLOGY.toml` (3 levels up from `backend/src/gateway/` to repo root).

## Final TOPOLOGY.toml Schema

```toml
[topology]
name = "development"
description = "Default 7-phase TDD build pipeline"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"
model_tier = "complex"
max_turns = 25
phase_type = "parse-brief"

[[phases]]
name = "architect"
agent = "build-architect"
model_tier = "complex"

[phases.post_validation]
paths = ["specs/architecture.md"]

[[phases]]
name = "test-writer"
agent = "build-test-writer"
model_tier = "complex"

[phases.pre_validation]
type = "file_exists"
paths = ["specs/architecture.md"]

[[phases]]
name = "developer"
agent = "build-developer"
model_tier = "complex"

[phases.pre_validation]
type = "file_patterns"
patterns = ["test", "spec", "_test."]

[[phases]]
name = "qa"
agent = "build-qa"
model_tier = "complex"
phase_type = "corrective-loop"

[phases.pre_validation]
type = "file_patterns"
patterns = [".rs", ".py", ".js", ".ts", ".go", ".java", ".rb", ".c", ".cpp"]

[phases.retry]
max = 3
fix_agent = "build-developer"

[[phases]]
name = "reviewer"
agent = "build-reviewer"
model_tier = "complex"
phase_type = "corrective-loop"

[phases.retry]
max = 2
fix_agent = "build-developer"

[[phases]]
name = "delivery"
agent = "build-delivery"
model_tier = "complex"
phase_type = "parse-summary"
```

**Note on `post_validation`:** The requirements document shows `post_validation = ["specs/architecture.md"]` as a simple array on the architect phase. However, the current code checks `arch_file.exists()` *after* the architect phase runs, which is semantically a post-validation. In the TOML schema, `post_validation` is `Option<Vec<String>>` -- a list of file paths that must exist after the phase completes. This is simpler than `pre_validation` (which needs a type discriminant for file_exists vs file_patterns) because post-validation is always "these files must exist."

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| Topology loaded once per build, not cached globally | LazyLock global cache; per-process cache with invalidation | Per-build load is simple, <5ms, and allows hot-editing topology files between builds. No cache invalidation complexity. |
| Agent content stored in HashMap, not Vec | Vec<(String, String)> like current BUILD_AGENTS | HashMap allows O(1) lookup by name, needed for fix_agent resolution in corrective loops. |
| PhaseType dispatches to Rust functions, not a generic executor | Template-based prompts with variable interpolation; WASM plugin system | Prompts depend on runtime state (project_dir, brief_text) that would need a template engine. Rust dispatch is safer and preserves identical behavior. Phase 1 is a refactoring, not a feature. |
| `phase_message_by_name` delegates to `phase_message(u8)` | Duplicate all 56 i18n strings in new function; use a lookup table | Delegation avoids duplication and ensures existing tests keep passing. Adding a name->number mapping is trivial. |
| `post_validation` is `Option<Vec<String>>` (simple path list) | Full `ValidationConfig` struct for post too; enum with FileExists/FilePatterns | Only one post-validation type exists today (file_exists for specs/architecture.md). Simple paths keep the TOML clean. Can be upgraded later. |
| Keep `validate_phase_output()` alongside new `run_validation()` | Remove old function, rewrite all callers | Transition safety. Old function can be deprecated after Phase 1 is stable. |
| Bundled defaults deployed on first build, not process startup | Deploy in main.rs startup; deploy in Gateway::new() | Lazy deployment means no disk I/O unless builds are actually used. Follows the same pattern as skills deployment. |
| Discovery agent included in topology agents but NOT in phases | Add discovery as phase 0 in topology; separate discovery topology | Discovery runs in pipeline.rs, not builds.rs. It uses the agent file but not the phase orchestrator. Including it in the agents map means AgentFilesGuard deploys it. |

## Design Questions Answered

### 1. How does the orchestrator loop carry state (brief, project_dir) between phases?

Via an `OrchestratorState` struct allocated on the stack at the start of `handle_build_request()`. The `ParseBrief` phase populates `brief_text`, `brief`, `project_dir`, and `project_dir_str`. Subsequent phases read from this state. The struct is not passed to the topology loader -- it's internal to the orchestrator.

### 2. How does AgentFilesGuard get agent content from the loaded topology?

`AgentFilesGuard::write_from_topology(project_dir, &loaded_topology)` replaces `write()`. It iterates over `loaded_topology.agents` (the HashMap) and writes each entry to disk. The guard does not own or reference the topology -- it only needs the content at write time.

### 3. Where is the topology cached?

Not cached. Loaded from disk once per build request, at the top of `handle_build_request()`. The load is fast (<5ms for 9 small files) and the per-build load allows users to edit topology files between builds without restarting the process. If performance becomes an issue in Phase 2 (multiple topologies), a process-level cache with file-modified-time invalidation can be added.

### 4. How do validation rules map to validate_phase_output()?

The existing `validate_phase_output()` function uses a hardcoded match on phase name ("test-writer" -> check specs/architecture.md, "developer" -> check test files, "qa" -> check source files). The new `run_validation()` function reads the same rules from `ValidationConfig` in the topology. Both functions coexist during transition. The orchestrator calls `run_validation()` when a phase has `pre_validation` in its topology config.

### 5. What happens if TOPOLOGY.toml references an agent that has no .md file?

`load_topology()` scans all phases for agent names and fix_agent names, then attempts to read each `.md` file. If any file is missing, it returns `Err(format!("agent '{name}' referenced in topology but file not found: {e}"))`. The build is blocked and the user sees a clear error message naming the specific missing file. No silent skipping, no empty content fallback.

## Integration Points

### `builds.rs` <-> `builds_topology.rs`
- `builds.rs` calls `load_topology()` at the top of `handle_build_request()`
- `builds.rs` reads `LoadedTopology.topology.phases` for the iteration loop
- `builds.rs` calls `LoadedTopology.resolve_model()` for each phase

### `builds.rs` <-> `builds_agents.rs`
- `builds.rs` calls `AgentFilesGuard::write_from_topology()` instead of `write()`
- Passes `&LoadedTopology` to the guard constructor

### `builds.rs` <-> `builds_loop.rs`
- `builds.rs` calls `run_corrective_loop()` with `&Phase` and `&RetryConfig` from topology
- `builds.rs` calls `run_validation()` with `&ValidationConfig` from topology

### `builds.rs` <-> `builds_parse.rs`
- `builds.rs` calls `phase_message_by_name()` instead of `phase_message(phase_num)`
- All parse functions unchanged in signature

### `pipeline.rs` <-> `builds_agents.rs` / `builds_topology.rs`
- `pipeline.rs` calls `load_topology()` independently for discovery flow (to get agent content for `AgentFilesGuard::write_from_topology()`)
- `builds.rs::handle_build_request()` calls `load_topology()` independently at the top of the function
- Each call site loads topology on its own rather than sharing a single load. This is simpler (no parameter threading) and the load is fast (<5ms). The topology is loaded at most twice per build request (once for discovery, once for the build itself).

### `gateway/mod.rs`
- Add `mod builds_topology;` to module registration

## Failure Modes (System-Level)

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| Topology directory deleted at runtime | builds_topology | `load_topology` fails on `read_to_string` | Re-deploy bundled default for "development" | Build delayed by re-deploy |
| Agent .md file corrupted (not valid markdown) | builds_agents, provider | Agent runs with garbled instructions | Provider call may fail or produce bad output | Same as today if const was wrong |
| TOML syntax error after user edit | builds_topology | `toml::from_str` returns descriptive error | User fixes TOML, retries build | Build blocked with clear error |
| Disk read latency spike (NFS, slow disk) | builds_topology | No explicit timeout on `fs::read_to_string` | Blocks the build task (async runtime not blocked -- fs ops are sync but fast) | Build takes longer to start |
| All topology files deleted mid-build | builds_agents guard | Agent files already written to workspace | Build completes normally (guard wrote files at start) | No impact on current build |

## Security Model

### Trust Boundaries

- **Topology name**: In Phase 1, hardcoded to "development" in Rust code. Validated anyway for Phase 2 safety. Users cannot inject topology names in Phase 1.
- **TOML content**: Read from `~/.omega/topologies/` which is user-writable. Treated as semi-trusted (user's own files). Parsed via serde with typed structs -- unknown fields ignored, type mismatches rejected.
- **Agent .md content**: Read from disk, written to workspace for Claude Code CLI consumption. Content is prompt text, not executable code. Same trust model as today's const strings.

### Data Classification

| Data | Classification | Storage | Access Control |
|------|---------------|---------|---------------|
| TOPOLOGY.toml | Internal | `~/.omega/topologies/` (user home) | User file permissions |
| Agent .md files | Internal | `~/.omega/topologies/` + temp in workspace | User file permissions |
| Bundled defaults | Public | Compiled into binary | Read-only after build |

### Attack Surface

- **Path traversal in topology name**: Risk: read arbitrary files. Mitigation: `validate_topology_name()` rejects `..`, `/`, `\`. Max 64 chars.
- **TOML injection**: Risk: None. Serde deserializes into typed structs. Unknown fields are ignored, not executed.
- **Agent content injection**: Risk: User could edit agent .md to include malicious prompts. Mitigation: Same risk as today (user has access to their own prompt files). Not a new attack vector.

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| `~/.omega/topologies/` directory | Load from disk | Re-deploy bundled default | Transparent, build proceeds |
| Individual agent .md file | Load from disk | Error with specific filename | Build blocked, user informed which file to fix |
| TOPOLOGY.toml parse | Load and parse | Error with line/column info | Build blocked, user gets actionable error |

## Performance Budgets

| Operation | Latency (p50) | Latency (p99) | Memory | Notes |
|-----------|---------------|---------------|--------|-------|
| Load topology from disk | < 2ms | < 5ms | ~50KB | 9 files, all small |
| Deploy bundled topology | < 10ms | < 50ms | ~50KB | First build only |
| Validate topology name | < 1us | < 1us | 0 | String checks only |
| Phase dispatch (per phase) | < 1us | < 1us | 0 | Match on enum |

## Data Flow

```
User says "build me X"
        |
        v
pipeline.rs: build keyword detected -> discovery flow
        |
        v
pipeline.rs: load_topology("development")           <-- NEW (for discovery)
        |
        v
pipeline.rs: AgentFilesGuard::write_from_topology()  <-- CHANGED
        |   (deploys all 8 agents to workspace)
        v
pipeline.rs: discovery agent runs (if needed)
        |
        v
pipeline.rs: handle_build_request(incoming, typing_handle)  <-- signature unchanged
        |
        v
builds.rs: load_topology("development")             <-- loads independently
        |
        v
builds.rs: iterate topology.phases
        |
        +--> Phase 1 (ParseBrief): run analyst, parse brief, create dir
        |
        +--> Phase 2 (Standard): run architect, post-validate specs/
        |
        +--> Phase 3 (Standard): pre-validate specs/, run test-writer
        |
        +--> Phase 4 (Standard): pre-validate tests, run developer
        |
        +--> Phase 5 (CorrectiveLoop): pre-validate sources, qa loop (max 3)
        |
        +--> Phase 6 (CorrectiveLoop): reviewer loop (max 2)
        |
        +--> Phase 7 (ParseSummary): run delivery, parse summary, send
        |
        v
    Build complete -> audit log -> user notified
```

## External Dependencies

- `toml` (0.8): Already a workspace dependency. Used for TOPOLOGY.toml deserialization.
- `serde` (1.x): Already a workspace dependency. Derive `Deserialize` for topology structs.
- No new crate dependencies added.

## Requirement Traceability

| Requirement ID | Architecture Section | Module(s) |
|---------------|---------------------|-----------|
| REQ-TOP-001 | Module 1: builds_topology.rs, Structs section | `backend/src/gateway/builds_topology.rs` |
| REQ-TOP-002 | Module 1: Bundled Defaults, Loader | `backend/src/gateway/builds_topology.rs`, `topologies/development/` |
| REQ-TOP-003 | Module 1: Loader (`load_topology()`) | `backend/src/gateway/builds_topology.rs` |
| REQ-TOP-004 | Module 2: builds.rs, Dispatch loop | `backend/src/gateway/builds.rs` |
| REQ-TOP-005 | Module 3: builds_agents.rs, `write_from_topology()` | `backend/src/gateway/builds_agents.rs` |
| REQ-TOP-006 | Module 4: builds_loop.rs, `run_corrective_loop()` | `backend/src/gateway/builds_loop.rs` |
| REQ-TOP-007 | Module 4: builds_loop.rs, `run_validation()` | `backend/src/gateway/builds_loop.rs` |
| REQ-TOP-008 | Module 5: builds_parse.rs, `phase_message_by_name()` | `backend/src/gateway/builds_parse.rs` |
| REQ-TOP-009 | Design Decisions, Dispatch loop, Data Flow | `backend/src/gateway/builds.rs` |
| REQ-TOP-010 | Module 5: "Keep everything else unchanged" | `backend/src/gateway/builds_parse.rs` |
| REQ-TOP-011 | Module 3: Line count impact | All gateway/builds_*.rs files |
| REQ-TOP-012 | Design Decisions: "Discovery agent included..." | `backend/src/gateway/builds_topology.rs` |
| REQ-TOP-013 | Module 1: `validate_topology_name()` | `backend/src/gateway/builds_topology.rs` |
| REQ-TOP-014 | Module 1: Loader error handling | `backend/src/gateway/builds_topology.rs` |
| REQ-TOP-015 | Module 4: `save_chain_state()` extension | `backend/src/gateway/builds_loop.rs` |
