//! # omega-sandbox
//!
//! OS-level system protection for the Omega agent.
//!
//! Uses a **blocklist** approach: everything is allowed by default, then
//! dangerous system directories and OMEGA's core data are blocked.
//!
//! - **macOS**: Apple Seatbelt via `sandbox-exec -p <profile>` — denies reads
//!   and writes to `{data_dir}/data/` (memory.db) and `config.toml`; denies
//!   writes to `/System`, `/bin`, `/sbin`, `/usr/{bin,sbin,lib,libexec}`,
//!   `/private/etc`, `/Library`.
//! - **Linux**: Landlock LSM via `pre_exec` hook (kernel 5.13+) — broad
//!   read-only on `/` with full access to `$HOME`, `/tmp`, `/var/tmp`, `/opt`,
//!   `/srv`, `/run`, `/media`, `/mnt`; restricted access to `{data_dir}/data/`
//!   and `config.toml`.
//! - **Other**: Falls back to a plain command with a warning.
//!
//! Also provides [`is_write_blocked`] and [`is_read_blocked`] for code-level
//! enforcement in HTTP provider tool executors (protects memory.db and
//! config.toml on all platforms).

use std::path::{Path, PathBuf};
use tokio::process::Command;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
use tracing::warn;

#[cfg(target_os = "macos")]
mod seatbelt;

#[cfg(target_os = "linux")]
mod landlock_sandbox;

/// Build a [`Command`] with OS-level system protection.
///
/// Always active — blocks writes to dangerous system directories and
/// OMEGA's core data directory (memory.db). No configuration needed.
///
/// `data_dir` is the Omega data directory (`~/.omega/`). Writes to
/// `{data_dir}/data/` are blocked (protects memory.db). All other
/// paths under `data_dir` (workspace, skills, projects) remain writable.
///
/// On unsupported platforms, logs a warning and returns a plain command.
pub fn protected_command(program: &str, data_dir: &Path) -> Command {
    platform_command(program, data_dir)
}

/// Best-effort path canonicalization. Returns the canonicalized path or the
/// original if canonicalization fails (file doesn't exist yet, permissions, etc.).
fn try_canonicalize(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Check if a write to the given path should be blocked.
///
/// Returns `true` if the path targets a protected location:
/// - Dangerous OS directories (`/System`, `/bin`, `/sbin`, `/usr/bin`, etc.)
/// - OMEGA's core data directory (`{data_dir}/data/`) — protects memory.db
///
/// Resolves symlinks before comparison to prevent bypass via symlink chains.
/// Used by the HTTP provider `ToolExecutor` for code-level enforcement.
pub fn is_write_blocked(path: &Path, data_dir: &Path) -> bool {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Fail closed: relative paths could bypass protection via traversal.
        return true;
    };

    // Resolve symlinks for both target and protected paths.
    let resolved = try_canonicalize(&abs);

    // Block writes to OMEGA's core data directory (memory.db, etc.).
    let data_data = try_canonicalize(&data_dir.join("data"));
    if resolved.starts_with(&data_data) {
        return true;
    }

    // Block writes to OMEGA's config file (API keys, auth settings).
    let config_file = try_canonicalize(&data_dir.join("config.toml"));
    if resolved == config_file {
        return true;
    }

    // Block writes to dangerous OS directories.
    // Uses Path::starts_with (component-aware) instead of string prefix matching
    // to prevent false positives like "/binaries/test" matching "/bin".
    let blocked_prefixes: &[&str] = &[
        "/System",
        "/bin",
        "/sbin",
        "/usr/bin",
        "/usr/sbin",
        "/usr/lib",
        "/usr/libexec",
        "/private/etc",
        "/Library",
        "/etc",
        "/boot",
        "/proc",
        "/sys",
        "/dev",
    ];

    for prefix in blocked_prefixes {
        if resolved.starts_with(prefix) {
            return true;
        }
    }

    false
}

/// Check if a read from the given path should be blocked.
///
/// Returns `true` if the path targets a protected location:
/// - OMEGA's core data directory (`{data_dir}/data/`) — protects memory.db
/// - OMEGA's config file (`{data_dir}/config.toml`) — protects API keys
/// - The actual config file at `config_path` (may differ from data_dir) — protects secrets
///
/// Resolves symlinks before comparison to prevent bypass via symlink chains.
/// Used by the HTTP provider `ToolExecutor` for code-level enforcement.
pub fn is_read_blocked(path: &Path, data_dir: &Path, config_path: Option<&Path>) -> bool {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Fail closed: relative paths could bypass protection via traversal.
        return true;
    };

    // Resolve symlinks for both target and protected paths.
    let resolved = try_canonicalize(&abs);

    // Block reads to OMEGA's core data directory (memory.db, etc.).
    let data_data = try_canonicalize(&data_dir.join("data"));
    if resolved.starts_with(&data_data) {
        return true;
    }

    // Block reads to OMEGA's config file in data_dir (API keys, secrets).
    let config_in_data = try_canonicalize(&data_dir.join("config.toml"));
    if resolved == config_in_data {
        return true;
    }

    // Block reads to the actual config file (may live outside data_dir).
    if let Some(cp) = config_path {
        let resolved_config = try_canonicalize(cp);
        if resolved == resolved_config {
            return true;
        }
    }

    false
}

