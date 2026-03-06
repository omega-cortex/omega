# Wizard Flow Specification: OMEGA TUI Onboarding & Reconfiguration

## Overview
- **Purpose**: Guide new users through first-time OMEGA setup (`omega init`) and allow existing users to reconfigure individual components (`omega setup`)
- **Target Medium**: TUI (terminal) -- keyboard-only interaction via the `cliclack` Rust crate (v0.3)
- **Target Users**: Developers and technical users setting up a personal AI agent. Expected to be comfortable with terminals but not necessarily familiar with TUI multiselect conventions (Space-to-toggle)
- **Estimated Completion Time**: 1-3 minutes (init), 30 seconds per component (setup)
- **Total Steps**: 9 interactive steps in `omega init` (4 conditional), 1 selection + N component steps in `omega setup`

## Medium Constraints

**TUI via cliclack (Rust):**
- Screen width: 80-120 columns typical
- Input: keyboard only (arrow keys, Space, Enter, Escape, typed text)
- Available widgets: `input`, `confirm`, `select`, `multiselect`, `spinner`, `note` (via `init_style`), `progress_bar`
- cliclack multiselect: requires **Space** to toggle items, **Enter** to submit. This is the root cause of user confusion -- many users press Enter expecting it to select the highlighted item
- cliclack select: uses **arrow keys** to navigate, **Enter** to pick. No Space required. More intuitive for single-choice scenarios
- cliclack multiselect `required(false)`: allows submitting with zero selections (Enter with nothing toggled = empty `Vec`)
- No inline hint mechanism on multiselect/select widgets -- hints must be part of the prompt string or shown as a preceding line
- Branded chrome: `init_style` module provides styled output (gutter bars, colored markers)
- Works over SSH, on macOS Terminal, iTerm2, Linux terminals with ANSI color support

---

## Audit Findings

### Finding 1: Skip + Google Mutual Exclusion (P0 -- breaks flow)

**Location**: `init.rs` lines 138-148

**Current code:**
```rust
let selected_tools: Vec<&str> = cliclack::multiselect("Optional tools -- select to set up")
    .item("skip", "Skip", "Continue without setting up optional tools")
    .item("google", "Google Workspace", google_hint)
    .required(true)
    .interact()?;

let selected_tools: Vec<&str> = if selected_tools.contains(&"skip") {
    vec![]
} else {
    selected_tools
};
```

**Problem**: The multiselect allows selecting BOTH "Skip" AND "Google Workspace" simultaneously. The code silently resolves this (skip wins), but the UX is contradictory -- the user sees two items checked that are mutually exclusive. Additionally, `.required(true)` forces the user to select at least one item, which is why "Skip" was added as a selectable option in the first place.

**Root cause**: Multiselect is the wrong widget for a binary choice. There is currently only one optional tool (Google Workspace). A single-choice `select` or a yes/no `confirm` is the correct pattern.

**Fix**: Replace the multiselect with `cliclack::confirm`.

**After:**
```rust
let setup_google: bool = cliclack::confirm("Set up Google Workspace?")
    .initial_value(false)
    .interact()?;
```

**Rationale**: With only one optional tool, a confirm prompt is the simplest correct widget. It eliminates the mutual exclusion bug, removes the Space/Enter confusion entirely, and reduces cognitive load. If more optional tools are added in the future, switch to a multiselect with `required(false)` (no "Skip" item needed -- empty selection = skip).

**Alternative considered**: `cliclack::select` with "Google Workspace" / "Skip" items. Rejected because `confirm` is even simpler for a binary choice and uses the same Enter-key interaction the user expects.

---

### Finding 2: Space + Enter Confusion on Multiselect (P1 -- confusing)

**Location**: `init.rs` lines 133-136, `init.rs` lines 286-289

**Current code:**
```rust
let hint = console::Style::new()
    .bold()
    .apply_to("Space to select, Enter to confirm");
init_style::omega_info(&hint.to_string())?;
```

**Problem**: The hint "Space to select, Enter to confirm" is displayed as a separate `omega_info` line BEFORE the multiselect widget renders. By the time the user sees the multiselect prompt, the hint has scrolled up and may not be visible. Users unfamiliar with TUI conventions press Enter immediately (expecting it to select the highlighted item) and submit with no selections or with the wrong result.

**Root cause**: cliclack does not support inline hint text within the multiselect widget itself. The external hint is disconnected from the widget it describes.

**Fix (two-part)**:

Part A -- Embed the hint in the prompt string:
```rust
// Before:
cliclack::multiselect("Select components to reconfigure")

// After:
cliclack::multiselect("Select components to reconfigure\n  Space = toggle, Enter = confirm")
```

Part B -- For binary choices (Finding 1), eliminate multiselect entirely. The Space/Enter confusion only exists on multiselect widgets. Every binary choice should use `confirm` or `select`. Multiselect should only appear where multi-selection is genuinely needed (omega setup component picker, Google API picker).

---

### Finding 3: "Exit" as Multiselect Item in `omega setup` (P1 -- confusing)

**Location**: `init.rs` lines 298-306

