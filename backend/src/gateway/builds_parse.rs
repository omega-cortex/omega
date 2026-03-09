//! Pure parsing functions, data structures, and prompt templates for the build pipeline.

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Parsed output from Phase 1 (Clarification).
///
/// Fields beyond `name` and `scope` are parsed for completeness and passed to Phase 2
/// via the raw `brief_text` string. They are available for future orchestrator logic
/// (e.g., conditional frontend phase, language-specific verification commands).
#[allow(dead_code)]
pub(super) struct ProjectBrief {
    pub(super) name: String,
    pub(super) language: String,
    pub(super) database: String,
    pub(super) frontend: bool,
    pub(super) scope: String,
    pub(super) components: Vec<String>,
}

/// Result of Phase 4 (Verification).
pub(super) enum VerificationResult {
    Pass,
    Fail(String),
}

/// Parsed output from Phase 5 (Delivery).
pub(super) struct BuildSummary {
    pub(super) project: String,
    pub(super) location: String,
    pub(super) language: String,
    pub(super) summary: String,
    pub(super) usage: String,
    pub(super) skill: Option<String>,
}

/// Result of Phase 6 (Review).
pub(super) enum ReviewResult {
    Pass,
    Fail(String),
}

/// Snapshot of build pipeline progress — written to `docs/.workflow/chain-state.md`
/// on failure so the user can resume or inspect partial results.
pub(super) struct ChainState {
    pub(super) project_name: String,
    pub(super) project_dir: String,
    pub(super) completed_phases: Vec<String>,
    pub(super) failed_phase: Option<String>,
    pub(super) failure_reason: Option<String>,
    /// Which topology was used for this build (REQ-TOP-015).
    pub(super) topology_name: Option<String>,
}

// Phase prompt templates have been replaced by embedded agent definitions
// in builds_agents.rs. Each agent's instructions are compiled into the binary
// and written as temporary files via AgentFilesGuard.

// ---------------------------------------------------------------------------
// Pure parsing functions (testable without mocking)
// ---------------------------------------------------------------------------

/// Strip markdown bold markers (`**`) and leading whitespace from a line.
fn strip_markdown(line: &str) -> String {
    line.trim().replace("**", "")
}

/// Parse structured output from Phase 1 into a `ProjectBrief`.
///
/// Resilient to LLM output that wraps fields in markdown bold (`**PROJECT_NAME:**`)
/// or includes prose before the structured fields.
pub(super) fn parse_project_brief(text: &str) -> Option<ProjectBrief> {
    let get_field = |key: &str| -> Option<String> {
        text.lines()
            .map(strip_markdown)
            .find(|line| line.starts_with(&format!("{key}:")))
            .map(|line| line[key.len() + 1..].trim().to_string())
    };

    let name = get_field("PROJECT_NAME")?;
    // Strip backticks that LLMs sometimes wrap values in.
    let name = name.trim_matches('`').trim().to_string();
    // Strict validation: alphanumeric start, hyphens/underscores allowed, max 64 chars.
    // Rejects spaces, shell metacharacters, path traversal, and unicode control chars.
    if name.is_empty()
        || name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        || name.starts_with('.')
        || name.contains("..")
    {
        return None;
    }

    let language = get_field("LANGUAGE").unwrap_or_else(|| "Rust".to_string());
    let database = get_field("DATABASE").unwrap_or_else(|| "SQLite".to_string());
    let frontend = get_field("FRONTEND")
        .map(|v| v.to_lowercase().starts_with('y'))
        .unwrap_or(false);
    let scope = get_field("SCOPE").unwrap_or_else(|| "A software project.".to_string());

    let components: Vec<String> = text
        .lines()
        .map(strip_markdown)
        .skip_while(|line| !line.starts_with("COMPONENTS:"))
        .skip(1)
        .take_while(|line| line.starts_with("- "))
        .map(|line| line[2..].trim().to_string())
        .collect();

    Some(ProjectBrief {
        name,
        language,
        database,
        frontend,
        scope,
        components,
    })
}

