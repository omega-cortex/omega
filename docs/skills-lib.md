# omega-skills — Developer Guide

## What is this crate?

`omega-skills` is a generic skill loader. It scans `~/.omega/skills/*.md` for skill definitions and makes them available to the AI via the system prompt.

## How It Works

1. **Startup**: `install_bundled_skills(data_dir)` deploys core skills from the binary to `{data_dir}/skills/` (skips existing files)
2. **Load**: `load_skills(data_dir)` scans `{data_dir}/skills/` for `.md` files
3. **Frontmatter**: Each file must have TOML frontmatter between `---` delimiters
4. **Dep check**: Required CLI tools are checked via `which`
5. **Prompt**: `build_skill_prompt()` builds a block appended to the system prompt listing all skills with their install status and file path
6. **On demand**: When the AI needs a skill, it reads the full `.md` file for instructions

## Skill File Format

Create `.md` files in `~/.omega/skills/`:

```markdown
---
name = "gog"
description = "Google Workspace CLI."
requires = ["gog"]
homepage = "https://gogcli.sh"
---

# Full usage instructions here
The AI reads this section when it needs to use the skill.
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Short identifier |
| `description` | Yes | One-line description for the AI |
| `requires` | No | List of CLI tools that must be on `$PATH` |
| `homepage` | No | URL for reference |

## Bot Command

`/skills` — Lists all loaded skills with their availability status.

## Bundled Skills

Core skills live in `skills/` at the repo root and are embedded into the binary at compile time via `include_str!`. On first startup (or after deletion), they are auto-deployed to `~/.omega/skills/`. User edits are never overwritten.

| File | Skill |
|------|-------|
| `google-workspace.md` | Google Workspace CLI (`gog`) |

To add a new bundled skill: create the `.md` file in `skills/`, then add it to the `BUNDLED_SKILLS` const in `crates/omega-skills/src/lib.rs`.

## Design Notes

- **Lean prompt**: Only name + description go into the system prompt. The AI reads the full file on demand.
- **Bundled + user skills**: Core skills ship with the binary; users can add their own `.md` files too.
- **No hot-reload**: Restart Omega to pick up new skill files.
- **Install on demand**: All skills appear in the prompt regardless of install status. The AI can install missing tools by reading the skill file.
- **No per-skill Rust code**: The loader is fully generic — skills are just markdown files.