**Current code:**
```rust
let selected: Vec<&str> = cliclack::multiselect("Select components to reconfigure")
    .item("claude", &lbl_claude, "OAuth token for Claude Code")
    .item("telegram", &lbl_telegram, "Bot token and allowed users")
    .item("whisper", &lbl_whisper, "OpenAI Whisper API key")
    .item("whatsapp", &lbl_whatsapp, "Pair via QR code")
    .item("google", &lbl_google, "Gmail, Calendar, Drive...")
    .item("service", &lbl_service, "Install or reinstall the service")
    .item("exit", "Exit", "Return to terminal")
    .interact()?;
```

**Problem**: "Exit" is semantically different from the other items -- it is an action, not a component to reconfigure. A user can select "Exit" + "Telegram" + "Google" simultaneously, which is contradictory. The code handles it (exit check wins), but the UX is confusing.

**Fix**: Remove "Exit" from the multiselect. Use `required(false)` so that submitting with no selections exits the loop.

**After:**
```rust
let selected: Vec<&str> =
    cliclack::multiselect("Select components to reconfigure\n  Space = toggle, Enter = confirm, Enter with none = exit")
        .item("claude", &lbl_claude, "OAuth token for Claude Code")
        .item("telegram", &lbl_telegram, "Bot token and allowed users")
        .item("whisper", &lbl_whisper, "OpenAI Whisper API key")
        .item("whatsapp", &lbl_whatsapp, "Pair via QR code")
        .item("google", &lbl_google, "Gmail, Calendar, Drive...")
        .item("service", &lbl_service, "Install or reinstall the service")
        .required(false)
        .interact()?;

if selected.is_empty() {
    init_style::omega_outro("Done")?;
    return Ok(());
}
```

**Rationale**: Empty selection as "exit" is a natural TUI convention. The prompt text makes this explicit. No contradictory states are possible.

---

### Finding 4: Google API Multiselect to Select Double-Prompt (P1 -- confusing)

**Location**: `init_google.rs` lines 417-437

**Current code:**
```rust
let mut ms = cliclack::multiselect("Step 2 -- Select Google APIs to enable");
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
```

**Problem**: If the user presses Enter without selecting anything in the multiselect, a SECOND prompt appears (a select widget) asking them to pick one or skip. This is a double-prompt for the same purpose. The user expected their Enter to mean "I'm done, continue" but instead got a new prompt.

**Fix**: Remove the select fallback entirely. Use `required(false)` on the multiselect. If the user submits with no selections, treat it as "enable recommended defaults" (Gmail, Calendar, Drive, Contacts -- the 4 most commonly needed APIs) and show which were auto-selected.

**After:**
```rust
let hint = "Step 2 -- Select Google APIs to enable\n  Space = toggle, Enter = confirm (defaults: Gmail, Calendar, Drive, Contacts)";
let mut ms = cliclack::multiselect(hint);
for (i, (name, api)) in apis.iter().enumerate() {
    ms = ms.item(i, *name, *api);
}
let selected: Vec<usize> = ms.required(false).interact()?;

let chosen: Vec<usize> = if selected.is_empty() {
    // Default to the 4 most common APIs
    let defaults = vec![0, 1, 2, 10]; // Gmail, Calendar, Drive, Contacts
    init_style::omega_info("Using recommended defaults: Gmail, Calendar, Drive, Contacts")?;
    defaults
} else {
    selected
};
```

**Alternative considered**: Remove the double-prompt but treat empty as "skip all APIs". Rejected because the user explicitly chose to set up Google Workspace -- skipping all APIs makes the setup pointless. Smart defaults respect the user's intent.

---

### Finding 5: Hint Text Placement (P2 -- polish)

**Location**: `init.rs` lines 133-136 (init), lines 286-289 (setup), `init_google.rs` lines 412-415

**Current code:**
```rust
let hint = console::Style::new()
    .bold()
    .apply_to("Space to select, Enter to confirm");
init_style::omega_info(&hint.to_string())?;
```

**Problem**: The hint is rendered as a separate styled line above the multiselect widget. Once the widget renders, the hint may scroll off-screen on small terminals. The hint text also varies inconsistently: "Space to select, Enter to confirm" vs "Space to select, Enter to pick one".

**Fix**: Embed the hint consistently in every multiselect prompt string. Remove the separate `omega_info` call.

**Pattern:**
```rust
// For multiselect where empty = skip/exit:
cliclack::multiselect("Your prompt here\n  Space = toggle, Enter = confirm")

// For multiselect where empty = done/exit:
cliclack::multiselect("Your prompt here\n  Space = toggle, Enter = confirm, Enter with none = exit")
```

**Rationale**: The hint stays visible because it is part of the prompt that cliclack renders alongside the widget. Consistent wording across all multiselects reduces cognitive load.

---

### Finding 6: No Review Step Before Config Write (P2 -- polish)

**Location**: `init.rs` lines 166-184

**Problem**: The wizard collects all inputs across multiple steps, then writes `config.toml` without showing the user a summary of what will be configured. The user has no opportunity to review their choices before the file is written. This violates the "confirmation before execution" principle.

**Fix**: Add a summary note before writing `config.toml`.

