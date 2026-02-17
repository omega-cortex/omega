# Technical Specification: `omega-skills/src/lib.rs`

## File

| Field | Value |
|-------|-------|
| **Path** | `crates/omega-skills/src/lib.rs` |
| **Crate** | `omega-skills` |
| **Role** | Generic skill loader — scans `~/.omega/skills/*/SKILL.md` and exposes them to the system prompt |

## Purpose

Loads skill definitions from `SKILL.md` files inside per-skill directories with TOML frontmatter. Each skill file declares a name, description, required CLI tools, and optional homepage. The loader checks whether required tools are installed and builds a prompt block that tells the AI what skills exist and where to read full instructions.

## Public API

| Item | Kind | Description |
|------|------|-------------|
| `Skill` | struct | Loaded skill definition (name, description, requires, homepage, available, path) |
| `install_bundled_skills(data_dir)` | fn | Deploy bundled core skills to `{data_dir}/skills/{name}/SKILL.md`, creating subdirs if needed. Never overwrites existing files. |
| `migrate_flat_skills(data_dir)` | fn | Auto-migrate legacy flat `.md` files to `{name}/SKILL.md` directory layout. Skips if target dir exists. |
| `load_skills(data_dir)` | fn | Scan `{data_dir}/skills/*/SKILL.md`, parse frontmatter, check deps, return `Vec<Skill>` |
| `build_skill_prompt(skills)` | fn | Build the system prompt block listing all skills with install status |

## Skill Directory Format

Skills are stored as directories in `{data_dir}/skills/` with a `SKILL.md` file containing TOML frontmatter between `---` delimiters:

```
~/.omega/skills/
├── google-workspace/
│   └── SKILL.md
└── my-tool/
    └── SKILL.md
```

```markdown
---
name = "gog"
description = "Google Workspace CLI."
requires = ["gog"]
homepage = "https://gogcli.sh"
---

(Body text — full instructions the AI reads on demand)
```

## Bundled Skills

Core skills are embedded at compile time from `skills/` in the repo root via `include_str!`. On startup, `install_bundled_skills()` writes them to `{data_dir}/skills/{name}/SKILL.md` only if absent, preserving user edits.

| Directory | Skill |
|-----------|-------|
| `skills/google-workspace/SKILL.md` | Google Workspace CLI (`gog`) |

## Internal Functions

| Function | Description |
|----------|-------------|
| `parse_skill_file(content)` | Extract and deserialize TOML frontmatter from `---` delimiters |
| `which_exists(tool)` | Check if a CLI tool exists on `$PATH` via `which` |
| `expand_tilde(path)` | Expand `~` to `$HOME` in data_dir paths |

## Dependencies

| Dependency | Usage |
|------------|-------|
| `serde` | Deserialize TOML frontmatter |
| `toml` | Parse TOML |
| `tracing` | Warn on invalid skill files |

## Projects

In addition to skills, this crate also handles project loading. Projects are user-defined instruction scopes stored in `~/.omega/projects/`.

### Public API (Projects)

| Item | Kind | Description |
|------|------|-------------|
| `Project` | struct | Loaded project definition (name, instructions, path) |
| `ensure_projects_dir(data_dir)` | fn | Create `{data_dir}/projects/` directory if missing |
| `load_projects(data_dir)` | fn | Scan `{data_dir}/projects/*/INSTRUCTIONS.md`, return `Vec<Project>` sorted by name |
| `get_project_instructions(projects, name)` | fn | Find project by name, return `Option<&str>` of its instructions |

### Project Directory Format

```
~/.omega/projects/
├── real-estate/
│   └── INSTRUCTIONS.md      # "You are a real estate analyst..."
├── nutrition/
│   └── INSTRUCTIONS.md      # "You are a nutrition coach..."
└── stocks/
    └── INSTRUCTIONS.md      # "You track my portfolio..."
```

- **Project name** = directory name
- **Instructions** = contents of `INSTRUCTIONS.md` (trimmed, must be non-empty)
- Directories without `INSTRUCTIONS.md` or with empty instructions are skipped
- Projects are loaded at startup (restart to pick up new ones)

## Tests

- Valid frontmatter parsing
- Missing frontmatter returns None
- Empty requires defaults to empty vec
- Empty skill list produces empty prompt
- Prompt format with installed/not-installed status (paths use `*/SKILL.md`)
- `which` detection for known and unknown tools
- Missing skills directory returns empty vec
- Valid skill directory with SKILL.md loads correctly
- Flat skill migration moves `.md` → `{name}/SKILL.md`, skips existing dirs
- Bundled skills deploy to `{name}/SKILL.md`, never overwrite existing files
- Missing projects directory returns empty vec
- Valid project with INSTRUCTIONS.md loads correctly
- Empty INSTRUCTIONS.md is skipped
- Directory without INSTRUCTIONS.md is skipped
- `get_project_instructions()` returns correct instructions or None