/// Parse Phase 4 verification output into a pass/fail result.
pub(super) fn parse_verification_result(text: &str) -> VerificationResult {
    if text.contains("VERIFICATION: PASS") {
        VerificationResult::Pass
    } else if let Some(reason_line) = text.lines().find(|l| l.starts_with("REASON:")) {
        VerificationResult::Fail(reason_line["REASON:".len()..].trim().to_string())
    } else if text.contains("VERIFICATION: FAIL") {
        VerificationResult::Fail("Verification failed (no reason provided)".to_string())
    } else {
        // No marker found — treat as failure to avoid silently passing a broken build.
        VerificationResult::Fail("No verification marker found in response".to_string())
    }
}

/// Parse Phase 6 reviewer output into a pass/fail result.
pub(super) fn parse_review_result(text: &str) -> ReviewResult {
    if text.contains("REVIEW: PASS") {
        ReviewResult::Pass
    } else if text.contains("REVIEW: FAIL") {
        // Collect all lines after the REVIEW: FAIL marker as findings.
        let findings: String = text
            .lines()
            .skip_while(|l| !l.contains("REVIEW: FAIL"))
            .skip(1) // skip the REVIEW: FAIL line itself
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if findings.is_empty() {
            ReviewResult::Fail("Review failed (no findings provided)".to_string())
        } else {
            ReviewResult::Fail(findings)
        }
    } else {
        // No marker — treat as failure to avoid silently passing a broken review.
        ReviewResult::Fail("No review marker found in response".to_string())
    }
}

/// Parse Phase 5 delivery output into a `BuildSummary`.
pub(super) fn parse_build_summary(text: &str) -> Option<BuildSummary> {
    if !text.contains("BUILD_COMPLETE") {
        return None;
    }

    let get_field = |key: &str| -> Option<String> {
        text.lines()
            .find(|line| line.starts_with(&format!("{key}:")))
            .map(|line| line[key.len() + 1..].trim().to_string())
    };

    Some(BuildSummary {
        project: get_field("PROJECT").unwrap_or_default(),
        location: get_field("LOCATION").unwrap_or_default(),
        language: get_field("LANGUAGE").unwrap_or_default(),
        summary: get_field("SUMMARY").unwrap_or_default(),
        usage: get_field("USAGE").unwrap_or_default(),
        skill: get_field("SKILL").filter(|s| !s.is_empty()),
    })
}