**After (insert before the config write):**
```rust
// Build summary of choices
let mut summary = String::new();
summary.push_str(&format!("  Telegram: {}\n",
    if bot_token.is_empty() { "skipped" } else { "configured" }));
if let Some(id) = user_id {
    summary.push_str(&format!("  Allowed user: {id}\n"));
}
summary.push_str(&format!("  Voice transcription: {}\n",
    if whisper_api_key.is_some() { "configured" } else { "skipped" }));
summary.push_str(&format!("  WhatsApp: {}\n",
    if whatsapp_enabled { "paired" } else { "skipped" }));
summary.push_str(&format!("  Google Workspace: {}\n",
    if google_email.is_some() { "connected" } else { "skipped" }));
summary.push_str(&format!("  Claude Auth: {}",
    if oauth_token.is_some() { "configured" } else { "skipped" }));

init_style::omega_note("Configuration summary", &summary)?;

let proceed: bool = cliclack::confirm("Write configuration to ~/.omega/config.toml?")
    .initial_value(true)
    .interact()?;

if !proceed {
    init_style::omega_outro_cancel("Setup cancelled -- no files written")?;
    return Ok(());
}
```

**Rationale**: The user sees exactly what will be configured and can bail out if something is wrong. The default is `true` (proceed), so users who are happy just press Enter. This adds one keypress for the common case but prevents "I didn't mean to configure that" frustration.

---

### Finding 7: `omega setup` Loop Exit Clarity (P2 -- polish)

**Location**: `init.rs` lines 239-411

**Problem**: The `omega setup` loop re-shows the full menu after each reconfiguration cycle. With "Exit" removed (Finding 3), the exit mechanism is empty-selection. But the loop behavior should be explicit: after completing the selected reconfiguration(s), ask whether to continue or exit, rather than silently re-rendering the entire logo and menu.

**Fix**: After applying changes, show a success summary and ask whether to reconfigure more components.

**After (at end of loop body, after config updates and service install):**
```rust
if !changed.is_empty() || selected.contains(&"service") {
    let continue_setup: bool = cliclack::confirm("Reconfigure more components?")
        .initial_value(false)
        .interact()?;
    if !continue_setup {
        init_style::omega_outro("Done")?;
        return Ok(());
    }
} else {
    init_style::omega_outro("No changes made")?;
    return Ok(());
}
```

---

## Step Sequence: `omega init`

### WIZ-INIT-001: Welcome & Prerequisites
- **Purpose**: Show branding, create data directory, verify Claude CLI is installed
- **Prerequisite**: None
- **Skip Condition**: Never skipped
- **Interactive**: No (automated checks with visual feedback)

#### Actions
1. Display OMEGA ASCII logo via `init_style::omega_intro(LOGO, "omega init")`
2. Create `~/.omega/` directory if missing; show success status
3. Spinner: check `claude --version`
   - Found: stop spinner with "claude CLI -- found"
   - NOT found: stop spinner with error, show install instructions via `omega_note`, exit with `omega_outro_cancel("Setup aborted")`

#### Error Messages
| Condition | Message |
|-----------|---------|
| `~/.omega/config.toml` already exists | "OMEGA is already installed. Use `omega setup` to reconfigure." |
| Claude CLI not found | "claude CLI -- NOT FOUND" + install instructions note |

#### UX Copy
- **Introduction**: Logo + "omega init" subtitle
- **Success**: "{data_dir} -- created" or "{data_dir} -- exists"

---

### WIZ-INIT-002: Anthropic Authentication
- **Purpose**: Collect and validate the OAuth token for Claude Code
- **Prerequisite**: WIZ-INIT-001 completed (Claude CLI available)
- **Skip Condition**: User presses Enter with empty input (token skipped)

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| OAuth token | password input | (empty) | Format: starts with `sk-ant-oat01-`. Async validation: runs `claude -p "ok"` with the token (15s timeout) | No |

#### Smart Defaults
- oauth_token: No sensible default. User must obtain this from `claude setup-token`.

#### UX Copy
- **Introduction**: Note box: "1. Run `claude login` in another terminal / 2. Run `claude setup-token` and authorize in the browser / 3. Paste the generated token below"
- **Field label**: "Paste OAuth token (or Enter to skip)"
- **Placeholder**: "sk-ant-oat01-..."
- **Validation spinner**: "Validating token..."
- **Success**: "Anthropic authentication -- configured"
- **Validation failure**: "Token validation failed -- attempt {n}/{max}. Try again."
- **Skip warning**: "Skipped -- set oauth_token in config.toml later, or run `omega init` again."

#### Error Messages
| Condition | Message |
|-----------|---------|
| Token validation fails (all 3 attempts) | "Token validation failed -- claude could not authenticate. The token will still be saved to config.toml. You can re-authenticate later with: claude setup-token" |

---

### WIZ-INIT-003: Telegram Bot Token
- **Purpose**: Collect the Telegram bot token for the messaging channel
- **Prerequisite**: WIZ-INIT-002 completed
- **Skip Condition**: User presses Enter with empty input

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Bot token | text input | (empty) | None (validated on first connection) | No |

#### Smart Defaults
- bot_token: No default. Intentionally -- each user has their own bot via @BotFather.

#### UX Copy
- **Field label**: "Telegram bot token"
- **Placeholder**: "Paste token from @BotFather (or Enter to skip)"
- **Skip info**: "Skipping Telegram -- you can add it later in config.toml"

