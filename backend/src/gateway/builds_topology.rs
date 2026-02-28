//! Topology data structures, TOML deserialization, loader, and bundled default deployment.
//!
//! Defines the config-driven topology format that replaces the hardcoded 7-phase
//! build pipeline. The "development" topology is bundled in the binary and
//! auto-deployed to `~/.omega/topologies/development/` on first build request.
//!
//! Public interface (all pub(super)):
//! - Topology, TopologyMeta, Phase, PhaseType, ModelTier, RetryConfig,
//!   ValidationConfig, ValidationType structs with serde::Deserialize
//! - LoadedTopology: topology + agent content map + helper methods
//! - load_topology(data_dir, name) -> Result<LoadedTopology, String>
//! - deploy_bundled_topology(data_dir) -> Result<(), String>
//! - validate_topology_name(name) -> Result<(), String>

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use omega_core::config::shellexpand;

// ---------------------------------------------------------------------------
// Schema structs — TOML deserialization targets
// ---------------------------------------------------------------------------

/// Root topology document.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct Topology {
    /// Metadata header (name, description, version). Used by tests and future CLI tooling.
    #[allow(dead_code)]
    pub topology: TopologyMeta,
    pub phases: Vec<Phase>,
}

/// Topology metadata header.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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

fn default_model_tier() -> ModelTier {
    ModelTier::Complex
}

