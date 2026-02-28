# Architecture: OMEGA Init Wizard Visual Identity

## Scope

This architecture covers `backend/src/init_style.rs` (new module), plus integration changes to `backend/src/init.rs` and `backend/src/init_wizard.rs`. It does NOT cover cliclack interactive widgets (input, select, confirm, spinner) or the non-interactive `run_noninteractive()` path.

## Overview

A single new module (`init_style.rs`) provides 10 branded output helper functions that replace all cliclack "chrome" calls (intro, outro, log::success, log::warning, log::error, log::info, log::step, note) in the interactive init wizard. cliclack interactive widgets remain untouched. The module uses `console::Style` exclusively for terminal coloring -- no raw ANSI escape codes.

```
init.rs  ──────┐
               ├──> init_style.rs (branded output helpers)
init_wizard.rs ┘         │
                          ├── omega_intro()
                          ├── omega_outro()
                          ├── omega_outro_cancel()
                          ├── omega_success()
                          ├── omega_info()
                          ├── omega_warning()
                          ├── omega_error()
                          ├── omega_step()
                          ├── omega_note()
                          └── typewrite()
```

## Color Palette

### Design Direction: Dark Terminal / Classified Briefing

The palette is designed for dark terminal backgrounds. Three accent colors provide clear semantic separation without decoration overload.

| Role | Color | `console::Style` Method | Hex Approx. | Usage |
|------|-------|------------------------|-------------|-------|
| **Primary accent** | Cyan | `.cyan()` | `#00FFFF` | Gutter bars, intro/outro chrome, step indicators |
| **Brand** | Cyan + Bold | `.cyan().bold()` | `#00FFFF` | Logo, OMEGA name, section titles |
| **Success** | Green | `.green()` | `#00FF00` | Success status markers |
| **Warning** | Yellow | `.yellow()` | `#FFFF00` | Warning status markers |
| **Error** | Red | `.red()` | `#FF0000` | Error status markers |
| **Info** | Cyan | `.cyan()` | `#00FFFF` | Info status markers, note borders |
| **Muted** | Dim (white) | `.dim()` | `#808080` | Gutter dots, secondary text, note body |
| **Text** | Default | (none) | terminal default | Message body text |

### Why Cyan

- **High visibility on dark backgrounds** without being aggressive (amber/gold can look muddy on some terminals)
- **Technical/cyberpunk association** -- cyan is the canonical "hacker terminal" color
- **Clear contrast with cliclack's purple** -- cliclack uses magenta/purple for its chrome, so cyan creates visual distinction without clashing when cliclack widgets appear inline
- **Works on both dark and light terminals** -- cyan has enough contrast for both, unlike amber which washes out on light backgrounds
- **Single-method in console crate** -- `.cyan()` is a first-class method, no custom color needed

### Color Constants

```rust
use console::Style;

/// Cyan accent for gutter bars, borders, structural chrome.
fn accent() -> Style {
    Style::new().cyan()
}

/// Cyan bold for brand elements (logo, titles).
fn brand() -> Style {
    Style::new().cyan().bold()
}

/// Dimmed for secondary/muted text.
fn muted() -> Style {
    Style::new().dim()
}
```

These are functions (not `const` or `lazy_static`) because `console::Style` does not implement `const fn` construction. The functions are trivially cheap (builder pattern, no allocation).

## Module: `init_style.rs`

### Responsibility
Provide branded terminal output helpers for the init wizard. Pure display functions -- no state, no async, no I/O beyond `eprintln!`/`println!` (via `console::Term`).

### Public Interface

All functions are `pub(crate)` -- internal to the binary crate.

```rust
pub(crate) fn omega_intro(logo: &str, subtitle: &str) -> io::Result<()>
pub(crate) fn omega_outro(message: &str) -> io::Result<()>
pub(crate) fn omega_outro_cancel(message: &str) -> io::Result<()>
pub(crate) fn omega_success(message: &str) -> io::Result<()>
pub(crate) fn omega_info(message: &str) -> io::Result<()>
pub(crate) fn omega_warning(message: &str) -> io::Result<()>
pub(crate) fn omega_error(message: &str) -> io::Result<()>
pub(crate) fn omega_step(message: &str) -> io::Result<()>
pub(crate) fn omega_note(title: &str, body: &str) -> io::Result<()>
pub(crate) fn typewrite(text: &str, delay_ms: u64)
```