---

### WIZ-INIT-004: Telegram User ID
- **Purpose**: Restrict bot access to the owner's Telegram account
- **Prerequisite**: WIZ-INIT-003 completed AND bot token is non-empty
- **Skip Condition**: Auto-skipped if bot token is empty. Also skipped if user enters blank (allows all users).

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| User ID | text input | (empty) | Parsed as i64; invalid = silently treated as "allow all" | No |

#### Smart Defaults
- user_id: No default. Cannot be auto-detected without calling Telegram API.

#### UX Copy
- **Field label**: "Your Telegram user ID"
- **Placeholder**: "Send /start to @userinfobot (blank = allow all)"

---

### WIZ-INIT-005: Voice Transcription
- **Purpose**: Optionally enable Whisper-based voice message transcription
- **Prerequisite**: WIZ-INIT-003 completed AND bot token is non-empty
- **Skip Condition**: Auto-skipped if bot token is empty. Also skipped if user declines the confirm prompt.

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Enable voice? | confirm | No | N/A | N/A |
| OpenAI API key | text input (shown only if enabled) | (empty) | None (validated on first use) | No |

#### UX Copy
- **Confirm label**: "Enable voice message transcription?"
- **Note title**: "Voice Transcription"
- **Note body**: "Voice messages will be transcribed using OpenAI Whisper.\nGet a key: https://platform.openai.com/api-keys"
- **Field label**: "OpenAI API key (for Whisper)"
- **Placeholder**: "sk-... (Enter to skip, or set OPENAI_API_KEY later)"

---

### WIZ-INIT-006: WhatsApp Pairing
- **Purpose**: Connect WhatsApp via QR code scanning
- **Prerequisite**: WIZ-INIT-005 completed
- **Skip Condition**: User declines the confirm prompt. Also auto-skipped if already paired (session file exists).

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Connect WhatsApp? | confirm | No | N/A | N/A |

#### UX Copy
- **Confirm label**: "Connect WhatsApp?"
- **Already paired**: "WhatsApp -- already paired"
- **Instructions**: "Open WhatsApp on your phone > Linked Devices > Link a Device"
- **QR display**: Note box with QR code rendered as Unicode
- **Spinner**: "Waiting for scan..."
- **Success**: "WhatsApp linked successfully"
- **Failure**: "Pairing did not complete"
- **Timeout**: "{error} -- you can try again later with /whatsapp."

---

### WIZ-INIT-007: Google Workspace
- **Purpose**: Optionally set up Google Workspace integration (Gmail, Calendar, Drive, etc.)
- **Prerequisite**: WIZ-INIT-006 completed
- **Skip Condition**: User declines the confirm prompt

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Set up Google? | confirm | No | N/A | N/A |

#### UX Copy (CHANGED -- was multiselect, now confirm)
- **Confirm label**: "Set up Google Workspace?"
- **Hint**: Shows "(installed)" or "(Ask OMEGA to manage Gmail, Calendar, Drive, Docs...)" based on whether `omg-gog` is detected

#### Smart Defaults
- Default: No. Google Workspace setup is complex (requires GCP project creation) and most users skip it on first install.

**Sub-wizard**: If the user accepts, delegates to `init_google::run_google_wizard()` which has its own multi-step flow (see Google Workspace Sub-Wizard section below).

---

### WIZ-INIT-008: Review & Confirm
- **Purpose**: Show the user a summary of all choices before writing configuration
- **Prerequisite**: WIZ-INIT-007 completed
- **Skip Condition**: Never skipped

**NEW STEP** -- not present in current implementation.

#### Display
Summary note showing all configured components:
```
|  Configuration summary
|  .  Claude Auth: configured / skipped
|  .  Telegram: configured / skipped
|  .  Allowed user: {id}
|  .  Voice transcription: configured / skipped
|  .  WhatsApp: paired / skipped
|  .  Google Workspace: connected / skipped
```

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Proceed? | confirm | Yes | N/A | Yes |

#### UX Copy
- **Note title**: "Configuration summary"
- **Confirm label**: "Write configuration to ~/.omega/config.toml?"
- **Cancel**: "Setup cancelled -- no files written"

---

### WIZ-INIT-009: Service Install & Next Steps
- **Purpose**: Offer system service installation and show what to do next
- **Prerequisite**: WIZ-INIT-008 confirmed (config written)
- **Skip Condition**: Never skipped

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Install service? | confirm | Yes | N/A | N/A |

#### UX Copy
- **Confirm label**: "Install Omega as a system service?"
- **Service success**: "System service installed"
- **Service failure**: "Service install failed: {error}. You can install later with: omega service install"
- **Next steps note title**: "Next steps"
- **Next steps body**:
  ```
  1. Review ~/.omega/config.toml
  2. Run: omega start
  3. Send a message to your bot
  ```
  (Plus conditional lines for WhatsApp, Google, service status)
- **Outro**: "Setup complete"
- **Animation**: Typewrite "enjoy OMEGA ..." on exit

---

## Step Sequence: `omega setup`