/// Dispatch to the platform-specific protection implementation.
#[cfg(target_os = "macos")]
fn platform_command(program: &str, data_dir: &Path) -> Command {
    seatbelt::protected_command(program, data_dir)
}

/// Dispatch to the platform-specific protection implementation.
#[cfg(target_os = "linux")]
fn platform_command(program: &str, data_dir: &Path) -> Command {
    landlock_sandbox::protected_command(program, data_dir)
}

/// Fallback for unsupported platforms.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn platform_command(program: &str, _data_dir: &Path) -> Command {
    warn!("OS-level protection not available on this platform; using code-level enforcement only");
    Command::new(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_protected_command_returns_command() {
        let data_dir = PathBuf::from("/tmp/ws");
        let cmd = protected_command("claude", &data_dir);
        let program = cmd.as_std().get_program().to_string_lossy().to_string();
        assert!(!program.is_empty());
    }

    #[test]
    fn test_is_write_blocked_data_dir() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(is_write_blocked(
            Path::new("/home/user/.omega/data/memory.db"),
            &data_dir
        ));
        assert!(is_write_blocked(
            Path::new("/home/user/.omega/data/"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_write_blocked_allows_workspace() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(!is_write_blocked(
            Path::new("/home/user/.omega/workspace/test.txt"),
            &data_dir
        ));
        assert!(!is_write_blocked(
            Path::new("/home/user/.omega/skills/test/SKILL.md"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_write_blocked_system_dirs() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(is_write_blocked(
            Path::new("/System/Library/test"),
            &data_dir
        ));
        assert!(is_write_blocked(Path::new("/bin/sh"), &data_dir));
        assert!(is_write_blocked(Path::new("/usr/bin/env"), &data_dir));
        assert!(is_write_blocked(Path::new("/private/etc/hosts"), &data_dir));
        assert!(is_write_blocked(
            Path::new("/Library/Preferences/test"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_write_blocked_allows_normal_paths() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(!is_write_blocked(Path::new("/tmp/test"), &data_dir));
        assert!(!is_write_blocked(
            Path::new("/home/user/documents/test"),
            &data_dir
        ));
        assert!(!is_write_blocked(
            Path::new("/usr/local/bin/something"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_write_blocked_no_string_prefix_false_positive() {
        // Path::starts_with is component-aware: "/binaries" should NOT match "/bin".
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(!is_write_blocked(Path::new("/binaries/test"), &data_dir));
    }

    #[test]
    fn test_is_write_blocked_relative_path() {
        let data_dir = PathBuf::from("/home/user/.omega");
        // Relative paths are blocked (fail closed) to prevent traversal bypass.
        assert!(is_write_blocked(Path::new("relative/path"), &data_dir));
        assert!(is_write_blocked(
            Path::new("../../data/memory.db"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_write_blocked_config_toml() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(is_write_blocked(
            Path::new("/home/user/.omega/config.toml"),
            &data_dir
        ));
    }

    #[test]
    fn test_is_read_blocked_data_dir() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(is_read_blocked(
            Path::new("/home/user/.omega/data/memory.db"),
            &data_dir,
            None
        ));
        assert!(is_read_blocked(
            Path::new("/home/user/.omega/data/"),
            &data_dir,
            None
        ));
    }

    #[test]
    fn test_is_read_blocked_config() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(is_read_blocked(
            Path::new("/home/user/.omega/config.toml"),
            &data_dir,
            None
        ));
    }

    #[test]
    fn test_is_read_blocked_external_config() {
        let data_dir = PathBuf::from("/home/user/.omega");
        let ext_config = PathBuf::from("/opt/omega/config.toml");
        assert!(is_read_blocked(
            Path::new("/opt/omega/config.toml"),
            &data_dir,
            Some(ext_config.as_path())
        ));
        // Non-matching path still allowed.
        assert!(!is_read_blocked(
            Path::new("/opt/omega/other.toml"),
            &data_dir,
            Some(ext_config.as_path())
        ));
    }

    #[test]
    fn test_is_read_blocked_allows_workspace() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(!is_read_blocked(
            Path::new("/home/user/.omega/workspace/test.txt"),
            &data_dir,
            None
        ));
        assert!(!is_read_blocked(
            Path::new("/home/user/.omega/skills/test/SKILL.md"),
            &data_dir,
            None
        ));
    }

    #[test]
    fn test_is_read_blocked_allows_stores() {
        let data_dir = PathBuf::from("/home/user/.omega");
        assert!(!is_read_blocked(
            Path::new("/home/user/.omega/stores/trading/store.db"),
            &data_dir,
            None
        ));
    }

    #[test]
    fn test_is_read_blocked_relative_path() {
        let data_dir = PathBuf::from("/home/user/.omega");
        // Relative paths are blocked (fail closed) to prevent traversal bypass.
        assert!(is_read_blocked(Path::new("relative/path"), &data_dir, None));
        assert!(is_read_blocked(
            Path::new("../../data/memory.db"),
            &data_dir,
            None
        ));
    }
}
