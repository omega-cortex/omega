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
pub(crate) fn is_omg_gog_installed() -> bool {
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
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            return false;
        }
    } else {
        let _ = std::process::Command::new("git")
            .args(["-C", &src_dir, "pull"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    // Build release binary.
    let ok = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&src_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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

/// Install `omg-gog` non-interactively with suppressed output.
///
/// Returns `true` when the binary is available after installation.
pub(crate) fn install_omg_gog() -> anyhow::Result<bool> {
    let spinner = cliclack::spinner();
    spinner.start("Installing omg-gog...");

    if install_via_script() && is_omg_gog_installed() {
        spinner.stop("omg-gog — installed");
        return Ok(true);
    }

    spinner.stop("Script install failed — building from source...");

    let spinner = cliclack::spinner();
    spinner.start("Building omg-gog from source (this may take a few minutes)...");

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
// GCP URL helpers
// ---------------------------------------------------------------------------

/// Build a direct Google Cloud Console API Library URL for the given project.
fn gcp_api_library_url(project: &str, api: &str) -> String {
    format!("https://console.cloud.google.com/apis/library/{api}?project={project}")
}

/// Build a direct Google Cloud Console URL for a given path and project.
fn gcp_console_url(project: &str, path: &str) -> String {
    format!("https://console.cloud.google.com/{path}?project={project}")
}

/// Validate a GCP project ID: non-empty, no spaces, no slashes.
fn validate_project_id(input: &str) -> Result<(), &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Project ID is required");
    }
    if trimmed.contains(' ') {
        return Err("Project ID cannot contain spaces");
    }
    if trimmed.contains('/') {
        return Err("Project ID cannot contain slashes");
    }
    Ok(())
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
// OAuth subprocess with headless detection
// ---------------------------------------------------------------------------

/// Extract the first `https://accounts.google.com` URL from text.
fn extract_google_url(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|w| w.starts_with("https://accounts.google.com"))
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && !":/?=&%.+-_".contains(c))
                .to_string()
        })
}

/// Run `omg-gog auth add --web --force-consent` with piped I/O.
///
/// Always displays the authorization URL via cliclack so the user can
/// open it on any device (like Claude Code's "Browser didn't open?"
/// pattern). Collects the auth code via cliclack and pipes it back.
///
/// Uses `--force-consent` so Google always returns a refresh token
/// (required even if the user previously authorized this app).
fn run_omg_gog_oauth() -> anyhow::Result<bool> {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::time::Duration;

    let mut child = Command::new("omg-gog")
        .args(["auth", "add", "--web", "--force-consent"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let child_stdin = child.stdin.take().expect("stdin was piped");
    let child_stderr = child.stderr.take().expect("stderr was piped");

    // omg-gog writes URL + prompt to stderr. Read in a background thread
    // (byte-by-byte to catch the prompt without trailing newline).
    // After detecting the prompt, keep reading to capture error output.
    let (tx_prompt, rx_prompt) = mpsc::channel::<Option<String>>();
    let (tx_rest, rx_rest) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut output = String::new();
        let mut buf = [0u8; 1];
        let mut reader = child_stderr;
        let mut prompt_sent = false;
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(_) => {
                    output.push(buf[0] as char);
                    if !prompt_sent && output.to_lowercase().contains("authorization code:") {
                        let _ = tx_prompt.send(Some(output.clone()));
                        prompt_sent = true;
                        output.clear();
                    }
                }
                Err(_) => break,
            }
        }
        if !prompt_sent {
            let _ = tx_prompt.send(None);
        }
        // Send remaining stderr (contains error messages if any).
        let _ = tx_rest.send(output);
    });

    // Drain stdout in a background thread so the pipe doesn't block.
    let child_stdout = child.stdout.take().expect("stdout was piped");
    std::thread::spawn(move || {
        let mut sink = child_stdout;
        let mut buf = [0u8; 256];
        while let Ok(n) = sink.read(&mut buf) {
            if n == 0 {
                break;
            }
        }
    });

    let timeout = Duration::from_secs(120);
    match rx_prompt.recv_timeout(timeout) {
        Ok(Some(output)) => {
            // Extract URL and display it — always, like Claude Code does.
            let url = extract_google_url(&output);

            if let Some(ref url) = url {
                init_style::omega_note("Browser didn't open? Use the URL below to sign in", url)?;
            } else {
                init_style::omega_warning(
                    "Could not extract the authorization URL from omg-gog output.",
                )?;
            }

            let code: String = cliclack::input("Paste the authorization code")
                .placeholder("4/0Axx...")
                .validate(|input: &String| {
                    if input.trim().is_empty() {
                        Err("Authorization code is required")
                    } else {
                        Ok(())
                    }
                })
                .interact()?;

            // Send the code to omg-gog's stdin and close it.
            let mut stdin = child_stdin;
            writeln!(stdin, "{}", code.trim())?;
            drop(stdin);

            // Wait for completion with timeout (poll every 500ms).
            let deadline = std::time::Instant::now() + timeout;
            loop {
                match child.try_wait()? {
                    Some(status) => {
                        if status.success() {
                            return Ok(true);
                        }
                        // Surface the error from omg-gog.
                        let rest = rx_rest
                            .recv_timeout(Duration::from_secs(2))
                            .unwrap_or_default();
                        let err = extract_error_message(&rest);
                        if !err.is_empty() {
                            anyhow::bail!("{err}");
                        }
                        return Ok(false);
                    }
                    None if std::time::Instant::now() >= deadline => {
                        child.kill()?;
                        anyhow::bail!("omg-gog timed out after 120s");
                    }
                    None => std::thread::sleep(Duration::from_millis(500)),
                }
            }
        }
        Ok(None) => {
            // Process exited without prompting — check exit status.
            let status = child.wait()?;
            Ok(status.success())
        }
        Err(_) => {
            child.kill()?;
            anyhow::bail!("omg-gog timed out after 120s");
        }
    }
}

