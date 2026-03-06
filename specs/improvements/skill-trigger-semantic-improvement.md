# Skill Trigger Semantic Improvement

## Problem

User says "check my excel" but trigger only has "sheets" — no match. Hardcoding every synonym, typo, and language variation is not scalable.

## Key Finding

The `trigger` field serves TWO different populations:

| Skill Type | Has MCP? | Trigger Effect | How AI Activates It |
|------------|----------|----------------|---------------------|
| omg-gog, doli-miner, ibkr-trader, claude-code | No | **Dead code** — `match_skill_triggers` skips skills with no MCP servers | AI reads SKILL.md via bash |
| playwright-mcp | Yes | **Gates MCP server startup** | AI uses MCP tools only if trigger matched |

For 4/5 bundled skills, `trigger` does nothing. The AI already sees all skill descriptions in the system prompt. The problem is prompt-level: Claude isn't explicitly told to use semantic reasoning for skill selection.

## Solution (3 changes)

### Change 1: Enhance `build_skill_prompt()` — semantic matching instruction

Add explicit instruction telling Claude to match user **intent** to skills, not just keywords. "excel" → Sheets → omg-gog.

### Change 2: Fix SYSTEM_PROMPT.md hardcoded routing

Line 48 references wrong path `skills/google-workspace/SKILL.md`. Replace hardcoded per-skill routing with generic intent-based instruction.

### Change 3: Always activate all MCP servers for Claude Code CLI

For Claude Code CLI (dominant provider), MCP activation is just a config write — essentially free. Always include all available MCP servers instead of gating on keyword match. For HTTP providers, keep trigger-based filtering (real per-message cost).

## Requirements

| ID | Priority | Requirement |
|----|----------|-------------|
| ST-1 | Must | `build_skill_prompt()` includes semantic matching instruction |
| ST-2 | Must | SYSTEM_PROMPT.md uses generic intent-based routing, no hardcoded skill paths |
| ST-3 | Must | Claude Code CLI provider always receives all available MCP servers |
| ST-4 | Must | HTTP providers retain trigger-based MCP filtering (cost control) |
| ST-5 | Must | All existing tests pass unchanged |
| ST-6 | Should | Remove `trigger` field from non-MCP bundled skills (dead code cleanup) |

## Impact

- `backend/crates/omega-skills/src/skills.rs` — `build_skill_prompt()`
- `backend/src/gateway/pipeline.rs` — MCP server injection
- `prompts/SYSTEM_PROMPT.md` — routing instructions
- `skills/omg-gog/SKILL.md`, `skills/doli-miner/SKILL.md`, `skills/ibkr-trader/SKILL.md` — remove dead `trigger` field
