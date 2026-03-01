# Functionalities: omega-skills

## Overview

Skill and project loader for Omega. Skills are loaded from `~/.omega/skills/*/SKILL.md` and provide tool definitions, MCP servers, and trigger patterns. Projects are loaded from `~/.omega/projects/*/ROLE.md` and define domain contexts with per-project skills and heartbeats.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | load_skills() | Service | `backend/crates/omega-skills/src/skills.rs` | Loads skill definitions from SKILL.md files in skills directory | parse module |
| 2 | install_bundled_skills() | Service | `backend/crates/omega-skills/src/skills.rs` | Deploys bundled skills to runtime directory (never overwrites) | -- |
| 3 | migrate_flat_skills() | Service | `backend/crates/omega-skills/src/skills.rs` | Migrates legacy flat skill files to structured directories | -- |
| 4 | build_skill_prompt() | Service | `backend/crates/omega-skills/src/skills.rs` | Builds skill listing for system prompt injection | Skill |
| 5 | match_skill_triggers() | Service | `backend/crates/omega-skills/src/skills.rs` | Matches message text against skill triggers to activate MCP servers | Skill |
| 6 | Skill | Model | `backend/crates/omega-skills/src/skills.rs` | Skill definition: name, description, path, triggers, available flag, MCP servers | McpServer |
| 7 | load_projects() | Service | `backend/crates/omega-skills/src/projects.rs` | Loads project definitions from ROLE.md files in projects directory | parse module |
| 8 | ensure_projects_dir() | Service | `backend/crates/omega-skills/src/projects.rs` | Creates projects directory if it doesn't exist | -- |
| 9 | get_project_instructions() | Service | `backend/crates/omega-skills/src/projects.rs` | Retrieves ROLE.md instructions for a named project | Project |
| 10 | Project | Model | `backend/crates/omega-skills/src/projects.rs` | Project definition: name, path, instructions, skills (declared by project) | -- |
| 11 | Parse module | Library | `backend/crates/omega-skills/src/parse.rs` | SKILL.md and ROLE.md parsing logic | -- |

## Internal Dependencies

- Gateway loads skills at startup and hot-reloads projects per message
- Skills inject MCP servers and tool definitions into provider context
- Projects inject ROLE.md into system prompt when active
- match_skill_triggers() produces McpServer list for Context

## Dead Code / Unused

- None detected.
