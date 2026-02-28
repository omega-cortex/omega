# backend/src/init_style.rs — Branded CLI Output Helpers Specification

## Path
`backend/src/init_style.rs`

## Purpose
Branded CLI output helpers for the OMEGA init wizard. Provides styled alternatives to cliclack chrome functions (intro, outro, log::success, log::info, log::warning, log::error, log::step, note) using `console::Style`. Interactive widgets (input, select, confirm, spinner) remain cliclack-managed in `init.rs` and `init_wizard.rs`.

## Module Overview

### Color Palette (private)
- `accent()` -- `Style::new().cyan()` -- Gutter bars, info markers, structural chrome
- `brand()` -- `Style::new().cyan().bold()` -- Logo, titles, outro messages
- `muted()` -- `Style::new().dim()` -- Note body text, continuation dots

### Gutter Characters (private)
- `BAR` = `"|"` -- Cyan-styled vertical gutter bar
- `DOT` = `"."` -- Dim-styled continuation marker for note body lines

### Private Helper
- `status_line(marker, marker_style, message)` -- Prints `|  {marker} {message}` with multi-line gutter support

### Public Interface (`pub(crate)`)

| Function | Signature | Replaces | Visual |
|----------|-----------|----------|--------|
| `omega_intro` | `(logo: &str, subtitle: &str) -> io::Result<()>` | `cliclack::intro` + typewrite logo | Cyan bold logo (instant), gutter bar + subtitle |
| `omega_outro` | `(message: &str) -> io::Result<()>` | `cliclack::outro` | Gutter bar + cyan bold message |
| `omega_outro_cancel` | `(message: &str) -> io::Result<()>` | `cliclack::outro_cancel` | Gutter bar + red `x` + message |
| `omega_success` | `(message: &str) -> io::Result<()>` | `cliclack::log::success` | `\|  + message` (green `+`) |
| `omega_info` | `(message: &str) -> io::Result<()>` | `cliclack::log::info` | `\|  - message` (cyan `-`) |
| `omega_warning` | `(message: &str) -> io::Result<()>` | `cliclack::log::warning` | `\|  ! message` (yellow `!`) |
| `omega_error` | `(message: &str) -> io::Result<()>` | `cliclack::log::error` | `\|  x message` (red bold `x`) |
| `omega_step` | `(message: &str) -> io::Result<()>` | `cliclack::log::step` | `\|  > message` (cyan `>`, bold message) |
| `omega_note` | `(title: &str, body: &str) -> io::Result<()>` | `cliclack::note` | Titled block with gutter-dot body lines |
| `typewrite` | `(text: &str, delay_ms: u64)` | (moved from init.rs) | Character-by-character animation |

### Output Target
All functions except `typewrite` write to `Term::stderr()` to match cliclack's convention. `typewrite` uses `Term::stdout()`.

## Called By
- `backend/src/init.rs` -- 14 call sites (replaces cliclack chrome)
- `backend/src/init_wizard.rs` -- 19 call sites (replaces cliclack chrome)

## Dependencies
- `console` v0.15 (`Style`, `Term`) -- already in Cargo.toml
- `std::io` -- `io::Result` return type
- `std::thread::sleep`, `std::time::Duration` -- for `typewrite()`

## Tech Debt
The following CLI files still use cliclack chrome and are not yet branded:
- `main.rs` (status command: 9 calls)
- `service.rs` (22 calls)
- `selfcheck.rs` (5 calls)
- `pair.rs` (10 calls)

## Unit Tests (11 tests)

| Test | Validates |
|------|-----------|
| `test_omega_success_returns_ok` | VIS-003: success helper returns Ok |
| `test_omega_warning_returns_ok` | VIS-005: warning helper returns Ok |
| `test_omega_error_returns_ok` | VIS-006: error helper returns Ok |
| `test_omega_info_returns_ok` | VIS-004: info helper returns Ok |
| `test_omega_step_returns_ok` | VIS-007: step helper returns Ok |
| `test_omega_note_returns_ok` | VIS-008: note with multi-line body returns Ok |
| `test_omega_note_empty_body` | VIS-008: note with empty body returns Ok |
| `test_omega_intro_returns_ok` | VIS-002: intro with logo and subtitle returns Ok |
| `test_omega_outro_returns_ok` | VIS-009: outro returns Ok |
| `test_omega_outro_cancel_returns_ok` | VIS-009: outro cancel returns Ok |
| `test_typewrite_does_not_panic` | VIS-011: typewrite with zero delay completes |
