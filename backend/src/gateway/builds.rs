//! Topology-driven build orchestrator — dispatches build phases from a loaded topology.
//!
//! Loads the "development" topology from `~/.omega/topologies/development/TOPOLOGY.toml`,
//! then iterates over `topology.phases`, dispatching each based on its `phase_type`:
//!
//! - ParseBrief: run agent, parse output via parse_project_brief(), create dir
//! - Standard: run agent, check for error, proceed
//! - CorrectiveLoop: run agent, parse result, retry with fix_agent on failure
//! - ParseSummary: run agent, parse output via parse_build_summary(), send final msg
//!
//! Safety controls:
//! - Pre-validation before phases (from topology config)
//! - Post-validation after phases (from topology config)
//! - Corrective loops with configurable retries
//! - Chain state persisted on failure for recovery inspection

use super::builds_agents::AgentFilesGuard;
use super::builds_parse::*;
use super::builds_topology::{self, PhaseType};
use super::Gateway;
use omega_core::{config::shellexpand, context::Context, message::IncomingMessage};
use omega_memory::audit::{AuditEntry, AuditStatus};
use std::path::PathBuf;
use tracing::warn;

/// State accumulated during orchestration, passed between phases.
#[derive(Default)]
pub(super) struct OrchestratorState {
    /// Raw text output from the analyst (ParseBrief) phase.
    pub(super) brief_text: Option<String>,
    /// Parsed project brief (name, scope, language, etc.).
    pub(super) brief: Option<ProjectBrief>,
    /// Project directory path (created after brief is parsed).
    pub(super) project_dir: Option<PathBuf>,
    /// Project directory as string (for prompt interpolation).
    pub(super) project_dir_str: Option<String>,
    /// Phases completed so far (names, for chain state).
    pub(super) completed_phases: Vec<String>,
}

// ---------------------------------------------------------------------------
// Gateway methods
// ---------------------------------------------------------------------------

impl Gateway {
    /// Main build orchestrator — topology-driven phase loop.
    pub(super) async fn handle_build_request(
        &self,
        incoming: &IncomingMessage,
        typing_handle: Option<tokio::task::JoinHandle<()>>,
    ) {
        // Resolve user language for localized messages.
        let user_lang = self
            .memory
            .get_fact(&incoming.sender_id, "preferred_language")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "English".to_string());

        // Load topology.
        let loaded = match builds_topology::load_topology(&self.data_dir, "development") {
            Ok(t) => t,
            Err(e) => {
                if let Some(h) = typing_handle {
                    h.abort();
                }
                self.send_text(incoming, &format!("Failed to load topology: {e}"))
                    .await;
                return;
            }
        };

        // Write agent files to workspace root BEFORE any phase runs.
        let workspace_dir = PathBuf::from(shellexpand(&self.data_dir)).join("workspace");
        let _agent_guard = match AgentFilesGuard::write_from_topology(&workspace_dir, &loaded).await
        {
            Ok(guard) => guard,
            Err(e) => {
                if let Some(h) = typing_handle {
                    h.abort();
                }
                self.send_text(incoming, &format!("Failed to write agent files: {e}"))
                    .await;
                return;
            }
        };

        let mut state = OrchestratorState::default();

        for phase in &loaded.topology.phases {
            let model = loaded.resolve_model(phase, &self.model_fast, &self.model_complex);

            // Send localized phase message.
            self.send_text(incoming, &phase_message_by_name(&user_lang, &phase.name))
                .await;

            // Run pre-validation if configured.
            if let Some(ref validation) = phase.pre_validation {
                if let Some(project_dir) = &state.project_dir {
                    if let Some(err) = Gateway::run_validation(project_dir, validation) {
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &err).await;
                        let cs = Self::chain_state_topo(
                            &state,
                            &format!("{} (validation)", phase.name),
                            err,
                        );
                        if let Some(pd) = &state.project_dir {
                            Gateway::save_chain_state(pd, &cs).await;
                        }
                        return;
                    }
                }
            }

