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
// Private helper
// ---------------------------------------------------------------------------

/// Print a status line: `|  {marker} {message}`.
/// Multi-line messages get the gutter bar on each continuation line.
fn status_line(marker: &str, marker_style: Style, message: &str) -> io::Result<()> {
    let term = Term::stderr();
    let mut first = true;
    for line in message.lines() {
        if first {
            term.write_line(&format!(
                "{}  {} {}",
                accent().apply_to(BAR),
                marker_style.apply_to(marker),
                line,
            ))?;
            first = false;
        } else {
            term.write_line(&format!("{}    {}", accent().apply_to(BAR), line,))?;
        }
    }
    if first {
        // Empty message — just print the marker line.
        term.write_line(&format!(
            "{}  {}",
            accent().apply_to(BAR),
            marker_style.apply_to(marker),
        ))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Intro / Outro
// ---------------------------------------------------------------------------

/// Print the OMEGA logo and wizard subtitle.
///
/// Logo is printed in cyan bold (instant). Subtitle appears below
/// the logo with a gutter bar.
pub(crate) fn omega_intro(logo: &str, subtitle: &str) -> io::Result<()> {
    let term = Term::stderr();
    for line in logo.lines() {
        term.write_line(&format!("{}", brand().apply_to(line)))?;
    }
    term.write_line("")?;
    term.write_line(&format!(
        "{}  {}",
        accent().apply_to(BAR),
        accent().apply_to(subtitle),
    ))?;
    term.write_line(&format!("{}", accent().apply_to(BAR)))
}

/// Close the wizard session with a success message.
pub(crate) fn omega_outro(message: &str) -> io::Result<()> {
    let term = Term::stderr();
    term.write_line(&format!("{}", accent().apply_to(BAR)))?;
    term.write_line(&format!(
        "{}  {}",
        accent().apply_to(BAR),
        brand().apply_to(message),
    ))
}

/// Close the wizard session with an abort/cancel message.
pub(crate) fn omega_outro_cancel(message: &str) -> io::Result<()> {
    let term = Term::stderr();
    term.write_line(&format!("{}", accent().apply_to(BAR)))?;
    term.write_line(&format!(
        "{}  {}",
        Style::new().red().apply_to("x"),
        message,
    ))
}

// ---------------------------------------------------------------------------
// Status Lines
// ---------------------------------------------------------------------------

/// Green `+` success status line.
pub(crate) fn omega_success(message: &str) -> io::Result<()> {
    status_line("+", Style::new().green(), message)
}

/// Cyan `-` informational status line.
pub(crate) fn omega_info(message: &str) -> io::Result<()> {
    status_line("-", Style::new().cyan(), message)
}

/// Yellow `!` warning status line.
pub(crate) fn omega_warning(message: &str) -> io::Result<()> {
    status_line("!", Style::new().yellow(), message)
}

/// Red `x` error status line.
pub(crate) fn omega_error(message: &str) -> io::Result<()> {
    status_line("x", Style::new().red().bold(), message)
}

/// Cyan `>` step/action indicator.
pub(crate) fn omega_step(message: &str) -> io::Result<()> {
    let term = Term::stderr();
    term.write_line(&format!(
        "{}  {} {}",
        accent().apply_to(BAR),
        accent().apply_to(">"),
        Style::new().bold().apply_to(message),
    ))
}

// ---------------------------------------------------------------------------
// Note Box
// ---------------------------------------------------------------------------

/// Titled multi-line note with gutter-dot body.
pub(crate) fn omega_note(title: &str, body: &str) -> io::Result<()> {
    let term = Term::stderr();
    term.write_line(&format!("{}", accent().apply_to(BAR)))?;
    term.write_line(&format!(
        "{}  {}",
        accent().apply_to(BAR),
        brand().apply_to(title),
    ))?;
    for line in body.lines() {
        if line.is_empty() {
            term.write_line(&format!(
                "{}  {}",
                accent().apply_to(BAR),
                muted().apply_to(DOT),
            ))?;
        } else {
            term.write_line(&format!(
                "{}  {}  {}",
                accent().apply_to(BAR),
                muted().apply_to(DOT),
                muted().apply_to(line),
            ))?;
        }
    }
    term.write_line(&format!("{}", accent().apply_to(BAR)))
}

// ---------------------------------------------------------------------------
// Animation
// ---------------------------------------------------------------------------

/// Print text character-by-character for a hacker-terminal feel.
pub(crate) fn typewrite(text: &str, delay_ms: u64) {
    let term = Term::stdout();
    for ch in text.chars() {
        let _ = term.write_str(&ch.to_string());
        sleep(Duration::from_millis(delay_ms));
    }
    let _ = term.flush();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ===================================================================
    // VIS-003 (Must): omega_success returns Ok
    // ===================================================================

    // Requirement: VIS-003 (Must)
    // Acceptance: Branded success status line executes without error
    #[test]
    fn test_omega_success_returns_ok() {
        let result = omega_success("test message");
        assert!(result.is_ok(), "omega_success must not return an error");
    }

    // ===================================================================
    // VIS-005 (Must): omega_warning returns Ok
    // ===================================================================

    // Requirement: VIS-005 (Must)
    // Acceptance: Branded warning status line executes without error
    #[test]
    fn test_omega_warning_returns_ok() {
        let result = omega_warning("test warning");
        assert!(result.is_ok(), "omega_warning must not return an error");
    }

    // ===================================================================
    // VIS-006 (Must): omega_error returns Ok
    // ===================================================================

    // Requirement: VIS-006 (Must)
    // Acceptance: Branded error status line executes without error
    #[test]
    fn test_omega_error_returns_ok() {
        let result = omega_error("test error");
        assert!(result.is_ok(), "omega_error must not return an error");
    }

    // ===================================================================
    // VIS-004 (Must): omega_info returns Ok
    // ===================================================================

    // Requirement: VIS-004 (Must)
    // Acceptance: Branded info status line executes without error
    #[test]
    fn test_omega_info_returns_ok() {
        let result = omega_info("test info");
        assert!(result.is_ok(), "omega_info must not return an error");
    }

    // ===================================================================
    // VIS-007 (Must): omega_step returns Ok
    // ===================================================================

    // Requirement: VIS-007 (Must)
    // Acceptance: Branded step indicator executes without error
    #[test]
    fn test_omega_step_returns_ok() {
        let result = omega_step("test step");
        assert!(result.is_ok(), "omega_step must not return an error");
    }

    // ===================================================================
    // VIS-008 (Must): omega_note returns Ok
    // ===================================================================

    // Requirement: VIS-008 (Must)
    // Acceptance: Branded note box with multi-line body executes without error
    #[test]
    fn test_omega_note_returns_ok() {
        let result = omega_note("title", "line1\nline2");
        assert!(result.is_ok(), "omega_note must not return an error");
    }

    // Requirement: VIS-008 (Must)
    // Edge case: Empty body string should not panic or error
    #[test]
    fn test_omega_note_empty_body() {
        let result = omega_note("title", "");
        assert!(
            result.is_ok(),
            "omega_note with empty body must not return an error"
        );
    }

    // ===================================================================
    // VIS-002 (Must): omega_intro returns Ok
    // ===================================================================

    // Requirement: VIS-002 (Must)
    // Acceptance: Branded intro with logo and subtitle executes without error
    #[test]
    fn test_omega_intro_returns_ok() {
        let result = omega_intro("LOGO", "subtitle");
        assert!(result.is_ok(), "omega_intro must not return an error");
    }

    // ===================================================================
    // VIS-009 (Must): omega_outro and omega_outro_cancel return Ok
    // ===================================================================

    // Requirement: VIS-009 (Must)
    // Acceptance: Branded outro executes without error
    #[test]
    fn test_omega_outro_returns_ok() {
        let result = omega_outro("done");
        assert!(result.is_ok(), "omega_outro must not return an error");
    }

    // Requirement: VIS-009 (Must)
    // Acceptance: Branded outro cancel executes without error
    #[test]
    fn test_omega_outro_cancel_returns_ok() {
        let result = omega_outro_cancel("abort");
        assert!(
            result.is_ok(),
            "omega_outro_cancel must not return an error"
        );
    }

    // ===================================================================
    // VIS-011 (Should): typewrite does not panic
    // ===================================================================

    // Requirement: VIS-011 (Should)
    // Acceptance: typewrite with zero delay completes without panic
    #[test]
    fn test_typewrite_does_not_panic() {
        typewrite("hello", 0);
    }
}
