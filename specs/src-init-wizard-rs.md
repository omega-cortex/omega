# backend/src/init_wizard.rs — Interactive Init Helpers Specification

## Path
`backend/src/init_wizard.rs`

## Purpose
Interactive-only helpers extracted from `init.rs` to keep the init module under the 500-line limit. Contains browser detection, Anthropic authentication, WhatsApp QR pairing, and Google Workspace OAuth setup — all of which require cliclack interactive prompts and are not used in non-interactive mode.

## Module Overview
- `PrivateBrowser` — `pub(crate)` struct: browser label, app name, incognito flag
- `PRIVATE_BROWSERS` — `pub(crate)` const: Chrome, Brave, Firefox, Edge
- `detect_private_browsers()` — `pub(crate)` fn: returns indices of installed browsers in `/Applications`
- `create_incognito_script(browser)` — `pub(crate)` fn: writes temp shell script for incognito URL opening
- `run_anthropic_auth()` — `pub(crate)` fn: interactive Anthropic auth (already-authed or setup-token)
- `run_whatsapp_setup()` — `pub(crate) async` fn: WhatsApp QR pairing via cliclack
- `run_google_setup()` — `pub(crate)` fn: Google OAuth flow via `gog` CLI with incognito browser offer
- `whatsapp_already_paired()` — private helper

## Called By
- `backend/src/init.rs` — `run()` (interactive wizard path only)

## Unit Tests (3 tests)
| Test | Assertions |
|------|------------|
| `test_private_browsers_constant_has_entries` | PRIVATE_BROWSERS has entries with non-empty label/app/flag |
| `test_detect_private_browsers_returns_valid_indices` | All returned indices are within bounds |
| `test_create_incognito_script` | Script created, has shebang, contains app name and flag, is executable |

## Dependencies
- `cliclack` — Interactive prompts (select, input, confirm, spinner)
- `crate::init_style` — Branded chrome (note, success, warning, error, step, info)
- `omega_channels::whatsapp` — WhatsApp pairing
- `omega_core::shellexpand` — Home directory expansion
- `std::process::Command` — Subprocess execution (`claude`, `gog`)
- `tokio::time` — Timeout for async pairing flow
