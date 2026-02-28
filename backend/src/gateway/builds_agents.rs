//! Build agent file lifecycle — RAII guard for writing agent .md files.
//!
//! Agent content is loaded from `topologies/development/agents/` via `include_str!()`.
//! The `AgentFilesGuard` writes them to `<project_dir>/.claude/agents/` at runtime
//! and removes them on drop (RAII). Reference-counted for concurrent builds.
//!
//! Public interface:
//! - BUILD_AGENTS: &[(&str, &str)] mapping names to content (backward compat)
//! - Individual BUILD_*_AGENT constants (backward compat, used by tests)
//! - AgentFilesGuard::write(project_dir) — writes from const BUILD_AGENTS
//! - AgentFilesGuard::write_from_topology(project_dir, topology) — writes from LoadedTopology

use super::builds_topology::LoadedTopology;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

/// Global reference counter for concurrent AgentFilesGuard instances.
/// Only the last guard to drop deletes the agent files.
static GUARD_REFCOUNTS: LazyLock<Mutex<HashMap<PathBuf, usize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// ---------------------------------------------------------------------------
// Agent constants — loaded from topology .md files via include_str!()
// Kept for backward compatibility: used by tests to verify bundled content.
// Production code uses LoadedTopology.agents from write_from_topology().
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub(super) const BUILD_ANALYST_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-analyst.md");

#[allow(dead_code)]
pub(super) const BUILD_ARCHITECT_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-architect.md");

#[allow(dead_code)]
pub(super) const BUILD_TEST_WRITER_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-test-writer.md");

#[allow(dead_code)]
pub(super) const BUILD_DEVELOPER_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-developer.md");

#[allow(dead_code)]
pub(super) const BUILD_QA_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-qa.md");

#[allow(dead_code)]
pub(super) const BUILD_REVIEWER_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-reviewer.md");

#[allow(dead_code)]
pub(super) const BUILD_DELIVERY_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-delivery.md");

#[allow(dead_code)]
pub(super) const BUILD_DISCOVERY_AGENT: &str =
    include_str!("../../../topologies/development/agents/build-discovery.md");

/// Name-to-content mapping for all 8 build agents (discovery + 7 pipeline phases).
#[allow(dead_code)]
pub(super) const BUILD_AGENTS: &[(&str, &str)] = &[
    ("build-discovery", BUILD_DISCOVERY_AGENT),
    ("build-analyst", BUILD_ANALYST_AGENT),
    ("build-architect", BUILD_ARCHITECT_AGENT),
    ("build-test-writer", BUILD_TEST_WRITER_AGENT),
    ("build-developer", BUILD_DEVELOPER_AGENT),
    ("build-qa", BUILD_QA_AGENT),
    ("build-reviewer", BUILD_REVIEWER_AGENT),
    ("build-delivery", BUILD_DELIVERY_AGENT),
];

// ---------------------------------------------------------------------------
// Agent file lifecycle — RAII guard
// ---------------------------------------------------------------------------

/// RAII guard that writes agent `.md` files on creation and removes them on drop.
///
/// Reference-counted per directory: multiple concurrent builds share the same agent files.
/// Files are only deleted when the last guard for that directory is dropped.
pub(super) struct AgentFilesGuard {
    agents_dir: PathBuf,
}

impl AgentFilesGuard {
    /// Write all build agent files to `<project_dir>/.claude/agents/`.
    ///
    /// Uses the compiled-in BUILD_AGENTS constants.
    /// Increments the per-directory reference count. Safe to call concurrently —
    /// all guards write identical content to the same directory.
    /// Kept for backward compatibility — production code uses write_from_topology().
    #[allow(dead_code)]
    pub async fn write(project_dir: &Path) -> std::io::Result<Self> {
        let agents_dir = project_dir.join(".claude").join("agents");
        tokio::fs::create_dir_all(&agents_dir).await?;
        for (name, content) in BUILD_AGENTS {
            let path = agents_dir.join(format!("{name}.md"));
            tokio::fs::write(&path, content).await?;
        }
        let mut counts = GUARD_REFCOUNTS.lock().unwrap();
        *counts.entry(agents_dir.clone()).or_insert(0) += 1;
        Ok(Self { agents_dir })
    }

