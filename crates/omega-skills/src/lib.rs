//! # omega-skills
//!
//! Generic skill loader for Omega. Scans `~/.omega/skills/*.md` for skill
//! definitions and exposes them to the system prompt so the AI knows what
//! tools are available.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Bundled core skills — embedded at compile time from `skills/` in the repo root.
const BUNDLED_SKILLS: &[(&str, &str)] = &[(
    "google-workspace.md",
    include_str!("../../../skills/google-workspace.md"),
)];

/// Deploy bundled skills to `{data_dir}/skills/`, creating the directory if needed.
///
/// Never overwrites existing files so user edits are preserved.
pub fn install_bundled_skills(data_dir: &str) {
    let dir = Path::new(&expand_tilde(data_dir)).join("skills");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("skills: failed to create {}: {e}", dir.display());
        return;
    }
    for (filename, content) in BUNDLED_SKILLS {
        let dest = dir.join(filename);
        if !dest.exists() {
            if let Err(e) = std::fs::write(&dest, content) {
                warn!("skills: failed to write {}: {e}", dest.display());
            } else {
                info!("skills: installed bundled skill {filename}");
            }
        }
    }
}

/// A loaded skill definition.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Short identifier (e.g. "gog").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// CLI tools this skill depends on.
    pub requires: Vec<String>,
    /// Homepage URL (informational).
    pub homepage: String,
    /// Whether all required CLIs are available on `$PATH`.
    pub available: bool,
    /// Absolute path to the skill file.
    pub path: PathBuf,
}

/// TOML frontmatter parsed from a skill `.md` file.
#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    homepage: String,
}

/// Scan `{data_dir}/skills/*.md` and return all valid skill definitions.
pub fn load_skills(data_dir: &str) -> Vec<Skill> {
    let dir = Path::new(&expand_tilde(data_dir)).join("skills");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                warn!("skills: failed to read {}: {e}", path.display());
                continue;
            }
        };
        let Some(fm) = parse_skill_file(&content) else {
            warn!("skills: no valid frontmatter in {}", path.display());
            continue;
        };
        let available = fm.requires.iter().all(|t| which_exists(t));
        skills.push(Skill {
            name: fm.name,
            description: fm.description,
            requires: fm.requires,
            homepage: fm.homepage,
            available,
            path,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Build the skill block appended to the system prompt.
///
/// Returns an empty string if there are no skills.
pub fn build_skill_prompt(skills: &[Skill]) -> String {
    if skills.is_empty() {
        return String::new();
    }

    let mut out = String::from(
        "\n\nYou have the following skills available. \
         Before using any skill, you MUST read its file for full instructions. \
         If a tool is not installed, the skill file contains installation \
         instructions — install it first, then use it.\n\nSkills:\n",
    );

    for s in skills {
        let status = if s.available {
            "installed"
        } else {
            "not installed"
        };
        out.push_str(&format!(
            "- {} [{}]: {} → Read {}\n",
            s.name,
            status,
            s.description,
            s.path.display(),
        ));
    }

    out
}

/// Expand `~` to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return format!("{}/{rest}", home.to_string_lossy());
        }
    }
    path.to_string()
}

/// Extract TOML frontmatter delimited by `---` lines.
fn parse_skill_file(content: &str) -> Option<SkillFrontmatter> {
    let trimmed = content.trim_start();
    let rest = trimmed.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    let toml_block = &rest[..end];
    toml::from_str(toml_block).ok()
}

/// Check whether a CLI tool exists on `$PATH`.
fn which_exists(tool: &str) -> bool {
    std::process::Command::new("which")
        .arg(tool)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = "\
---
name = \"gog\"
description = \"Google Workspace CLI.\"
requires = [\"gog\"]
homepage = \"https://gogcli.sh\"
---

Some body text.
";
        let fm = parse_skill_file(content).unwrap();
        assert_eq!(fm.name, "gog");
        assert_eq!(fm.description, "Google Workspace CLI.");
        assert_eq!(fm.requires, vec!["gog"]);
        assert_eq!(fm.homepage, "https://gogcli.sh");
    }

    #[test]
    fn test_parse_no_frontmatter() {
        assert!(parse_skill_file("Just plain text.").is_none());
    }

    #[test]
    fn test_parse_empty_requires() {
        let content = "\
---
name = \"simple\"
description = \"No deps.\"
---
";
        let fm = parse_skill_file(content).unwrap();
        assert!(fm.requires.is_empty());
    }

    #[test]
    fn test_build_skill_prompt_empty() {
        assert!(build_skill_prompt(&[]).is_empty());
    }

    #[test]
    fn test_build_skill_prompt_formats_correctly() {
        let skills = vec![
            Skill {
                name: "gog".into(),
                description: "Google Workspace CLI.".into(),
                requires: vec!["gog".into()],
                homepage: "https://gogcli.sh".into(),
                available: true,
                path: PathBuf::from("/home/user/.omega/skills/gog.md"),
            },
            Skill {
                name: "missing".into(),
                description: "Not installed tool.".into(),
                requires: vec!["nope".into()],
                homepage: String::new(),
                available: false,
                path: PathBuf::from("/home/user/.omega/skills/missing.md"),
            },
        ];
        let prompt = build_skill_prompt(&skills);
        assert!(prompt.contains("gog [installed]"));
        assert!(prompt.contains("missing [not installed]"));
        assert!(prompt.contains("Read /home/user/.omega/skills/gog.md"));
    }

    #[test]
    fn test_which_exists_known_tool() {
        // `ls` should exist on any Unix system.
        assert!(which_exists("ls"));
    }

    #[test]
    fn test_which_exists_missing_tool() {
        assert!(!which_exists("__omega_nonexistent_tool_42__"));
    }

    #[test]
    fn test_load_skills_missing_dir() {
        let skills = load_skills("/tmp/__omega_test_no_such_dir__");
        assert!(skills.is_empty());
    }

    #[test]
    fn test_install_bundled_skills_creates_files() {
        let tmp = std::env::temp_dir().join("__omega_test_bundled__");
        let _ = std::fs::remove_dir_all(&tmp);
        install_bundled_skills(tmp.to_str().unwrap());
        let dest = tmp.join("skills/google-workspace.md");
        assert!(dest.exists(), "bundled skill should be deployed");
        let content = std::fs::read_to_string(&dest).unwrap();
        assert!(content.contains("google-workspace"));
        // Run again — should not overwrite.
        std::fs::write(&dest, "custom").unwrap();
        install_bundled_skills(tmp.to_str().unwrap());
        let after = std::fs::read_to_string(&dest).unwrap();
        assert_eq!(after, "custom", "should not overwrite user edits");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
