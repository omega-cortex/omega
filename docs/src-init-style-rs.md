# backend/src/init_style.rs — Branded CLI Output Helpers

## Overview

`init_style.rs` provides branded terminal output helpers for the OMEGA init wizard. It replaces cliclack's generic chrome functions (intro, outro, log::success, etc.) with OMEGA-branded equivalents using `console::Style` and a cyan gutter-bar visual language.

Interactive widgets (input, select, confirm, spinner) remain cliclack-managed. Only the surrounding "chrome" -- status lines, note blocks, intro/outro sequences -- uses the branded helpers.

## Visual Language

- **Gutter bar** (`|`) in cyan on the left margin provides visual continuity
- **Status markers**: `+` (green/success), `-` (cyan/info), `!` (yellow/warning), `x` (red/error), `>` (cyan/step)
- **Note blocks**: titled sections with dim `.` continuation dots for body lines
- **Logo**: printed instantly in cyan bold (no animation)
- **Outro signature**: `typewrite()` for character-by-character "enjoy OMEGA" animation

## Available Helpers

| Function | Purpose |
|----------|---------|
| `omega_intro(logo, subtitle)` | Branded logo + session opener |
| `omega_outro(message)` | Session close (success) |
| `omega_outro_cancel(message)` | Session close (abort) |
| `omega_success(message)` | Green `+` status line |
| `omega_info(message)` | Cyan `-` status line |
| `omega_warning(message)` | Yellow `!` status line |
| `omega_error(message)` | Red `x` status line |
| `omega_step(message)` | Cyan `>` step indicator |
| `omega_note(title, body)` | Titled note with gutter-dot body |
| `typewrite(text, delay_ms)` | Character-by-character animation |

## Used By

- `backend/src/init.rs` -- 14 call sites
- `backend/src/init_wizard.rs` -- 19 call sites

## Tech Debt

The following CLI files still use generic cliclack chrome and have not been branded:
- `main.rs` (9 calls), `service.rs` (22 calls), `selfcheck.rs` (5 calls), `pair.rs` (10 calls)

## Related

- [Technical specification](../specs/src-init-style-rs.md)
- [Init wizard documentation](src-init-rs.md)
