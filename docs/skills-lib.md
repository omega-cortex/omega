# omega-skills — Developer Guide

## What is this crate?

`omega-skills` is a generic skill loader. It scans `~/.omega/skills/*.md` for skill definitions and makes them available to the AI via the system prompt.

## How It Works

1. **Startup**: `load_skills(data_dir)` scans `{data_dir}/skills/` for `.md` files
2. **Frontmatter**: Each file must have TOML frontmatter between `---` delimiters
3. **Dep check**: Required CLI tools are checked via `which`
4. **Prompt**: `build_skill_prompt()` builds a block appended to the system prompt listing all skills with their install status and file path
5. **On demand**: When the AI needs a skill, it reads the full `.md` file for instructions

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

## Design Notes

- **Lean prompt**: Only name + description go into the system prompt. The AI reads the full file on demand.
- **No hot-reload**: Restart Omega to pick up new skill files.
- **Install on demand**: All skills appear in the prompt regardless of install status. The AI can install missing tools by reading the skill file.
- **No per-skill Rust code**: The loader is fully generic — skills are just markdown files.
