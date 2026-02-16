//! Init wizard — interactive 2-minute setup for new users.

use omega_core::shellexpand;
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Run the interactive init wizard.
pub fn run() -> anyhow::Result<()> {
    println!();
    println!("  Omega — Setup Wizard");
    println!("  ====================");
    println!();

    // 1. Create data directory.
    let data_dir = shellexpand("~/.omega");
    if !Path::new(&data_dir).exists() {
        std::fs::create_dir_all(&data_dir)?;
        println!("  Created {data_dir}");
    } else {
        println!("  {data_dir} already exists");
    }

    // 2. Check claude CLI.
    print!("  Checking claude CLI... ");
    io::stdout().flush()?;
    let claude_ok = std::process::Command::new("claude")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if claude_ok {
        println!("found");
    } else {
        println!("NOT FOUND");
        println!();
        println!("  Install claude CLI first:");
        println!("    npm install -g @anthropic-ai/claude-code");
        println!();
        println!("  Then run 'omega init' again.");
        return Ok(());
    }

    // 3. Telegram bot token.
    println!();
    println!("  Telegram Bot Setup");
    println!("  ------------------");
    println!("  Create a bot with @BotFather on Telegram, then paste the token.");
    println!();
    let bot_token = prompt("  Bot token: ")?;
    if bot_token.is_empty() {
        println!("  Skipping Telegram setup.");
        println!("  You can add it later in config.toml.");
    }

    // 4. User ID (optional).
    let user_id = if !bot_token.is_empty() {
        println!();
        println!("  Your Telegram user ID (send /start to @userinfobot to find it).");
        println!("  Leave blank to allow all users.");
        let id = prompt("  User ID: ")?;
        id.parse::<i64>().ok()
    } else {
        None
    };

    // 5. Generate config.toml.
    let config_path = "config.toml";
    if Path::new(config_path).exists() {
        println!();
        println!("  config.toml already exists — skipping generation.");
        println!("  Delete it and run 'omega init' again to regenerate.");
    } else {
        let allowed_users = match user_id {
            Some(id) => format!("[{id}]"),
            None => "[]".to_string(),
        };
        let telegram_enabled = if bot_token.is_empty() {
            "false"
        } else {
            "true"
        };

        let config = format!(
            r#"[omega]
name = "Omega"
data_dir = "~/.omega"
log_level = "info"

[auth]
enabled = true

[provider]
default = "claude-code"

[provider.claude-code]
enabled = true
max_turns = 10
allowed_tools = ["Bash", "Read", "Write", "Edit"]

[channel.telegram]
enabled = {telegram_enabled}
bot_token = "{bot_token}"
allowed_users = {allowed_users}

[memory]
backend = "sqlite"
db_path = "~/.omega/memory.db"
max_context_messages = 50
"#
        );

        std::fs::write(config_path, config)?;
        println!();
        println!("  Generated config.toml");
    }

    // 6. Summary and next steps.
    println!();
    println!("  Setup Complete");
    println!("  ==============");
    println!();
    println!("  Next steps:");
    println!("    1. Review config.toml");
    println!("    2. Run: omega start");
    println!("    3. Send a message to your bot on Telegram");
    println!();

    Ok(())
}

/// Read a line from stdin with a prompt.
fn prompt(msg: &str) -> anyhow::Result<String> {
    print!("{msg}");
    io::stdout().flush()?;
    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}
