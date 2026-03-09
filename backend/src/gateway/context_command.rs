//! `/context` command handler — shows the assembled system prompt sections.
//!
//! Intercepted in pipeline.rs (like `/setup` and `/google`) because it needs
//! access to `Gateway` fields (`prompts`, `provider`, `projects`, etc.) to
//! build the prompt.

use omega_core::message::IncomingMessage;
use tracing::info;

use super::keywords::*;
use super::Gateway;
use crate::i18n;

impl Gateway {
    /// Handle the `/context` command: build the system prompt and show a section breakdown.
    pub(super) async fn handle_context_command(
        &self,
        incoming: &IncomingMessage,
        active_project: Option<&str>,
    ) {
        let lang = self
            .memory
            .get_fact(&incoming.sender_id, "preferred_language")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "English".to_string());

        let projects = &self.projects;

        // Parse optional argument: `/context scheduling` simulates keyword activation.
        let arg = incoming
            .text
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_lowercase();

        let show_full = arg == "full";

        // Determine which sections would activate.
        let needs_scheduling = arg == "scheduling" || kw_match(&arg, SCHEDULING_KW);
        let needs_projects = arg == "projects" || kw_match(&arg, PROJECTS_KW);
        let needs_builds = arg == "builds" || kw_match(&arg, BUILDS_KW);
        let needs_meta = arg == "meta" || kw_match(&arg, META_KW);

        // Build the prompt exactly as the pipeline would.
        let prompt = self.build_system_prompt(
            incoming,
            &arg,
            active_project,
            projects,
            needs_scheduling,
            needs_projects,
            needs_builds,
            needs_meta,
        );

        if show_full {
            // Send the raw prompt (split if needed for Telegram's 4096 char limit).
            let chunks = split_message(&prompt, 4000);
            for chunk in chunks {
                self.send_text(incoming, &chunk).await;
            }
            return;
        }

        // Build structured summary.
        let mut sections = Vec::new();

        let identity_chars = self.prompts.identity.len();
        let soul_chars = self.prompts.soul.len();
        let system_chars = self.prompts.system.len();

        let on = i18n::t("context_on", &lang);
        let off = i18n::t("context_off", &lang);

        sections.push(format!("[{on}] Identity ({identity_chars} chars)"));
        sections.push(format!("[{on}] Soul ({soul_chars} chars)"));
        sections.push(format!("[{on}] System ({system_chars} chars)"));

        let sched_status = if needs_scheduling { &on } else { &off };
        sections.push(format!(
            "[{sched_status}] Scheduling ({} chars)",
            self.prompts.scheduling.len()
        ));

        let proj_status = if needs_projects { &on } else { &off };
        sections.push(format!(
            "[{proj_status}] Project rules ({} chars)",
            self.prompts.projects_rules.len()
        ));

        let build_status = if needs_builds { &on } else { &off };
        sections.push(format!(
            "[{build_status}] Builds ({} chars)",
            self.prompts.builds.len()
        ));

        let meta_status = if needs_meta { &on } else { &off };
        sections.push(format!(
            "[{meta_status}] Meta ({} chars)",
            self.prompts.meta.len()
        ));

        if let Some(project_name) = active_project {
            if let Some(proj) = projects.iter().find(|p| p.name == project_name) {
                sections.push(format!(
                    "[{on}] ROLE.md: {project_name} ({} chars)",
                    proj.instructions.len()
                ));
            }
        }

        let total_chars = prompt.len();
        let total_tokens = total_chars / 4;

        let sections_text = sections.join("\n  ");
        let response = format!(
            "{}\n\n  {sections_text}\n\n{} ~{total_tokens} (~{total_chars} chars)\n\n{}",
            i18n::t("context_header", &lang),
            i18n::t("context_total", &lang),
            i18n::t("context_tip", &lang),
        );

        info!(
            "[{}] /context: {} chars, ~{} tokens",
            incoming.channel, total_chars, total_tokens
        );

        self.send_text(incoming, &response).await;
    }
}

/// Split a long message into chunks that fit within `max_chars`.
fn split_message(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        if remaining.len() <= max_chars {
            chunks.push(remaining.to_string());
            break;
        }
        // Try to split at a newline boundary.
        let split_at = remaining[..max_chars].rfind('\n').unwrap_or(max_chars);
        chunks.push(remaining[..split_at].to_string());
        remaining = remaining[split_at..].trim_start_matches('\n');
    }
    chunks
}
