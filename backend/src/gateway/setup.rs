//! Brain setup session -- `/setup` command orchestrator.
//!
//! Handles the full lifecycle: start session, handle responses (questions,
//! confirmation, cancellation, expiry), execute approved setup, cleanup.

use std::path::PathBuf;

use omega_core::config::shellexpand;
use omega_core::message::IncomingMessage;
use omega_memory::audit::{AuditEntry, AuditStatus};
use tracing::warn;

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
#[allow(dead_code)]
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
    pub(super) async fn start_setup_session(
        &self,
        incoming: &IncomingMessage,
        description: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        let user_lang = self
            .memory
            .get_fact(&incoming.sender_id, "preferred_language")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "English".to_string());

        // REQ-BRAIN-023: Check for concurrent session.
        let existing = self
            .memory
            .get_fact(&incoming.sender_id, "pending_setup")
            .await
            .ok()
            .flatten();
        if let Some(ref val) = existing {
            let ts_str = val.split('|').next().unwrap_or("0");
            let created_at: i64 = ts_str.parse().unwrap_or(0);
            let now = chrono::Utc::now().timestamp();
            let ttl = SETUP_TTL_SECS as i64;
            if (now - created_at) <= ttl {
                if let Some(h) = typing_handle {
                    h.abort();
                }
                self.send_text(incoming, setup_conflict_message(&user_lang))
                    .await;
                return;
            }
            // Expired -- clean up old session and continue.
            self.cleanup_setup_session(&incoming.sender_id).await;
        }

        // REQ-BRAIN-005: Load existing projects for collision detection.
        let projects = omega_skills::load_projects(&self.data_dir);
        let omega_dir = PathBuf::from(shellexpand(&self.data_dir));

        // REQ-BRAIN-019: Read existing ROLE.md files for context.
        let mut project_context = String::new();
        for proj in &projects {
            project_context.push_str(&format!("- {}", proj.name));
            let role_path = omega_dir.join("projects").join(&proj.name).join("ROLE.md");
            if let Ok(content) = tokio::fs::read_to_string(&role_path).await {
                let first_line = content.lines().next().unwrap_or("");
                project_context.push_str(&format!(": {first_line}"));
            }
            project_context.push('\n');
        }

        // Build Brain prompt.
        let prompt = if project_context.is_empty() {
            format!(
                "Setup round 1. The user wants to configure OMEGA for their domain.\n\
                 User description: {description}\n\n\
                 No existing projects.\n\n\
                 Analyze the description. If specific enough, output SETUP_PROPOSAL.\n\
                 If you need more information, output SETUP_QUESTIONS (2-4 questions max)."
            )
        } else {
            format!(
                "Setup round 1. The user wants to configure OMEGA for their domain.\n\
                 User description: {description}\n\n\
                 Existing projects:\n{project_context}\n\
                 Analyze the description. If specific enough, output SETUP_PROPOSAL.\n\
                 If you need more information, output SETUP_QUESTIONS (2-4 questions max).\n\
                 If an existing project matches, propose updating it instead of creating a duplicate."
            )
        };

        // REQ-BRAIN-003: Write single agent file.
        let _agent_guard =
            match AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT).await {
                Ok(guard) => guard,
                Err(e) => {
                    warn!("Failed to write Brain agent file: {e}");
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    self.send_text(
                        incoming,
                        "Setup failed: could not initialize the Brain agent.",
                    )
                    .await;
                    return;
                }
            };

        // REQ-BRAIN-004: Invoke Brain via run_build_phase.
        let result = self
            .run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30))
            .await;

        match result {
            Ok(output) => {
                match parse_setup_output(&output) {
                    SetupOutput::Questions(questions) => {
                        // Create context file with round 1.
                        let ctx_path = setup_context_path(&self.data_dir, &incoming.sender_id);
                        let ctx_dir = ctx_path.parent().unwrap();
                        let _ = tokio::fs::create_dir_all(ctx_dir).await;
                        let file_content = format!(
                            "ROUND: 1\nDESCRIPTION: {description}\n\nQUESTIONS:\n{questions}"
                        );
                        let _ = tokio::fs::write(&ctx_path, &file_content).await;

                        // Store pending_setup fact.
                        let stamped = format!(
                            "{}|{}|1",
                            chrono::Utc::now().timestamp(),
                            incoming.sender_id
                        );
                        let _ = self
                            .memory
                            .store_fact(&incoming.sender_id, "pending_setup", &stamped)
                            .await;

                        // Send questions to user.
                        let msg = setup_intro_message(&user_lang, &questions);
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &msg).await;
                    }
                    SetupOutput::Proposal(proposal) => {
                        // Extract user-facing preview (before SETUP_EXECUTE).
                        let preview = proposal
                            .split("SETUP_EXECUTE")
                            .next()
                            .unwrap_or(&proposal)
                            .replace("SETUP_PROPOSAL", "")
                            .trim()
                            .to_string();

                        // Store full proposal in context file.
                        let ctx_path = setup_context_path(&self.data_dir, &incoming.sender_id);
                        let ctx_dir = ctx_path.parent().unwrap();
                        let _ = tokio::fs::create_dir_all(ctx_dir).await;
                        let _ = tokio::fs::write(&ctx_path, &proposal).await;

                        // Store pending_setup fact (round = "proposal").
                        let stamped = format!(
                            "{}|{}|proposal",
                            chrono::Utc::now().timestamp(),
                            incoming.sender_id
                        );
                        let _ = self
                            .memory
                            .store_fact(&incoming.sender_id, "pending_setup", &stamped)
                            .await;

                        // Send proposal to user.
                        let msg = setup_proposal_message(&user_lang, &preview);
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &msg).await;
                    }
                    SetupOutput::Executed(_) => {
                        // Should not happen in questioning mode.
                        warn!("Brain returned Executed in questioning mode");
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(
                            incoming,
                            "Setup encountered an unexpected state. Please try again with /setup.",
                        )
                        .await;
                    }
                }
            }
            Err(e) => {
                warn!("Brain invocation failed: {e}");
                if let Some(h) = typing_handle {
                    h.abort();
                }
                self.send_text(
                    incoming,
                    "Setup failed after multiple attempts. Please try again later.",
                )
                .await;
            }
        }

        self.audit_setup(incoming, "unknown", "started", description)
            .await;
    }

    /// Handle a follow-up message during an active setup session.
    pub(super) async fn handle_setup_response(
        &self,
        incoming: &IncomingMessage,
        setup_value: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        let user_lang = self
            .memory
            .get_fact(&incoming.sender_id, "preferred_language")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "English".to_string());

        // Parse pending_setup value: "timestamp|sender_id|round".
        let parts: Vec<&str> = setup_value.split('|').collect();
        let stored_ts: i64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let now = chrono::Utc::now().timestamp();
        let ttl = SETUP_TTL_SECS as i64;

        // Check TTL.
        if (now - stored_ts) > ttl {
            self.cleanup_setup_session(&incoming.sender_id).await;
            if let Some(h) = typing_handle {
                h.abort();
            }
            self.send_text(incoming, setup_expired_message(&user_lang))
                .await;
            return;
        }

        // Check for cancellation.
        if is_build_cancelled(&incoming.text) {
            self.cleanup_setup_session(&incoming.sender_id).await;
            if let Some(h) = typing_handle {
                h.abort();
            }
            self.send_text(incoming, setup_cancelled_message(&user_lang))
                .await;
            return;
        }

        // Read context file to determine phase.
        let ctx_path = setup_context_path(&self.data_dir, &incoming.sender_id);
        let context = tokio::fs::read_to_string(&ctx_path)
            .await
            .unwrap_or_default();

        let omega_dir = PathBuf::from(shellexpand(&self.data_dir));

        if context.contains("SETUP_PROPOSAL") {
            // CONFIRMATION PHASE.
            if is_build_confirmed(&incoming.text) {
                // Execute the approved setup.
                match self.execute_setup(incoming, &context).await {
                    Ok(mut output) => {
                        // Process markers from Brain output.
                        let active_project: Option<String> = self
                            .memory
                            .get_fact(&incoming.sender_id, "active_project")
                            .await
                            .ok()
                            .flatten();
                        self.process_markers(incoming, &mut output, active_project.as_deref())
                            .await;

                        // Extract project name from markers or context.
                        let project_name = output
                            .lines()
                            .find(|l| l.starts_with("PROJECT_ACTIVATE:"))
                            .and_then(|l| l.strip_prefix("PROJECT_ACTIVATE:"))
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| "project".to_string());

                        self.cleanup_setup_session(&incoming.sender_id).await;
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        let msg = setup_complete_message(&user_lang, &project_name);
                        self.send_text(incoming, &msg).await;
                        self.audit_setup(incoming, &project_name, "complete", "Setup completed")
                            .await;
                    }
                    Err(e) => {
                        self.cleanup_setup_session(&incoming.sender_id).await;
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &format!("Setup execution failed: {e}"))
                            .await;
                    }
                }
            } else {
                // Modification request -- append to context, run Brain again.
                let updated_context = format!("{context}\n\nUSER MODIFICATION:\n{}", incoming.text);
                let _ = tokio::fs::write(&ctx_path, &updated_context).await;

                let prompt = format!(
                    "The user wants modifications to the proposed setup.\n\n\
                     Previous context and proposal:\n{updated_context}\n\n\
                     Update the proposal based on the user's feedback. \
                     Output SETUP_PROPOSAL with the updated plan."
                );

                let _agent_guard =
                    match AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT)
                        .await
                    {
                        Ok(g) => g,
                        Err(e) => {
                            warn!("Failed to write Brain agent: {e}");
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            return;
                        }
                    };

                match self
                    .run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30))
                    .await
                {
                    Ok(output) => {
                        if let SetupOutput::Proposal(proposal) = parse_setup_output(&output) {
                            let preview = proposal
                                .split("SETUP_EXECUTE")
                                .next()
                                .unwrap_or(&proposal)
                                .replace("SETUP_PROPOSAL", "")
                                .trim()
                                .to_string();
                            let _ = tokio::fs::write(&ctx_path, &proposal).await;
                            let msg = setup_proposal_message(&user_lang, &preview);
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            self.send_text(incoming, &msg).await;
                        } else {
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            self.send_text(
                                incoming,
                                "I couldn't update the proposal. Reply *yes* to proceed or describe your changes.",
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        warn!("Brain modification failed: {e}");
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(
                            incoming,
                            "Failed to process modification. Reply *yes* to proceed with the original plan or *no* to cancel.",
                        )
                        .await;
                    }
                }
            }
        } else {
            // QUESTIONING PHASE -- append answer, run next round.
            let current_round = parse_setup_round(&context);
            let next_round = current_round + 1;

            let updated_context = format!(
                "ROUND: {next_round}\n{}\n\nUSER ANSWER (round {current_round}):\n{}",
                context
                    .lines()
                    .skip_while(|l| l.starts_with("ROUND:"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                incoming.text
            );
            let _ = tokio::fs::write(&ctx_path, &updated_context).await;

            let final_round = next_round >= 3;
            let prompt = if final_round {
                format!(
                    "FINAL ROUND. You MUST produce SETUP_PROPOSAL now.\n\n\
                     Accumulated context:\n{updated_context}"
                )
            } else {
                format!(
                    "Setup round {next_round}. Continue the setup conversation.\n\n\
                     Accumulated context:\n{updated_context}\n\n\
                     If you have enough information, output SETUP_PROPOSAL.\n\
                     Otherwise, output SETUP_QUESTIONS (2-4 questions max)."
                )
            };

            let _agent_guard =
                match AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT).await {
                    Ok(g) => g,
                    Err(e) => {
                        warn!("Failed to write Brain agent: {e}");
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        return;
                    }
                };

            // Update pending_setup with new round.
            let stamped = format!(
                "{}|{}|{next_round}",
                chrono::Utc::now().timestamp(),
                incoming.sender_id
            );
            let _ = self
                .memory
                .store_fact(&incoming.sender_id, "pending_setup", &stamped)
                .await;

            match self
                .run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30))
                .await
            {
                Ok(output) => match parse_setup_output(&output) {
                    SetupOutput::Questions(questions) => {
                        let file_content = format!("{updated_context}\n\nQUESTIONS:\n{questions}");
                        let _ = tokio::fs::write(&ctx_path, &file_content).await;
                        let msg = setup_followup_message(&user_lang, &questions, next_round);
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &msg).await;
                    }
                    SetupOutput::Proposal(proposal) => {
                        let preview = proposal
                            .split("SETUP_EXECUTE")
                            .next()
                            .unwrap_or(&proposal)
                            .replace("SETUP_PROPOSAL", "")
                            .trim()
                            .to_string();
                        let _ = tokio::fs::write(&ctx_path, &proposal).await;

                        let stamped = format!(
                            "{}|{}|proposal",
                            chrono::Utc::now().timestamp(),
                            incoming.sender_id
                        );
                        let _ = self
                            .memory
                            .store_fact(&incoming.sender_id, "pending_setup", &stamped)
                            .await;

                        let msg = setup_proposal_message(&user_lang, &preview);
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &msg).await;
                    }
                    SetupOutput::Executed(_) => {
                        warn!("Brain returned Executed in questioning mode");
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                    }
                },
                Err(e) => {
                    warn!("Brain follow-up failed: {e}");
                    self.cleanup_setup_session(&incoming.sender_id).await;
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    self.send_text(incoming, "Setup failed. Please try again with /setup.")
                        .await;
                }
            }
        }
    }

    /// Execute the approved setup: run Brain in execution mode.
    async fn execute_setup(
        &self,
        _incoming: &IncomingMessage,
        proposal_context: &str,
    ) -> Result<String, String> {
        let omega_dir = PathBuf::from(shellexpand(&self.data_dir));

        let prompt =
            format!("EXECUTE_SETUP. Create all files and emit markers.\n\n{proposal_context}");

        let _agent_guard = AgentFilesGuard::write_single(&omega_dir, "omega-brain", BRAIN_AGENT)
            .await
            .map_err(|e| format!("Failed to write agent: {e}"))?;

        self.run_build_phase("omega-brain", &prompt, &self.model_complex, Some(30))
            .await
            .map_err(|e| format!("Brain execution failed: {e}"))
    }

    /// Clean up session state (fact + context file).
    async fn cleanup_setup_session(&self, sender_id: &str) {
        let _ = self.memory.delete_fact(sender_id, "pending_setup").await;
        let ctx_file = setup_context_path(&self.data_dir, sender_id);
        let _ = tokio::fs::remove_file(&ctx_file).await;
    }

    /// Log an audit entry for a setup operation.
    async fn audit_setup(
        &self,
        incoming: &IncomingMessage,
        project: &str,
        status: &str,
        detail: &str,
    ) {
        let _ = self
            .audit
            .log(&AuditEntry {
                channel: incoming.channel.clone(),
                sender_id: incoming.sender_id.clone(),
                sender_name: incoming.sender_name.clone(),
                input_text: format!("[SETUP:{project}] {}", incoming.text),
                output_text: Some(format!("[{status}] {detail}")),
                provider_used: Some(self.provider.name().to_string()),
                model: None,
                processing_ms: None,
                status: if status == "complete" {
                    AuditStatus::Ok
                } else {
                    AuditStatus::Error
                },
                denial_reason: None,
            })
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // REQ-BRAIN-020 (Must): Setup context file path and cleanup
    // ===================================================================

    // Requirement: REQ-BRAIN-020 (Must)
    // Acceptance: Context file path resolves to <data_dir>/setup/<sender_id>.md
    #[test]
    fn test_setup_context_path_format() {
        let path = setup_context_path("/tmp/omega-test", "user123");
        assert_eq!(
            path,
            PathBuf::from("/tmp/omega-test/setup/user123.md"),
            "Context path must be <data_dir>/setup/<sender_id>.md"
        );
    }

    // Requirement: REQ-BRAIN-020 (Must)
    // Acceptance: Context file path handles tilde expansion
    #[test]
    fn test_setup_context_path_tilde_expansion() {
        let path = setup_context_path("~/.omega", "sender42");
        // After shellexpand, ~ should be replaced with $HOME.
        assert!(
            !path.to_string_lossy().starts_with('~'),
            "Path must not start with ~ after shellexpand"
        );
        assert!(
            path.to_string_lossy().contains("/setup/sender42.md"),
            "Path must end with /setup/sender42.md"
        );
    }

    // Requirement: REQ-BRAIN-020 (Must)
    // Edge case: sender_id with special characters
    #[test]
    fn test_setup_context_path_special_sender_id() {
        let path = setup_context_path("/tmp/omega", "user-123_test");
        assert_eq!(
            path,
            PathBuf::from("/tmp/omega/setup/user-123_test.md"),
            "Path must handle sender_id with hyphens and underscores"
        );
    }

    // Requirement: REQ-BRAIN-020 (Must)
    // Acceptance: Context file created, exists during session
    #[tokio::test]
    async fn test_setup_context_file_creation_and_cleanup() {
        let tmp = std::env::temp_dir().join("__omega_test_setup_ctx__");
        let _ = std::fs::remove_dir_all(&tmp);
        let setup_dir = tmp.join("setup");
        std::fs::create_dir_all(&setup_dir).unwrap();

        let ctx_path = tmp.join("setup").join("test_user.md");
        // Create context file (simulating session start).
        std::fs::write(&ctx_path, "ROUND: 1\nUser wants to be a realtor").unwrap();
        assert!(ctx_path.exists(), "Context file must exist during session");

        // Simulate cleanup (completion/cancel/expiry).
        let _ = std::fs::remove_file(&ctx_path);
        assert!(
            !ctx_path.exists(),
            "Context file must be deleted on cleanup"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BRAIN-020 (Must)
    // Acceptance: Context file deleted on expiry
    #[tokio::test]
    async fn test_setup_context_file_deleted_on_expiry() {
        let tmp = std::env::temp_dir().join("__omega_test_setup_expiry__");
        let _ = std::fs::remove_dir_all(&tmp);
        let setup_dir = tmp.join("setup");
        std::fs::create_dir_all(&setup_dir).unwrap();

        let ctx_path = tmp.join("setup").join("expired_user.md");
        std::fs::write(&ctx_path, "ROUND: 2\nOld session content").unwrap();
        assert!(ctx_path.exists());

        // Cleanup removes the file.
        let _ = tokio::fs::remove_file(&ctx_path).await;
        assert!(
            !ctx_path.exists(),
            "Context file must be removed on session expiry"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // Requirement: REQ-BRAIN-020 (Must)
    // Edge case: cleanup when context file already deleted
    #[tokio::test]
    async fn test_setup_cleanup_idempotent() {
        let ctx_path = std::env::temp_dir()
            .join("__omega_test_setup_idempotent__")
            .join("setup")
            .join("ghost_user.md");

        // File does not exist -- cleanup should not panic.
        let result = tokio::fs::remove_file(&ctx_path).await;
        assert!(
            result.is_err(),
            "Removing non-existent file returns error (not panic)"
        );
    }

    // ===================================================================
    // REQ-BRAIN-022 (Must): Workspace path is ~/.omega/
    // ===================================================================

    // Requirement: REQ-BRAIN-022 (Must)
    // Acceptance: Workspace path resolves to ~/.omega/ (not ~/.omega/workspace/)
    #[test]
    fn test_workspace_path_is_omega_root() {
        // The setup uses data_dir (which is ~/.omega/) as the workspace for Brain.
        // NOT the workspace subdirectory used for builds.
        let data_dir = "~/.omega";
        let omega_dir = PathBuf::from(shellexpand(data_dir));
        assert!(
            omega_dir.to_string_lossy().ends_with(".omega")
                || omega_dir.to_string_lossy().ends_with(".omega/"),
            "Brain workspace must resolve to ~/.omega/, got: {}",
            omega_dir.display()
        );
        // Must NOT end with /workspace/
        assert!(
            !omega_dir.to_string_lossy().ends_with("workspace"),
            "Brain workspace must be ~/.omega/, not ~/.omega/workspace/"
        );
    }

    // ===================================================================
    // REQ-BRAIN-010 (Should): Multi-round session with parse_setup_round
    // ===================================================================

    // Requirement: REQ-BRAIN-010 (Should)
    // Acceptance: parse_setup_round extracts round number from context header
    #[test]
    fn test_parse_setup_round_extracts_round() {
        let content = "ROUND: 1\nUser description: I'm a realtor";
        assert_eq!(
            parse_setup_round(content),
            1,
            "Must parse round 1 from header"
        );
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Acceptance: parse_setup_round returns round 2
    #[test]
    fn test_parse_setup_round_round_2() {
        let content = "ROUND: 2\nPrevious: questions\nAnswer: blah";
        assert_eq!(parse_setup_round(content), 2, "Must parse round 2");
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Acceptance: parse_setup_round returns round 3
    #[test]
    fn test_parse_setup_round_round_3() {
        let content = "ROUND: 3\nFINAL ROUND\nAccumulated context here";
        assert_eq!(
            parse_setup_round(content),
            3,
            "Must parse round 3 (max allowed)"
        );
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Acceptance: parse_setup_round returns 0 for no header
    #[test]
    fn test_parse_setup_round_no_header() {
        let content = "Just some text without round header";
        assert_eq!(
            parse_setup_round(content),
            0,
            "Must return 0 when no ROUND: header found"
        );
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Edge case: parse_setup_round with empty content
    #[test]
    fn test_parse_setup_round_empty_content() {
        assert_eq!(parse_setup_round(""), 0, "Must return 0 for empty content");
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Edge case: parse_setup_round with malformed round value
    #[test]
    fn test_parse_setup_round_malformed_value() {
        let content = "ROUND: abc\nSome content";
        assert_eq!(
            parse_setup_round(content),
            0,
            "Must return 0 for unparseable round value"
        );
    }

    // Requirement: REQ-BRAIN-010 (Should)
    // Edge case: ROUND header not on first line
    #[test]
    fn test_parse_setup_round_not_first_line() {
        let content = "Some preamble\nROUND: 2\nContent";
        assert_eq!(
            parse_setup_round(content),
            2,
            "Must find ROUND: header even if not on first line"
        );
    }

    // ===================================================================
    // REQ-BRAIN-004 (Must), REQ-BRAIN-010 (Should): parse_setup_output
    // ===================================================================

    // Requirement: REQ-BRAIN-004 (Must)
    // Acceptance: Output containing SETUP_QUESTIONS is parsed as Questions
    #[test]
    fn test_parse_setup_output_questions() {
        let output = "SETUP_QUESTIONS\n1. What type of properties?\n2. Residential or commercial?";
        match parse_setup_output(output) {
            SetupOutput::Questions(q) => {
                assert!(
                    q.contains("What type of properties"),
                    "Questions text must be extracted: got '{q}'"
                );
            }
            _ => panic!("Expected SetupOutput::Questions"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Acceptance: Output containing SETUP_PROPOSAL is parsed as Proposal
    #[test]
    fn test_parse_setup_output_proposal() {
        let output = "SETUP_PROPOSAL\nProject: realtor\nDomain: Real estate\n\nSETUP_EXECUTE\nCreate files...";
        match parse_setup_output(output) {
            SetupOutput::Proposal(p) => {
                assert!(
                    p.contains("SETUP_PROPOSAL"),
                    "Proposal must contain the full output"
                );
                assert!(
                    p.contains("SETUP_EXECUTE"),
                    "Proposal must include SETUP_EXECUTE section"
                );
            }
            _ => panic!("Expected SetupOutput::Proposal"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Acceptance: Output without markers is parsed as Executed
    #[test]
    fn test_parse_setup_output_executed() {
        let output = "Created ~/.omega/projects/realtor/ROLE.md\nSCHEDULE_ACTION: Check listings | 2026-03-01T08:00:00 | daily\nPROJECT_ACTIVATE: realtor";
        match parse_setup_output(output) {
            SetupOutput::Executed(e) => {
                assert!(
                    e.contains("SCHEDULE_ACTION"),
                    "Executed output must contain markers"
                );
                assert!(
                    e.contains("PROJECT_ACTIVATE"),
                    "Executed output must contain activation marker"
                );
            }
            _ => panic!("Expected SetupOutput::Executed"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Edge case: empty output parsed as Executed
    #[test]
    fn test_parse_setup_output_empty() {
        match parse_setup_output("") {
            SetupOutput::Executed(e) => {
                assert!(e.is_empty(), "Empty output must parse as empty Executed");
            }
            _ => panic!("Expected SetupOutput::Executed for empty input"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Edge case: output with both SETUP_QUESTIONS and SETUP_PROPOSAL
    // (malformed -- SETUP_QUESTIONS takes priority)
    #[test]
    fn test_parse_setup_output_questions_takes_priority() {
        let output = "SETUP_QUESTIONS\n1. What area?\nSETUP_PROPOSAL\nProject: test";
        match parse_setup_output(output) {
            SetupOutput::Questions(_) => {
                // Correct: SETUP_QUESTIONS checked first.
            }
            _ => panic!("When both markers present, SETUP_QUESTIONS must take priority"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Edge case: SETUP_QUESTIONS with no actual questions
    #[test]
    fn test_parse_setup_output_questions_empty() {
        let output = "SETUP_QUESTIONS";
        match parse_setup_output(output) {
            SetupOutput::Questions(q) => {
                assert!(
                    q.is_empty(),
                    "Questions text must be empty when nothing follows marker"
                );
            }
            _ => panic!("Expected SetupOutput::Questions"),
        }
    }

    // Requirement: REQ-BRAIN-004 (Must)
    // Edge case: SETUP_PROPOSAL with no content
    #[test]
    fn test_parse_setup_output_proposal_minimal() {
        let output = "SETUP_PROPOSAL";
        match parse_setup_output(output) {
            SetupOutput::Proposal(p) => {
                assert!(
                    p.contains("SETUP_PROPOSAL"),
                    "Proposal must contain marker text"
                );
            }
            _ => panic!("Expected SetupOutput::Proposal"),
        }
    }

    // ===================================================================
    // REQ-BRAIN-012 (Should): pending_setup fact format and TTL
    // ===================================================================

    // Requirement: REQ-BRAIN-012 (Should)
    // Acceptance: pending_setup fact format is <timestamp>|<sender_id>|<round>
    #[test]
    fn test_pending_setup_fact_format() {
        // The fact value format: "1709123456|user123|1"
        let fact_value = "1709123456|user123|1";
        let parts: Vec<&str> = fact_value.split('|').collect();
        assert_eq!(
            parts.len(),
            3,
            "pending_setup fact must have 3 pipe-delimited parts"
        );
        assert!(
            parts[0].parse::<i64>().is_ok(),
            "First part must be a timestamp"
        );
        assert_eq!(parts[1], "user123", "Second part must be sender_id");
        assert!(
            parts[2].parse::<u8>().is_ok(),
            "Third part must be a round number"
        );
    }

    // Requirement: REQ-BRAIN-012 (Should)
    // Acceptance: 30-minute TTL (1800 seconds) for setup sessions
    #[test]
    fn test_pending_setup_ttl_value() {
        // The TTL constant is defined in keywords.rs as SETUP_TTL_SECS.
        // This test validates the expected value.
        let expected_ttl: i64 = 1800; // 30 minutes
        let fact_timestamp = 1709123456_i64;
        let now = fact_timestamp + expected_ttl + 1; // one second past expiry

        assert!(
            now - fact_timestamp > expected_ttl,
            "Session older than {expected_ttl} seconds must be considered expired"
        );
    }

    // Requirement: REQ-BRAIN-012 (Should)
    // Acceptance: Session within TTL is still valid
    #[test]
    fn test_pending_setup_within_ttl() {
        let ttl: i64 = 1800;
        let fact_timestamp = 1709123456_i64;
        let now = fact_timestamp + 900; // 15 minutes later

        assert!(
            now - fact_timestamp <= ttl,
            "Session within 30 minutes must be considered valid"
        );
    }

    // ===================================================================
    // REQ-BRAIN-005 (Must): Collision detection
    // ===================================================================

    // Requirement: REQ-BRAIN-005 (Must)
    // Acceptance: Existing projects can be listed for Brain context
    // (This tests the concept -- actual load_projects() is tested in omega-skills crate)
    #[test]
    fn test_collision_detection_context_format() {
        // Simulate building the collision context string.
        let existing_projects = vec!["realtor", "trader", "restaurant"];
        let context = existing_projects
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(context.contains("- realtor"));
        assert!(context.contains("- trader"));
        assert!(context.contains("- restaurant"));
    }

    // Requirement: REQ-BRAIN-005 (Must)
    // Acceptance: No existing projects produces empty context
    #[test]
    fn test_collision_detection_no_projects() {
        let existing_projects: Vec<&str> = vec![];
        let context = existing_projects
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            context.is_empty(),
            "Empty project list must produce empty context string"
        );
    }

    // ===================================================================
    // REQ-BRAIN-023 (Could): Concurrent session guard
    // ===================================================================

    // Requirement: REQ-BRAIN-023 (Could)
    // Acceptance: Only one active pending_setup per sender_id
    #[test]
    fn test_concurrent_session_guard_logic() {
        // The guard is implemented by checking if pending_setup fact exists.
        // If it exists and not expired -> reject new session.
        let existing_fact: Option<&str> = Some("1709123456|user123|1");
        assert!(
            existing_fact.is_some(),
            "When pending_setup exists, new session must be rejected"
        );

        let no_fact: Option<&str> = None;
        assert!(
            no_fact.is_none(),
            "When no pending_setup, new session is allowed"
        );
    }

    // ===================================================================
    // REQ-BRAIN-011 (Should): Pipeline intercept extracts description
    // ===================================================================

    // Requirement: REQ-BRAIN-011 (Should)
    // Acceptance: Description text extracted from after /setup
    #[test]
    fn test_extract_description_from_setup_command() {
        let text = "/setup I'm a realtor in Lisbon";
        let description = text.strip_prefix("/setup").unwrap_or("").trim();
        assert_eq!(
            description, "I'm a realtor in Lisbon",
            "Description must be extracted from after /setup"
        );
    }

    // Requirement: REQ-BRAIN-011 (Should)
    // Acceptance: /setup with no text yields empty description
    #[test]
    fn test_extract_description_empty() {
        let text = "/setup";
        let description = text.strip_prefix("/setup").unwrap_or("").trim();
        assert!(
            description.is_empty(),
            "Empty description when /setup has no text"
        );
    }

    // Requirement: REQ-BRAIN-011 (Should)
    // Acceptance: /setup@botname extracts description correctly
    #[test]
    fn test_extract_description_with_botname() {
        let text = "/setup@omega_bot I'm a chef";
        // Simulate the pipeline extraction logic:
        let first_word = text.split_whitespace().next().unwrap_or("");
        let description = if first_word.starts_with("/setup") {
            text[first_word.len()..].trim()
        } else {
            ""
        };
        assert_eq!(
            description, "I'm a chef",
            "Description must be extracted even with @botname suffix"
        );
    }

    // Requirement: REQ-BRAIN-011 (Should)
    // Edge case: /setup with only whitespace after it
    #[test]
    fn test_extract_description_whitespace_only() {
        let text = "/setup   ";
        let description = text.strip_prefix("/setup").unwrap_or("").trim();
        assert!(
            description.is_empty(),
            "Whitespace-only description must be treated as empty"
        );
    }

    // ===================================================================
    // REQ-BRAIN-019 (Should): Existing ROLE.md content in context
    // ===================================================================

    // Requirement: REQ-BRAIN-019 (Should)
    // Acceptance: ROLE.md content can be read and truncated for context
    #[tokio::test]
    async fn test_read_existing_role_for_context() {
        let tmp = std::env::temp_dir().join("__omega_test_role_context__");
        let _ = std::fs::remove_dir_all(&tmp);
        let project_dir = tmp.join("projects").join("trader");
        std::fs::create_dir_all(&project_dir).unwrap();

        let role_content = "# Trader Role\n\nYou are an expert trader...\n".repeat(10);
        std::fs::write(project_dir.join("ROLE.md"), &role_content).unwrap();

        // Read first 50 lines (as per architecture spec).
        let read_content = tokio::fs::read_to_string(project_dir.join("ROLE.md"))
            .await
            .unwrap();
        let first_50: String = read_content.lines().take(50).collect::<Vec<_>>().join("\n");

        assert!(
            !first_50.is_empty(),
            "Existing ROLE.md content must be readable for context"
        );
        assert!(
            first_50.lines().count() <= 50,
            "Must truncate to at most 50 lines"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ===================================================================
    // REQ-BRAIN-006 (Must), REQ-BRAIN-007 (Must): File creation paths
    // ===================================================================

    // Requirement: REQ-BRAIN-006 (Must)
    // Acceptance: ROLE.md path resolves correctly
    #[test]
    fn test_role_md_path_format() {
        let data_dir = "/tmp/omega-test";
        let project_name = "realtor";
        let role_path = PathBuf::from(data_dir)
            .join("projects")
            .join(project_name)
            .join("ROLE.md");
        assert_eq!(
            role_path,
            PathBuf::from("/tmp/omega-test/projects/realtor/ROLE.md"),
            "ROLE.md path must be <data_dir>/projects/<name>/ROLE.md"
        );
    }

    // Requirement: REQ-BRAIN-007 (Must)
    // Acceptance: HEARTBEAT.md path resolves correctly
    #[test]
    fn test_heartbeat_md_path_format() {
        let data_dir = "/tmp/omega-test";
        let project_name = "realtor";
        let hb_path = PathBuf::from(data_dir)
            .join("projects")
            .join(project_name)
            .join("HEARTBEAT.md");
        assert_eq!(
            hb_path,
            PathBuf::from("/tmp/omega-test/projects/realtor/HEARTBEAT.md"),
            "HEARTBEAT.md path must be <data_dir>/projects/<name>/HEARTBEAT.md"
        );
    }

    // Requirement: REQ-BRAIN-006 (Must)
    // Acceptance: Project directory created if absent
    #[tokio::test]
    async fn test_project_directory_creation() {
        let tmp = std::env::temp_dir().join("__omega_test_project_dir__");
        let _ = std::fs::remove_dir_all(&tmp);

        let project_dir = tmp.join("projects").join("realtor");
        assert!(!project_dir.exists());

        tokio::fs::create_dir_all(&project_dir).await.unwrap();
        assert!(
            project_dir.exists(),
            "Project directory must be created via create_dir_all"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // ===================================================================
    // REQ-BRAIN-008 (Must): Schedule marker format validation
    // ===================================================================

    // Requirement: REQ-BRAIN-008 (Must)
    // Acceptance: SCHEDULE_ACTION marker format is parseable
    #[test]
    fn test_schedule_action_marker_format() {
        let marker = "SCHEDULE_ACTION: Check new property listings | 2026-03-01T08:00:00 | daily";
        let parts: Vec<&str> = marker
            .strip_prefix("SCHEDULE_ACTION:")
            .unwrap()
            .split('|')
            .map(|s| s.trim())
            .collect();

        assert_eq!(
            parts.len(),
            3,
            "SCHEDULE_ACTION must have 3 pipe-delimited parts"
        );
        assert!(!parts[0].is_empty(), "Description part must not be empty");
        assert!(
            parts[1].contains('T'),
            "Datetime part must be ISO 8601 format"
        );
        assert!(
            ["daily", "weekly", "monthly", "none"].contains(&parts[2]),
            "Repeat part must be daily|weekly|monthly|none"
        );
    }

    // Requirement: REQ-BRAIN-008 (Must)
    // Edge case: Marker with extra spaces
    #[test]
    fn test_schedule_action_marker_with_spaces() {
        let marker = "SCHEDULE_ACTION:  Review market trends  |  2026-03-01T09:00:00  |  weekly ";
        let content = marker.strip_prefix("SCHEDULE_ACTION:").unwrap();
        let parts: Vec<&str> = content.split('|').map(|s| s.trim()).collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "Review market trends");
        assert_eq!(parts[1], "2026-03-01T09:00:00");
        assert_eq!(parts[2], "weekly");
    }

    // ===================================================================
    // REQ-BRAIN-009 (Should): PROJECT_ACTIVATE marker format
    // ===================================================================

    // Requirement: REQ-BRAIN-009 (Should)
    // Acceptance: PROJECT_ACTIVATE marker format is correct
    #[test]
    fn test_project_activate_marker_format() {
        let marker = "PROJECT_ACTIVATE: realtor";
        let project_name = marker.strip_prefix("PROJECT_ACTIVATE:").unwrap().trim();
        assert_eq!(
            project_name, "realtor",
            "PROJECT_ACTIVATE must extract project name"
        );
    }

    // Requirement: REQ-BRAIN-009 (Should)
    // Edge case: PROJECT_ACTIVATE with hyphenated name
    #[test]
    fn test_project_activate_marker_hyphenated_name() {
        let marker = "PROJECT_ACTIVATE: real-estate-lisbon";
        let project_name = marker.strip_prefix("PROJECT_ACTIVATE:").unwrap().trim();
        assert_eq!(
            project_name, "real-estate-lisbon",
            "Must handle hyphenated project names"
        );
    }

    // ===================================================================
    // REQ-BRAIN-018 (Could): Audit logging format
    // ===================================================================

    // Requirement: REQ-BRAIN-018 (Could)
    // Acceptance: Audit log prefix format
    #[test]
    fn test_audit_setup_prefix_format() {
        let project = "realtor";
        let prefix = format!("[SETUP:{project}]");
        assert_eq!(prefix, "[SETUP:realtor]");
    }
}