            // Dispatch based on phase type.
            match phase.phase_type {
                PhaseType::ParseBrief => {
                    if let Err(reason) = self
                        .execute_parse_brief(incoming, phase, model, &mut state)
                        .await
                    {
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &reason).await;
                        return;
                    }
                }
                PhaseType::Standard => {
                    if let Err(reason) = self.execute_standard(incoming, phase, model, &state).await
                    {
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        self.send_text(incoming, &reason).await;
                        return;
                    }
                }
                PhaseType::CorrectiveLoop => {
                    let retry = match &phase.retry {
                        Some(r) => r,
                        None => {
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            self.send_text(
                                incoming,
                                &format!(
                                    "Configuration error: phase '{}' is corrective-loop but has no retry config",
                                    phase.name
                                ),
                            )
                            .await;
                            return;
                        }
                    };

                    if let Err(reason) = self
                        .run_corrective_loop(incoming, &state, &user_lang, phase, retry, model)
                        .await
                    {
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        let project_dir_str =
                            state.project_dir_str.as_deref().unwrap_or("(unknown)");
                        let brief_name = state
                            .brief
                            .as_ref()
                            .map(|b| b.name.as_str())
                            .unwrap_or("(unknown)");

                        // Send exhausted message matching the loop type.
                        let exhausted_msg = if phase.name == "qa" {
                            qa_exhausted_message(&user_lang, &reason, project_dir_str)
                        } else {
                            review_exhausted_message(&user_lang, &reason, project_dir_str)
                        };
                        self.send_text(incoming, &exhausted_msg).await;
                        self.audit_build(incoming, brief_name, "failed", &reason)
                            .await;
                        let cs = Self::chain_state_topo(&state, &phase.name, reason);
                        if let Some(pd) = &state.project_dir {
                            Gateway::save_chain_state(pd, &cs).await;
                        }
                        return;
                    }
                }
                PhaseType::ParseSummary => {
                    if let Err(reason) = self
                        .execute_parse_summary(incoming, phase, model, &state)
                        .await
                    {
                        // Delivery error is partial success.
                        if let Some(h) = typing_handle {
                            h.abort();
                        }
                        let brief_name = state
                            .brief
                            .as_ref()
                            .map(|b| b.name.as_str())
                            .unwrap_or("(unknown)");
                        self.send_text(
                            incoming,
                            &format!(
                                "Build complete but delivery had issues: {reason}\nProject: `{brief_name}`"
                            ),
                        )
                        .await;
                        self.audit_build(incoming, brief_name, "partial", &reason)
                            .await;
                        return;
                    }
                }
            }

            // Run post-validation if configured.
            if let Some(ref paths) = phase.post_validation {
                if let Some(project_dir) = &state.project_dir {
                    for path in paths {
                        // Reject path traversal in post_validation paths.
                        if path.contains("..") || path.starts_with('/') || path.contains('\\') {
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            let msg = format!(
                                "Post-validation rejected: path '{}' contains invalid characters.",
                                path
                            );
                            self.send_text(incoming, &msg).await;
                            return;
                        }
                        if !project_dir.join(path).exists() {
                            if let Some(h) = typing_handle {
                                h.abort();
                            }
                            let msg = format!(
                                "{} phase completed but {} was not generated. Build stopped.",
                                phase.name, path
                            );
                            self.send_text(incoming, &msg).await;
                            return;
                        }
                    }
                    // Post-validation passed — send confirmation for architect.
                    if phase.name == "architect" {
                        self.send_text(incoming, "Architecture defined.").await;
                    }
                }
            }

            state.completed_phases.push(phase.name.clone());
        }

        // All phases completed successfully.
        if let Some(h) = typing_handle {
            h.abort();
        }
        let brief_name = state
            .brief
            .as_ref()
            .map(|b| b.name.as_str())
            .unwrap_or("(unknown)");
        self.audit_build(incoming, brief_name, "success", "").await;
    }

    /// Execute a ParseBrief phase: run analyst, parse brief, create project dir.
    async fn execute_parse_brief(
        &self,
        incoming: &IncomingMessage,
        phase: &builds_topology::Phase,
        model: &str,
        state: &mut OrchestratorState,
    ) -> Result<(), String> {
        let brief_text = self
            .run_build_phase(&phase.agent, &incoming.text, model, phase.max_turns)
            .await
            .map_err(|e| format!("Could not analyze your build request: {e}"))?;

        let brief = parse_project_brief(&brief_text)
            .ok_or("Could not parse the build brief. Please try rephrasing.")?;

        let project_dir = PathBuf::from(shellexpand(&self.data_dir))
            .join("workspace/builds")
            .join(&brief.name);
        let project_dir_str = project_dir.display().to_string();

        tokio::fs::create_dir_all(&project_dir)
            .await
            .map_err(|e| format!("Failed to create project directory: {e}"))?;

        self.send_text(
            incoming,
            &format!(
                "Building `{}` \u{2014} {}. I'll keep you posted.",
                brief.name, brief.scope
            ),
        )
        .await;

        state.brief_text = Some(brief_text);
        state.project_dir = Some(project_dir);
        state.project_dir_str = Some(project_dir_str);
        state.brief = Some(brief);
        Ok(())
    }

    /// Execute a Standard phase: run agent, check for error.
    async fn execute_standard(
        &self,
        incoming: &IncomingMessage,
        phase: &builds_topology::Phase,
        model: &str,
        state: &OrchestratorState,
    ) -> Result<(), String> {
        let project_dir_str = state.project_dir_str.as_deref().unwrap_or("");
        let brief_text = state.brief_text.as_deref().unwrap_or("");
        let brief_name = state.brief.as_ref().map(|b| b.name.as_str()).unwrap_or("");

        // Build phase-specific prompt.
        let prompt = match phase.name.as_str() {
            "architect" => {
                format!(
                    "Project brief:\n{brief_text}\nBegin architecture design in {project_dir_str}."
                )
            }
            "test-writer" => {
                format!("Read specs/ in {project_dir_str} and write failing tests. Begin.")
            }
            "developer" => {
                format!("Read the tests and specs/ in {project_dir_str}. Implement until all tests pass. Begin.")
            }
            _ => format!("Execute phase '{}' in {project_dir_str}.", phase.name),
        };

        self.run_build_phase(&phase.agent, &prompt, model, phase.max_turns)
            .await
            .map_err(|e| {
                format!(
                    "{} phase failed: {e}. Partial results in `{brief_name}`.",
                    phase.name
                )
            })?;

        // Phase-specific completion messages.
        if phase.name == "test-writer" {
            self.send_text(incoming, "Tests written.").await;
        } else if phase.name == "developer" {
            self.send_text(incoming, "Implementation complete \u{2014} verifying...")
                .await;
        }

        Ok(())
    }

    /// Execute a ParseSummary phase: run delivery, parse summary, send final message.
    ///
    /// Note: typing handle lifecycle is managed by the caller (handle_build_request).
    /// This method does NOT abort the typing handle.
    async fn execute_parse_summary(
        &self,
        incoming: &IncomingMessage,
        phase: &builds_topology::Phase,
        model: &str,
        state: &OrchestratorState,
    ) -> Result<(), String> {
        let project_dir_str = state.project_dir_str.as_deref().unwrap_or("");
        let brief_name = state.brief.as_ref().map(|b| b.name.as_str()).unwrap_or("");
        let skills_dir = PathBuf::from(shellexpand(&self.data_dir)).join("skills");
        let skills_dir_str = skills_dir.display().to_string();

        let delivery_prompt = format!(
            "Create docs and skill file for {brief_name} in {project_dir_str}. Skills dir: {skills_dir_str}.",
        );

        let delivery_text = self
            .run_build_phase(&phase.agent, &delivery_prompt, model, phase.max_turns)
            .await?;

        // Parse and send final summary.
        let final_msg = if let Some(summary) = parse_build_summary(&delivery_text) {
            format!(
                "Build complete!\n\n\
                 *{}*\n\
                 {}\n\n\
                 Location: `{}`\n\
                 Language: {}\n\
                 Usage: `{}`{}",
                summary.project,
                summary.summary,
                summary.location,
                summary.language,
                summary.usage,
                summary
                    .skill
                    .as_ref()
                    .map(|s| format!("\nSkill: {s}"))
                    .unwrap_or_default(),
            )
        } else {
            format!("Build complete!\n\nProject `{brief_name}` is ready.",)
        };

        self.send_text(incoming, &final_msg).await;
        Ok(())
    }

    /// Build a `ChainState` from the current orchestrator state at a given failure point.
    fn chain_state_topo(state: &OrchestratorState, failed: &str, reason: String) -> ChainState {
        ChainState {
            project_name: state
                .brief
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_default(),
            project_dir: state.project_dir_str.clone().unwrap_or_default(),
            completed_phases: state.completed_phases.clone(),
            failed_phase: Some(failed.to_string()),
            failure_reason: Some(reason),
            topology_name: Some("development".to_string()),
        }
    }

    /// Generic phase runner with retry logic (3 attempts, 2s delay).
    ///
    /// Each phase gets a fresh Context with `agent_name` set and no session_id.
    /// The agent file provides the system prompt; only the user message is sent via `-p`.
    pub(super) async fn run_build_phase(
        &self,
        agent_name: &str,
        user_message: &str,
        model: &str,
        max_turns: Option<u32>,
    ) -> Result<String, String> {
        let mut ctx = Context::new(user_message);
        ctx.system_prompt = String::new();
        ctx.agent_name = Some(agent_name.to_string());
        ctx.model = Some(model.to_string());
        // Explicit max_turns prevents auto-resume from losing agent context.
        ctx.max_turns = Some(max_turns.unwrap_or(100));

        for attempt in 1..=3u32 {
            match self.provider.complete(&ctx).await {
                Ok(resp) => return Ok(resp.text),
                Err(e) => {
                    warn!("build phase '{agent_name}' attempt {attempt}/3 failed: {e}");
                    if attempt < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }
        Err(format!("phase '{agent_name}' failed after 3 attempts"))
    }

    /// Log an audit entry for a build operation.
    async fn audit_build(
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
                input_text: format!("[BUILD:{project}] {}", incoming.text),
                output_text: Some(format!("[{status}] {detail}")),
                provider_used: Some(self.provider.name().to_string()),
                model: None,
                processing_ms: None,
                status: if status == "success" {
                    AuditStatus::Ok
                } else {
                    AuditStatus::Error
                },
                denial_reason: None,
            })
            .await;
    }
}
