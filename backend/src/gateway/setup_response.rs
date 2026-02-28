//! Setup session response handling — questioning and confirmation phases.

use std::path::{Path, PathBuf};

use omega_core::config::shellexpand;
use omega_core::message::IncomingMessage;
use tracing::warn;

use super::builds_agents::{AgentFilesGuard, BRAIN_AGENT};
use super::keywords::*;
use super::setup::{parse_setup_output, parse_setup_round, setup_context_path, SetupOutput};
use super::Gateway;

/// Extract the user-facing preview from a proposal (text before SETUP_EXECUTE).
fn extract_proposal_preview(proposal: &str) -> String {
    proposal
        .split("SETUP_EXECUTE")
        .next()
        .unwrap_or(proposal)
        .replace("SETUP_PROPOSAL", "")
        .trim()
        .to_string()
}

impl Gateway {
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
        let ttl = SETUP_TTL_SECS;

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
            self.handle_setup_confirmation(
                incoming,
                &context,
                &ctx_path,
                &omega_dir,
                &user_lang,
                typing_handle,
            )
            .await;
        } else {
            self.handle_setup_questioning(
                incoming,
                &context,
                &ctx_path,
                &omega_dir,
                &user_lang,
                typing_handle,
            )
            .await;
        }
    }

    /// Handle the confirmation phase: user approved, rejected, or wants modifications.
    async fn handle_setup_confirmation(
        &self,
        incoming: &IncomingMessage,
        context: &str,
        ctx_path: &Path,
        omega_dir: &Path,
        user_lang: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        if is_build_confirmed(&incoming.text) {
            // Execute the approved setup.
            match self.execute_setup(incoming, context).await {
                Ok(mut output) => {
                    // Extract project name BEFORE process_markers strips it.
                    let project_name = output
                        .lines()
                        .find(|l| l.starts_with("PROJECT_ACTIVATE:"))
                        .and_then(|l| l.strip_prefix("PROJECT_ACTIVATE:"))
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "project".to_string());

                    // Process markers from Brain output.
                    let active_project: Option<String> = self
                        .memory
                        .get_fact(&incoming.sender_id, "active_project")
                        .await
                        .ok()
                        .flatten();
                    self.process_markers(incoming, &mut output, active_project.as_deref())
                        .await;

                    self.cleanup_setup_session(&incoming.sender_id).await;
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    let msg = setup_complete_message(user_lang, &project_name);
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
            let _ = tokio::fs::write(ctx_path, &updated_context).await;

            let prompt = format!(
                "The user wants modifications to the proposed setup.\n\n\
                 Previous context and proposal:\n{updated_context}\n\n\
                 Update the proposal based on the user's feedback. \
                 Output SETUP_PROPOSAL with the updated plan."
            );

            let _agent_guard =
                match AgentFilesGuard::write_single(omega_dir, "omega-brain", BRAIN_AGENT).await {
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
                        let preview = extract_proposal_preview(&proposal);
                        let _ = tokio::fs::write(ctx_path, &proposal).await;
                        let msg = setup_proposal_message(user_lang, &preview);
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
    }

    /// Handle the questioning phase: append user answer, run next round.
    async fn handle_setup_questioning(
        &self,
        incoming: &IncomingMessage,
        context: &str,
        ctx_path: &Path,
        omega_dir: &Path,
        user_lang: &str,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        let current_round = parse_setup_round(context);
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
        let _ = tokio::fs::write(ctx_path, &updated_context).await;

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
            match AgentFilesGuard::write_single(omega_dir, "omega-brain", BRAIN_AGENT).await {
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
                    let _ = tokio::fs::write(ctx_path, &file_content).await;
                    let msg = setup_followup_message(user_lang, &questions, next_round);
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    self.send_text(incoming, &msg).await;
                }
                SetupOutput::Proposal(proposal) => {
                    let preview = extract_proposal_preview(&proposal);
                    let _ = tokio::fs::write(ctx_path, &proposal).await;

                    let stamped = format!(
                        "{}|{}|proposal",
                        chrono::Utc::now().timestamp(),
                        incoming.sender_id
                    );
                    let _ = self
                        .memory
                        .store_fact(&incoming.sender_id, "pending_setup", &stamped)
                        .await;

                    let msg = setup_proposal_message(user_lang, &preview);
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    self.send_text(incoming, &msg).await;
                }
                SetupOutput::Executed(_) => {
                    warn!("Brain returned Executed in questioning mode");
                    self.cleanup_setup_session(&incoming.sender_id).await;
                    if let Some(h) = typing_handle {
                        h.abort();
                    }
                    self.send_text(
                        incoming,
                        "Setup encountered an unexpected state. Please try again with /setup.",
                    )
                    .await;
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