// i18n functions (phase_message, phase_message_by_name, qa_pass/retry/exhausted,
// review_pass/retry/exhausted) extracted to builds_i18n.rs for the 500-line limit.
// Re-exported here so callers using `use super::builds_parse::*` still compile.
pub(super) use super::builds_i18n::{
    phase_message_by_name, qa_exhausted_message, qa_pass_message, qa_retry_message,
    review_exhausted_message, review_pass_message, review_retry_message,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_project_brief_valid() {
        let text = "PROJECT_NAME: price-tracker\nLANGUAGE: Rust\nDATABASE: SQLite\nFRONTEND: no\nSCOPE: A CLI tool that tracks cryptocurrency prices.\nCOMPONENTS:\n- price fetcher\n- storage engine\n- alert system";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "price-tracker");
        assert_eq!(brief.language, "Rust");
        assert_eq!(brief.database, "SQLite");
        assert!(!brief.frontend);
        assert!(brief.scope.contains("cryptocurrency"));
        assert_eq!(brief.components.len(), 3);
    }

    #[test]
    fn test_parse_project_brief_minimal() {
        let text = "PROJECT_NAME: my-tool\nSCOPE: Does stuff";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "my-tool");
        assert_eq!(brief.language, "Rust"); // default
        assert_eq!(brief.database, "SQLite"); // default
        assert!(!brief.frontend); // default
    }

    #[test]
    fn test_parse_project_brief_missing_name() {
        let text = "LANGUAGE: Python\nSCOPE: A web scraper";
        assert!(parse_project_brief(text).is_none());
    }

    #[test]
    fn test_parse_project_brief_empty_name() {
        let text = "PROJECT_NAME: \nLANGUAGE: Rust";
        assert!(parse_project_brief(text).is_none());
    }

    #[test]
    fn test_parse_project_brief_path_traversal_rejected() {
        assert!(parse_project_brief("PROJECT_NAME: ../../../etc\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: foo/bar\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: .hidden\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: foo\\bar\nSCOPE: evil").is_none());
    }

    // BUG-M2: Strict name validation — reject spaces, shell metacharacters, overlength
    #[test]
    fn test_parse_project_brief_spaces_rejected() {
        assert!(
            parse_project_brief("PROJECT_NAME: my cool project\nSCOPE: test").is_none(),
            "Names with spaces must be rejected"
        );
    }

    #[test]
    fn test_parse_project_brief_shell_metacharacters_rejected() {
        assert!(parse_project_brief("PROJECT_NAME: test;rm -rf\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: test|cat /etc\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: $(whoami)\nSCOPE: evil").is_none());
        assert!(parse_project_brief("PROJECT_NAME: test&bg\nSCOPE: evil").is_none());
    }

    #[test]
    fn test_parse_project_brief_overlength_rejected() {
        let long_name = "a".repeat(65);
        let text = format!("PROJECT_NAME: {long_name}\nSCOPE: test");
        assert!(
            parse_project_brief(&text).is_none(),
            "Names over 64 chars must be rejected"
        );
    }

    #[test]
    fn test_parse_project_brief_max_length_accepted() {
        let name_64 = "a".repeat(64);
        let text = format!("PROJECT_NAME: {name_64}\nSCOPE: test");
        assert!(
            parse_project_brief(&text).is_some(),
            "Names at exactly 64 chars must be accepted"
        );
    }

    #[test]
    fn test_parse_project_brief_valid_kebab_and_snake() {
        // These must still work after the stricter validation.
        assert!(parse_project_brief("PROJECT_NAME: price-tracker\nSCOPE: test").is_some());
        assert!(parse_project_brief("PROJECT_NAME: my_tool_v2\nSCOPE: test").is_some());
        assert!(parse_project_brief("PROJECT_NAME: CamelCase\nSCOPE: test").is_some());
    }

    #[test]
    fn test_parse_project_brief_with_frontend() {
        let text = "PROJECT_NAME: dashboard\nFRONTEND: yes\nSCOPE: A web dashboard";
        let brief = parse_project_brief(text).unwrap();
        assert!(brief.frontend);
    }

    #[test]
    fn test_parse_project_brief_components_parsing() {
        let text =
            "PROJECT_NAME: my-app\nCOMPONENTS:\n- auth module\n- api layer\n- database\nSome other text";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(
            brief.components,
            vec!["auth module", "api layer", "database"]
        );
    }

    #[test]
    fn test_parse_verification_pass() {
        let text = "All tests passed.\n\nVERIFICATION: PASS";
        assert!(matches!(
            parse_verification_result(text),
            VerificationResult::Pass
        ));
    }

    #[test]
    fn test_parse_verification_fail_with_reason() {
        let text = "VERIFICATION: FAIL\nREASON: cargo test failed with 3 errors";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => assert!(reason.contains("3 errors")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_verification_fail_no_reason() {
        let text = "VERIFICATION: FAIL";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => assert!(reason.contains("no reason")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_verification_no_marker_implicit_fail() {
        let text = "Fixed all issues. Everything compiles now.";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => {
                assert!(reason.contains("No verification marker"))
            }
            _ => panic!("expected Fail when no marker present"),
        }
    }

    #[test]
    fn test_parse_build_summary_valid() {
        let text = "BUILD_COMPLETE\nPROJECT: price-tracker\nLOCATION: /home/user/.omega/workspace/builds/price-tracker\nLANGUAGE: Rust\nSUMMARY: A CLI tool for tracking crypto prices with alerts.\nUSAGE: price-tracker watch BTC\nSKILL: price-tracker";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.project, "price-tracker");
        assert!(summary.location.contains("price-tracker"));
        assert_eq!(summary.language, "Rust");
        assert!(summary.summary.contains("crypto"));
        assert_eq!(summary.usage, "price-tracker watch BTC");
        assert_eq!(summary.skill, Some("price-tracker".to_string()));
    }

    #[test]
    fn test_parse_build_summary_no_marker() {
        let text = "Here's what I built: a price tracker tool.";
        assert!(parse_build_summary(text).is_none());
    }

    #[test]
    fn test_parse_build_summary_no_skill() {
        let text = "BUILD_COMPLETE\nPROJECT: one-off\nLOCATION: /tmp/one-off\nLANGUAGE: Python\nSUMMARY: A quick script\nUSAGE: python main.py\nSKILL: ";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.skill, None); // empty string filtered out
    }

    // =======================================================================
    // REQ-BAP-010 (Must): Preserve existing parse functions — regression
    // =======================================================================
    //
    // These tests lock the CURRENT behavior of parse functions. They must
    // pass both before and after the build agent pipeline implementation.

    // Edge case: LLM wraps field names in markdown bold (**FIELD:**)
    #[test]
    fn test_parse_project_brief_markdown_bold_fields() {
        let text = "Here is the structured project brief:\n\n\
                     **PROJECT_NAME:** crm-tool\n\
                     **LANGUAGE:** Rust\n\
                     **DATABASE:** SQLite\n\
                     **FRONTEND:** none\n\
                     **SCOPE:** CLI-first CRM system\n\
                     **COMPONENTS:**\n\
                     - contacts module\n\
                     - deals pipeline\n\
                     - reporting engine";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "crm-tool");
        assert_eq!(brief.language, "Rust");
        assert_eq!(brief.database, "SQLite");
        assert!(!brief.frontend);
        assert!(brief.scope.contains("CRM"));
        assert_eq!(brief.components.len(), 3);
    }

    // Edge case: LLM wraps values in backticks
    #[test]
    fn test_parse_project_brief_backtick_name() {
        let text = "PROJECT_NAME: `my-tool`\nSCOPE: Does stuff";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "my-tool");
    }

    // Requirement: REQ-BAP-010 (Must)
    // Acceptance: parse_project_brief remains functional
    // Edge case: extra whitespace around field values
    #[test]
    fn test_regression_parse_project_brief_whitespace_in_values() {
        let text = "PROJECT_NAME:   my-tool  \nLANGUAGE:  Python \nSCOPE: Does stuff";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "my-tool", "Should trim whitespace from name");
        assert_eq!(
            brief.language, "Python",
            "Should trim whitespace from language"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: multiline text with BUILD_COMPLETE embedded in middle
    #[test]
    fn test_regression_parse_project_brief_no_false_positive() {
        // Brief text that also happens to contain BUILD_COMPLETE should still
        // parse as a brief, not as a build summary.
        let text = "PROJECT_NAME: my-tool\nSCOPE: Does stuff\nBUILD_COMPLETE";
        let brief = parse_project_brief(text);
        assert!(
            brief.is_some(),
            "Brief should still parse even with BUILD_COMPLETE present"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: VERIFICATION: PASS appears multiple times
    #[test]
    fn test_regression_parse_verification_multiple_pass_markers() {
        let text = "Phase 1: VERIFICATION: PASS\nPhase 2: VERIFICATION: PASS";
        assert!(matches!(
            parse_verification_result(text),
            VerificationResult::Pass
        ));
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: VERIFICATION: FAIL with REASON on non-adjacent line
    #[test]
    fn test_regression_parse_verification_reason_non_adjacent() {
        let text = "VERIFICATION: FAIL\nSome other output\nREASON: tests fail";
        match parse_verification_result(text) {
            VerificationResult::Fail(reason) => {
                assert!(
                    reason.contains("tests fail"),
                    "Should find REASON even if not adjacent to FAIL"
                );
            }
            _ => panic!("expected Fail"),
        }
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: empty text to all three parse functions
    #[test]
    fn test_regression_parse_functions_empty_input() {
        assert!(parse_project_brief("").is_none(), "Empty string -> None");
        match parse_verification_result("") {
            VerificationResult::Fail(reason) => {
                assert!(reason.contains("No verification marker"));
            }
            _ => panic!("Empty string should fail verification"),
        }
        assert!(
            parse_build_summary("").is_none(),
            "Empty string -> None for build summary"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: very large input (stress test)
    #[test]
    fn test_regression_parse_project_brief_large_input() {
        let mut text = String::from("PROJECT_NAME: huge-project\nSCOPE: Test\nCOMPONENTS:\n");
        for i in 0..1000 {
            text.push_str(&format!("- component {i}\n"));
        }
        let brief = parse_project_brief(&text).unwrap();
        assert_eq!(brief.name, "huge-project");
        assert_eq!(brief.components.len(), 1000);
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: unicode in project name
    #[test]
    fn test_regression_parse_project_brief_unicode_scope() {
        let text =
            "PROJECT_NAME: emoji-tracker\nSCOPE: Tracks emojis like \u{1f600} and \u{1f4a5}\nLANGUAGE: Rust";
        let brief = parse_project_brief(text).unwrap();
        assert_eq!(brief.name, "emoji-tracker");
        assert!(brief.scope.contains('\u{1f600}'));
    }

    // Requirement: REQ-BAP-010 (Must)
    // Edge case: BUILD_COMPLETE with missing fields
    #[test]
    fn test_regression_parse_build_summary_partial_fields() {
        let text = "BUILD_COMPLETE\nPROJECT: my-app";
        let summary = parse_build_summary(text).unwrap();
        assert_eq!(summary.project, "my-app");
        assert!(
            summary.location.is_empty(),
            "Missing field should default to empty"
        );
        assert!(summary.language.is_empty());
        assert!(summary.summary.is_empty());
        assert!(summary.usage.is_empty());
        assert_eq!(summary.skill, None);
    }

    // Requirement: REQ-BAP-010 (Must)
    // Security: script injection in project name
    #[test]
    fn test_regression_parse_project_brief_script_injection_with_slash() {
        // Names with / are rejected — </script> contains a slash.
        let text = "PROJECT_NAME: <script>alert(1)</script>\nSCOPE: evil";
        assert!(
            parse_project_brief(text).is_none(),
            "Name containing / (from </script>) should be rejected"
        );
    }

    // Requirement: REQ-BAP-010 (Must)
    // Security: special chars in project name (no path separators)
    #[test]
    fn test_regression_parse_project_brief_special_chars_no_slash() {
        // Names without / \ .. or leading . are accepted by the parser.
        let text = "PROJECT_NAME: my-app-v2.0\nSCOPE: test";
        let brief = parse_project_brief(text);
        assert!(
            brief.is_some(),
            "Name with dots (not leading) should be accepted"
        );
        assert_eq!(brief.unwrap().name, "my-app-v2.0");
    }
    // ===================================================================
    // Review result parsing
    // ===================================================================

    #[test]
    fn test_parse_review_result_pass() {
        let text = "All code looks good.\n\nREVIEW: PASS";
        assert!(matches!(parse_review_result(text), ReviewResult::Pass));
    }

    #[test]
    fn test_parse_review_result_fail_with_findings() {
        let text = "REVIEW: FAIL\n- security: SQL injection in query.rs\n- bug: off-by-one in pagination.rs";
        match parse_review_result(text) {
            ReviewResult::Fail(findings) => {
                assert!(findings.contains("SQL injection"));
                assert!(findings.contains("off-by-one"));
            }
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_review_result_fail_no_findings() {
        let text = "REVIEW: FAIL";
        match parse_review_result(text) {
            ReviewResult::Fail(reason) => assert!(reason.contains("no findings")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_review_result_no_marker() {
        let text = "The code looks fine but I didn't use the marker format.";
        match parse_review_result(text) {
            ReviewResult::Fail(reason) => assert!(reason.contains("No review marker")),
            _ => panic!("expected Fail"),
        }
    }

    #[test]
    fn test_parse_review_result_empty_input() {
        match parse_review_result("") {
            ReviewResult::Fail(reason) => assert!(reason.contains("No review marker")),
            _ => panic!("expected Fail"),
        }
    }

    // ===================================================================
    // ChainState construction
    // ===================================================================

    #[test]
    fn test_chain_state_construction() {
        let state = ChainState {
            project_name: "test-project".to_string(),
            project_dir: "/tmp/builds/test-project".to_string(),
            completed_phases: vec!["analyst".to_string(), "architect".to_string()],
            failed_phase: Some("qa".to_string()),
            failure_reason: Some("tests failing".to_string()),
            topology_name: None,
        };
        assert_eq!(state.project_name, "test-project");
        assert_eq!(state.completed_phases.len(), 2);
        assert_eq!(state.failed_phase.as_deref(), Some("qa"));
    }

    // i18n tests (phase_message, qa/review pass/retry/exhausted, phase_message_by_name)
    // moved to builds_i18n.rs alongside the functions they test.
}