### WIZ-SETUP-001: Component Selection
- **Purpose**: Let the user pick which components to reconfigure
- **Prerequisite**: `~/.omega/config.toml` exists
- **Skip Condition**: N/A

#### Fields
| Field | Type | Default | Validation | Required |
|-------|------|---------|------------|----------|
| Components | multiselect | None pre-selected | `required(false)` -- empty = exit | No |

Items (each shows "(configured)" suffix if already set up):
1. Claude Auth -- "OAuth token for Claude Code"
2. Telegram -- "Bot token and allowed users"
3. Voice Transcription -- "OpenAI Whisper API key"
4. WhatsApp -- "Pair via QR code"
5. Google Workspace -- "Gmail, Calendar, Drive..."
6. System Service -- "Install or reinstall the service"

NO "Exit" item.

#### UX Copy
- **Prompt**: "Select components to reconfigure\n  Space = toggle, Enter = confirm, Enter with none = exit"
- **Empty selection**: Exits the loop with `omega_outro("Done")`
- **After changes applied**: Confirm "Reconfigure more components?" (default: No)

---

## Google Workspace Sub-Wizard (within WIZ-INIT-007 / WIZ-SETUP Google)

This sub-wizard is delegated to `init_google.rs` and runs when the user opts to set up Google Workspace.

### GOG-001: Ensure omg-gog CLI
- Spinner: check if `omg-gog` is installed
- If missing: attempt install via script, then from source
- If still missing: show manual install instructions, abort sub-wizard

### GOG-002: Create Google Cloud Project
- Note with instructions + direct URL
- Confirm: "Done? Continue to next step"

### GOG-003: Collect GCP Project ID
- Input: "Google Cloud Project ID or number"
- Validation: non-empty, no spaces, no slashes

### GOG-004: Enable Google APIs (CHANGED)
- **Multiselect** with `required(false)`, hint embedded in prompt
- 14 API items listed
- **Empty selection = use recommended defaults** (Gmail, Calendar, Drive, Contacts)
- Show direct enable links for selected/defaulted APIs
- Confirm: "Done enabling? Continue to next step"

**Before:**
```rust
// Multiselect -> if empty, fall back to select -> if "Skip", empty vec
let selected: Vec<usize> = ms.required(false).interact()?;
let chosen: Vec<usize> = if selected.is_empty() {
    let mut sel = cliclack::select("Pick one API to enable (or skip)");
    // ... second prompt
```

**After:**
```rust
let selected: Vec<usize> = ms.required(false).interact()?;
let chosen: Vec<usize> = if selected.is_empty() {
    let defaults = vec![0, 1, 2, 10]; // Gmail, Calendar, Drive, Contacts
    init_style::omega_info("Using recommended defaults: Gmail, Calendar, Drive, Contacts")?;
    defaults
} else {
    selected
};
```

### GOG-005: Configure OAuth Consent Screen
- Note with instructions + direct URL
- Confirm: "Done? Continue to next step"

### GOG-006: Create OAuth Client Credentials
- Note with instructions + direct URL
- Confirm: "Done? Continue to next step"

### GOG-007: Publish the App
- Note with instructions + direct URL
- Confirm: "Done? Continue to paste credentials"

### GOG-008: Collect Credentials JSON
- Input: paste JSON content or file path
- Validation: valid JSON with "web" or "installed" key
- Spinner: "Registering credentials with omg-gog..."

### GOG-009: OAuth Authorization
- Instructions for the authorization flow
- Display authorization URL via note
- Input: "Paste the authorization code"
- Success/failure feedback

### GOG-010: Detect Connected Account
- Auto-detect email from `omg-gog auth list`
- Success: "Google Workspace connected -- {email}"

---

## Flow Diagram

```
omega init
==========

[WIZ-INIT-001] Welcome & Prerequisites
    |
    +-- config.toml exists? --> STOP: "Use omega setup"
    +-- claude CLI missing? --> STOP: "Install claude CLI"
    |
[WIZ-INIT-002] Anthropic Auth (skippable)
    |
[WIZ-INIT-003] Telegram Bot Token (skippable)
    |
    +-- token empty? --+
    |                   |
    v                   |
[WIZ-INIT-004]          |
  User ID               |
    |                   |
[WIZ-INIT-005]          |
  Voice Transcription   |
    |                   |
    +<------------------+
    |
[WIZ-INIT-006] WhatsApp Pairing (skippable)
    |
[WIZ-INIT-007] Google Workspace (skippable)
    |                   |
    +-- yes? --> [GOG-001..010 sub-wizard]
    |                   |
    +<------------------+
    |
[WIZ-INIT-008] Review & Confirm (NEW)
    |
    +-- cancel? --> STOP: "No files written"
    |
[WIZ-INIT-009] Service Install & Next Steps
    |
    DONE


omega setup
============

[Guard] config.toml exists? --> No: STOP "Run omega init"
    |
    v
+-> [WIZ-SETUP-001] Component Selection (multiselect)
|       |
|       +-- empty selection? --> DONE
|       |
|       +-- components selected --> run each sub-wizard
|       |
|       +-- "Reconfigure more?" confirm
|       |       |
|       +<------+-- Yes
|
+-- No --> DONE
```

---

