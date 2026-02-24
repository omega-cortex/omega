//! Background conversation summarization and fact extraction.

use super::keywords::is_valid_fact;
use super::Gateway;
use crate::i18n;
use omega_core::{context::Context, traits::Provider};
use omega_memory::Store;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Summarize a conversation and extract facts in a single provider call.
/// Designed for background use — all errors are logged, never surfaced.
pub(super) async fn summarize_and_extract(
    store: &Store,
    provider: &Arc<dyn Provider>,
    conversation_id: &str,
    summarize_prompt: &str,
    facts_prompt: &str,
) -> Result<(), anyhow::Error> {
    let messages = store.get_conversation_messages(conversation_id).await?;
    if messages.is_empty() {
        store
            .close_conversation(conversation_id, "(empty conversation)")
            .await?;
        return Ok(());
    }

    // Build transcript.
    let mut transcript = String::new();
    for (role, content) in &messages {
        let label = if role == "user" { "User" } else { "Assistant" };
        transcript.push_str(&format!("{label}: {content}\n"));
    }

    // Single combined prompt for summary + facts.
    let combined_prompt = format!(
        "{summarize_prompt}\n\n\
         Additionally, extract personal facts about the user from this conversation.\n\
         {facts_prompt}\n\n\
         Transcript:\n{transcript}\n\n\
         Respond in this exact format:\n\
         SUMMARY: <1-2 sentence summary>\n\
         FACTS:\n\
         <key: value per line, or \"none\">"
    );
    let ctx = Context::new(&combined_prompt);

    match provider.complete(&ctx).await {
        Ok(resp) => {
            let text = resp.text.trim();
            // Parse response: split on FACTS: line.
            let (summary, facts_section) = if let Some(idx) = text.find("\nFACTS:") {
                let summary_part = text[..idx].trim();
                let facts_part = text[idx + 7..].trim(); // skip "\nFACTS:"
                                                         // Strip "SUMMARY: " prefix if present.
                let summary_clean = summary_part
                    .strip_prefix("SUMMARY:")
                    .unwrap_or(summary_part)
                    .trim();
                (summary_clean.to_string(), facts_part.to_string())
            } else {
                // No FACTS: section — treat entire response as summary.
                let summary_clean = text.strip_prefix("SUMMARY:").unwrap_or(text).trim();
                (summary_clean.to_string(), String::new())
            };

            // Store facts if present.
            if !facts_section.is_empty() && facts_section.to_lowercase() != "none" {
                let conv_info: Option<(String,)> =
                    sqlx::query_as("SELECT sender_id FROM conversations WHERE id = ?")
                        .bind(conversation_id)
                        .fetch_optional(store.pool())
                        .await
                        .ok()
                        .flatten();

                if let Some((sender_id,)) = conv_info {
                    for line in facts_section.lines() {
                        if let Some((key, value)) = line.split_once(':') {
                            let key = key.trim().trim_start_matches("- ").to_lowercase();
                            let value = value.trim().to_string();
                            if !key.is_empty() && !value.is_empty() && is_valid_fact(&key, &value) {
                                let _ = store.store_fact(&sender_id, &key, &value).await;
                            }
                        }
                    }
                }
            }

            // Update the already-closed conversation with the summary.
            store.close_conversation(conversation_id, &summary).await?;
            info!("Conversation {conversation_id} summarized in background");
        }
        Err(e) => {
            warn!("background summarization failed: {e}");
            // Conversation is already closed — just log, no fallback needed.
        }
    }

    Ok(())
}

