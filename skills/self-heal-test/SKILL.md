---
name: self-heal-test
description: Regression test for the SELF_HEAL verification pipeline. Use when asked to test self-healing, run a self-heal regression test, or verify the self-healing pipeline works end-to-end.
---

# Self-Heal Pipeline Regression Test

Run a functional end-to-end test of the SELF_HEAL verification pipeline.

## What This Tests

1. `SELF_HEAL: description | verification test` marker emission and gateway parsing
2. `~/.omega/self-healing.json` state file creation with `verification` field
3. Follow-up task scheduling with verification test embedded in the prompt
4. `SELF_HEAL_RESOLVED` marker processing and state cleanup

## Test Procedure

### Step 1: Create a test anomaly

Create a harmless flag file that represents a fake anomaly:

```bash
echo "TEST ANOMALY: created at $(date -u +%Y-%m-%dT%H:%M:%SZ)" > ~/.omega/self-healing-test.flag
```

### Step 2: Schedule an action task that emits the marker

Insert a scheduled action task due in 1 minute that instructs the AI to emit the raw `SELF_HEAL:` marker:

```bash
DUE=$(date -u -v+1M +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -d '+1 minute' +%Y-%m-%dT%H:%M:%SZ)
sqlite3 ~/.omega/memory.db "INSERT INTO scheduled_tasks (id, channel, sender_id, reply_target, description, due_at, repeat, status, task_type) VALUES (
  'selfheal-regression-test',
  'telegram',
  '$(sqlite3 ~/.omega/memory.db "SELECT sender_id FROM audit_log ORDER BY id DESC LIMIT 1;")',
  '$(sqlite3 ~/.omega/memory.db "SELECT sender_id FROM audit_log ORDER BY id DESC LIMIT 1;")',
  'IMPORTANT: Include this EXACT line on its own line in your response — do NOT fix anything, just emit the marker:

SELF_HEAL: regression test anomaly | check if ~/.omega/self-healing-test.flag exists and confirm it is gone

After emitting the marker, say: Marker emitted for regression test. Do NOT attempt to fix anything or emit SELF_HEAL_RESOLVED.',
  '$DUE',
  NULL,
  'pending',
  'action'
);"
```

### Step 3: Wait and verify (within ~2 minutes)

After the scheduler fires (~60s poll interval + provider response time), verify:

**Check 1 — State file created with verification field:**
```bash
cat ~/.omega/self-healing.json
```
Expected: JSON with `"anomaly": "regression test anomaly"` and `"verification": "check if ~/.omega/self-healing-test.flag exists and confirm it is gone"` and `"iteration": 1`.

**Check 2 — Follow-up task scheduled with verification in prompt:**
```bash
sqlite3 ~/.omega/memory.db "SELECT description FROM scheduled_tasks WHERE description LIKE '%regression test anomaly%' AND status='pending' ORDER BY rowid DESC LIMIT 1;"
```
Expected: Contains `Run this verification: check if ~/.omega/self-healing-test.flag exists and confirm it is gone`.

**Check 3 — Owner notification sent:**
```bash
grep 'self-heal: scheduled verification task' ~/.omega/omega.log | tail -1
```
Expected: Log line with `(iteration 1)`.

### Step 4: Cleanup

Cancel the follow-up task and remove test artifacts:

```bash
sqlite3 ~/.omega/memory.db "UPDATE scheduled_tasks SET status='delivered' WHERE description LIKE '%regression test anomaly%' AND status='pending';"
rm -f ~/.omega/self-healing.json ~/.omega/self-healing-test.flag
```

## Pass Criteria

All three checks must pass:
- State file exists with both `anomaly` and `verification` fields
- Follow-up task contains the verification test in its description
- Gateway logged `self-heal: scheduled verification task ... (iteration 1)`

## Why Direct Chat Testing Fails

Sending "emit SELF_HEAL:" via Telegram chat usually fails because:
- Sonnet (DIRECT path) refuses to emit raw markers for test requests — the system prompt restricts self-healing to "genuine infrastructure/code bugs"
- The action task scheduler path bypasses this restriction because the AI receives the marker text as part of its task instruction