## State Management
- **Storage**: In-memory variables within the `run()` / `run_setup()` function scope. No external state file.
- **Persistence**: None across interruptions. If the wizard is Ctrl+C'd, all progress is lost. This is acceptable because:
  - The wizard takes 1-3 minutes
  - No destructive operations occur until the config write step
  - The review step (WIZ-INIT-008) is the final gate before any file is written
- **Recovery**: Re-run `omega init` from scratch. Since the wizard does not write anything until WIZ-INIT-008 confirms, interruption is always safe.
- **State File**: N/A (not needed for a 1-3 minute wizard)

## Navigation Rules
| Action | Behavior |
|--------|----------|
| Enter (on input) | Submit current value (or empty/default if blank) |
| Enter (on confirm) | Accept the current selection (yes/no) |
| Enter (on select) | Pick the highlighted item |
| Enter (on multiselect) | Submit current toggle state |
| Space (on multiselect) | Toggle the highlighted item |
| Arrow keys | Navigate between items in select/multiselect |
| Ctrl+C | Abort wizard entirely. Safe: no files written until WIZ-INIT-008 |
| Back navigation | Not supported by cliclack. Mitigated by the review step (WIZ-INIT-008) which lets users cancel and restart |

**Note on back-navigation**: cliclack does not support backward navigation between steps. This is a framework limitation. The review step (WIZ-INIT-008) mitigates this: the user sees all choices before anything is committed. If they notice a mistake, they cancel and re-run the wizard. For a 1-3 minute wizard, this is an acceptable trade-off.

---

## Error Recovery
| Error Type | Detection | User Experience | Recovery |
|------------|-----------|-----------------|----------|
| Validation failure (inline) | cliclack `.validate()` | Error shown below field, field retains input | Edit and resubmit |
| OAuth token invalid | `claude -p "ok"` returns non-zero | "Token validation failed -- attempt N/3. Try again." | 3 retries, then saves anyway with warning |
| Network failure (WhatsApp) | `tokio::time::timeout` | Spinner error + "you can try again later with /whatsapp." | Non-fatal, wizard continues |
| Network failure (omg-gog) | Subprocess exit code / timeout | Spinner error + specific message | Non-fatal, wizard continues |
| Permission error (directory) | `std::fs::create_dir_all` failure | Anyhow error propagates | User fixes permissions, re-runs |
| Config write failure | `std::fs::write` failure | Anyhow error propagates | User fixes disk/permissions, re-runs |
| Ctrl+C interruption | Terminal signal | Immediate exit | Re-run wizard (no cleanup needed) |

---

## Expert/Fast-Path Mode

### Non-interactive deployment (`omega init --telegram-token ...`)
All wizard inputs can be provided as CLI flags or environment variables:

| Flag | Env Var | Purpose |
|------|---------|---------|
| `--telegram-token` | `OMEGA_TELEGRAM_TOKEN` | Telegram bot token |
| `--allowed-users` | `OMEGA_ALLOWED_USERS` | Comma-separated user IDs |
| `--claude-setup-token` | `OMEGA_CLAUDE_SETUP_TOKEN` | OAuth token |
| `--whisper-key` | `OMEGA_WHISPER_KEY` | OpenAI API key |
| `--google-credentials` | `OMEGA_GOOGLE_CREDENTIALS` | Path to client_secret.json |
| `--google-email` | `OMEGA_GOOGLE_EMAIL` | Gmail address |

When any of these flags are provided, the wizard runs non-interactively (no prompts, no spinners, text-only output).

### Manual configuration
Users can skip the wizard entirely by creating `~/.omega/config.toml` manually (using `config.example.toml` as reference) and running `omega start`.

---

## Accessibility
| Requirement | Implementation |
|-------------|----------------|
| Keyboard navigation | All interactions are keyboard-driven (cliclack is keyboard-only). Arrow keys, Space, Enter, Ctrl+C. No mouse required. |
| Screen reader | cliclack writes to stderr with clear text labels. Each prompt has a text label. Status changes (spinner start/stop/error) emit text. Progress is communicated via step-by-step prompts. |
| Color independence | All status indicators use text markers alongside color: `+` (success/green), `-` (info/cyan), `!` (warning/yellow), `x` (error/red), `>` (step/cyan). Color enhances but is not required to understand the output. |
| Focus management | cliclack manages focus automatically. Each step receives focus when it renders. Completed steps scroll up naturally. |
| High contrast | Cyan-on-dark palette. Brand colors (cyan bold) meet contrast requirements on standard dark terminal backgrounds. Light terminal backgrounds may have lower contrast -- this is a known limitation of colored TUI output. |

---

## Post-Wizard Experience

### On Success (`omega init`)
- **Message**: "Setup complete" + typewrite "enjoy OMEGA ..."
- **Next Steps** (shown as note box):
  1. Review ~/.omega/config.toml
  2. Run: omega start
  3. Send a message to your bot
  - (Conditional) WhatsApp is linked and ready!
  - (Conditional) Google Workspace is connected!
  - (Conditional) System service installed -- OMEGA starts on login!
  - (Conditional) Tip: Run `omega service install` to auto-start on login
- **Generated Artifacts**: `~/.omega/config.toml`, `~/.omega/` directory

