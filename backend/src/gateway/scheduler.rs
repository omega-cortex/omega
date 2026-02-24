//! Scheduled task delivery — reminders and action tasks.

use super::scheduler_action;
use super::Gateway;
use omega_core::{
    config::Prompts,
    message::{MessageMetadata, OutgoingMessage},
    traits::{Channel, Provider},
};
use omega_memory::{audit::AuditLogger, Store};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tracing::{error, info, warn};

impl Gateway {
    /// Background task: deliver due scheduled tasks.
    ///
    /// Reminder tasks send a text message. Action tasks invoke the provider
    /// with full tool access and process response markers.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn scheduler_loop(
        store: Store,
        channels: HashMap<String, Arc<dyn Channel>>,
        poll_secs: u64,
        provider: Arc<dyn Provider>,
        skills: Vec<omega_skills::Skill>,
        prompts: Prompts,
        model_complex: String,
        heartbeat_interval: Arc<AtomicU64>,
        audit: AuditLogger,
        provider_name: String,
        data_dir: String,
    ) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(poll_secs)).await;

            match store.get_due_tasks().await {
                Ok(tasks) => {
                    for (
                        id,
                        channel_name,
                        sender_id,
                        reply_target,
                        description,
                        repeat,
                        task_type,
                        project,
                    ) in &tasks
                    {
                        if task_type == "action" {
                            scheduler_action::execute_action_task(
                                id,
                                channel_name,
                                sender_id,
                                reply_target,
                                description,
                                repeat.as_deref(),
                                project,
                                &store,
                                &channels,
                                &*provider,
                                &skills,
                                &prompts,
                                &model_complex,
                                &heartbeat_interval,
                                &audit,
                                &provider_name,
                                &data_dir,
                            )
                            .await;
                            continue; // Action tasks handle their own completion.
                        }

                        // --- Reminder task: send text ---
                        let msg = OutgoingMessage {
                            text: format!("Reminder: {description}"),
                            metadata: MessageMetadata::default(),
                            reply_target: Some(reply_target.clone()),
                        };

                        if let Some(ch) = channels.get(channel_name) {
                            if let Err(e) = ch.send(msg).await {
                                error!("failed to deliver task {id}: {e}");
                                continue;
                            }
                        } else {
                            warn!("scheduler: no channel '{channel_name}' for task {id}");
                            continue;
                        }

                        if let Err(e) = store.complete_task(id, repeat.as_deref()).await {
                            error!("failed to complete task {id}: {e}");
                        } else {
                            info!("delivered scheduled task {id}: {description}");
                        }
                    }
                }
                Err(e) => {
                    error!("scheduler: failed to get due tasks: {e}");
                }
            }
        }
    }
}
