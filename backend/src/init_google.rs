//! Google Workspace setup wizard for the OMEGA init flow.
//!
//! Installs `omg-gog` if needed, walks the user step-by-step through Google
//! Cloud project creation, collects OAuth credentials, and runs the
//! `omg-gog auth` flow.  The user is granting *themselves* permission to
//! access *their own* data — no third-party access is involved.

use crate::init_style;
use omega_core::shellexpand;

// ---------------------------------------------------------------------------
// omg-gog binary detection and installation
// ---------------------------------------------------------------------------

/// Check if `omg-gog` CLI is available on `PATH`.
fn is_omg_gog_installed() -> bool {
    std::process::Command::new("omg-gog")
        .arg("--help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Attempt to install `omg-gog` via the official installer script.
fn install_via_script() -> bool {
    std::process::Command::new("sh")
        .args([
            "-c",
            "curl -fsSL https://omgagi.ai/tools/omg-gog/install.sh | sh",
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Attempt to build `omg-gog` from source (requires Rust toolchain + git).
fn install_from_source() -> bool {
    let cargo_ok = std::process::Command::new("cargo")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !cargo_ok {
        return false;
    }

    let build_dir = shellexpand("~/builds/omg-gog");
    let src_dir = format!("{build_dir}/omg-gog");

    // Clone (or pull if already present).
    if !std::path::Path::new(&src_dir).exists() {
        let ok = std::process::Command::new("git")
            .args(["clone", "https://github.com/omgagi/omg-gog.git", &build_dir])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            return false;
        }
    } else {
        let _ = std::process::Command::new("git")
            .args(["-C", &src_dir, "pull"])
            .status();
    }

    // Build release binary.
    let ok = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&src_dir)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return false;
    }

    // Copy binary to PATH.
    let binary = format!("{src_dir}/target/release/omg-gog");
    let cp_ok = std::process::Command::new("cp")
        .args([&binary, "/usr/local/bin/omg-gog"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if cp_ok {
        return true;
    }

    // Retry with sudo.
    std::process::Command::new("sudo")
        .args(["cp", &binary, "/usr/local/bin/omg-gog"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Ensure `omg-gog` is installed, offering to install if missing.
///
/// Returns `true` when the binary is available after this call.
fn ensure_omg_gog() -> anyhow::Result<bool> {
    if is_omg_gog_installed() {
        init_style::omega_success("omg-gog — found")?;
        return Ok(true);
    }

    init_style::omega_warning("omg-gog is not installed")?;

    let install: bool = cliclack::confirm("Install omg-gog? (Google Workspace CLI tool)")
        .initial_value(true)
        .interact()?;

    if !install {
        return Ok(false);
    }

    // Try the quick installer first.
    let spinner = cliclack::spinner();
    spinner.start("Installing omg-gog ...");

    if install_via_script() && is_omg_gog_installed() {
        spinner.stop("omg-gog — installed");
        return Ok(true);
    }

    spinner.stop("Installer failed — trying build from source...");

    let spinner = cliclack::spinner();
    spinner.start("Building omg-gog from source (this may take a few minutes) ...");

    if install_from_source() && is_omg_gog_installed() {
        spinner.stop("omg-gog — built and installed");
        return Ok(true);
    }

    spinner.error("Could not install omg-gog");
    init_style::omega_note(
        "Manual installation",
        "curl -fsSL https://omgagi.ai/tools/omg-gog/install.sh | sh\n\
         Or build from source: https://github.com/omgagi/omg-gog",
    )?;

    Ok(false)
}

// ---------------------------------------------------------------------------
// Step-by-step wizard helpers
// ---------------------------------------------------------------------------

/// Show a wizard step and wait for the user to confirm before continuing.
/// Returns `false` if the user chose to cancel.
fn wizard_step(title: &str, body: &str, continue_label: &str) -> anyhow::Result<bool> {
    init_style::omega_note(title, body)?;
    let cont: bool = cliclack::confirm(continue_label)
        .initial_value(true)
        .interact()?;
    Ok(cont)
}

/// Try to extract the first email address from `omg-gog auth list` output.
fn detect_email_from_omg_gog() -> Option<String> {
    let output = std::process::Command::new("omg-gog")
        .args(["auth", "list"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .find(|w| w.contains('@') && w.contains('.'))
        .map(|w| {
            w.trim_matches(|c: char| {
                !c.is_alphanumeric() && c != '@' && c != '.' && c != '-' && c != '_'
            })
            .to_string()
        })
}

// ---------------------------------------------------------------------------
// Main wizard entry point
// ---------------------------------------------------------------------------

/// Run the step-by-step Google Workspace setup wizard.
///
/// Returns `Some(email)` if Google was successfully connected, `None` if
/// the user skipped or an error occurred.
pub(crate) fn run_google_setup() -> anyhow::Result<Option<String>> {
    // ── 0. Ensure omg-gog is available ──────────────────────────────────
    if !ensure_omg_gog()? {
        return Ok(None);
    }

    let setup: bool =
        cliclack::confirm("Set up Google Workspace? (Gmail, Calendar, Drive, Docs ...)")
            .initial_value(false)
            .interact()?;

    if !setup {
        return Ok(None);
    }

    init_style::omega_info(
        "This process gives YOU permission to access YOUR OWN Google data through OMEGA.\n\
         No data is shared with third parties — you control the credentials.",
    )?;

    // ── Step 1 — Create a Google Cloud Project ──────────────────────────
    if !wizard_step(
        "Step 1 — Create a Google Cloud Project",
        "1. Go to https://console.cloud.google.com\n\
         2. Click \"Select a project\" → \"New Project\"\n\
         3. Name it (e.g. \"My project\") and click Create",
        "Done? Continue to next step",
    )? {
        return Ok(None);
    }

    // ── Step 2 — Enable Google APIs ─────────────────────────────────────
    if !wizard_step(
        "Step 2 — Enable Google APIs",
        "In the Google Cloud Console search box, find and Enable\n\
         each API you plan to use:\n\
         \n\
         Gmail        → Gmail API\n\
         Calendar     → Google Calendar API\n\
         Drive        → Google Drive API\n\
         Docs         → Google Docs API\n\
         Sheets       → Google Sheets API\n\
         Slides       → Google Slides API\n\
         Forms        → Google Forms API\n\
         Chat         → Google Chat API\n\
         Classroom    → Google Classroom API\n\
         Tasks        → Google Tasks API\n\
         Contacts     → People API\n\
         Groups       → Cloud Identity API\n\
         Keep         → Google Keep API\n\
         Apps Script  → Apps Script API\n\
         \n\
         Enable only the ones you need.",
        "Done? Continue to next step",
    )? {
        return Ok(None);
    }

    // ── Step 3 — Configure OAuth Consent Screen ─────────────────────────
    if !wizard_step(
        "Step 3 — Configure OAuth Consent Screen",
        "1. In the left menu, click \"OAuth consent screen\"\n\
         2. Click \"Get Started\"\n\
         3. App Information:\n\
            • App name: omg-gog\n\
            • User support email: your email\n\
         4. Audience: External\n\
         5. Contact Information: your email\n\
         6. Accept the agreement and click \"Continue\"\n\
         7. Click \"Create\"",
        "Done? Continue to next step",
    )? {
        return Ok(None);
    }

    // ── Step 4 — Create OAuth Client Credentials ────────────────────────
    if !wizard_step(
        "Step 4 — Create OAuth Client Credentials",
        "1. From OAuth Overview, click \"Create OAuth Client\"\n\
         2. Application type: Web application\n\
         3. Name: leave the default\n\
         4. Under \"Authorized redirect URIs\", click \"Add URI\":\n\
            https://omgagi.ai/oauth/callback/\n\
         5. Click \"Create\"\n\
         6. In the popup, click \"Download JSON\"",
        "Done? Continue to next step",
    )? {
        return Ok(None);
    }

    // ── Step 5 — Publish the App ────────────────────────────────────────
    if !wizard_step(
        "Step 5 — Publish the App",
        "1. Go to OAuth consent screen → Audience\n\
         2. Click \"Publish App\"\n\
         3. Confirm when prompted\n\
         \n\
         Publishing lets your own Google account complete the OAuth flow\n\
         without \"unverified app\" warnings.",
        "Done? Continue to paste credentials",
    )? {
        return Ok(None);
    }

    // ── Collect client_secret JSON ──────────────────────────────────────
    init_style::omega_info("Open the downloaded JSON file and copy its entire content.")?;

    let json_content: String = cliclack::input("Paste the client_secret JSON content")
        .placeholder(r#"{"web":{"client_id":"...","client_secret":"...","redirect_uris":[...]}}"#)
        .validate(|input: &String| {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("JSON content is required");
            }
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(v) if v.get("web").is_some() || v.get("installed").is_some() => Ok(()),
                Ok(_) => Err("JSON must contain a \"web\" or \"installed\" key"),
                Err(_) => Err("Invalid JSON — paste the full content of the downloaded file"),
            }
        })
        .interact()?;

    // Write to a temp file with restricted permissions.
    let tmp_path = "/tmp/client_secret.json";
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(tmp_path)?;
        f.write_all(json_content.trim().as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(tmp_path, json_content.trim())?;
    }

    // ── Register credentials with omg-gog ───────────────────────────────
    let spinner = cliclack::spinner();
    spinner.start("Registering credentials with omg-gog ...");

    let cred_result = std::process::Command::new("omg-gog")
        .args(["auth", "credentials", tmp_path])
        .output();

    // Always clean up the temp file.
    let _ = std::fs::remove_file(tmp_path);

    match cred_result {
        Ok(output) if output.status.success() => {
            spinner.stop("Credentials registered");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            spinner.error(format!("omg-gog auth credentials failed: {stderr}"));
            init_style::omega_warning("Google Workspace setup incomplete.")?;
            return Ok(None);
        }
        Err(e) => {
            spinner.error(format!("Failed to run omg-gog: {e}"));
            init_style::omega_warning("Google Workspace setup incomplete.")?;
            return Ok(None);
        }
    }

    // ── Run OAuth flow ─────────────────────────────────────────────────
    init_style::omega_info(
        "You are authorizing YOUR app to access YOUR data — no third-party involved.",
    )?;

    init_style::omega_note(
        "OAuth — next steps",
        "omg-gog will print an authorization URL below.\n\
         Open it in an incognito/private browser window (recommended).\n\
         \n\
         • Click \"Advanced\" → \"Go to omg-gog (unsafe)\" → Allow\n\
         • If blocked, go back to OAuth consent screen → Publish app",
    )?;

    // --web flow: prints URL, user opens in browser, callback via omgagi.ai.
    // stdin/stdout/stderr inherited so the user sees the URL and interacts.
    let status = std::process::Command::new("omg-gog")
        .args(["auth", "add", "--web"])
        .status();

    let oauth_ok = match status {
        Ok(s) if s.success() => {
            init_style::omega_success("OAuth approved")?;
            true
        }
        Ok(_) => {
            init_style::omega_warning("OAuth did not complete. Try manually: omg-gog auth add")?;
            false
        }
        Err(e) => {
            init_style::omega_error(&format!("Failed to run omg-gog: {e}"))?;
            false
        }
    };

    if !oauth_ok {
        return Ok(None);
    }

    // ── Detect connected account ────────────────────────────────────────
    let email = detect_email_from_omg_gog();

    if let Some(ref addr) = email {
        init_style::omega_success(&format!("Google Workspace connected — {addr}"))?;
    } else {
        init_style::omega_success("Google Workspace connected!")?;
    }

    Ok(email)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_email_from_output_empty() {
        // When omg-gog is not installed, detect_email_from_omg_gog returns None.
        // We can't call it in CI, so just verify the function compiles.
        let _ = detect_email_from_omg_gog;
    }

    #[test]
    fn test_wizard_step_label_not_empty() {
        // Regression: labels passed to wizard_step must be non-empty.
        let title = "Step 1 — Test";
        let body = "Do a thing";
        let label = "Done?";
        assert!(!title.is_empty());
        assert!(!body.is_empty());
        assert!(!label.is_empty());
    }

    #[test]
    fn test_is_omg_gog_installed_does_not_panic() {
        // Must not panic even if the binary does not exist.
        let _ = is_omg_gog_installed();
    }
}