### On Success (`omega setup`)
- **Message**: "Updated ~/.omega/config.toml -- {changed components}"
- **Next Steps**: Implicit -- user knows they are reconfiguring an existing installation

### On Failure
- **Claude CLI missing**: "Setup aborted" with install instructions. No artifacts created.
- **User cancels at review step**: "Setup cancelled -- no files written". No artifacts created.
- **Service install fails**: Warning shown, wizard continues. User told to run `omega service install` later.
- **WhatsApp/Google sub-wizard fails**: Warning shown, wizard continues. Config written without those components.

---

## Complete Before/After Code Reference

### Change 1: Optional Tools (init.rs) -- P0

**BEFORE (lines 126-148):**
```rust
// 8. Optional tools.
let omg_gog_installed = crate::init_google::is_omg_gog_installed();
let google_hint = if omg_gog_installed {
    "Ask OMEGA to manage Gmail, Calendar, Drive, Docs... (installed)"
} else {
    "Ask OMEGA to manage Gmail, Calendar, Drive, Docs..."
};

let hint = console::Style::new()
    .bold()
    .apply_to("Space to select, Enter to confirm");
init_style::omega_info(&hint.to_string())?;

let selected_tools: Vec<&str> = cliclack::multiselect("Optional tools -- select to set up")
    .item("skip", "Skip", "Continue without setting up optional tools")
    .item("google", "Google Workspace", google_hint)
    .required(true)
    .interact()?;

let selected_tools: Vec<&str> = if selected_tools.contains(&"skip") {
    vec![]
} else {
    selected_tools
};
```

**AFTER:**
```rust
// 8. Optional tools -- Google Workspace.
let omg_gog_installed = crate::init_google::is_omg_gog_installed();
let google_hint = if omg_gog_installed {
    "Gmail, Calendar, Drive, Docs... (omg-gog installed)"
} else {
    "Gmail, Calendar, Drive, Docs..."
};

let setup_google: bool = cliclack::confirm(
    &format!("Set up Google Workspace? {}", init_style::muted_text(google_hint))
)
    .initial_value(false)
    .interact()?;
```

**Rationale**: Binary choice = `confirm`. Eliminates mutual exclusion bug. Eliminates Space/Enter confusion. Fewer lines of code. Default is `false` because Google setup is the longest sub-wizard and most users skip it on first run.

---

### Change 2: Setup Component Selection (init.rs) -- P1

**BEFORE (lines 286-310):**
```rust
let hint = console::Style::new()
    .bold()
    .apply_to("Space to select, Enter to confirm");
init_style::omega_info(&hint.to_string())?;

let selected: Vec<&str> = cliclack::multiselect("Select components to reconfigure")
    .item("claude", &lbl_claude, "OAuth token for Claude Code")
    .item("telegram", &lbl_telegram, "Bot token and allowed users")
    .item("whisper", &lbl_whisper, "OpenAI Whisper API key")
    .item("whatsapp", &lbl_whatsapp, "Pair via QR code")
    .item("google", &lbl_google, "Gmail, Calendar, Drive...")
    .item("service", &lbl_service, "Install or reinstall the service")
    .item("exit", "Exit", "Return to terminal")
    .interact()?;

if selected.contains(&"exit") {
    init_style::omega_outro("Done")?;
    return Ok(());
}
```

**AFTER:**
```rust
let selected: Vec<&str> = cliclack::multiselect(
    "Select components to reconfigure\n  Space = toggle, Enter = confirm, Enter with none = exit"
)
    .item("claude", &lbl_claude, "OAuth token for Claude Code")
    .item("telegram", &lbl_telegram, "Bot token and allowed users")
    .item("whisper", &lbl_whisper, "OpenAI Whisper API key")
    .item("whatsapp", &lbl_whatsapp, "Pair via QR code")
    .item("google", &lbl_google, "Gmail, Calendar, Drive...")
    .item("service", &lbl_service, "Install or reinstall the service")
    .required(false)
    .interact()?;

if selected.is_empty() {
    init_style::omega_outro("Done")?;
    return Ok(());
}
```

**Rationale**: "Exit" removed from multiselect. Hint embedded in prompt. `required(false)` enables empty-selection-as-exit. No contradictory states possible.

---

### Change 3: Setup Loop Exit (init.rs) -- P2

**BEFORE (end of loop body, line ~410):**
```rust
    } // end of loop body -- silently re-renders full menu
}
```

**AFTER (insert before closing brace of loop body):**
```rust
        // After applying changes, ask whether to continue
        let continue_setup: bool = cliclack::confirm("Reconfigure more components?")
            .initial_value(false)
            .interact()?;
        if !continue_setup {
            init_style::omega_outro("Done")?;
            return Ok(());
        }
    } // end of loop body
}
```

---

### Change 4: Google API Double-Prompt (init_google.rs) -- P1

**BEFORE (lines 417-437):**
```rust
let hint = console::Style::new()
    .bold()
    .apply_to("Space to select, Enter to pick one");
init_style::omega_info(&hint.to_string())?;

let mut ms = cliclack::multiselect("Step 2 -- Select Google APIs to enable");
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
```

