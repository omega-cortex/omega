//! Periodic heartbeat check-in loop.

use super::Gateway;
use crate::markers::*;
use omega_core::{
    config::{HeartbeatConfig, Prompts},
    context::Context,
    message::{MessageMetadata, OutgoingMessage},
    traits::{Channel, Provider},
};
use omega_memory::{
    audit::{AuditEntry, AuditLogger, AuditStatus},
    Store,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

impl Gateway {
    /// Background task: periodic heartbeat check-in.
    ///
    /// Skips the provider call entirely when no checklist is configured.
    /// When a checklist exists, enriches the prompt with recent memory context.
    /// Actively executes checklist items using Opus and processes response markers.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn heartbeat_loop(
        provider: Arc<dyn Provider>,
        channels: HashMap<String, Arc<dyn Channel>>,
        config: HeartbeatConfig,
        prompts: Prompts,
        sandbox_prompt: Option<String>,
        memory: Store,
        interval: Arc<AtomicU64>,
        model_complex: String,
        skills: Vec<omega_skills::Skill>,
        audit: AuditLogger,
        provider_name: String,
    ) {
        loop {
            // Clock-aligned sleep: fire at clean boundaries (e.g. :00, :30).
            let mins = interval.load(Ordering::Relaxed);
            let now = chrono::Local::now();
            use chrono::Timelike;
            let current_minute = u64::from(now.hour()) * 60 + u64::from(now.minute());
            let next_boundary = ((current_minute / mins) + 1) * mins;
            let wait_secs = (next_boundary - current_minute) * 60 - u64::from(now.second());
            tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;

            // Check active hours.
            if !config.active_start.is_empty()
                && !config.active_end.is_empty()
                && !is_within_active_hours(&config.active_start, &config.active_end)
            {
                info!("heartbeat: outside active hours, skipping");
                continue;
            }

            // Read optional checklist — skip API call if none configured.
            let checklist = match read_heartbeat_file() {
                Some(cl) => cl,
                None => {
                    info!("heartbeat: no checklist configured, skipping");
                    continue;
                }
            };

            let mut prompt = prompts
                .heartbeat_checklist
                .replace("{checklist}", &checklist);

            // Enrich heartbeat context with recent memory.
            if let Ok(facts) = memory.get_all_facts().await {
                if !facts.is_empty() {
                    prompt.push_str("\n\nKnown about the user:");
                    for (key, value) in &facts {
                        prompt.push_str(&format!("\n- {key}: {value}"));
                    }
                }
            }
            if let Ok(summaries) = memory.get_all_recent_summaries(3).await {
                if !summaries.is_empty() {
                    prompt.push_str("\n\nRecent activity:");
                    for (summary, timestamp) in &summaries {
                        prompt.push_str(&format!("\n- [{timestamp}] {summary}"));
                    }
                }
            }

            let mut system = format!(
                "{}\n\n{}\n\n{}",
                prompts.identity, prompts.soul, prompts.system
            );
            if let Some(ref sp) = sandbox_prompt {
                system.push_str("\n\n");
                system.push_str(sp);
            }
            system.push_str(&format!(
                "\n\nCurrent time: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M %Z")
            ));

            let mut ctx = Context::new(&prompt);
            ctx.system_prompt = system;
            ctx.model = Some(model_complex.clone());

            // Match skill triggers on checklist content to inject MCP servers.
            let matched_servers = omega_skills::match_skill_triggers(&skills, &checklist);
            ctx.mcp_servers = matched_servers;

            let started = Instant::now();
            match provider.complete(&ctx).await {
                Ok(resp) => {
                    let elapsed_ms = started.elapsed().as_millis() as i64;
                    let mut text = resp.text.clone();

                    // Process markers from heartbeat response (same as scheduler).
                    let sender_id = &config.reply_target;
                    let channel_name = &config.channel;

                    for sched_line in extract_all_schedule_markers(&text) {
                        if let Some((desc, due, rep)) = parse_schedule_line(&sched_line) {
                            let rep_opt = if rep == "once" {
                                None
                            } else {
                                Some(rep.as_str())
                            };
                            match memory
                                .create_task(
                                    channel_name,
                                    sender_id,
                                    sender_id,
                                    &desc,
                                    &due,
                                    rep_opt,
                                    "reminder",
                                )
                                .await
                            {
                                Ok(new_id) => info!("heartbeat spawned reminder {new_id}: {desc}"),
                                Err(e) => error!("heartbeat: failed to create reminder: {e}"),
                            }
                        }
                    }
                    text = strip_schedule_marker(&text);

                    for sched_line in extract_all_schedule_action_markers(&text) {
                        if let Some((desc, due, rep)) = parse_schedule_action_line(&sched_line) {
                            let rep_opt = if rep == "once" {
                                None
                            } else {
                                Some(rep.as_str())
                            };
                            match memory
                                .create_task(
                                    channel_name,
                                    sender_id,
                                    sender_id,
                                    &desc,
                                    &due,
                                    rep_opt,
                                    "action",
                                )
                                .await
                            {
                                Ok(new_id) => info!("heartbeat spawned action {new_id}: {desc}"),
                                Err(e) => error!("heartbeat: failed to create action: {e}"),
                            }
                        }
                    }
                    text = strip_schedule_action_markers(&text);

                    let hb_actions = extract_heartbeat_markers(&text);
                    if !hb_actions.is_empty() {
                        apply_heartbeat_changes(&hb_actions);
                        for action in &hb_actions {
                            if let HeartbeatAction::SetInterval(mins) = action {
                                interval.store(*mins, Ordering::Relaxed);
                                info!("heartbeat: interval changed to {mins} minutes (via heartbeat loop)");
                            }
                        }
                        text = strip_heartbeat_markers(&text);
                    }

                    for id_prefix in extract_all_cancel_tasks(&text) {
                        match memory.cancel_task(&id_prefix, sender_id).await {
                            Ok(true) => info!("heartbeat cancelled task: {id_prefix}"),
                            Ok(false) => {
                                warn!("heartbeat: no matching task to cancel: {id_prefix}")
                            }
                            Err(e) => error!("heartbeat: failed to cancel task: {e}"),
                        }
                    }
                    text = strip_cancel_task(&text);

                    for update_line in extract_all_update_tasks(&text) {
                        if let Some((id_prefix, desc, due_at, repeat)) =
                            parse_update_task_line(&update_line)
                        {
                            match memory
                                .update_task(
                                    &id_prefix,
                                    sender_id,
                                    desc.as_deref(),
                                    due_at.as_deref(),
                                    repeat.as_deref(),
                                )
                                .await
                            {
                                Ok(true) => info!("heartbeat updated task: {id_prefix}"),
                                Ok(false) => {
                                    warn!("heartbeat: no matching task to update: {id_prefix}")
                                }
                                Err(e) => error!("heartbeat: failed to update task: {e}"),
                            }
                        }
                    }
                    text = strip_update_task(&text);

                    // Check for HEARTBEAT_OK after stripping all markers.
                    let cleaned: String = text.chars().filter(|c| *c != '*' && *c != '`').collect();
                    if cleaned.trim().contains("HEARTBEAT_OK") {
                        info!("heartbeat: OK");
                    } else {
                        // Audit log the heartbeat execution.
                        let audit_entry = AuditEntry {
                            channel: channel_name.clone(),
                            sender_id: sender_id.clone(),
                            sender_name: None,
                            input_text: "[HEARTBEAT]".to_string(),
                            output_text: Some(text.clone()),
                            provider_used: Some(provider_name.clone()),
                            model: Some(model_complex.clone()),
                            processing_ms: Some(elapsed_ms),
                            status: AuditStatus::Ok,
                            denial_reason: None,
                        };
                        if let Err(e) = audit.log(&audit_entry).await {
                            error!("heartbeat: audit log failed: {e}");
                        }

                        if let Some(ch) = channels.get(channel_name) {
                            let msg = OutgoingMessage {
                                text: text.trim().to_string(),
                                metadata: MessageMetadata::default(),
                                reply_target: Some(config.reply_target.clone()),
                            };
                            if let Err(e) = ch.send(msg).await {
                                error!("heartbeat: failed to send alert: {e}");
                            }
                        } else {
                            warn!(
                                "heartbeat: channel '{}' not found, alert dropped",
                                config.channel
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("heartbeat: provider error: {e}");
                }
            }
        }
    }
}
