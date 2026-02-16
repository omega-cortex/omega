//! Startup self-check — verify all components are operational before running.

use omega_core::config::Config;
use omega_memory::Store;

/// Result of a single check.
struct CheckResult {
    name: String,
    detail: String,
    ok: bool,
}

/// Run all startup checks. Returns true if all passed.
pub async fn run(config: &Config, store: &Store) -> bool {
    let mut results = Vec::new();

    // 1. Database check.
    results.push(check_database(store).await);

    // 2. Provider check.
    results.push(check_provider(config).await);

    // 3. Channel checks.
    if let Some(ref tg) = config.channel.telegram {
        if tg.enabled {
            results.push(check_telegram(tg).await);
        }
    }

    // Print results.
    println!("\nOmega Self-Check");
    println!("================");
    let mut all_ok = true;
    for r in &results {
        let icon = if r.ok { "+" } else { "x" };
        println!("  {icon} {} — {}", r.name, r.detail);
        if !r.ok {
            all_ok = false;
        }
    }
    println!();

    all_ok
}

async fn check_database(store: &Store) -> CheckResult {
    // Try a simple query to verify the database is accessible.
    let detail = match store.db_size().await {
        Ok(size) => {
            let size_str = if size < 1024 {
                format!("{size} B")
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            };
            format!("accessible ({size_str})")
        }
        Err(e) => {
            return CheckResult {
                name: "Database".to_string(),
                detail: format!("FAILED: {e}"),
                ok: false,
            }
        }
    };

    CheckResult {
        name: "Database".to_string(),
        detail,
        ok: true,
    }
}

async fn check_provider(config: &Config) -> CheckResult {
    match config.provider.default.as_str() {
        "claude-code" => {
            let available = omega_providers::claude_code::ClaudeCodeProvider::check_cli().await;
            CheckResult {
                name: "Provider".to_string(),
                detail: if available {
                    "claude-code (available)".to_string()
                } else {
                    "claude-code (NOT FOUND — install claude CLI)".to_string()
                },
                ok: available,
            }
        }
        other => CheckResult {
            name: "Provider".to_string(),
            detail: format!("{other} (unchecked)"),
            ok: true,
        },
    }
}

async fn check_telegram(tg: &omega_core::config::TelegramConfig) -> CheckResult {
    if tg.bot_token.is_empty() {
        return CheckResult {
            name: "Channel".to_string(),
            detail: "telegram (missing bot_token)".to_string(),
            ok: false,
        };
    }

    // Call getMe to verify the token.
    let url = format!("https://api.telegram.org/bot{}/getMe", tg.bot_token);
    let client = reqwest::Client::new();
    match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                // Try to extract bot username.
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let username = body["result"]["username"].as_str().unwrap_or("unknown");
                CheckResult {
                    name: "Channel".to_string(),
                    detail: format!("telegram (@{username})"),
                    ok: true,
                }
            } else {
                CheckResult {
                    name: "Channel".to_string(),
                    detail: format!("telegram (token invalid — HTTP {})", resp.status()),
                    ok: false,
                }
            }
        }
        Err(e) => CheckResult {
            name: "Channel".to_string(),
            detail: format!("telegram (network error: {e})"),
            ok: false,
        },
    }
}