**AFTER:**
```rust
let mut ms = cliclack::multiselect(
    "Step 2 -- Select Google APIs to enable\n  Space = toggle, Enter = confirm (defaults: Gmail, Calendar, Drive, Contacts)"
);
for (i, (name, api)) in apis.iter().enumerate() {
    ms = ms.item(i, *name, *api);
}
let selected: Vec<usize> = ms.required(false).interact()?;

let chosen: Vec<usize> = if selected.is_empty() {
    // User pressed Enter without selecting -- use recommended defaults
    let defaults = vec![0, 1, 2, 10]; // Gmail, Calendar, Drive, Contacts (People API)
    init_style::omega_info("Using recommended defaults: Gmail, Calendar, Drive, Contacts")?;
    defaults
} else {
    selected
};
```

**Rationale**: Single prompt instead of two. Smart defaults instead of a second prompt. The user chose to set up Google Workspace -- giving them sensible defaults respects their intent.

---

### Change 5: Review Step (init.rs) -- P2

**INSERT before config write (between current line 165 and 166):**
```rust
    // 8.5 Review configuration before writing.
    let mut summary_lines = Vec::new();
    summary_lines.push(format!("  Claude Auth: {}",
        if oauth_token.is_some() { "configured" } else { "skipped" }));
    summary_lines.push(format!("  Telegram: {}",
        if bot_token.is_empty() { "skipped" } else { "configured" }));
    if let Some(id) = user_id {
        summary_lines.push(format!("  Allowed user: {id}"));
    }
    summary_lines.push(format!("  Voice transcription: {}",
        if whisper_api_key.is_some() { "configured" } else { "skipped" }));
    summary_lines.push(format!("  WhatsApp: {}",
        if whatsapp_enabled { "paired" } else { "skipped" }));
    summary_lines.push(format!("  Google Workspace: {}",
        match &google_email { Some(e) => format!("connected ({e})"), None => "skipped".to_string() }));

    init_style::omega_note("Configuration summary", &summary_lines.join("\n"))?;

    let proceed: bool = cliclack::confirm("Write configuration to ~/.omega/config.toml?")
        .initial_value(true)
        .interact()?;

    if !proceed {
        init_style::omega_outro_cancel("Setup cancelled -- no files written")?;
        return Ok(());
    }
```

---

## Design Decisions

| Decision | Alternatives Considered | Rationale |
|----------|------------------------|-----------|
| Replace optional tools multiselect with `confirm` | (A) `select` with Skip/Google items, (B) multiselect with `required(false)`, (C) keep multiselect + validation | `confirm` is the simplest widget for a binary choice. Only one optional tool exists currently. If more are added, switch to multiselect with `required(false)`. |
| Remove "Exit" from setup multiselect | (A) Keep Exit but validate mutual exclusion, (B) Replace multiselect with repeated select | `required(false)` with empty-as-exit is cleaner. Every item in the multiselect is now semantically consistent (all are components). |
| Smart defaults for Google APIs | (A) Empty = skip all, (B) Empty = select fallback prompt, (C) Pre-check defaults with `initial_values` | (A) contradicts user's intent (they chose Google setup). (B) is the existing double-prompt bug. (C) is ideal but `initial_values` would pre-toggle items which some users might not notice. Smart defaults with an info message is explicit and correct. |
| Add review step before config write | (A) No review (current), (B) Full editable summary with jump-back | (A) violates confirmation-before-execution principle. (B) impossible with cliclack (no back-nav). A non-editable summary with cancel option is the best possible within framework constraints. |
| Embed hints in prompt strings | (A) Keep separate `omega_info` hint, (B) Add cliclack `.hint()` method (doesn't exist) | Prompt-embedded hints stay visible alongside the widget. Separate hints scroll away. |
| "Reconfigure more?" confirm after setup changes | (A) Silent loop restart, (B) Always exit after one round | (A) is disorienting (full screen clear + logo re-render). (B) forces users to re-run `omega setup` for multiple changes. The confirm gives the user control. |
| No state persistence across interruptions | (A) Save partial state to temp file, (B) Resume prompt on restart | Wizard takes 1-3 minutes and writes nothing until the review step. The cost of re-running is low. Adding state persistence would add complexity disproportionate to the benefit, violating OMEGA's "less is more" principle. |

---

## Priority Summary

| Priority | Issue | Fix | Impact |
|----------|-------|-----|--------|
| **P0** | Skip + Google mutual exclusion | Replace multiselect with `confirm` | Eliminates contradictory state bug |
| **P1** | Space + Enter confusion | Embed hints in prompt; use `confirm` for binary choices | Reduces user confusion on first interaction |
| **P1** | "Exit" in setup multiselect | Remove "Exit", use `required(false)` | Eliminates contradictory state |
| **P1** | Google API double-prompt | Remove select fallback, use smart defaults | Eliminates confusing second prompt |
| **P2** | Hint text placement | Embed in prompt string | Hints stay visible |
| **P2** | No review before config write | Add summary + confirm step | User sees choices before commit |
| **P2** | Setup loop exit clarity | Add "Reconfigure more?" confirm | Clear exit path |

**Implementation order**: P0 first (1 change), then P1 (3 changes), then P2 (3 changes). Total: 7 changes across 2 files (`init.rs`, `init_google.rs`).
