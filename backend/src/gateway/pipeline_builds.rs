//! Build-related pipeline stages — build confirmation handling.
//!
//! Build intent detection is handled by the AI via `BUILD_PROPOSAL:` markers.
//! This module handles the confirmation step before starting the build pipeline.

use tracing::info;

use omega_core::message::IncomingMessage;

use super::keywords::*;
use super::Gateway;

impl Gateway {
    /// Handle a pending build confirmation (pending_build_request fact exists).
    ///
    /// Returns `true` if the caller should early-return from `handle_message`,
    /// `false` if processing should continue (expired or not confirmed).
    pub(super) async fn handle_pending_build_confirmation(
        &self,
        incoming: &IncomingMessage,
        clean_text: &str,
        typing_handle: &mut Option<tokio::task::JoinHandle<()>>,
    ) -> bool {
        let pending_build: Option<String> = self
            .memory
            .get_fact(&incoming.sender_id, "pending_build_request")
            .await
            .ok()
            .flatten();

        let stored_value = match pending_build {
            Some(v) => v,
            None => return false,
        };

        // Always clear the pending state — one-shot.
        let _ = self
            .memory
            .delete_fact(&incoming.sender_id, "pending_build_request")
            .await;

        // Parse "timestamp|request_text" and check TTL.
        let (stored_ts, stored_request) =
            stored_value.split_once('|').unwrap_or(("0", &stored_value));
        let created_at: i64 = stored_ts.parse().unwrap_or(0);
        let now = chrono::Utc::now().timestamp();
        let expired = (now - created_at) > BUILD_CONFIRM_TTL_SECS;

        if expired {
            info!(
                "[{}] pending build expired ({}s ago) — ignoring",
                incoming.channel,
                now - created_at
            );
        } else if is_build_confirmed(clean_text) {
            info!(
                "[{}] build CONFIRMED → multi-phase pipeline",
                incoming.channel
            );
            let mut build_incoming = incoming.clone();
            build_incoming.text = stored_request.to_string();
            self.handle_build_request(&build_incoming, typing_handle.take())
                .await;
            return true;
        } else if is_build_cancelled(clean_text) {
            info!("[{}] build explicitly CANCELLED by user", incoming.channel);
            let user_lang = self
                .memory
                .get_fact(&incoming.sender_id, "preferred_language")
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "English".to_string());
            if let Some(h) = typing_handle.take() {
                h.abort();
            }
            self.send_text(incoming, build_cancelled_message(&user_lang))
                .await;
            return true;
        } else {
            info!(
                "[{}] build NOT confirmed — proceeding with normal pipeline",
                incoming.channel
            );
        }
        // Fall through to normal message processing.
        false
    }
}
