//! Interactive-only helpers for the init wizard (browser detection, auth, WhatsApp).
//! Google Workspace setup lives in `init_google.rs`.

use crate::init_style;
use omega_channels::whatsapp;
use omega_core::shellexpand;
use std::path::Path;

/// Browser that supports private/incognito mode from the command line.
pub(crate) struct PrivateBrowser {
    pub label: &'static str,
    pub app: &'static str,
    pub flag: &'static str,
}

/// Known browsers with incognito/private mode support on macOS.
pub(crate) const PRIVATE_BROWSERS: &[PrivateBrowser] = &[
    PrivateBrowser {
        label: "Google Chrome",
        app: "Google Chrome",
        flag: "--incognito",
    },
    PrivateBrowser {
        label: "Brave",
        app: "Brave Browser",
        flag: "--incognito",
    },
    PrivateBrowser {
        label: "Firefox",
        app: "Firefox",
        flag: "--private-window",
    },
    PrivateBrowser {
        label: "Microsoft Edge",
        app: "Microsoft Edge",
        flag: "--inprivate",
    },
];

/// Detect installed browsers that support incognito/private mode (macOS).
///
/// Returns indices into `PRIVATE_BROWSERS` for browsers found in `/Applications`.
pub(crate) fn detect_private_browsers() -> Vec<usize> {
    PRIVATE_BROWSERS
        .iter()
        .enumerate()
        .filter(|(_, b)| Path::new(&format!("/Applications/{}.app", b.app)).exists())
        .map(|(i, _)| i)
        .collect()
}

/// Create a temporary shell script that opens a URL in incognito/private mode.
///
/// Returns the path to the script on success.
pub(crate) fn create_incognito_script(
    browser: &PrivateBrowser,
) -> anyhow::Result<std::path::PathBuf> {
    let script_path = std::env::temp_dir().join("omega_incognito_browser.sh");
    let script = format!(
        "#!/bin/sh\nopen -na '{}' --args {} \"$1\"\n",
        browser.app, browser.flag
    );
    // Create with restricted permissions first (0o700), then write content.
    // Prevents TOCTOU: no window where the file is world-readable.
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o700)
            .open(&script_path)?;
        f.write_all(script.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&script_path, script)?;
    }
    Ok(script_path)
}

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

// Google Workspace setup moved to `init_google.rs`.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_browsers_constant_has_entries() {
        assert!(
            !PRIVATE_BROWSERS.is_empty(),
            "should have at least one browser defined"
        );
        for b in PRIVATE_BROWSERS {
            assert!(!b.label.is_empty(), "label must not be empty");
            assert!(!b.app.is_empty(), "app must not be empty");
            assert!(!b.flag.is_empty(), "flag must not be empty");
        }
    }

    #[test]
    fn test_detect_private_browsers_returns_valid_indices() {
        let indices = detect_private_browsers();
        for &idx in &indices {
            assert!(
                idx < PRIVATE_BROWSERS.len(),
                "index {idx} out of bounds for PRIVATE_BROWSERS"
            );
        }
    }

    #[test]
    fn test_create_incognito_script() {
        let browser = &PRIVATE_BROWSERS[0]; // Google Chrome
        let path = create_incognito_script(browser).expect("should create script");
        assert!(path.exists(), "script file should exist");

        let content = std::fs::read_to_string(&path).expect("should read script");
        assert!(content.starts_with("#!/bin/sh\n"), "should have shebang");
        assert!(
            content.contains(browser.app),
            "should contain browser app name"
        );
        assert!(
            content.contains(browser.flag),
            "should contain browser flag"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&path)
                .expect("should get metadata")
                .permissions();
            assert_eq!(perms.mode() & 0o700, 0o700, "script should be executable");
        }

        // Cleanup.
        let _ = std::fs::remove_file(path);
    }
}
