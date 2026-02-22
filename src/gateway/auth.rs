//! Authentication checks and WhatsApp QR pairing flow.

use super::Gateway;
use omega_channels::whatsapp;
use omega_core::message::IncomingMessage;
use tracing::warn;

impl Gateway {
    /// Check if an incoming message is authorized.
    /// Returns `None` if allowed, `Some(reason)` if denied.
    pub(super) fn check_auth(&self, incoming: &IncomingMessage) -> Option<String> {
        match incoming.channel.as_str() {
            "telegram" => {
                let allowed = self
                    .channel_config
                    .telegram
                    .as_ref()
                    .map(|tg| &tg.allowed_users);

                match allowed {
                    Some(users) if users.is_empty() => {
                        // Empty list = allow all (for easy testing).
                        None
                    }
                    Some(users) => {
                        let sender_id: i64 = incoming.sender_id.parse().unwrap_or(-1);
                        if users.contains(&sender_id) {
                            None
                        } else {
                            Some(format!(
                                "telegram user {} not in allowed_users",
                                incoming.sender_id
                            ))
                        }
                    }
                    None => Some("telegram channel not configured".to_string()),
                }
            }
            "whatsapp" => {
                let allowed = self
                    .channel_config
                    .whatsapp
                    .as_ref()
                    .map(|wa| &wa.allowed_users);

                match allowed {
                    Some(users) if users.is_empty() => None,
                    Some(users) => {
                        if users.contains(&incoming.sender_id) {
                            None
                        } else {
                            Some(format!(
                                "whatsapp user {} not in allowed_users",
                                incoming.sender_id
                            ))
                        }
                    }
                    None => Some("whatsapp channel not configured".to_string()),
                }
            }
            other => Some(format!("unknown channel: {other}")),
        }
    }

    /// Handle the WHATSAPP_QR flow: use the running bot's event stream for pairing.
    pub(super) async fn handle_whatsapp_qr(&self, incoming: &IncomingMessage) {
        use omega_channels::whatsapp::WhatsAppChannel;

        // Downcast the whatsapp channel to access pairing_channels().
        let wa_channel = match self.channels.get("whatsapp") {
            Some(ch) => match ch.as_any().downcast_ref::<WhatsAppChannel>() {
                Some(wa) => wa,
                None => {
                    self.send_text(incoming, "WhatsApp channel not available.")
                        .await;
                    return;
                }
            },
            None => {
                self.send_text(incoming, "WhatsApp channel not configured.")
                    .await;
                return;
            }
        };

        // If already connected, no need to pair again.
        if wa_channel.is_connected().await {
            self.send_text(
                incoming,
                "WhatsApp is already connected! Send yourself a message to test.",
            )
            .await;
            return;
        }

        self.send_text(incoming, "Starting WhatsApp pairing...")
            .await;

        // Delete stale session and restart bot so it generates fresh QR codes.
        // This handles the case where WhatsApp was unlinked from the phone —
        // the library won't generate QR codes with invalidated session keys.
        if let Err(e) = wa_channel.restart_for_pairing().await {
            warn!("WhatsApp restart_for_pairing failed: {e}");
            self.send_text(incoming, &format!("WhatsApp pairing failed: {e}"))
                .await;
            return;
        }

        // Get fresh receivers from the restarted bot.
        let (mut qr_rx, mut done_rx) = wa_channel.pairing_channels().await;

        // Wait for the first QR code (with timeout).
        let qr_timeout = tokio::time::timeout(std::time::Duration::from_secs(30), qr_rx.recv());

        match qr_timeout.await {
            Ok(Some(qr_data)) => {
                // Generate QR image and send it.
                match whatsapp::generate_qr_image(&qr_data) {
                    Ok(png_bytes) => {
                        if let Some(channel) = self.channels.get(&incoming.channel) {
                            let target = incoming.reply_target.as_deref().unwrap_or("");
                            if let Err(e) = channel
                                .send_photo(
                                    target,
                                    &png_bytes,
                                    "Scan with WhatsApp (Link a Device > QR Code)",
                                )
                                .await
                            {
                                warn!("failed to send QR image: {e}");
                                self.send_text(incoming, &format!("Failed to send QR image: {e}"))
                                    .await;
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        self.send_text(incoming, &format!("QR generation failed: {e}"))
                            .await;
                        return;
                    }
                }

                // Wait for pairing confirmation (up to 60s).
                let pair_timeout =
                    tokio::time::timeout(std::time::Duration::from_secs(60), done_rx.recv());

                match pair_timeout.await {
                    Ok(Some(true)) => {
                        self.send_text(incoming, "WhatsApp connected!").await;
                    }
                    _ => {
                        self.send_text(
                            incoming,
                            "WhatsApp pairing timed out. Try /whatsapp again.",
                        )
                        .await;
                    }
                }
            }
            _ => {
                self.send_text(incoming, "Failed to generate QR code. Try again.")
                    .await;
            }
        }
    }
}