### Dependencies
- `console` v0.15 (`Style`, `Term`) -- already in `Cargo.toml`
- `std::io` -- for `io::Result`
- `std::thread::sleep`, `std::time::Duration` -- for `typewrite()`

### Implementation Order
1. Color palette functions (`accent()`, `brand()`, `muted()`)
2. Status line helpers (`omega_success`, `omega_info`, `omega_warning`, `omega_error`, `omega_step`)
3. Note box (`omega_note`)
4. Intro/outro (`omega_intro`, `omega_outro`, `omega_outro_cancel`)
5. `typewrite()` (moved from `init.rs`)
6. Unit tests

---

## Visual Design: Each Helper Function

### Design Vocabulary

The branded chrome uses a **gutter-bar** visual language:

- **Cyan vertical bar** (`|`) on the left as the structural element (replaces cliclack's box-drawing characters)
- **Status markers** are single Unicode characters with semantic color
- **Whitespace is the separator** -- no box-drawing, no heavy decoration
- The gutter bar provides visual continuity through the wizard while being minimal

### Gutter Characters

```rust
const BAR: &str = "|";       // Gutter bar (styled cyan)
const DOT: &str = ".";       // Continuation (styled dim)
```

These are intentionally plain ASCII. No Unicode box-drawing characters. The styling comes from color, not from character complexity.

---

### 1. `omega_intro(logo, subtitle)`

**Replaces:** `typewrite(LOGO, 2)` + `typewrite("omega init\n", 30)` + `cliclack::intro("omega init")`

**Signature:**
```rust
pub(crate) fn omega_intro(logo: &str, subtitle: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan().bold()` for logo, `.cyan()` for bar and subtitle, `.dim()` for separator

**Visual output:**
```
              ██████╗ ███╗   ███╗███████╗ ██████╗  █████╗        █████╗
             ██╔═══██╗████╗ ████║██╔════╝██╔════╝ ██╔══██╗      ██╔══██╗
             ██║   ██║██╔████╔██║█████╗  ██║  ███╗███████║      ██║  ██║
             ██║   ██║██║╚██╔╝██║██╔══╝  ██║   ██║██╔══██║      ╚██╗██╔╝
             ╚██████╔╝██║ ╚═╝ ██║███████╗╚██████╔╝██║  ██║    ████╔╝╚████╗
              ╚═════╝ ╚═╝     ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝    ╚═══╝  ╚═══╝

|  omega init
|
```

**Behavior:**
1. Print logo in cyan bold (instant, no typewrite)
2. Print empty line
3. Print `|  {subtitle}` with cyan bar and cyan subtitle
4. Print `|` (bare gutter bar for spacing)

**Implementation notes:**
- Logo is printed line-by-line through `Term::stderr()` with `write_line()` for consistent handling
- The logo itself uses `brand()` styling (cyan bold)
- The subtitle line uses `accent()` for the bar and subtitle text

---

### 2. `omega_outro(message)`

**Replaces:** `cliclack::outro("Setup complete")`

**Signature:**
```rust
pub(crate) fn omega_outro(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar, `.cyan().bold()` for message

**Visual output:**
```
|
|  Setup complete
```

**Behavior:**
1. Print `|` (bare gutter bar)
2. Print `|  {message}` with cyan bar and cyan bold message

**Implementation notes:** Outro is deliberately minimal -- the typewrite "enjoy OMEGA" that follows it provides the personality. The outro just closes the structural gutter.

---

### 3. `omega_outro_cancel(message)`

**Replaces:** `cliclack::outro_cancel("Setup aborted")`

**Signature:**
```rust
pub(crate) fn omega_outro_cancel(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar, `.red()` for `x` marker, default for message

**Visual output:**
```
|
x  Setup aborted
```

**Behavior:**
1. Print `|` (bare gutter bar)
2. Print `x  {message}` with red `x` and default-color message

**Implementation notes:** The red `x` replaces cliclack's cancel icon. The gutter switches from `|` to `x` to signal termination.

---

### 4. `omega_success(message)`

**Replaces:** `cliclack::log::success(...)`

**Signature:**
```rust
pub(crate) fn omega_success(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar, `.green()` for `+` marker, default for message

**Visual output:**
```
|  + ~/.omega -- created
```

**Behavior:** Print `|  + {message}` with cyan bar, green `+`, default-color message.

**Marker choice:** `+` (plus) for success. Minimal, immediately readable, no Unicode dependency.

---

### 5. `omega_info(message)`

**Replaces:** `cliclack::log::info(...)`

**Signature:**
```rust
pub(crate) fn omega_info(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar and `-` marker, default for message

**Visual output:**
```
|  - Skipping Telegram -- you can add it later in config.toml
```

**Behavior:** Print `|  - {message}` with cyan bar, cyan `-`, default-color message.

**Marker choice:** `-` (dash) for informational. Neutral, does not demand attention.

---

### 6. `omega_warning(message)`

**Replaces:** `cliclack::log::warning(...)`

**Signature:**
```rust
pub(crate) fn omega_warning(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar, `.yellow()` for `!` marker, default for message

**Visual output:**
```
|  ! config.toml already exists -- skipping.
```

**Behavior:** Print `|  ! {message}` with cyan bar, yellow `!`, default-color message.

**Marker choice:** `!` (exclamation) for warning. Universal symbol, no ambiguity.

---

### 7. `omega_error(message)`

**Replaces:** `cliclack::log::error(...)`

**Signature:**
```rust
pub(crate) fn omega_error(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar, `.red().bold()` for `x` marker, default for message

**Visual output:**
```
|  x timed out -- you can try again later with /whatsapp.
```

**Behavior:** Print `|  x {message}` with cyan bar, red bold `x`, default-color message.

**Marker choice:** `x` (lowercase x) for error. Same as outro_cancel for visual consistency.

---

### 8. `omega_step(message)`

**Replaces:** `cliclack::log::step(...)`

**Signature:**
```rust
pub(crate) fn omega_step(message: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bar and `>` marker, `.bold()` for message

**Visual output:**
```
|  > Starting WhatsApp pairing...
```

**Behavior:** Print `|  > {message}` with cyan bar, cyan `>`, bold message.

**Marker choice:** `>` (chevron) for step/action. Conveys forward movement.

---

### 9. `omega_note(title, body)`

**Replaces:** `cliclack::note(title, body)`

**Signature:**
```rust
pub(crate) fn omega_note(title: &str, body: &str) -> io::Result<()>
```

**`console::Style` used:** `.cyan()` for bars, `.cyan().bold()` for title, `.dim()` for body lines

**Visual output:**
```
|
|  Install claude CLI
|  .  npm install -g @anthropic-ai/claude-code
|  .
|  .  Then run 'omega init' again.
|
```

**Behavior:**
1. Print `|` (spacer)
2. Print `|  {title}` with cyan bar, cyan bold title
3. For each line of body: print `|  .  {line}` with cyan bar, dim dot, dim text
4. Print `|` (spacer)

**Multi-line handling:** Split body on `\n`. Each line gets its own gutter-dot prefix. Empty lines in the body become `|  .` (bare dot, no trailing text).

**QR code rendering:** The QR code body from WhatsApp pairing contains Unicode block characters. The dim styling on body text does NOT interfere with QR rendering because `console::Style` applies to the text string, and Unicode block characters render correctly regardless of dim/bright styling. The QR code is already monochrome (black blocks on terminal background), so dim styling actually helps it blend into the chrome without being visually jarring.

---

### 10. `typewrite(text, delay_ms)`

**Moved from:** `init.rs` line 22-28

**Signature:**
```rust
pub(crate) fn typewrite(text: &str, delay_ms: u64)
```

**`console` used:** `Term::stdout()` for character-by-character output

**Behavior:** Unchanged from current implementation. Prints text character-by-character with `delay_ms` millisecond pause between characters.

**No `io::Result` return:** This function intentionally ignores write errors (uses `let _ = term.write_str(...)`) to match the existing behavior. Typewrite is used for cosmetic "enjoy OMEGA" output only -- failure is not actionable.

---

## Full Module Layout: `init_style.rs`

```rust
//! Branded CLI output helpers for the OMEGA init wizard.
//!
//! Provides styled alternatives to cliclack chrome functions (intro, outro,
//! log::success, etc.) using `console::Style`. Interactive widgets (input,
//! select, confirm, spinner) remain cliclack-managed.
//!
//! ## Tech Debt
//! The following CLI files still use cliclack chrome and are not yet branded:
//! - `main.rs` (status command: 9 calls)
//! - `service.rs` (22 calls)
//! - `selfcheck.rs` (5 calls)
//! - `pair.rs` (10 calls)

use console::{Style, Term};
use std::io;
use std::thread::sleep;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// Cyan accent for gutter bars, info markers, structural chrome.
fn accent() -> Style {
    Style::new().cyan()
}

/// Cyan bold for brand elements (logo, titles, outro).
fn brand() -> Style {
    Style::new().cyan().bold()
}

/// Dimmed for secondary text (note body, continuation dots).
fn muted() -> Style {
    Style::new().dim()
}

// Markers
const BAR: &str = "|";
const DOT: &str = ".";

// ---------------------------------------------------------------------------
// Intro / Outro
// ---------------------------------------------------------------------------

/// Print the OMEGA logo and wizard subtitle.
///
/// Logo is printed in cyan bold (instant). Subtitle appears below
/// the logo with a gutter bar.
pub(crate) fn omega_intro(logo: &str, subtitle: &str) -> io::Result<()> { ... }

/// Close the wizard session with a success message.
pub(crate) fn omega_outro(message: &str) -> io::Result<()> { ... }

/// Close the wizard session with an abort/cancel message.
pub(crate) fn omega_outro_cancel(message: &str) -> io::Result<()> { ... }

// ---------------------------------------------------------------------------
// Status Lines
// ---------------------------------------------------------------------------

/// Green `+` success status line.
pub(crate) fn omega_success(message: &str) -> io::Result<()> { ... }

/// Cyan `-` informational status line.
pub(crate) fn omega_info(message: &str) -> io::Result<()> { ... }

/// Yellow `!` warning status line.
pub(crate) fn omega_warning(message: &str) -> io::Result<()> { ... }

/// Red `x` error status line.
pub(crate) fn omega_error(message: &str) -> io::Result<()> { ... }

/// Cyan `>` step/action indicator.
pub(crate) fn omega_step(message: &str) -> io::Result<()> { ... }

// ---------------------------------------------------------------------------
// Note Box
// ---------------------------------------------------------------------------

/// Titled multi-line note with gutter-dot body.
pub(crate) fn omega_note(title: &str, body: &str) -> io::Result<()> { ... }

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Print text character-by-character for a hacker-terminal feel.
pub(crate) fn typewrite(text: &str, delay_ms: u64) { ... }

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests { ... }
```

**Estimated line count:** ~120-150 production lines (excluding tests). Well under the 500-line limit.

---

## Implementation Pattern: Status Lines

All five status line helpers follow an identical pattern, differing only in marker character and marker style:

```rust
fn status_line(marker: &str, marker_style: Style, message: &str) -> io::Result<()> {
    let term = Term::stderr();
    term.write_line(&format!(
        "{}  {} {}",
        accent().apply_to(BAR),
        marker_style.apply_to(marker),
        message,
    ))
}

pub(crate) fn omega_success(message: &str) -> io::Result<()> {
    status_line("+", Style::new().green(), message)
}

pub(crate) fn omega_warning(message: &str) -> io::Result<()> {
    status_line("!", Style::new().yellow(), message)
}
// ... etc
```

This internal `status_line` helper is `fn` (private), not `pub(crate)`. It prevents duplication across the five status functions without exposing implementation details.

### Why `Term::stderr()`

All output uses `Term::stderr()` (not stdout). This matches cliclack's behavior -- cliclack writes all chrome to stderr so that piping/scripting can capture stdout cleanly. Using stderr ensures the branded chrome coexists properly with cliclack widgets (which also write to stderr).

---

## Integration Pattern

### `main.rs` Change

Add one line:

```rust
// Before:
mod init;
mod init_wizard;

// After:
mod init;
mod init_style;
mod init_wizard;
```

### `init.rs` Changes

#### Before (current):
```rust
use crate::init_wizard;
use crate::service;
use console::Term;
use omega_core::shellexpand;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

const LOGO: &str = r#"..."#;

pub(crate) fn typewrite(text: &str, delay_ms: u64) {
    let term = Term::stdout();
    for ch in text.chars() {
        let _ = term.write_str(&ch.to_string());
        sleep(Duration::from_millis(delay_ms));
    }
    let _ = term.flush();
}

pub async fn run() -> anyhow::Result<()> {
    typewrite(LOGO, 2);
    println!();
    typewrite("omega init\n", 30);
    cliclack::intro("omega init")?;
    // ...
    cliclack::log::success(format!("{data_dir} — created"))?;
    // ...
    cliclack::note("Install claude CLI", "npm install -g ...")?;
    cliclack::outro_cancel("Setup aborted")?;
    // ...
    cliclack::outro("Setup complete")?;
    typewrite("\n  enjoy OMEGA Ω!\n\n", 30);
```

#### After (branded):
```rust
use crate::init_style;
use crate::init_wizard;
use crate::service;
use omega_core::shellexpand;
use std::path::Path;

const LOGO: &str = r#"..."#;

pub async fn run() -> anyhow::Result<()> {
    init_style::omega_intro(LOGO, "omega init")?;
    // ...
    init_style::omega_success(&format!("{data_dir} — created"))?;
    // ...
    init_style::omega_note("Install claude CLI", "npm install -g @anthropic-ai/claude-code\n\nThen run 'omega init' again.")?;
    init_style::omega_outro_cancel("Setup aborted")?;
    // ...
    init_style::omega_outro("Setup complete")?;
    init_style::typewrite("\n  enjoy OMEGA Ω!\n\n", 30);
```

**Key changes:**
- Remove `use console::Term;`, `use std::thread::sleep;`, `use std::time::Duration;` (no longer needed in init.rs)
- Remove the `typewrite()` function definition (moved to init_style.rs)
- Add `use crate::init_style;`
- Replace all 14 cliclack chrome calls with `init_style::` equivalents
- LOGO constant stays in init.rs (it belongs to the init wizard, not to the style module)

### `init_wizard.rs` Changes

#### Before (example call sites):
```rust
cliclack::note("Anthropic setup-token", "Run `claude setup-token` ...")?;
cliclack::log::warning("You can authenticate later with: claude setup-token")?;
cliclack::log::success("Anthropic authentication — already configured")?;
cliclack::log::step("Starting WhatsApp pairing...")?;
cliclack::log::info("Open WhatsApp on your phone > ...")?;
cliclack::note("Scan this QR code with WhatsApp", &qr_text)?;
cliclack::log::error(format!("{e} — you can try again later with /whatsapp."))?;
```

#### After (branded):
```rust
use crate::init_style;

init_style::omega_note("Anthropic setup-token", "Run `claude setup-token` ...")?;
init_style::omega_warning("You can authenticate later with: claude setup-token")?;
init_style::omega_success("Anthropic authentication — already configured")?;
init_style::omega_step("Starting WhatsApp pairing...")?;
init_style::omega_info("Open WhatsApp on your phone > ...")?;
init_style::omega_note("Scan this QR code with WhatsApp", &qr_text)?;
init_style::omega_error(&format!("{e} — you can try again later with /whatsapp."))?;
```

**Key change:** Add `use crate::init_style;` and replace all 19 cliclack chrome calls. No other changes to init_wizard.rs.

---

## Visual Coexistence: Full Wizard Flow Mock

This shows how branded chrome interleaves with cliclack's interactive widgets during a complete wizard run. Branded lines are marked `[OMEGA]`, cliclack widget output is marked `[cliclack]`.

```
              ██████╗ ███╗   ███╗███████╗ ██████╗  █████╗        █████╗         [OMEGA]
             ██╔═══██╗████╗ ████║██╔════╝██╔════╝ ██╔══██╗      ██╔══██╗       [OMEGA]
             ██║   ██║██╔████╔██║█████╗  ██║  ███╗███████║      ██║  ██║       [OMEGA]
             ██║   ██║██║╚██╔╝██║██╔══╝  ██║   ██║██╔══██║      ╚██╗██╔╝      [OMEGA]
             ╚██████╔╝██║ ╚═╝ ██║███████╗╚██████╔╝██║  ██║    ████╔╝╚████╗    [OMEGA]
              ╚═════╝ ╚═╝     ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝    ╚═══╝  ╚═══╝   [OMEGA]
                                                                                 [OMEGA]
|  omega init                                                                    [OMEGA]
|                                                                                [OMEGA]
|  + /Users/you/.omega -- exists                                                 [OMEGA]
◆  Checking claude CLI...                                                        [cliclack spinner]
│  claude CLI -- found                                                           [cliclack spinner.stop]
│                                                                                [cliclack]
◆  Anthropic auth method                                                         [cliclack select]
│  ● Already authenticated (Recommended)                                         [cliclack select]
│  ○ Paste setup-token                                                           [cliclack select]
└                                                                                [cliclack select]
|  + Anthropic authentication -- already configured                              [OMEGA]
│                                                                                [cliclack]
◆  Telegram bot token                                                            [cliclack input]
│  Paste token from @BotFather (or Enter to skip)_                               [cliclack input]
└                                                                                [cliclack input]
|  - Skipping Telegram -- you can add it later in config.toml                    [OMEGA]
│                                                                                [cliclack]
◆  Connect WhatsApp?                                                             [cliclack confirm]
│  No / Yes                                                                      [cliclack confirm]
└                                                                                [cliclack confirm]
|  > Starting WhatsApp pairing...                                                [OMEGA]
|  - Open WhatsApp on your phone > Linked Devices > Link a Device               [OMEGA]
|                                                                                [OMEGA]
|  Scan this QR code with WhatsApp                                               [OMEGA note]
|  .  ██████████████████████████████                                             [OMEGA note]
|  .  ██  ██████████  ██  ████  ████                                             [OMEGA note]
|  .  ██████████████████████████████                                             [OMEGA note]
|                                                                                [OMEGA]
◆  Waiting for scan...                                                           [cliclack spinner]
│  WhatsApp linked successfully                                                  [cliclack spinner.stop]
│                                                                                [cliclack]
◆  Install Omega as a system service?                                            [cliclack confirm]
│  No / Yes                                                                      [cliclack confirm]
└                                                                                [cliclack confirm]
|  + Generated config.toml                                                       [OMEGA]
|                                                                                [OMEGA]
|  Next steps                                                                    [OMEGA note]
|  .  1. Review config.toml                                                      [OMEGA note]
|  .  2. Run: omega start                                                        [OMEGA note]
|  .  3. Send a message to your bot                                              [OMEGA note]
|  .  4. WhatsApp is linked and ready!                                           [OMEGA note]
|                                                                                [OMEGA]
|                                                                                [OMEGA]
|  Setup complete                                                                [OMEGA outro]
                                                                                 [OMEGA]
  enjoy OMEGA Ω!                                                                 [OMEGA typewrite]
```

### Coexistence Analysis

The visual transition between branded chrome and cliclack widgets is **not jarring** because:

1. **Gutter alignment**: The branded `|` bar and cliclack's `│` (box-drawing vertical) occupy the same left-margin position. The gutter is continuous even though the character differs.

2. **Color separation**: Branded chrome is cyan-dominant. cliclack widgets are purple/magenta-dominant. The color change signals "this is interactive now" vs "this is status output" -- which is semantically correct.

3. **No doubled borders**: Branded chrome never wraps content in boxes. cliclack widgets have their own box-drawing. There is no collision.

4. **Spacing**: Every branded note and intro/outro includes leading/trailing `|` spacer lines, creating breathing room before cliclack widgets appear.

---

## Failure Modes

### Module-Level

| Failure | Cause | Detection | Recovery | Impact |
|---------|-------|-----------|----------|--------|
| `Term::write_line` returns `Err` | Broken pipe, terminal closed | `io::Result` propagated via `?` | Caller handles; wizard aborts cleanly | Wizard session ends early |
| Dim/color not rendered | Terminal does not support ANSI | `console` crate auto-detects and falls back to plain text | None needed -- output is readable without color | Visual identity degraded but functional |
| Wide logo wraps on narrow terminal | Terminal width < 80 columns | Not detected at runtime | None -- cosmetic degradation only | Logo looks broken; wizard still works |

### Integration-Level

| Scenario | Affected Modules | Detection | Recovery Strategy | Degraded Behavior |
|----------|-----------------|-----------|-------------------|-------------------|
| cliclack widget fails mid-wizard | init.rs, init_wizard.rs | `?` propagation | Wizard aborts; user reruns | Same as current behavior -- no change |
| Mixed stderr buffering | init_style.rs + cliclack | Visual: output appears interleaved | `Term::stderr()` flushes per line | May see brief visual glitch |

---

## Security Considerations

### Trust Boundary
- **No untrusted input**: All message strings passed to helpers come from hardcoded string literals or formatted system output (directory paths, error messages). No user-controlled input flows into the styling functions without prior formatting by the caller.

### Sensitive Data
- **None**: The style module displays status messages only. It never touches tokens, passwords, or config values.

### Attack Surface
- **None added**: Pure display functions with no network, no file I/O, no deserialization. The attack surface is identical to `println!`.

---

## Performance Budgets

| Operation | Latency Target | Memory | Notes |
|-----------|---------------|--------|-------|
| `omega_intro()` | < 5ms | < 1KB | Instant logo print (no animation) |
| `omega_success/info/warning/error/step()` | < 1ms | < 256B | Single `write_line` call |
| `omega_note()` | < 5ms | < 2KB | Proportional to body line count |
| `omega_outro()` | < 1ms | < 256B | Single `write_line` call |
| `typewrite()` | `len * delay_ms` | < 256B | Only used for "enjoy OMEGA" (~18 chars * 30ms = ~540ms) |

All targets are well within interactive CLI tolerances.

---

## Graceful Degradation

| Dependency | Normal Behavior | Degraded Behavior | User Impact |
|-----------|----------------|-------------------|-------------|
| Terminal ANSI support | Full color (cyan, green, yellow, red) | Plain uncolored text | Functional but generic-looking |
| Terminal width >= 80 | Logo renders correctly | Logo lines wrap | Cosmetic only |
| stderr writable | All output appears | `io::Error` propagated, wizard may abort | User reruns wizard |

---

## Design Decisions

| Decision | Alternatives Considered | Justification |
|----------|------------------------|---------------|
| Cyan as primary accent | Amber/gold, magenta, green | Cyan has maximum dark-terminal contrast, cyberpunk association, and is distinct from cliclack's purple. Amber washes out on light terminals. |
| ASCII markers (`+`, `!`, `x`, `-`, `>`) | Unicode markers (checkmark, warning triangle) | ASCII is universally supported. Unicode markers can render as `?` on misconfigured terminals. "Less is more." |
| `Term::stderr()` for all output | `Term::stdout()`, `println!` | Matches cliclack's stderr convention. Keeps stdout clean for potential piping. |
| No box-drawing characters | Light box-drawing, heavy box-drawing | Boxes are cliclack's visual language. The gutter-bar is OMEGA's. Using boxes would look like a cliclack clone with different colors. |
| Gutter bar `\|` (pipe) | `│` (Unicode box-drawing vertical) | Pipe is ASCII-safe and visually distinct from cliclack's `│`. The intentional difference signals "this is OMEGA chrome, not cliclack chrome." |
| Functions over lazy_static for palette | `lazy_static!`, `const`, `once_cell` | `Style::new().cyan()` is a zero-allocation builder. No need for static caching. Functions are cleaner and have no initialization cost. |
| `io::Result<()>` return type | `anyhow::Result<()>`, no return | Matches the caller's `?` pattern. `io::Error` is the natural error type for terminal writes. Callers already use `?` on cliclack calls. |
| Body lines in `omega_note` use dim styling | No styling, bold, colored | Dim creates visual hierarchy: title is prominent (bold cyan), body is secondary (dim). Prevents note bodies from competing with status lines for attention. |

---

## External Dependencies

- `console` v0.15 -- already in `Cargo.toml` line 69. Used for `Style` and `Term`.
- No new dependencies.

---

## Test Plan

### Unit Tests for `init_style.rs` (VIS-017)

Tests verify that helper functions execute without error and produce non-empty output. Tests do NOT verify exact color codes (terminal-dependent) but DO verify structural content (markers, titles, message text).

| Test | Description | Validates |
|------|-------------|-----------|
| `test_omega_success_returns_ok` | Call `omega_success("test")`, assert `Ok(())` | VIS-003 |
| `test_omega_warning_returns_ok` | Call `omega_warning("test")`, assert `Ok(())` | VIS-005 |
| `test_omega_error_returns_ok` | Call `omega_error("test")`, assert `Ok(())` | VIS-006 |
| `test_omega_info_returns_ok` | Call `omega_info("test")`, assert `Ok(())` | VIS-004 |
| `test_omega_step_returns_ok` | Call `omega_step("test")`, assert `Ok(())` | VIS-007 |
| `test_omega_note_returns_ok` | Call `omega_note("title", "line1\nline2")`, assert `Ok(())` | VIS-008 |
| `test_omega_note_empty_body` | Call `omega_note("title", "")`, assert `Ok(())` | VIS-008 edge case |
| `test_omega_intro_returns_ok` | Call `omega_intro("LOGO", "subtitle")`, assert `Ok(())` | VIS-002 |
| `test_omega_outro_returns_ok` | Call `omega_outro("done")`, assert `Ok(())` | VIS-009 |
| `test_omega_outro_cancel_returns_ok` | Call `omega_outro_cancel("abort")`, assert `Ok(())` | VIS-009 |
| `test_typewrite_does_not_panic` | Call `typewrite("hello", 0)`, no assertion needed | VIS-011 |

**Note:** These tests write to stderr/stdout. In CI, they will produce visible output but this is harmless. The tests verify correctness (no panics, no errors) rather than visual appearance.

---

## Requirement Traceability

| Requirement | Priority | Architecture Section | Module(s) |
|-------------|----------|---------------------|-----------|
| VIS-001 | Must | Module: `init_style.rs` | `backend/src/init_style.rs` |
| VIS-002 | Must | `omega_intro()` | `backend/src/init_style.rs`, `backend/src/init.rs` |
| VIS-003 | Must | `omega_success()` | `backend/src/init_style.rs` |
| VIS-004 | Must | `omega_info()` | `backend/src/init_style.rs` |
| VIS-005 | Must | `omega_warning()` | `backend/src/init_style.rs` |
| VIS-006 | Must | `omega_error()` | `backend/src/init_style.rs` |
| VIS-007 | Must | `omega_step()` | `backend/src/init_style.rs` |
| VIS-008 | Must | `omega_note()` | `backend/src/init_style.rs` |
| VIS-009 | Must | `omega_outro()`, `omega_outro_cancel()` | `backend/src/init_style.rs`, `backend/src/init.rs` |
| VIS-010 | Must | `omega_intro()` — instant print, no typewrite | `backend/src/init.rs` (caller removes `typewrite(LOGO, 2)`) |
| VIS-011 | Should | `typewrite()` — retained for outro only | `backend/src/init_style.rs` |
| VIS-012 | Should | Color Palette — cyan accent, ASCII markers, gutter bar | `backend/src/init_style.rs` |
| VIS-013 | Should | Visual Coexistence — full flow mock | `backend/src/init.rs`, `backend/src/init_wizard.rs` |
| VIS-014 | Must | Integration Pattern — all chrome calls replaced | `backend/src/init.rs`, `backend/src/init_wizard.rs` |
| VIS-015 | Must | Module Layout — ~150 production lines | `backend/src/init_style.rs` |
| VIS-016 | Should | `io::Result<()>` return type on all helpers except `typewrite` | `backend/src/init_style.rs` |
| VIS-017 | Should | Test Plan — 11 tests | `backend/src/init_style.rs` |
| VIS-018 | Must | N/A — existing tests are not affected | existing test suites |
| VIS-019 | Could | Tech Debt doc comment in module header | `backend/src/init_style.rs` |