impl Gateway {
    /// Background task: periodically find and summarize idle conversations.
    pub(super) async fn background_summarizer(
        store: Store,
        provider: Arc<dyn Provider>,
        summarize_prompt: String,
        facts_prompt: String,
    ) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            match store.find_idle_conversations().await {
                Ok(convos) => {
                    for (conv_id, channel, sender_id, project) in &convos {
                        if let Err(e) = Self::summarize_conversation(
                            &store,
                            &provider,
                            conv_id,
                            &summarize_prompt,
                            &facts_prompt,
                        )
                        .await
                        {
                            error!("failed to summarize conversation {conv_id}: {e}");
                        }
                        // Clear CLI session for this project when conversation is closed.
                        let _ = store.clear_session(channel, sender_id, project).await;
                    }
                }
                Err(e) => {
                    error!("failed to find idle conversations: {e}");
                }
            }
        }
    }

    /// Summarize a conversation using the provider, extract facts, then close it.
    pub async fn summarize_conversation(
        store: &Store,
        provider: &Arc<dyn Provider>,
        conversation_id: &str,
        summarize_prompt: &str,
        facts_prompt_template: &str,
    ) -> Result<(), anyhow::Error> {
        let messages = store.get_conversation_messages(conversation_id).await?;
        if messages.is_empty() {
            store
                .close_conversation(conversation_id, "(empty conversation)")
                .await?;
            return Ok(());
        }

        // Build a transcript for summarization.
        let mut transcript = String::new();
        for (role, content) in &messages {
            let label = if role == "user" { "User" } else { "Assistant" };
            transcript.push_str(&format!("{label}: {content}\n"));
        }

        // Ask provider to summarize.
        let full_summary_prompt = format!("{summarize_prompt}\n\n{transcript}");
        let summary_ctx = Context::new(&full_summary_prompt);
        let summary = match provider.complete(&summary_ctx).await {
            Ok(resp) => resp.text,
            Err(e) => {
                warn!("summarization failed, using fallback: {e}");
                format!("({} messages, summary unavailable)", messages.len())
            }
        };

        // Ask provider to extract facts.
        let facts_prompt = format!("{facts_prompt_template}\n\n{transcript}");
        let facts_ctx = Context::new(&facts_prompt);
        if let Ok(facts_resp) = provider.complete(&facts_ctx).await {
            let text = facts_resp.text.trim().to_string();
            if text.to_lowercase() != "none" {
                // Find sender_id from the conversation messages context.
                // We need the sender_id — extract from the conversation row.
                let conv_info: Option<(String,)> =
                    sqlx::query_as("SELECT sender_id FROM conversations WHERE id = ?")
                        .bind(conversation_id)
                        .fetch_optional(store.pool())
                        .await
                        .ok()
                        .flatten();

                if let Some((sender_id,)) = conv_info {
                    for line in text.lines() {
                        if let Some((key, value)) = line.split_once(':') {
                            let key = key.trim().trim_start_matches("- ").to_lowercase();
                            let value = value.trim().to_string();
                            if !key.is_empty() && !value.is_empty() {
                                if is_valid_fact(&key, &value) {
                                    let _ = store.store_fact(&sender_id, &key, &value).await;
                                } else {
                                    debug!("rejected invalid fact: {key}: {value}");
                                }
                            }
                        }
                    }
                }
            }
        }

        store.close_conversation(conversation_id, &summary).await?;
        info!("Conversation {conversation_id} summarized and closed");
        Ok(())
    }

    /// Handle /forget: close conversation instantly, summarize in background.
    pub(super) async fn handle_forget(&self, channel: &str, sender_id: &str) -> String {
        let lang = self
            .memory
            .get_fact(sender_id, "preferred_language")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "English".to_string());

        // Scope forget to the active project.
        let active_project = self
            .memory
            .get_fact(sender_id, "active_project")
            .await
            .ok()
            .flatten()
            .unwrap_or_default();
        let project_key = active_project.as_str();

        // Find the active conversation for this sender + project.
        let conv: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM conversations \
             WHERE channel = ? AND sender_id = ? AND project = ? AND status = 'active' \
             ORDER BY last_activity DESC LIMIT 1",
        )
        .bind(channel)
        .bind(sender_id)
        .bind(project_key)
        .fetch_optional(self.memory.pool())
        .await
        .ok()
        .flatten();

        match conv {
            Some((conversation_id,)) => {
                // Close immediately so new messages start a fresh conversation.
                let _ = self
                    .memory
                    .close_current_conversation(channel, sender_id, project_key)
                    .await;

                // Clear CLI session — next message starts fresh.
                let _ = self
                    .memory
                    .clear_session(channel, sender_id, project_key)
                    .await;

                // Summarize + extract facts in the background.
                let store = self.memory.clone();
                let provider = Arc::clone(&self.provider);
                let summarize_prompt = self.prompts.summarize.clone();
                let facts_prompt = self.prompts.facts.clone();
                tokio::spawn(async move {
                    if let Err(e) = summarize_and_extract(
                        &store,
                        &provider,
                        &conversation_id,
                        &summarize_prompt,
                        &facts_prompt,
                    )
                    .await
                    {
                        warn!("background summarization after /forget failed: {e}");
                    }
                });

                i18n::t("conversation_cleared", &lang).to_string()
            }
            None => i18n::t("no_active_conversation", &lang).to_string(),
        }
    }
}
