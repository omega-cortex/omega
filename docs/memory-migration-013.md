# Migration 013: Multi-Lesson Support

## What Changed
The `lessons` table no longer has a `UNIQUE(sender_id, domain, project)` constraint. This means multiple distinct rules can be stored per domain per project, enabling richer long-term learning.

## Why
Previously, each `LESSON: trading|<rule>` marker **replaced** the previous trading lesson for that user and project. This forced the AI to dump accumulated knowledge into `HEARTBEAT.md` instead, causing unbounded growth. Now each distinct insight becomes its own persistent row.

## How It Works

### Content Deduplication
If the AI emits the exact same rule text that already exists, the existing row's `occurrences` counter is incremented instead of creating a duplicate. This tracks reinforcement strength.

### Cap Enforcement
A maximum of **10 lessons per (sender_id, domain, project)** is enforced after each insert. When the cap is exceeded, the oldest lessons (by `updated_at`) are pruned. This prevents unbounded growth while keeping the most recent and actively reinforced rules.

### Query Safety
All lesson query functions (`get_lessons()`, `get_all_lessons()`) include a `LIMIT 50` safety cap to prevent prompt bloat even if multiple domains accumulate many lessons.

## Migration Details
The migration recreates the `lessons` table without the UNIQUE constraint, preserving all existing data. It follows the same `CREATE new → INSERT SELECT → DROP old → RENAME` pattern used by migration 011.

## Prompt Changes
`SYSTEM_PROMPT.md` was updated with guidance:
- LESSON markers should be used for durable rules (multiple per domain allowed)
- HEARTBEAT_ADD should only be used for temporary monitoring items
- HEARTBEAT_ADD must NOT be used as a scratchpad for accumulated knowledge

## Verification
```sql
-- Check multiple lessons per domain
sqlite3 ~/.omega/data/memory.db "SELECT domain, rule, occurrences FROM lessons WHERE sender_id = 'YOUR_ID' ORDER BY domain, updated_at DESC;"

-- Check cap enforcement (should never exceed 10 per group)
sqlite3 ~/.omega/data/memory.db "SELECT sender_id, domain, project, COUNT(*) as cnt FROM lessons GROUP BY sender_id, domain, project HAVING cnt > 10;"
```
