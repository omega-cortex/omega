# Technical Specification: `omega-skills/src/lib.rs`

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-skills/src/lib.rs` |
| **Crate** | `omega-skills` |
| **Role** | Generic skill loader — scans `~/.omega/skills/*.md` and exposes them to the system prompt |

## Purpose

Loads skill definitions from markdown files with TOML frontmatter. Each skill file declares a name, description, required CLI tools, and optional homepage. The loader checks whether required tools are installed and builds a prompt block that tells the AI what skills exist and where to read full instructions.

## Public API

| Item | Kind | Description |
|------|------|-------------|
| `Skill` | struct | Loaded skill definition (name, description, requires, homepage, available, path) |
| `load_skills(data_dir)` | fn | Scan `{data_dir}/skills/*.md`, parse frontmatter, check deps, return `Vec<Skill>` |
| `build_skill_prompt(skills)` | fn | Build the system prompt block listing all skills with install status |

## Skill File Format

Files in `{data_dir}/skills/*.md` with TOML frontmatter between `---` delimiters:

```markdown
---
name = "gog"
description = "Google Workspace CLI."
requires = ["gog"]
homepage = "https://gogcli.sh"
---

(Body text — full instructions the AI reads on demand)
```

## Internal Functions

| Function | Description |
|----------|-------------|
| `parse_skill_file(content)` | Extract and deserialize TOML frontmatter from `---` delimiters |
| `which_exists(tool)` | Check if a CLI tool exists on `$PATH` via `which` |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `serde` | Deserialize TOML frontmatter |
| `toml` | Parse TOML |
| `tracing` | Warn on invalid skill files |

## Tests

- Valid frontmatter parsing
- Missing frontmatter returns None
- Empty requires defaults to empty vec
- Empty skill list produces empty prompt
- Prompt format with installed/not-installed status
- `which` detection for known and unknown tools
- Missing skills directory returns empty vec