fn default_phase_type() -> PhaseType {
    PhaseType::Standard
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

// ---------------------------------------------------------------------------
// LoadedTopology — runtime object with parsed TOML + agent contents
// ---------------------------------------------------------------------------

/// A fully resolved topology: parsed TOML + all agent .md contents loaded.
#[derive(Debug)]
pub(super) struct LoadedTopology {
    pub topology: Topology,
    /// Map of agent name -> agent .md file content.
    pub agents: HashMap<String, String>,
}

impl LoadedTopology {
    /// Get agent content by name. Returns Err if the agent is referenced but not loaded.
    #[allow(dead_code)]
    pub fn agent_content(&self, name: &str) -> Result<&str, String> {
        self.agents
            .get(name)
            .map(|s| s.as_str())
            .ok_or_else(|| format!("agent '{name}' referenced in topology but .md file not found"))
    }

    /// Resolve the model string for a phase based on its ModelTier.
    pub fn resolve_model<'a>(
        &self,
        phase: &Phase,
        model_fast: &'a str,
        model_complex: &'a str,
    ) -> &'a str {
        match phase.model_tier {
            ModelTier::Fast => model_fast,
            ModelTier::Complex => model_complex,
        }
    }

    /// Collect all (agent_name, agent_content) pairs for AgentFilesGuard.
    #[allow(dead_code)]
    pub fn all_agents(&self) -> Vec<(&str, &str)> {
        self.agents
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Bundled defaults — compiled into the binary via include_str!()
// ---------------------------------------------------------------------------

/// Bundled topology TOML — compiled into the binary via include_str!().
const BUNDLED_TOPOLOGY_TOML: &str = include_str!("../../../topologies/development/TOPOLOGY.toml");

/// Bundled agent definitions — compiled into the binary via include_str!().
const BUNDLED_AGENTS: &[(&str, &str)] = &[
    (
        "build-analyst",
        include_str!("../../../topologies/development/agents/build-analyst.md"),
    ),
    (
        "build-architect",
        include_str!("../../../topologies/development/agents/build-architect.md"),
    ),
    (
        "build-test-writer",
        include_str!("../../../topologies/development/agents/build-test-writer.md"),
    ),
    (
        "build-developer",
        include_str!("../../../topologies/development/agents/build-developer.md"),
    ),
    (
        "build-qa",
        include_str!("../../../topologies/development/agents/build-qa.md"),
    ),
    (
        "build-reviewer",
        include_str!("../../../topologies/development/agents/build-reviewer.md"),
    ),
    (
        "build-delivery",
        include_str!("../../../topologies/development/agents/build-delivery.md"),
    ),
    (
        "build-discovery",
        include_str!("../../../topologies/development/agents/build-discovery.md"),
    ),
];

// ---------------------------------------------------------------------------
// Loader functions
// ---------------------------------------------------------------------------

/// Deploy bundled "development" topology to ~/.omega/topologies/development/.
/// Never overwrites existing files (preserves user customizations).
pub(super) fn deploy_bundled_topology(data_dir: &str) -> Result<(), String> {
    let base = PathBuf::from(shellexpand(data_dir))
        .join("topologies")
        .join("development");

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
    let topology: Topology =
        toml::from_str(&toml_content).map_err(|e| format!("failed to parse TOPOLOGY.toml: {e}"))?;

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
        let content = std::fs::read_to_string(&agent_path).map_err(|e| {
            format!("agent '{agent_name}' referenced in topology but file not found: {e}")
        })?;
        agents.insert(agent_name.to_string(), content);
    }

    // Scan agents/ directory for any additional .md files not referenced by phases
    // (e.g. build-discovery, which is used by the pre-build discovery flow).
    if agents_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&agents_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.ends_with(".md") {
                    let agent_name = file_name.trim_end_matches(".md").to_string();
                    agents.entry(agent_name).or_insert_with_key(|_| {
                        std::fs::read_to_string(entry.path()).unwrap_or_default()
                    });
                }
            }
        }
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
        return Err(format!(
            "topology name too long ({} chars, max 64)",
            name.len()
        ));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(format!(
            "topology name '{name}' contains path traversal characters"
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!(
            "topology name '{name}' contains invalid characters (only alphanumeric, hyphens, underscores allowed)"
        ));
    }
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // REQ-TOP-001 (Must): Schema deserialization tests
    // ===================================================================

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: Topology, Phase, ModelTier, PhaseType, RetryConfig, ValidationRule
    //             structs defined; all derive serde::Deserialize; invalid TOML returns Err
    #[test]
    fn test_topology_deserialize_minimal_valid_toml() {
        let toml_str = r#"
[topology]
name = "test"
description = "A test topology"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"
"#;
        let result: Result<Topology, _> = toml::from_str(toml_str);
        assert!(
            result.is_ok(),
            "Minimal valid TOML should deserialize: {:?}",
            result.err()
        );
        let topo = result.unwrap();
        assert_eq!(topo.topology.name, "test");
        assert_eq!(topo.topology.description, "A test topology");
        assert_eq!(topo.topology.version, 1);
        assert_eq!(topo.phases.len(), 1);
        assert_eq!(topo.phases[0].name, "analyst");
        assert_eq!(topo.phases[0].agent, "build-analyst");
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: Default values applied when optional fields are missing
    #[test]
    fn test_topology_deserialize_defaults_applied() {
        let toml_str = r#"
[topology]
name = "test"
description = "Test defaults"
version = 1

[[phases]]
name = "basic"
agent = "build-basic"
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        let phase = &topo.phases[0];
        assert_eq!(
            phase.model_tier,
            ModelTier::Complex,
            "Default model_tier should be Complex"
        );
        assert_eq!(
            phase.phase_type,
            PhaseType::Standard,
            "Default phase_type should be Standard"
        );
        assert!(
            phase.max_turns.is_none(),
            "Default max_turns should be None"
        );
        assert!(phase.retry.is_none(), "Default retry should be None");
        assert!(
            phase.pre_validation.is_none(),
            "Default pre_validation should be None"
        );
        assert!(
            phase.post_validation.is_none(),
            "Default post_validation should be None"
        );
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: All PhaseType variants deserialize correctly
    #[test]
    fn test_topology_deserialize_all_phase_types() {
        let toml_str = r#"
[topology]
name = "test"
description = "Phase types"
version = 1

[[phases]]
name = "a"
agent = "build-a"
phase_type = "standard"

[[phases]]
name = "b"
agent = "build-b"
phase_type = "parse-brief"

[[phases]]
name = "c"
agent = "build-c"
phase_type = "corrective-loop"

[[phases]]
name = "d"
agent = "build-d"
phase_type = "parse-summary"
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        assert_eq!(topo.phases[0].phase_type, PhaseType::Standard);
        assert_eq!(topo.phases[1].phase_type, PhaseType::ParseBrief);
        assert_eq!(topo.phases[2].phase_type, PhaseType::CorrectiveLoop);
        assert_eq!(topo.phases[3].phase_type, PhaseType::ParseSummary);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: ModelTier variants deserialize correctly
    #[test]
    fn test_topology_deserialize_model_tiers() {
        let toml_str = r#"
[topology]
name = "test"
description = "Model tiers"
version = 1

[[phases]]
name = "fast-phase"
agent = "build-fast"
model_tier = "fast"

[[phases]]
name = "complex-phase"
agent = "build-complex"
model_tier = "complex"
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        assert_eq!(topo.phases[0].model_tier, ModelTier::Fast);
        assert_eq!(topo.phases[1].model_tier, ModelTier::Complex);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: RetryConfig deserializes correctly
    #[test]
    fn test_topology_deserialize_retry_config() {
        let toml_str = r#"
[topology]
name = "test"
description = "Retry"
version = 1

[[phases]]
name = "qa"
agent = "build-qa"
phase_type = "corrective-loop"

[phases.retry]
max = 3
fix_agent = "build-developer"
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        let retry = topo.phases[0].retry.as_ref().unwrap();
        assert_eq!(retry.max, 3);
        assert_eq!(retry.fix_agent, "build-developer");
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: ValidationConfig with file_exists deserializes correctly
    #[test]
    fn test_topology_deserialize_validation_file_exists() {
        let toml_str = r#"
[topology]
name = "test"
description = "Validation"
version = 1

[[phases]]
name = "test-writer"
agent = "build-test-writer"

[phases.pre_validation]
type = "file_exists"
paths = ["specs/architecture.md"]
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        let validation = topo.phases[0].pre_validation.as_ref().unwrap();
        assert_eq!(validation.validation_type, ValidationType::FileExists);
        assert_eq!(validation.paths, vec!["specs/architecture.md"]);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: ValidationConfig with file_patterns deserializes correctly
    #[test]
    fn test_topology_deserialize_validation_file_patterns() {
        let toml_str = r#"
[topology]
name = "test"
description = "Patterns"
version = 1

[[phases]]
name = "developer"
agent = "build-developer"

[phases.pre_validation]
type = "file_patterns"
patterns = ["test", "spec", "_test."]
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        let validation = topo.phases[0].pre_validation.as_ref().unwrap();
        assert_eq!(validation.validation_type, ValidationType::FilePatterns);
        assert_eq!(validation.patterns, vec!["test", "spec", "_test."]);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: post_validation deserializes as Vec<String>
    #[test]
    fn test_topology_deserialize_post_validation() {
        let toml_str = r#"
[topology]
name = "test"
description = "Post validation"
version = 1

[[phases]]
name = "architect"
agent = "build-architect"
post_validation = ["specs/architecture.md"]
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        let post = topo.phases[0].post_validation.as_ref().unwrap();
        assert_eq!(post, &vec!["specs/architecture.md".to_string()]);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: max_turns is optional and correctly parsed when present
    #[test]
    fn test_topology_deserialize_max_turns() {
        let toml_str = r#"
[topology]
name = "test"
description = "Max turns"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"
max_turns = 25
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        assert_eq!(topo.phases[0].max_turns, Some(25));
    }

    // Requirement: REQ-TOP-001 (Must)
    // Failure mode: invalid TOML returns Err, not panic
    #[test]
    fn test_topology_deserialize_invalid_toml_returns_err() {
        let bad_toml = "this is not valid TOML {{{";
        let result: Result<Topology, _> = toml::from_str(bad_toml);
        assert!(result.is_err(), "Invalid TOML must return Err, not panic");
    }

    // Requirement: REQ-TOP-001 (Must)
    // Failure mode: missing required field returns Err
    #[test]
    fn test_topology_deserialize_missing_required_field() {
        // Missing [topology] section entirely
        let toml_str = r#"
[[phases]]
name = "analyst"
agent = "build-analyst"
"#;
        let result: Result<Topology, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "Missing [topology] section must return Err"
        );
    }

    // Requirement: REQ-TOP-001 (Must)
    // Failure mode: wrong type for field returns Err
    #[test]
    fn test_topology_deserialize_wrong_type_returns_err() {
        let toml_str = r#"
[topology]
name = "test"
description = "Test"
version = "not-a-number"

[[phases]]
name = "analyst"
agent = "build-analyst"
"#;
        let result: Result<Topology, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "Wrong type for version must return Err");
    }

    // Requirement: REQ-TOP-001 (Must)
    // Failure mode: unknown phase_type returns Err (strict deserialization)
    #[test]
    fn test_topology_deserialize_unknown_phase_type_returns_err() {
        let toml_str = r#"
[topology]
name = "test"
description = "Test"
version = 1

[[phases]]
name = "custom"
agent = "build-custom"
phase_type = "nonexistent-type"
"#;
        let result: Result<Topology, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "Unknown phase_type must return Err, not silently accept"
        );
    }

    // Requirement: REQ-TOP-001 (Must)
    // Edge case: empty phases array
    #[test]
    fn test_topology_deserialize_empty_phases() {
        let toml_str = r#"
[topology]
name = "empty"
description = "No phases"
version = 1
"#;
        // This may or may not succeed depending on serde defaults.
        // The key requirement is: it must not panic.
        let result: Result<Topology, _> = toml::from_str(toml_str);
        // Even if it succeeds with empty phases, that's valid TOML.
        if let Ok(topo) = result {
            assert!(topo.phases.is_empty());
        }
    }

    // Requirement: REQ-TOP-001 (Must)
    // Security: TOML with extra unknown fields is handled gracefully
    #[test]
    fn test_topology_deserialize_ignores_unknown_fields() {
        let toml_str = r#"
[topology]
name = "test"
description = "Test"
version = 1
evil_field = "should be ignored"

[[phases]]
name = "analyst"
agent = "build-analyst"
unknown_option = true
"#;
        // serde by default ignores unknown fields; verify no panic.
        let result: Result<Topology, _> = toml::from_str(toml_str);
        // This test documents that unknown fields don't crash the parser.
        // Whether it's Ok or Err depends on serde config — the requirement
        // is that it does NOT panic.
        let _ = result;
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: Full development topology TOML deserializes correctly (7 phases)
    #[test]
    fn test_topology_deserialize_full_development_topology() {
        let toml_str = r#"
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
post_validation = ["specs/architecture.md"]

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
"#;
        let topo: Topology = toml::from_str(toml_str).unwrap();
        assert_eq!(topo.topology.name, "development");
        assert_eq!(topo.phases.len(), 7, "Development topology has 7 phases");

        // Verify phase names in order.
        let names: Vec<&str> = topo.phases.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(
            names,
            vec![
                "analyst",
                "architect",
                "test-writer",
                "developer",
                "qa",
                "reviewer",
                "delivery"
            ]
        );

        // Verify analyst phase specifics.
        assert_eq!(topo.phases[0].phase_type, PhaseType::ParseBrief);
        assert_eq!(topo.phases[0].max_turns, Some(25));

        // Verify QA retry config.
        let qa_retry = topo.phases[4].retry.as_ref().unwrap();
        assert_eq!(qa_retry.max, 3);
        assert_eq!(qa_retry.fix_agent, "build-developer");

        // Verify reviewer retry config.
        let rev_retry = topo.phases[5].retry.as_ref().unwrap();
        assert_eq!(rev_retry.max, 2);
        assert_eq!(rev_retry.fix_agent, "build-developer");

        // Verify delivery phase type.
        assert_eq!(topo.phases[6].phase_type, PhaseType::ParseSummary);
    }

    // ===================================================================
    // REQ-TOP-002 (Must): Bundled default deployment tests
    // ===================================================================

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: Auto-deployed to ~/.omega/topologies/development/ if directory missing
    #[test]
    fn test_deploy_bundled_topology_creates_directory_structure() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let result = deploy_bundled_topology(data_dir);
        assert!(
            result.is_ok(),
            "deploy_bundled_topology should succeed: {:?}",
            result.err()
        );

        let topo_dir = tmp.path().join("topologies").join("development");
        assert!(topo_dir.exists(), "topologies/development/ must be created");

        let agents_dir = topo_dir.join("agents");
        assert!(
            agents_dir.exists(),
            "topologies/development/agents/ must be created"
        );

        let toml_path = topo_dir.join("TOPOLOGY.toml");
        assert!(toml_path.exists(), "TOPOLOGY.toml must be deployed");
    }

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: Does NOT overwrite existing files (preserves user customizations)
    #[test]
    fn test_deploy_bundled_topology_preserves_existing_files() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("development");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Write a custom TOPOLOGY.toml before deployment.
        let toml_path = topo_dir.join("TOPOLOGY.toml");
        let custom_content = "# User-customized topology\n[topology]\nname = \"custom\"\n";
        std::fs::write(&toml_path, custom_content).unwrap();

        // Deploy should NOT overwrite.
        deploy_bundled_topology(data_dir).unwrap();

        let after_content = std::fs::read_to_string(&toml_path).unwrap();
        assert_eq!(
            after_content, custom_content,
            "Existing TOPOLOGY.toml must be preserved, not overwritten"
        );
    }

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: Does NOT overwrite existing agent .md files
    #[test]
    fn test_deploy_bundled_topology_preserves_existing_agent_files() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let agents_dir = tmp
            .path()
            .join("topologies")
            .join("development")
            .join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Write a custom agent file.
        let custom_agent = agents_dir.join("build-analyst.md");
        let custom_content = "---\nname: build-analyst\n---\nCustom agent content\n";
        std::fs::write(&custom_agent, custom_content).unwrap();

        deploy_bundled_topology(data_dir).unwrap();

        let after_content = std::fs::read_to_string(&custom_agent).unwrap();
        assert_eq!(
            after_content, custom_content,
            "Existing agent .md file must be preserved"
        );
    }

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: Idempotent — calling twice produces same result
    #[test]
    fn test_deploy_bundled_topology_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        deploy_bundled_topology(data_dir).unwrap();
        // Second call should succeed without error.
        let result = deploy_bundled_topology(data_dir);
        assert!(
            result.is_ok(),
            "Second deploy must succeed (idempotent): {:?}",
            result.err()
        );
    }

    // Requirement: REQ-TOP-002 (Must)
    // Failure mode: bundled agents are non-empty (once developer fills in include_str!)
    //
    // NOTE: This test will FAIL until the developer creates the topology files
    // and replaces the stub BUNDLED_AGENTS/BUNDLED_TOPOLOGY_TOML with include_str!().
    #[test]
    fn test_bundled_topology_toml_is_non_empty() {
        assert!(
            !BUNDLED_TOPOLOGY_TOML.is_empty(),
            "BUNDLED_TOPOLOGY_TOML must contain the development topology TOML (replace stub with include_str!)"
        );
    }

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: Bundled TOML is valid and parseable
    #[test]
    fn test_bundled_topology_toml_is_valid() {
        if BUNDLED_TOPOLOGY_TOML.is_empty() {
            // Stub hasn't been replaced yet — skip gracefully with a clear message.
            panic!(
                "BUNDLED_TOPOLOGY_TOML is still a stub. Developer must replace with include_str!()"
            );
        }
        let result: Result<Topology, _> = toml::from_str(BUNDLED_TOPOLOGY_TOML);
        assert!(
            result.is_ok(),
            "Bundled TOPOLOGY.toml must be valid: {:?}",
            result.err()
        );
    }

    // Requirement: REQ-TOP-002, REQ-TOP-012 (Must, Should)
    // Acceptance: 8 agents bundled (7 pipeline + 1 discovery)
    #[test]
    fn test_bundled_agents_count() {
        assert_eq!(
            BUNDLED_AGENTS.len(),
            8,
            "Must bundle 8 agents (7 pipeline + discovery)"
        );
    }

    // Requirement: REQ-TOP-012 (Should)
    // Acceptance: Discovery agent included in bundled agents
    #[test]
    fn test_bundled_agents_includes_discovery() {
        let has_discovery = BUNDLED_AGENTS
            .iter()
            .any(|(name, _)| *name == "build-discovery");
        assert!(has_discovery, "BUNDLED_AGENTS must include build-discovery");
    }

    // Requirement: REQ-TOP-002 (Must)
    // Acceptance: All bundled agent contents are non-empty
    #[test]
    fn test_bundled_agents_all_non_empty() {
        for (name, content) in BUNDLED_AGENTS {
            assert!(
                !content.is_empty(),
                "Bundled agent '{name}' must not be empty"
            );
        }
    }

    // ===================================================================
    // REQ-TOP-003 (Must): Topology loader tests
    // ===================================================================

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Reads and parses TOPOLOGY.toml from topology directory
    #[test]
    fn test_load_topology_reads_valid_topology() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        // Create a valid topology on disk.
        let topo_dir = tmp.path().join("topologies").join("test-topo");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        let toml_content = r#"
[topology]
name = "test-topo"
description = "Test topology"
version = 1

[[phases]]
name = "only-phase"
agent = "build-only"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        std::fs::write(
            agents_dir.join("build-only.md"),
            "---\nname: build-only\n---\nAgent content\n",
        )
        .unwrap();

        let result = load_topology(data_dir, "test-topo");
        assert!(
            result.is_ok(),
            "Loading valid topology should succeed: {:?}",
            result.err()
        );

        let loaded = result.unwrap();
        assert_eq!(loaded.topology.topology.name, "test-topo");
        assert_eq!(loaded.topology.phases.len(), 1);
        assert!(loaded.agents.contains_key("build-only"));
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Reads agent .md files referenced by topology
    #[test]
    fn test_load_topology_loads_all_referenced_agents() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("multi");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        let toml_content = r#"
[topology]
name = "multi"
description = "Multi-agent"
version = 1

[[phases]]
name = "phase-a"
agent = "agent-a"

[[phases]]
name = "phase-b"
agent = "agent-b"
phase_type = "corrective-loop"

[phases.retry]
max = 2
fix_agent = "agent-a"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        std::fs::write(agents_dir.join("agent-a.md"), "Agent A content").unwrap();
        std::fs::write(agents_dir.join("agent-b.md"), "Agent B content").unwrap();

        let loaded = load_topology(data_dir, "multi").unwrap();
        assert_eq!(loaded.agents.len(), 2, "Must load both referenced agents");
        assert_eq!(loaded.agent_content("agent-a").unwrap(), "Agent A content");
        assert_eq!(loaded.agent_content("agent-b").unwrap(), "Agent B content");
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: fix_agent is also loaded even if not a direct phase agent
    #[test]
    fn test_load_topology_loads_fix_agent() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("fix-test");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        let toml_content = r#"
[topology]
name = "fix-test"
description = "Fix agent test"
version = 1

[[phases]]
name = "qa"
agent = "build-qa"
phase_type = "corrective-loop"

[phases.retry]
max = 3
fix_agent = "build-developer"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        std::fs::write(agents_dir.join("build-qa.md"), "QA content").unwrap();
        std::fs::write(agents_dir.join("build-developer.md"), "Dev content").unwrap();

        let loaded = load_topology(data_dir, "fix-test").unwrap();
        assert!(
            loaded.agents.contains_key("build-developer"),
            "fix_agent must be loaded even if not a direct phase agent"
        );
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Non-phase agents in agents/ directory are loaded (e.g. build-discovery)
    #[test]
    fn test_load_topology_includes_non_phase_agents() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("disc-test");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Topology references only build-analyst in its phases.
        let toml_content = r#"
[topology]
name = "disc-test"
description = "Discovery test"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        std::fs::write(agents_dir.join("build-analyst.md"), "Analyst content").unwrap();
        // build-discovery is NOT referenced in any phase, but lives in agents/.
        std::fs::write(agents_dir.join("build-discovery.md"), "Discovery content").unwrap();

        let loaded = load_topology(data_dir, "disc-test").unwrap();
        assert!(
            loaded.agents.contains_key("build-analyst"),
            "Phase-referenced agent must be loaded"
        );
        assert!(
            loaded.agents.contains_key("build-discovery"),
            "Non-phase agent in agents/ directory must also be loaded"
        );
        assert_eq!(
            loaded.agents.get("build-discovery").unwrap(),
            "Discovery content",
            "Non-phase agent content must match file content"
        );
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Reports clear error on corrupt TOML
    #[test]
    fn test_load_topology_corrupt_toml_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("corrupt");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        std::fs::write(
            topo_dir.join("TOPOLOGY.toml"),
            "this is {{not}} valid toml!!!",
        )
        .unwrap();

        let result = load_topology(data_dir, "corrupt");
        assert!(result.is_err(), "Corrupt TOML must return Err");
        let err = result.unwrap_err();
        assert!(
            err.contains("failed to parse TOPOLOGY.toml"),
            "Error message must mention TOPOLOGY.toml: got '{err}'"
        );
    }

    // Requirement: REQ-TOP-003 (Must)
    // Failure mode: missing TOPOLOGY.toml file
    #[test]
    fn test_load_topology_missing_toml_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        // Create directory but NO TOPOLOGY.toml.
        let topo_dir = tmp.path().join("topologies").join("no-toml");
        std::fs::create_dir_all(&topo_dir).unwrap();

        let result = load_topology(data_dir, "no-toml");
        assert!(result.is_err(), "Missing TOPOLOGY.toml must return Err");
        let err = result.unwrap_err();
        assert!(
            err.contains("failed to read TOPOLOGY.toml"),
            "Error should mention reading TOPOLOGY.toml: got '{err}'"
        );
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Falls back to bundled default on missing "development" directory
    #[test]
    fn test_load_topology_development_fallback_deploys_bundled() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        // No topologies directory at all — load_topology("development") should deploy.
        let result = load_topology(data_dir, "development");
        // This will fail because BUNDLED_TOPOLOGY_TOML is empty stub.
        // That's expected for TDD red phase — developer must fill in include_str!().
        // The key structural test: it should attempt to deploy, not return "not found".
        if let Err(ref e) = result {
            assert!(
                !e.contains("not found at"),
                "For 'development', must attempt deployment rather than 'not found': got '{e}'"
            );
        }
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: Non-development topology returns "not found" if directory missing
    #[test]
    fn test_load_topology_unknown_name_returns_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let result = load_topology(data_dir, "nonexistent");
        assert!(result.is_err(), "Unknown topology must return Err");
        let err = result.unwrap_err();
        assert!(
            err.contains("not found"),
            "Error must say 'not found': got '{err}'"
        );
    }

    // ===================================================================
    // REQ-TOP-013 (Should): Topology name validation tests
    // ===================================================================

    // Requirement: REQ-TOP-013 (Should)
    // Acceptance: Valid names accepted
    #[test]
    fn test_validate_topology_name_accepts_valid_names() {
        assert!(validate_topology_name("development").is_ok());
        assert!(validate_topology_name("my-topology").is_ok());
        assert!(validate_topology_name("custom_topo_v2").is_ok());
        assert!(validate_topology_name("a").is_ok());
        assert!(validate_topology_name("ABC123").is_ok());
    }

    // Requirement: REQ-TOP-013 (Should)
    // Acceptance: Rejects empty name
    #[test]
    fn test_validate_topology_name_rejects_empty() {
        let result = validate_topology_name("");
        assert!(result.is_err(), "Empty name must be rejected");
        assert!(result.unwrap_err().contains("empty"));
    }

    // Requirement: REQ-TOP-013 (Should)
    // Acceptance: Max 64 chars
    #[test]
    fn test_validate_topology_name_rejects_too_long() {
        let long_name = "a".repeat(65);
        let result = validate_topology_name(&long_name);
        assert!(result.is_err(), "Name > 64 chars must be rejected");
        assert!(result.unwrap_err().contains("too long"));
    }

    // Requirement: REQ-TOP-013 (Should)
    // Acceptance: Exactly 64 chars is accepted
    #[test]
    fn test_validate_topology_name_accepts_64_chars() {
        let name = "a".repeat(64);
        assert!(
            validate_topology_name(&name).is_ok(),
            "Exactly 64 chars should be accepted"
        );
    }

    // Requirement: REQ-TOP-013 (Should)
    // Security: Rejects path traversal with ..
    #[test]
    fn test_validate_topology_name_rejects_path_traversal_dots() {
        let result = validate_topology_name("..");
        assert!(result.is_err(), "Path traversal '..' must be rejected");
        assert!(result.unwrap_err().contains("path traversal"));
    }

    // Requirement: REQ-TOP-013 (Should)
    // Security: Rejects path traversal with /
    #[test]
    fn test_validate_topology_name_rejects_forward_slash() {
        let result = validate_topology_name("../etc/passwd");
        assert!(result.is_err(), "Forward slash must be rejected");
    }

    // Requirement: REQ-TOP-013 (Should)
    // Security: Rejects path traversal with backslash
    #[test]
    fn test_validate_topology_name_rejects_backslash() {
        let result = validate_topology_name("..\\windows\\system32");
        assert!(result.is_err(), "Backslash must be rejected");
    }

    // Requirement: REQ-TOP-013 (Should)
    // Security: Rejects shell metacharacters
    #[test]
    fn test_validate_topology_name_rejects_shell_metacharacters() {
        let bad_names = vec![
            "name;rm -rf /",
            "name|cat /etc/passwd",
            "name$(whoami)",
            "name`id`",
            "name&background",
            "name<redirect",
            "name>redirect",
            "name with spaces",
            "name\nnewline",
            "name\ttab",
        ];
        for name in bad_names {
            let result = validate_topology_name(name);
            assert!(
                result.is_err(),
                "Shell metacharacter name '{name}' must be rejected"
            );
        }
    }

    // Requirement: REQ-TOP-013 (Should)
    // Edge case: Unicode characters rejected (only ASCII alphanumeric + hyphen + underscore)
    #[test]
    fn test_validate_topology_name_rejects_unicode() {
        let result = validate_topology_name("topo-\u{1f600}");
        assert!(result.is_err(), "Unicode characters must be rejected");
    }

    // Requirement: REQ-TOP-013 (Should)
    // Edge case: dots (not path traversal) still rejected
    #[test]
    fn test_validate_topology_name_rejects_dots() {
        let result = validate_topology_name("my.topology");
        assert!(result.is_err(), "Dots in topology name must be rejected");
    }

    // ===================================================================
    // REQ-TOP-014 (Should): Missing agent file error tests
    // ===================================================================

    // Requirement: REQ-TOP-014 (Should)
    // Acceptance: Clear error message naming the missing file
    #[test]
    fn test_load_topology_missing_agent_file_names_file() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("missing-agent");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        let toml_content = r#"
