# Init Wizard Helpers (backend/src/init_wizard.rs)

## Overview

Interactive-only helpers extracted from `init.rs` to keep the init module under the 500-line limit. Contains browser detection, Anthropic authentication, and WhatsApp QR pairing -- all of which require `cliclack` interactive prompts and are **not used** in non-interactive mode. Google Workspace OAuth setup has moved to `backend/src/init_google.rs`.

**Called by:** `backend/src/init.rs` (interactive wizard path only)

## Functions

### `detect_private_browsers() -> Vec<usize>`

Detects installed browsers on macOS that support incognito/private mode. Checks `/Applications/<browser>.app` for Chrome, Brave, Firefox, and Edge. Returns indices into the `PRIVATE_BROWSERS` constant.

Used to offer users an incognito browser option during Google OAuth (avoids cached session issues).

### `create_incognito_script(browser) -> Result<PathBuf>`

Creates a temporary shell script at `/tmp/omega_incognito_browser.sh` that opens a URL in the selected browser's incognito/private mode. Written with `0o700` permissions (TOCTOU-safe). The script is set as `$BROWSER` env var when launching the OAuth flow.

### `is_claude_authenticated() -> bool` (private)

Probes Claude CLI authentication by running `claude -p "ok" --output-format json --max-turns 1`. Returns `true` if the command exits successfully (credentials are valid), `false` otherwise. Stdout/stderr are suppressed.

### `run_anthropic_auth() -> Result<()>`

Interactive Anthropic authentication with auto-detection:
1. Probes auth via `is_claude_authenticated()` (shown as spinner)
2. If already authenticated -- confirms and returns immediately
3. If not authenticated -- offers two choices:
   - **Paste setup-token (Recommended)** -- prompts for token, runs `claude setup-token <token>`, reports result
   - **Skip for now** -- warns user about post-init auth options (`claude login` or `claude setup-token`)

### `run_whatsapp_setup() -> Result<bool>`

WhatsApp QR pairing flow:
1. If already paired (`~/.omega/whatsapp_session/whatsapp.db` exists), reports success and returns `true`
2. Asks user if they want to connect WhatsApp
3. Starts pairing bot, waits up to 30s for QR code
4. Renders QR in terminal via `cliclack::note()`
5. Waits up to 60s for scan completion
6. Returns `true` on success, `false` on failure/decline

### `run_google_setup() -> Result<Option<String>>` (moved to `init_google.rs`)

Google Workspace OAuth setup has moved to `backend/src/init_google.rs`. It uses the `omg-gog` CLI tool:
1. Checks if `omg-gog` is installed (offers install if missing)
2. Walks user through GCP project creation
3. Collects GCP Project ID (validated: non-empty, no spaces, no slashes)
4. Generates direct one-click GCP Console links via **multiselect with Enter-fallback**:
   - Shows bold hint: "Space to select multiple, or just press Enter to go one by one"
   - Presents `cliclack::multiselect` with all 14 APIs (`.required(false)`)
   - If user selects with Space: shows links only for selected APIs
   - If user presses Enter (empty): walks through each API with `cliclack::confirm` (default true)
   - 14 APIs: Gmail, Calendar, Drive, Docs, Sheets, Slides, Forms, Chat, Classroom, Tasks, People, CloudIdentity, Keep, Apps Script
   - OAuth consent screen link
   - OAuth client creation link
   - App publish link
5. Collects `client_secret.json` (paste or file path)
6. Runs `omg-gog auth credentials <path>`
7. Runs OAuth flow via `omg-gog auth add --web --force-consent` with piped I/O
8. Verifies with `omg-gog auth list`
9. Returns `Some(email)` on success, `None` on failure/decline

## Data Structures

### `PrivateBrowser`
```rust
pub(crate) struct PrivateBrowser {
    pub label: &'static str,  // Display name (e.g., "Google Chrome")
    pub app: &'static str,    // macOS app name (e.g., "Google Chrome")
    pub flag: &'static str,   // Incognito flag (e.g., "--incognito")
}
```

### `PRIVATE_BROWSERS` (const)
Chrome, Brave, Firefox, Edge -- with their respective incognito/private flags.

## Visibility

All exports are `pub(crate)` -- only accessible within the binary crate, not exposed as public API.