    /// Write agent files from a loaded topology to `<project_dir>/.claude/agents/`.
    ///
    /// Replaces the old `write()` source with topology-loaded content. Same RAII
    /// behavior: increments ref count, files cleaned up on last guard drop.
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

    /// Current number of active guards for a given directory (for testing).
    #[cfg(test)]
    pub fn active_count_for(dir: &Path) -> usize {
        let counts = GUARD_REFCOUNTS.lock().unwrap();
        counts.get(dir).copied().unwrap_or(0)
    }
}

impl Drop for AgentFilesGuard {
    fn drop(&mut self) {
        let should_cleanup = {
            let mut counts = GUARD_REFCOUNTS.lock().unwrap();
            if let Some(count) = counts.get_mut(&self.agents_dir) {
                *count -= 1;
                if *count == 0 {
                    counts.remove(&self.agents_dir);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };
        // Only the last guard for this directory cleans up.
        if should_cleanup {
            let _ = std::fs::remove_dir_all(&self.agents_dir);
            // Remove the parent .claude/ directory if it is now empty.
            if let Some(claude_dir) = self.agents_dir.parent() {
                let _ = std::fs::remove_dir(claude_dir);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ===================================================================
    // REQ-BAP-002 (Must): Embedded agent content — 7 agent definitions
    // ===================================================================

    // Requirement: REQ-BAP-002 (Must), REQ-BDP-002 (Must)
    // Acceptance: all 8 build agent definitions compiled into the binary
    // (7 original pipeline agents + 1 discovery agent)
    #[test]
    fn test_build_agents_has_exactly_8_entries() {
        assert_eq!(
            BUILD_AGENTS.len(),
            8,
            "BUILD_AGENTS must contain exactly 8 agent definitions (7 pipeline + 1 discovery)"
        );
    }

    // Requirement: REQ-BAP-002 (Must), REQ-BDP-002 (Must)
    // Acceptance: correct agent names in the mapping (discovery + 7 pipeline agents)
    #[test]
    fn test_build_agents_correct_names() {
        let expected_names = [
            "build-discovery",
            "build-analyst",
            "build-architect",
            "build-test-writer",
            "build-developer",
            "build-qa",
            "build-reviewer",
            "build-delivery",
        ];
        let actual_names: Vec<&str> = BUILD_AGENTS.iter().map(|(name, _)| *name).collect();
        assert_eq!(
            actual_names, expected_names,
            "Agent names must match expected order: discovery first, then 7-phase pipeline"
        );
    }

    // Requirement: REQ-BAP-002 (Must)
    // Acceptance: no .md files shipped on disk; content accessible via constants
    #[test]
    fn test_build_agent_constants_are_non_empty() {
        assert!(
            !BUILD_ANALYST_AGENT.is_empty(),
            "BUILD_ANALYST_AGENT must not be empty"
        );
        assert!(
            !BUILD_ARCHITECT_AGENT.is_empty(),
            "BUILD_ARCHITECT_AGENT must not be empty"
        );
        assert!(
            !BUILD_TEST_WRITER_AGENT.is_empty(),
            "BUILD_TEST_WRITER_AGENT must not be empty"
        );
        assert!(
            !BUILD_DEVELOPER_AGENT.is_empty(),
            "BUILD_DEVELOPER_AGENT must not be empty"
        );
        assert!(
            !BUILD_QA_AGENT.is_empty(),
            "BUILD_QA_AGENT must not be empty"
        );
        assert!(
            !BUILD_REVIEWER_AGENT.is_empty(),
            "BUILD_REVIEWER_AGENT must not be empty"
        );
        assert!(
            !BUILD_DELIVERY_AGENT.is_empty(),
            "BUILD_DELIVERY_AGENT must not be empty"
        );
    }

    // Requirement: REQ-BAP-002 (Must)
    // Acceptance: each agent has YAML frontmatter
    #[test]
    fn test_build_agents_have_yaml_frontmatter() {
        for (name, content) in BUILD_AGENTS {
            assert!(
                content.starts_with("---"),
                "Agent '{name}' must start with YAML frontmatter delimiter '---'"
            );
            // Must have a closing --- delimiter.
            let after_open = &content[3..];
            assert!(
                after_open.contains("\n---"),
                "Agent '{name}' must have closing YAML frontmatter delimiter '---'"
            );
        }
    }

    // Requirement: REQ-BAP-002 (Must)
    // Acceptance: each agent frontmatter contains required keys
    #[test]
    fn test_build_agents_frontmatter_required_keys() {
        let required_keys = [
            "name:",
            "description:",
            "tools:",
            "model:",
            "permissionMode:",
        ];
        for (agent_name, content) in BUILD_AGENTS {
            // Extract frontmatter (between first --- and second ---).
            let after_open = &content[3..];
            let close_idx = after_open
                .find("\n---")
                .unwrap_or_else(|| panic!("Agent '{agent_name}' missing closing ---"));
            let frontmatter = &after_open[..close_idx];

            for key in &required_keys {
                assert!(
                    frontmatter.contains(key),
                    "Agent '{agent_name}' frontmatter must contain '{key}'"
                );
            }
        }
    }

    // Requirement: REQ-BAP-002 (Must)
    // Acceptance: frontmatter name matches the mapping key
    #[test]
    fn test_build_agents_frontmatter_name_matches_key() {
        for (agent_name, content) in BUILD_AGENTS {
            let after_open = &content[3..];
            let close_idx = after_open.find("\n---").unwrap();
            let frontmatter = &after_open[..close_idx];

            // Find the "name:" line and extract value.
            let name_line = frontmatter
                .lines()
                .find(|l| l.starts_with("name:"))
                .unwrap_or_else(|| panic!("Agent '{agent_name}' has no name: line"));
            let name_value = name_line["name:".len()..].trim();
            assert_eq!(
                name_value, *agent_name,
                "Agent frontmatter name '{name_value}' must match mapping key '{agent_name}'"
            );
        }
    }

    // ===================================================================
    // REQ-BAP-014 (Must): Permission bypass in build agents
    // ===================================================================

    // Requirement: REQ-BAP-014 (Must)
    // Acceptance: build agents use bypassPermissions
    #[test]
    fn test_build_agents_permission_bypass() {
        for (name, content) in BUILD_AGENTS {
            assert!(
                content.contains("permissionMode: bypassPermissions"),
                "Agent '{name}' must have permissionMode: bypassPermissions"
            );
        }
    }

    // ===================================================================
    // REQ-BAP-011 (Must): Non-interactive build agents
    // ===================================================================

    // Requirement: REQ-BAP-011 (Must)
    // Acceptance: "Do NOT ask questions" in every agent
    #[test]
    fn test_build_agents_non_interactive() {
        for (name, content) in BUILD_AGENTS {
            let lower = content.to_lowercase();
            assert!(
                lower.contains("do not ask question")
                    || lower.contains("don't ask question")
                    || lower.contains("never ask question")
                    || lower.contains("do not ask the user")
                    || lower.contains("never ask the user"),
                "Agent '{name}' must contain non-interactive instruction \
                 (e.g. 'Do NOT ask questions')"
            );
        }
    }

    // Requirement: REQ-BAP-011 (Must)
    // Acceptance: "Make reasonable defaults for anything ambiguous"
    #[test]
    fn test_build_agents_reasonable_defaults_instruction() {
        for (name, content) in BUILD_AGENTS {
            let lower = content.to_lowercase();
            assert!(
                lower.contains("reasonable default")
                    || lower.contains("sensible default")
                    || lower.contains("make default")
                    || lower.contains("assume reasonable"),
                "Agent '{name}' must instruct making reasonable defaults for ambiguity"
            );
        }
    }

    // ===================================================================
    // REQ-BAP-012 (Must): Analyst output format
    // ===================================================================

    // Requirement: REQ-BAP-012 (Must)
    // Acceptance: analyst agent instructions include parseable output format
    #[test]
    fn test_analyst_agent_output_format() {
        let content = BUILD_ANALYST_AGENT;
        assert!(
            content.contains("PROJECT_NAME"),
            "Analyst agent must reference PROJECT_NAME output format"
        );
        assert!(
            content.contains("LANGUAGE"),
            "Analyst agent must reference LANGUAGE output format"
        );
        assert!(
            content.contains("SCOPE"),
            "Analyst agent must reference SCOPE output format"
        );
        assert!(
            content.contains("COMPONENTS"),
            "Analyst agent must reference COMPONENTS output format"
        );
    }

    // ===================================================================
    // REQ-BAP-021 (Should): Agent tool restrictions per role
    // ===================================================================

    // Requirement: REQ-BAP-021 (Should)
    // Acceptance: Analyst has restricted tools (Read, Grep, Glob)
    #[test]
    fn test_analyst_agent_restricted_tools() {
        let after_open = &BUILD_ANALYST_AGENT[3..];
        let close_idx = after_open.find("\n---").unwrap();
        let frontmatter = &after_open[..close_idx];
        let tools_line = frontmatter
            .lines()
            .find(|l| l.starts_with("tools:"))
            .expect("Analyst must have tools: in frontmatter");
        // Analyst should NOT have Write or Edit tools.
        assert!(
            !tools_line.contains("Write"),
            "Analyst should not have Write tool"
        );
        assert!(
            !tools_line.contains("Edit"),
            "Analyst should not have Edit tool"
        );
        // Should have Read.
        assert!(tools_line.contains("Read"), "Analyst must have Read tool");
    }

    // Requirement: REQ-BAP-021 (Should)
    // Acceptance: Reviewer has tools (Read, Write, Grep, Glob, Bash)
    #[test]
    fn test_reviewer_agent_tools() {
        let after_open = &BUILD_REVIEWER_AGENT[3..];
        let close_idx = after_open.find("\n---").unwrap();
        let frontmatter = &after_open[..close_idx];
        let tools_line = frontmatter
            .lines()
            .find(|l| l.starts_with("tools:"))
            .expect("Reviewer must have tools: in frontmatter");
        // Reviewer should NOT have Edit tool (cannot modify source code).
        assert!(
            !tools_line.contains("Edit"),
            "Reviewer should not have Edit tool"
        );
        // Should have Read, Write (for report), and Bash.
        assert!(tools_line.contains("Read"), "Reviewer must have Read tool");
        assert!(
            tools_line.contains("Write"),
            "Reviewer must have Write tool for report"
        );
        assert!(tools_line.contains("Bash"), "Reviewer must have Bash tool");
    }

    // Requirement: REQ-BAP-021 (Should)
    // Acceptance: Developer/Test-writer/QA/Delivery have full tools
    #[test]
    fn test_developer_agents_have_full_tools() {
        let full_tool_agents = [
            ("build-test-writer", BUILD_TEST_WRITER_AGENT),
            ("build-developer", BUILD_DEVELOPER_AGENT),
            ("build-qa", BUILD_QA_AGENT),
            ("build-delivery", BUILD_DELIVERY_AGENT),
        ];
        for (name, content) in full_tool_agents {
            let after_open = &content[3..];
            let close_idx = after_open.find("\n---").unwrap();
            let frontmatter = &after_open[..close_idx];
            let tools_line = frontmatter
                .lines()
                .find(|l| l.starts_with("tools:"))
                .unwrap_or_else(|| panic!("Agent '{name}' must have tools:"));
            assert!(
                tools_line.contains("Read"),
                "Agent '{name}' must have Read tool"
            );
            assert!(
                tools_line.contains("Write"),
                "Agent '{name}' must have Write tool"
            );
            assert!(
                tools_line.contains("Edit"),
                "Agent '{name}' must have Edit tool"
            );
            assert!(
                tools_line.contains("Bash"),
                "Agent '{name}' must have Bash tool"
            );
        }
    }

    // ===================================================================
    // REQ-BAP-025 (Could): maxTurns in frontmatter
    // ===================================================================

    // Requirement: REQ-BAP-025 (Could)
    // Acceptance: analyst has maxTurns: 25 in frontmatter
    #[test]
    fn test_analyst_agent_max_turns() {
        let after_open = &BUILD_ANALYST_AGENT[3..];
        let close_idx = after_open.find("\n---").unwrap();
        let frontmatter = &after_open[..close_idx];
        assert!(
            frontmatter.contains("maxTurns:"),
            "Analyst agent should have maxTurns in frontmatter"
        );
    }

    // ===================================================================
    // REQ-BAP-001 (Must): Agent file lifecycle — AgentFilesGuard
    // ===================================================================

    // Requirement: REQ-BAP-001 (Must)
    // Acceptance: Agent files written to <project_dir>/.claude/agents/ before phase invocation
    #[tokio::test]
    async fn test_agent_files_guard_writes_all_agent_files() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_write__");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let guard = AgentFilesGuard::write(&tmp).await.unwrap();
        let agents_dir = tmp.join(".claude").join("agents");

        assert!(agents_dir.exists(), ".claude/agents/ directory must exist");

        // Verify all 7 agent files were written.
        for (name, _content) in BUILD_AGENTS {
            let file_path = agents_dir.join(format!("{name}.md"));
            assert!(
                file_path.exists(),
                "Agent file '{name}.md' must exist after write"
            );
            let file_content = std::fs::read_to_string(&file_path).unwrap();
            assert!(
                !file_content.is_empty(),
                "Agent file '{name}.md' must not be empty"
            );
            assert!(
                file_content.starts_with("---"),
                "Agent file '{name}.md' must start with YAML frontmatter"
            );
        }

        // Cleanup.
        drop(guard);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Acceptance: Agent file content matches the embedded constant
    #[tokio::test]
    async fn test_agent_files_guard_content_matches_constants() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_content__");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let guard = AgentFilesGuard::write(&tmp).await.unwrap();
        let agents_dir = tmp.join(".claude").join("agents");

        for (name, expected_content) in BUILD_AGENTS {
            let file_path = agents_dir.join(format!("{name}.md"));
            let actual_content = std::fs::read_to_string(&file_path).unwrap();
            assert_eq!(
                actual_content, *expected_content,
                "File content for '{name}.md' must match BUILD_AGENTS constant"
            );
        }

        drop(guard);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Acceptance: cleanup runs even on panic (RAII guard pattern) — test Drop
    #[tokio::test]
    async fn test_agent_files_guard_drop_cleans_up() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_drop__");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let agents_dir = tmp.join(".claude").join("agents");

        {
            let _guard = AgentFilesGuard::write(&tmp).await.unwrap();
            assert!(
                agents_dir.exists(),
                "agents/ must exist while guard is alive"
            );
            // Guard goes out of scope here — Drop should clean up.
        }

        assert!(
            !agents_dir.exists(),
            ".claude/agents/ must be removed after guard is dropped"
        );

        // Also verify .claude/ directory is removed (if empty).
        let claude_dir = tmp.join(".claude");
        assert!(
            !claude_dir.exists(),
            ".claude/ should be removed if empty after guard drop"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Failure mode: project_dir doesn't exist
    #[tokio::test]
    async fn test_agent_files_guard_creates_directory_hierarchy() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_nested__");
        let _ = std::fs::remove_dir_all(&tmp);
        // Do NOT create tmp — the guard must create the full path.
        std::fs::create_dir_all(&tmp).unwrap();

        let nested = tmp.join("deep").join("nested").join("project");
        // nested doesn't exist yet.
        assert!(!nested.exists());

        // Guard should create_dir_all internally.
        let guard = AgentFilesGuard::write(&nested).await.unwrap();
        let agents_dir = nested.join(".claude").join("agents");
        assert!(
            agents_dir.exists(),
            "Guard must create full directory hierarchy"
        );

        drop(guard);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Edge case: overwrite behavior when .claude/agents/ already exists
    #[tokio::test]
    async fn test_agent_files_guard_overwrites_existing_files() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_overwrite__");
        let _ = std::fs::remove_dir_all(&tmp);
        let agents_dir = tmp.join(".claude").join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();

        // Write a pre-existing file with stale content.
        let stale_file = agents_dir.join("build-analyst.md");
        std::fs::write(&stale_file, "stale content").unwrap();

        let guard = AgentFilesGuard::write(&tmp).await.unwrap();

        // File should be overwritten with correct content.
        let content = std::fs::read_to_string(&stale_file).unwrap();
        assert_ne!(content, "stale content", "Must overwrite existing files");
        assert!(
            content.starts_with("---"),
            "Overwritten content must be valid agent definition"
        );

        drop(guard);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Edge case: multiple guards for the same directory
    #[tokio::test]
    async fn test_agent_files_guard_second_write_succeeds() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_double__");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let guard1 = AgentFilesGuard::write(&tmp).await.unwrap();
        drop(guard1); // Clean up first.

        // Second write should succeed even though directory was removed.
        let guard2 = AgentFilesGuard::write(&tmp).await.unwrap();
        let agents_dir = tmp.join(".claude").join("agents");
        assert!(agents_dir.exists());

        drop(guard2);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BAP-001 (Must)
    // Edge case: guard Drop doesn't panic if files already removed
    #[tokio::test]
    async fn test_agent_files_guard_drop_idempotent() {
        let tmp = std::env::temp_dir().join("__omega_test_agents_idempotent__");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let guard = AgentFilesGuard::write(&tmp).await.unwrap();
        let agents_dir = tmp.join(".claude").join("agents");

        // Manually delete the directory before drop.
        std::fs::remove_dir_all(&agents_dir).unwrap();

        // Drop should NOT panic.
        drop(guard);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ===================================================================
    // REQ-BAP-019 (Should): QA outputs parseable VERIFICATION marker
    // ===================================================================

    // Requirement: REQ-BAP-019 (Should)
    // Acceptance: QA agent instructions include VERIFICATION: PASS/FAIL output
    #[test]
    fn test_qa_agent_verification_output_format() {
        let content = BUILD_QA_AGENT;
        assert!(
            content.contains("VERIFICATION: PASS") || content.contains("VERIFICATION:"),
            "QA agent must instruct outputting VERIFICATION: PASS/FAIL"
        );
    }

    // ===================================================================
    // REQ-BAP-020 (Should): Reviewer outputs parseable REVIEW marker
    // ===================================================================

    // Requirement: REQ-BAP-020 (Should)
    // Acceptance: Reviewer agent outputs REVIEW: PASS/FAIL
    #[test]
    fn test_reviewer_agent_review_output_format() {
        let content = BUILD_REVIEWER_AGENT;
        assert!(
            content.contains("REVIEW: PASS") || content.contains("REVIEW:"),
            "Reviewer agent must instruct outputting REVIEW: PASS/FAIL"
        );
    }

    // ===================================================================
    // REQ-BAP-016 (Should): Architect creates TDD-ready specs
    // ===================================================================

    // Requirement: REQ-BAP-016 (Should)
    // Acceptance: architect agent mentions specs/ and testable criteria
    #[test]
    fn test_architect_agent_tdd_specs() {
        let content = BUILD_ARCHITECT_AGENT;
        assert!(
            content.contains("specs/") || content.contains("specs\\"),
            "Architect agent must reference specs/ directory"
        );
        assert!(
            content.to_lowercase().contains("test")
                || content.to_lowercase().contains("acceptance"),
            "Architect agent must mention testable/acceptance criteria"
        );
    }

    // ===================================================================
    // REQ-BAP-017 (Should): Test writer references specs
    // ===================================================================

    // Requirement: REQ-BAP-017 (Should)
    // Acceptance: test-writer reads specs/ and writes tests
    #[test]
    fn test_test_writer_agent_references_specs() {
        let content = BUILD_TEST_WRITER_AGENT;
        assert!(
            content.contains("specs/") || content.contains("specs\\"),
            "Test-writer agent must reference specs/ directory"
        );
        assert!(
            content.to_lowercase().contains("fail"),
            "Test-writer agent must mention tests failing initially (TDD red phase)"
        );
    }

    // ===================================================================
    // REQ-BAP-018 (Should): Developer reads tests first
    // ===================================================================

    // Requirement: REQ-BAP-018 (Should)
    // Acceptance: developer reads tests before implementing
    #[test]
    fn test_developer_agent_reads_tests_first() {
        let content = BUILD_DEVELOPER_AGENT;
        assert!(
            content.to_lowercase().contains("test"),
            "Developer agent must reference tests"
        );
    }

    // ===================================================================
    // REQ-BAP-018 (Should): 500-line file limit
    // ===================================================================

    // Requirement: REQ-BAP-018 (Should)
    // Acceptance: 500-line file limit enforced in developer agent
    #[test]
    fn test_developer_agent_500_line_limit() {
        let content = BUILD_DEVELOPER_AGENT;
        assert!(
            content.contains("500")
                || content.contains("file limit")
                || content.contains("line limit"),
            "Developer agent should enforce 500-line file limit"
        );
    }
}
