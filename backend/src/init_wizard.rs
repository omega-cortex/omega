//! Interactive-only helpers for the init wizard (auth, WhatsApp).
//! Google Workspace setup lives in `init_google.rs`.

use crate::init_style;
use omega_channels::whatsapp;
use omega_core::shellexpand;
use std::path::Path;

/// Probe whether the Claude CLI has valid authentication.
///
/// Runs a minimal `claude -p` invocation. If the CLI has credentials the
/// command succeeds (exit 0); otherwise it fails fast before any network call.
fn is_claude_authenticated() -> bool {
    std::process::Command::new("claude")
        .args(["-p", "ok", "--output-format", "json", "--max-turns", "1"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run Anthropic authentication setup.
///
/// Probes Claude CLI authentication first. If credentials are already valid the
/// step is auto-completed. Otherwise the user is guided through the setup-token
/// flow (or may skip and authenticate later).
pub(crate) fn run_anthropic_auth() -> anyhow::Result<()> {
    let spinner = cliclack::spinner();
    spinner.start("Checking Anthropic authentication...");
    let authenticated = is_claude_authenticated();

    if authenticated {
        spinner.stop("Anthropic authentication — already configured");
        return Ok(());
    }

    spinner.stop("Anthropic authentication — not detected");

    let auth_method: &str = cliclack::select("Anthropic auth method")
        .item(
            "setup-token",
            "Paste setup-token (Recommended)",
            "Run `claude setup-token` elsewhere, then paste the token here",
        )
        .item(
            "skip",
            "Skip for now",
            "Authenticate later with: claude login or claude setup-token",
        )
        .interact()?;

    if auth_method == "skip" {
        init_style::omega_warning(
            "You can authenticate later with: claude login or claude setup-token",
        )?;
        return Ok(());
    }

    init_style::omega_note(
        "Anthropic setup-token",
        "Run `claude setup-token` in your terminal.\nThen paste the generated token below.",
    )?;

    let token: String = cliclack::input("Paste Anthropic setup-token")
        .placeholder("Paste the token here")
        .validate(|input: &String| {
            if input.trim().is_empty() {
                return Err("Token is required");
            }
            Ok(())
        })
        .interact()?;

    let spinner = cliclack::spinner();
    spinner.start("Applying setup-token...");

    let result = std::process::Command::new("claude")
        .args(["setup-token", token.trim()])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            spinner.stop("Anthropic authentication — configured");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            spinner.error(format!("setup-token failed: {stderr}"));
            init_style::omega_warning("You can authenticate later with: claude setup-token")?;
        }
        Err(e) => {
            spinner.error(format!("Failed to run claude: {e}"));
            init_style::omega_warning("You can authenticate later with: claude setup-token")?;
        }
    }

    Ok(())
}

/// Check if a WhatsApp session already exists.
fn whatsapp_already_paired() -> bool {
    let dir = shellexpand("~/.omega/whatsapp_session");
    Path::new(&dir).join("whatsapp.db").exists()
}

/// Run WhatsApp pairing as part of the init wizard.
///
/// Returns `true` if WhatsApp was successfully paired.
pub(crate) async fn run_whatsapp_setup() -> anyhow::Result<bool> {
    // If already paired, skip the QR flow.
    if whatsapp_already_paired() {
        init_style::omega_success("WhatsApp — already paired")?;
        return Ok(true);
    }

    let connect: bool = cliclack::confirm("Connect WhatsApp?")
        .initial_value(false)
        .interact()?;

    if !connect {
        return Ok(false);
    }

    init_style::omega_step("Starting WhatsApp pairing...")?;
    init_style::omega_info("Open WhatsApp on your phone > Linked Devices > Link a Device")?;

    let result = async {
        let (mut qr_rx, mut done_rx) = whatsapp::start_pairing("~/.omega").await?;

        // Wait for the first QR code.
        let qr_data = tokio::time::timeout(std::time::Duration::from_secs(30), qr_rx.recv())
            .await
            .map_err(|_| anyhow::anyhow!("timed out waiting for QR code"))?
            .ok_or_else(|| anyhow::anyhow!("QR channel closed"))?;

        // Render QR in terminal.
        let qr_text = whatsapp::generate_qr_terminal(&qr_data)?;
        // Display QR code inside a cliclack note.
        init_style::omega_note("Scan this QR code with WhatsApp", &qr_text)?;

        let spinner = cliclack::spinner();
        spinner.start("Waiting for scan...");

        // Wait for pairing confirmation.
        let paired = tokio::time::timeout(std::time::Duration::from_secs(60), done_rx.recv())
            .await
            .map_err(|_| anyhow::anyhow!("pairing timed out"))?
            .unwrap_or(false);

        if paired {
            spinner.stop("WhatsApp linked successfully");
        } else {
            spinner.error("Pairing did not complete");
        }

        Ok::<bool, anyhow::Error>(paired)
    }
    .await;

    match result {
        Ok(true) => Ok(true),
        Ok(false) => {
            init_style::omega_warning("You can try again later with /whatsapp.")?;
            Ok(false)
        }
        Err(e) => {
            init_style::omega_error(&format!("{e} — you can try again later with /whatsapp."))?;
            Ok(false)
        }
    }
}