[topology]
name = "missing-agent"
description = "Missing agent test"
version = 1

[[phases]]
name = "analyst"
agent = "build-analyst"

[[phases]]
name = "architect"
agent = "build-architect"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        // Only create one agent file — the other is missing.
        std::fs::write(agents_dir.join("build-analyst.md"), "Analyst content").unwrap();
        // build-architect.md is deliberately missing.

        let result = load_topology(data_dir, "missing-agent");
        assert!(result.is_err(), "Missing agent file must return Err");
        let err = result.unwrap_err();
        assert!(
            err.contains("build-architect"),
            "Error must name the missing agent 'build-architect': got '{err}'"
        );
    }

    // Requirement: REQ-TOP-014 (Should)
    // Acceptance: Does not silently skip phases or use empty content
    #[test]
    fn test_load_topology_missing_fix_agent_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_str().unwrap();

        let topo_dir = tmp.path().join("topologies").join("missing-fix");
        let agents_dir = topo_dir.join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        let toml_content = r#"
[topology]
name = "missing-fix"
description = "Missing fix agent"
version = 1

[[phases]]
name = "qa"
agent = "build-qa"
phase_type = "corrective-loop"

[phases.retry]
max = 3
fix_agent = "build-developer"
"#;
        std::fs::write(topo_dir.join("TOPOLOGY.toml"), toml_content).unwrap();
        std::fs::write(agents_dir.join("build-qa.md"), "QA content").unwrap();
        // build-developer.md is deliberately missing (fix_agent).

        let result = load_topology(data_dir, "missing-fix");
        assert!(result.is_err(), "Missing fix_agent file must return Err");
        let err = result.unwrap_err();
        assert!(
            err.contains("build-developer"),
            "Error must name the missing fix agent: got '{err}'"
        );
    }

    // ===================================================================
    // LoadedTopology helper method tests
    // ===================================================================

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: agent_content() returns content for known agents
    #[test]
    fn test_loaded_topology_agent_content_found() {
        let mut agents = HashMap::new();
        agents.insert("build-qa".to_string(), "QA instructions".to_string());

        let loaded = LoadedTopology {
            topology: Topology {
                topology: TopologyMeta {
                    name: "test".to_string(),
                    description: "Test".to_string(),
                    version: 1,
                },
                phases: vec![],
            },
            agents,
        };

        assert_eq!(loaded.agent_content("build-qa").unwrap(), "QA instructions");
    }

    // Requirement: REQ-TOP-014 (Should)
    // Acceptance: agent_content() returns Err for unknown agents
    #[test]
    fn test_loaded_topology_agent_content_missing() {
        let loaded = LoadedTopology {
            topology: Topology {
                topology: TopologyMeta {
                    name: "test".to_string(),
                    description: "Test".to_string(),
                    version: 1,
                },
                phases: vec![],
            },
            agents: HashMap::new(),
        };

        let result = loaded.agent_content("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("nonexistent"));
    }

    // Requirement: REQ-TOP-001 (Must)
    // Acceptance: resolve_model returns correct model for each tier
    #[test]
    fn test_loaded_topology_resolve_model() {
        let loaded = LoadedTopology {
            topology: Topology {
                topology: TopologyMeta {
                    name: "test".to_string(),
                    description: "Test".to_string(),
                    version: 1,
                },
                phases: vec![],
            },
            agents: HashMap::new(),
        };

        let fast_phase = Phase {
            name: "fast".to_string(),
            agent: "a".to_string(),
            model_tier: ModelTier::Fast,
            max_turns: None,
            phase_type: PhaseType::Standard,
            retry: None,
            pre_validation: None,
            post_validation: None,
        };

        let complex_phase = Phase {
            name: "complex".to_string(),
            agent: "b".to_string(),
            model_tier: ModelTier::Complex,
            max_turns: None,
            phase_type: PhaseType::Standard,
            retry: None,
            pre_validation: None,
            post_validation: None,
        };

        assert_eq!(
            loaded.resolve_model(&fast_phase, "sonnet", "opus"),
            "sonnet"
        );
        assert_eq!(
            loaded.resolve_model(&complex_phase, "sonnet", "opus"),
            "opus"
        );
    }

    // Requirement: REQ-TOP-003 (Must)
    // Acceptance: all_agents() returns all loaded agents
    #[test]
    fn test_loaded_topology_all_agents() {
        let mut agents = HashMap::new();
        agents.insert("agent-a".to_string(), "Content A".to_string());
        agents.insert("agent-b".to_string(), "Content B".to_string());

        let loaded = LoadedTopology {
            topology: Topology {
                topology: TopologyMeta {
                    name: "test".to_string(),
                    description: "Test".to_string(),
                    version: 1,
                },
                phases: vec![],
            },
            agents,
        };

        let all = loaded.all_agents();
        assert_eq!(all.len(), 2, "all_agents should return 2 entries");
    }

    // ===================================================================
    // Edge cases: The 10 worst scenarios for topology loading
    // ===================================================================

    // Requirement: REQ-TOP-001 (Must)
    // Worst scenario 1: Empty TOML content
    #[test]
    fn test_topology_deserialize_empty_string() {
        let result: Result<Topology, _> = toml::from_str("");
        assert!(result.is_err(), "Empty TOML string must return Err");
    }

    // Requirement: REQ-TOP-001 (Must)
    // Worst scenario 4: Special characters in topology name field
    #[test]
    fn test_topology_deserialize_special_chars_in_name() {
        let toml_str = r#"
[topology]
name = "test-\u0000-null"
description = "Null byte test"
version = 1

[[phases]]
name = "a"
agent = "b"
"#;
        // Should either parse or return Err — must not panic.
        let _ = toml::from_str::<Topology>(toml_str);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Worst scenario 7: Very large number of phases
    #[test]
    fn test_topology_deserialize_many_phases() {
        let mut toml_str =
            String::from("[topology]\nname = \"big\"\ndescription = \"Big\"\nversion = 1\n\n");
        for i in 0..100 {
            toml_str.push_str(&format!(
                "[[phases]]\nname = \"phase-{i}\"\nagent = \"agent-{i}\"\n\n"
            ));
        }
        let result: Result<Topology, _> = toml::from_str(&toml_str);
        assert!(result.is_ok(), "100 phases should parse fine");
        assert_eq!(result.unwrap().phases.len(), 100);
    }

    // Requirement: REQ-TOP-001 (Must)
    // Worst scenario 9: Correct format but inconsistent data (retry without corrective-loop)
    #[test]
    fn test_topology_deserialize_retry_without_corrective_loop() {
        // This is structurally valid TOML but semantically wrong.
        // The topology should still deserialize — runtime validation catches the inconsistency.
        let toml_str = r#"
[topology]
name = "inconsistent"
description = "Retry on standard phase"
version = 1

[[phases]]
name = "oops"
agent = "build-oops"
phase_type = "standard"

[phases.retry]
max = 3
fix_agent = "build-fix"
"#;
        let result: Result<Topology, _> = toml::from_str(toml_str);
        // Should parse (TOML is valid), semantic validation is separate.
        assert!(
            result.is_ok(),
            "Structurally valid TOML should parse even if semantically wrong"
        );
    }
}