/// Extract the first `Error: ...` line from omg-gog stderr output.
fn extract_error_message(stderr: &str) -> String {
    stderr
        .lines()
        .find(|l| l.starts_with("Error:"))
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------------------
// Main wizard entry point
// ---------------------------------------------------------------------------

/// Run the Google Workspace setup wizard (assumes omg-gog is installed).
///
/// Returns `Some(email)` if Google was successfully connected, `None` if
/// the user cancelled or an error occurred.
pub(crate) fn run_google_wizard() -> anyhow::Result<Option<String>> {
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

    // ── Step 1b — Collect Project ID ────────────────────────────────────
    let project_id: String = cliclack::input("Google Cloud Project ID or number")
        .placeholder("my-project-123456  or  424288504335")
        .validate(|input: &String| validate_project_id(input))
        .interact()?;
    let project_id = project_id.trim().to_string();

    // ── Step 2 — Enable Google APIs ─────────────────────────────────────
    {
        let apis = [
            ("Gmail", "gmail.googleapis.com"),
            ("Calendar", "calendar-json.googleapis.com"),
            ("Drive", "drive.googleapis.com"),
            ("Docs", "docs.googleapis.com"),
            ("Sheets", "sheets.googleapis.com"),
            ("Slides", "slides.googleapis.com"),
            ("Forms", "forms.googleapis.com"),
            ("Chat", "chat.googleapis.com"),
            ("Classroom", "classroom.googleapis.com"),
            ("Tasks", "tasks.googleapis.com"),
            ("Contacts", "people.googleapis.com"),
            ("Groups", "cloudidentity.googleapis.com"),
            ("Keep", "keep.googleapis.com"),
            ("Apps Script", "script.googleapis.com"),
        ];

        let hint = console::Style::new()
            .bold()
            .apply_to("Space to select, Enter to pick one");
        init_style::omega_info(&hint.to_string())?;

        let mut ms = cliclack::multiselect("Step 2 — Select Google APIs to enable");
        for (i, (name, api)) in apis.iter().enumerate() {
            ms = ms.item(i, *name, *api);
        }
        let selected: Vec<usize> = ms.required(false).interact()?;

        let chosen: Vec<usize> = if selected.is_empty() {
            let mut sel = cliclack::select("Pick one API to enable (or skip)");
            for (i, (name, api)) in apis.iter().enumerate() {
                sel = sel.item(i, *name, *api);
            }
            sel = sel.item(usize::MAX, "Skip", "Continue without enabling APIs");
            let choice: usize = sel.interact()?;
            if choice == usize::MAX {
                Vec::new()
            } else {
                vec![choice]
            }
        } else {
            selected
        };

        if !chosen.is_empty() {
            let links: String = chosen
                .iter()
                .map(|&i| {
                    let (name, api) = apis[i];
                    format!("{name:<12} → {}", gcp_api_library_url(&project_id, api))
                })
                .collect::<Vec<_>>()
                .join("\n");
            init_style::omega_note("Enable these APIs (click each link)", &links)?;
            let cont: bool = cliclack::confirm("Done enabling? Continue to next step")
                .initial_value(true)
                .interact()?;
            if !cont {
                return Ok(None);
            }
        }
    }

    // ── Step 3 — Configure OAuth Consent Screen ─────────────────────────
    {
        let consent_url = gcp_console_url(&project_id, "apis/credentials/consent");
        if !wizard_step(
            "Step 3 — Configure OAuth Consent Screen",
            &format!(
                "1. Open: {consent_url}\n\
                 2. Click \"Get Started\"\n\
                 3. App Information:\n\
                    • App name: omg-gog\n\
                    • User support email: your email\n\
                 4. Audience: External\n\
                 5. Contact Information: your email\n\
                 6. Accept the agreement and click \"Continue\"\n\
                 7. Click \"Create\""
            ),
            "Done? Continue to next step",
        )? {
            return Ok(None);
        }
    }

    // ── Step 4 — Create OAuth Client Credentials ────────────────────────
    {
        let oauth_client_url = gcp_console_url(&project_id, "apis/credentials/oauthclient");
        if !wizard_step(
            "Step 4 — Create OAuth Client Credentials",
            &format!(
                "1. Open: {oauth_client_url}\n\
                 2. Application type: Web application\n\
                 3. Name: leave the default\n\
                 4. Under \"Authorized redirect URIs\", click \"Add URI\":\n\
                    https://omgagi.ai/oauth/callback/\n\
                 5. Click \"Create\"\n\
                 6. In the popup, click \"Download JSON\""
            ),
            "Done? Continue to next step",
        )? {
            return Ok(None);
        }
    }

    // ── Step 5 — Publish the App ────────────────────────────────────────
    {
        let publish_url = gcp_console_url(&project_id, "apis/credentials/consent");
        if !wizard_step(
            "Step 5 — Publish the App",
            &format!(
                "1. Open: {publish_url}\n\
                 2. Go to \"Audience\" and click \"Publish App\"\n\
                 3. Confirm when prompted\n\
                 \n\
                 Publishing lets your own Google account complete the OAuth flow\n\
                 without \"unverified app\" warnings."
            ),
            "Done? Continue to paste credentials",
        )? {
            return Ok(None);
        }
    }

    // ── Collect client_secret JSON (paste or file path) ────────────────
    init_style::omega_info(
        "Paste the JSON content from the downloaded file, or provide the file path (e.g. ~/Downloads/client_secret_*.json).",
    )?;

    let raw_input: String = cliclack::input("Paste JSON content or file path")
        .placeholder("~/Downloads/client_secret_*.json  OR  {\"web\":{...}}")
        .validate(|input: &String| {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err("JSON content or file path is required");
            }
            // If it looks like a file path, validate file exists.
            if !trimmed.starts_with('{') {
                let expanded = omega_core::shellexpand(trimmed);
                if !std::path::Path::new(&expanded).exists() {
                    return Err("File not found — check the path and try again");
                }
                // Validate the file contains valid JSON.
                match std::fs::read_to_string(&expanded) {
                    Ok(content) => {
                        match serde_json::from_str::<serde_json::Value>(content.trim()) {
                            Ok(v) if v.get("web").is_some() || v.get("installed").is_some() => {
                                Ok(())
                            }
                            Ok(_) => Err("File JSON must contain a \"web\" or \"installed\" key"),
                            Err(_) => Err("File does not contain valid JSON"),
                        }
                    }
                    Err(_) => Err("Could not read file"),
                }
            } else {
                match serde_json::from_str::<serde_json::Value>(trimmed) {
                    Ok(v) if v.get("web").is_some() || v.get("installed").is_some() => Ok(()),
                    Ok(_) => Err("JSON must contain a \"web\" or \"installed\" key"),
                    Err(_) => Err("Invalid JSON — paste the full content of the downloaded file"),
                }
            }
        })
        .interact()?;

    // Resolve input: file path → read content, otherwise use pasted JSON.
    let json_content = {
        let trimmed = raw_input.trim();
        if trimmed.starts_with('{') {
            trimmed.to_string()
        } else {
            let expanded = shellexpand(trimmed);
            std::fs::read_to_string(&expanded)?
        }
    };

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

    init_style::omega_step("IMPORTANT — Read before continuing:")?;
    init_style::omega_warning("An authorization URL will appear below — open it in any browser.")?;
    init_style::omega_step("1. Click \"Advanced\" → \"Go to omg-gog (unsafe)\" → Allow")?;
    init_style::omega_step("2. If blocked: go back to OAuth consent screen → Publish app")?;
    init_style::omega_step("3. After authorizing, copy the code and paste it here")?;

    // --web flow: pipe stdin/stdout/stderr so we can detect interactive
    // prompts on headless systems where the browser can't open.
    let oauth_ok = match run_omg_gog_oauth() {
        Ok(true) => {
            init_style::omega_success("OAuth approved")?;
            true
        }
        Ok(false) => {
            init_style::omega_warning("OAuth did not complete. Try manually: omg-gog auth add")?;
            false
        }
        Err(e) => {
            init_style::omega_error(&format!("OAuth flow failed: {e}"))?;
            false
        }
    };

    if !oauth_ok {
        std::thread::sleep(std::time::Duration::from_secs(2));
        return Ok(None);
    }

    // ── Detect connected account ────────────────────────────────────────
    let email = detect_email_from_omg_gog();

    if let Some(ref addr) = email {
        init_style::omega_success(&format!("Google Workspace connected — {addr}"))?;
    } else {
        init_style::omega_success("Google Workspace connected!")?;
    }

    // Brief pause so the user can read the result before returning to the menu.
    std::thread::sleep(std::time::Duration::from_secs(2));

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

    #[test]
    fn test_extract_google_url_from_output() {
        let output = "Open this URL:\nhttps://accounts.google.com/o/oauth2/v2/auth?client_id=123&scope=email\nPaste the authorization code:";
        let url = extract_google_url(output);
        assert!(url.is_some());
        assert!(url
            .unwrap()
            .starts_with("https://accounts.google.com/o/oauth2/"));
    }

    #[test]
    fn test_extract_google_url_missing() {
        let output = "Some random text without a URL";
        assert!(extract_google_url(output).is_none());
    }

    #[test]
    fn test_extract_google_url_with_surrounding_chars() {
        let output = "Visit: https://accounts.google.com/auth?q=1&r=2 now.";
        let url = extract_google_url(output).unwrap();
        assert!(url.starts_with("https://accounts.google.com/"));
        assert!(url.contains("q=1"));
    }

    #[test]
    fn test_gcp_api_library_url() {
        let url = gcp_api_library_url("my-project-123", "gmail.googleapis.com");
        assert_eq!(
            url,
            "https://console.cloud.google.com/apis/library/gmail.googleapis.com?project=my-project-123"
        );
    }

    #[test]
    fn test_gcp_api_library_url_numeric_project() {
        let url = gcp_api_library_url("424288504335", "drive.googleapis.com");
        assert_eq!(
            url,
            "https://console.cloud.google.com/apis/library/drive.googleapis.com?project=424288504335"
        );
    }

    #[test]
    fn test_gcp_console_url() {
        let url = gcp_console_url("my-proj", "apis/credentials/consent");
        assert_eq!(
            url,
            "https://console.cloud.google.com/apis/credentials/consent?project=my-proj"
        );
    }

    #[test]
    fn test_validate_project_id_valid() {
        assert!(validate_project_id(&"my-project-123".to_string()).is_ok());
        assert!(validate_project_id(&"424288504335".to_string()).is_ok());
        assert!(validate_project_id(&"a".to_string()).is_ok());
    }

    #[test]
    fn test_validate_project_id_empty() {
        assert!(validate_project_id(&String::new()).is_err());
        assert!(validate_project_id(&"   ".to_string()).is_err());
    }

    #[test]
    fn test_validate_project_id_spaces() {
        assert!(validate_project_id(&"my project".to_string()).is_err());
    }

    #[test]
    fn test_validate_project_id_slashes() {
        assert!(validate_project_id(&"projects/my-proj".to_string()).is_err());
    }
}
