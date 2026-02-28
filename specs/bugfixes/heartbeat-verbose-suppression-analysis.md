# BUG: Heartbeat Verbosely Announces Suppression Instead of Silently Skipping

**ID:** BUG-HB-VERBOSE-SUPPRESS
**Severity:** Medium (UX annoyance, defeats purpose of learned rules)
**Reported:** 2026-02-28

## Symptom

Every heartbeat cycle, the user receives messages like:
> "Calisthenics check — Suppressed per standing learned rule: 'Never send unsolicited hydration or health reminders unless the user explicitly asks for them.'"

This is functionally identical to the reminder itself — telling the user "I'm not reminding you about your workouts" IS a reminder about workouts.

## Root Cause

Two compounding issues:

### 1. Prompt gap (primary) — REQ-HBVS-001

`heartbeat_checklist` template in `omega-core/src/config/prompts.rs:95-103` says:
- "If confirmed today, acknowledge briefly instead of nagging"
- "If ANY unconfirmed accountability item exists, do NOT respond with HEARTBEAT_OK"

**Missing instruction:** What to do when a learned behavioral rule **completely suppresses** an item type. The AI has no guidance for this conflict and resolves it by explaining the suppression verbosely.

### 2. Stale checklist item (secondary) — REQ-HBVS-002

`~/.omega/prompts/HEARTBEAT.md` still contains:
> "Daily training accountability — push Antonio to complete his calisthenics session"

The learned rule `[hydration] Never send unsolicited hydration or health reminders...If a health reminder was set up, remove it` should have triggered a `HEARTBEAT_REMOVE:` marker, but the AI never issued one.

## Fix Requirements

| ID | Priority | Description | Acceptance Criteria |
|----|----------|-------------|---------------------|
| REQ-HBVS-001 | Must | Update `heartbeat_checklist` prompt to instruct silent suppression when learned rules block an item | AI produces no user-visible text about suppressed items; counts them as resolved for HEARTBEAT_OK |
| REQ-HBVS-002 | Must | Add prompt instruction that learned rules prohibiting a notification type override checklist items | Items blocked by learned rules are treated as "checked and resolved" |

## Impact Analysis

- **Affected file:** `backend/crates/omega-core/src/config/prompts.rs` (prompt template)
- **No structural code changes needed** — the HEARTBEAT_OK detection logic in `heartbeat.rs` is correct
- **No specs/docs drift** — this is a prompt template bug, not a feature gap
- **Risk:** Low — only changes prompt text, no logic changes
