//! Authentication checks and WhatsApp QR pairing flow.

use super::Gateway;
use omega_channels::whatsapp;
use omega_core::config::ChannelConfig;
use omega_core::message::IncomingMessage;
use tracing::warn;

/// Core auth logic — pure function operating on config, testable without a full Gateway.
fn check_auth_inner(channel_config: &ChannelConfig, incoming: &IncomingMessage) -> Option<String> {
    match incoming.channel.as_str() {
        "telegram" => {
            let allowed = channel_config.telegram.as_ref().map(|tg| &tg.allowed_users);

            match allowed {
                Some(users) if users.is_empty() => {
                    // Empty list with auth enabled = deny all.
                    Some("no users configured in telegram allowed_users".to_string())
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
            let allowed = channel_config.whatsapp.as_ref().map(|wa| &wa.allowed_users);

            match allowed {
                Some(users) if users.is_empty() => {
                    // Empty list = allow all. WhatsApp has no upfront user IDs —
                    // the phone number is only known after first message.
                    None
                }
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

impl Gateway {
    /// Check if an incoming message is authorized.
    /// Returns `None` if allowed, `Some(reason)` if denied.
    pub(super) fn check_auth(&self, incoming: &IncomingMessage) -> Option<String> {
        check_auth_inner(&self.channel_config, incoming)
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

#[cfg(test)]
mod tests {
    use super::*;
    use omega_core::config::{TelegramConfig, WhatsAppConfig};

    /// Build a minimal `IncomingMessage` for testing auth.
    fn msg(channel: &str, sender_id: &str) -> IncomingMessage {
        IncomingMessage {
            id: uuid::Uuid::new_v4(),
            channel: channel.to_string(),
            sender_id: sender_id.to_string(),
            sender_name: None,
            text: String::new(),
            timestamp: chrono::Utc::now(),
            reply_to: None,
            attachments: vec![],
            reply_target: None,
            is_group: false,
            source: None,
        }
    }

    #[test]
    fn telegram_valid_user_allowed() {
        let config = ChannelConfig {
            telegram: Some(TelegramConfig {
                enabled: true,
                bot_token: String::new(),
                allowed_users: vec![12345],
                whisper_api_key: None,
            }),
            whatsapp: None,
        };
        let result = check_auth_inner(&config, &msg("telegram", "12345"));
        assert!(result.is_none(), "Valid telegram user should be allowed");
    }

    #[test]
    fn telegram_invalid_user_denied() {
        let config = ChannelConfig {
            telegram: Some(TelegramConfig {
                enabled: true,
                bot_token: String::new(),
                allowed_users: vec![12345],
                whisper_api_key: None,
            }),
            whatsapp: None,
        };
        let result = check_auth_inner(&config, &msg("telegram", "99999"));
        assert!(result.is_some(), "Invalid telegram user should be denied");
        assert!(result.unwrap().contains("not in allowed_users"));
    }

    #[test]
    fn telegram_empty_allowed_users_denied() {
        let config = ChannelConfig {
            telegram: Some(TelegramConfig {
                enabled: true,
                bot_token: String::new(),
                allowed_users: vec![],
                whisper_api_key: None,
            }),
            whatsapp: None,
        };
        let result = check_auth_inner(&config, &msg("telegram", "12345"));
        assert!(
            result.is_some(),
            "Empty allowed_users should deny all telegram users"
        );
        assert!(result.unwrap().contains("no users configured"));
    }

    #[test]
    fn whatsapp_valid_user_allowed() {
        let config = ChannelConfig {
            telegram: None,
            whatsapp: Some(WhatsAppConfig {
                enabled: true,
                allowed_users: vec!["5511999887766".to_string()],
                whisper_api_key: None,
            }),
        };
        let result = check_auth_inner(&config, &msg("whatsapp", "5511999887766"));
        assert!(result.is_none(), "Valid whatsapp user should be allowed");
    }

    #[test]
    fn whatsapp_empty_allowed_users_allows_all() {
        let config = ChannelConfig {
            telegram: None,
            whatsapp: Some(WhatsAppConfig {
                enabled: true,
                allowed_users: vec![],
                whisper_api_key: None,
            }),
        };
        let result = check_auth_inner(&config, &msg("whatsapp", "5511999887766"));
        assert!(
            result.is_none(),
            "Empty allowed_users should allow all whatsapp users"
        );
    }

    #[test]
    fn unknown_channel_denied() {
        let config = ChannelConfig::default();
        let result = check_auth_inner(&config, &msg("discord", "user123"));
        assert!(result.is_some(), "Unknown channel should be denied");
        assert!(result.unwrap().contains("unknown channel"));
    }
}
