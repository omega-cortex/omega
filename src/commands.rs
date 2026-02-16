//! Built-in bot commands — instant responses, no provider call.

use omega_memory::Store;
use std::time::Instant;

/// Known bot commands.
pub enum Command {
    Status,
    Memory,
    History,
    Facts,
    Forget,
    Help,
}

impl Command {
    /// Parse a command from message text. Returns `None` for unknown `/` prefixes
    /// (which should pass through to the provider).
    pub fn parse(text: &str) -> Option<Self> {
        let cmd = text.split_whitespace().next()?;
        match cmd {
            "/status" => Some(Self::Status),
            "/memory" => Some(Self::Memory),
            "/history" => Some(Self::History),
            "/facts" => Some(Self::Facts),
            "/forget" => Some(Self::Forget),
            "/help" => Some(Self::Help),
            _ => None,
        }
    }
}

/// Handle a command and return the response text.
pub async fn handle(
    cmd: Command,
    store: &Store,
    channel: &str,
    sender_id: &str,
    uptime: &Instant,
    provider_name: &str,
) -> String {
    match cmd {
        Command::Status => handle_status(store, uptime, provider_name).await,
        Command::Memory => handle_memory(store, sender_id).await,
        Command::History => handle_history(store, channel, sender_id).await,
        Command::Facts => handle_facts(store, sender_id).await,
        Command::Forget => handle_forget(store, channel, sender_id).await,
        Command::Help => handle_help(),
    }
}

async fn handle_status(store: &Store, uptime: &Instant, provider_name: &str) -> String {
    let elapsed = uptime.elapsed();
    let hours = elapsed.as_secs() / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let secs = elapsed.as_secs() % 60;

    let db_size = store
        .db_size()
        .await
        .map(format_bytes)
        .unwrap_or_else(|_| "unknown".to_string());

    format!(
        "Omega Status\n\
         Uptime: {hours}h {minutes}m {secs}s\n\
         Provider: {provider_name}\n\
         Database: {db_size}"
    )
}

async fn handle_memory(store: &Store, sender_id: &str) -> String {
    match store.get_memory_stats(sender_id).await {
        Ok((convos, msgs, facts)) => {
            format!(
                "Your Memory\n\
                 Conversations: {convos}\n\
                 Messages: {msgs}\n\
                 Facts: {facts}"
            )
        }
        Err(e) => format!("Error: {e}"),
    }
}

async fn handle_history(store: &Store, channel: &str, sender_id: &str) -> String {
    match store.get_history(channel, sender_id, 5).await {
        Ok(entries) if entries.is_empty() => "No conversation history yet.".to_string(),
        Ok(entries) => {
            let mut out = String::from("Recent Conversations\n");
            for (summary, timestamp) in &entries {
                out.push_str(&format!("\n[{timestamp}]\n{summary}\n"));
            }
            out
        }
        Err(e) => format!("Error: {e}"),
    }
}

async fn handle_facts(store: &Store, sender_id: &str) -> String {
    match store.get_facts(sender_id).await {
        Ok(facts) if facts.is_empty() => "No facts stored yet.".to_string(),
        Ok(facts) => {
            let mut out = String::from("Known Facts\n");
            for (key, value) in &facts {
                out.push_str(&format!("\n- {key}: {value}"));
            }
            out
        }
        Err(e) => format!("Error: {e}"),
    }
}

async fn handle_forget(store: &Store, channel: &str, sender_id: &str) -> String {
    match store.close_current_conversation(channel, sender_id).await {
        Ok(true) => "Conversation cleared. Starting fresh.".to_string(),
        Ok(false) => "No active conversation to clear.".to_string(),
        Err(e) => format!("Error: {e}"),
    }
}

fn handle_help() -> String {
    "\
Omega Commands\n\n\
/status  — Uptime, provider, database info\n\
/memory  — Your conversation and facts stats\n\
/history — Last 5 conversation summaries\n\
/facts   — List known facts about you\n\
/forget  — Clear current conversation\n\
/help    — This message"
        .to_string()
}

/// Format bytes into a human-readable string.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
