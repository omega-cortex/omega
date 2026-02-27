---
name: workflow:post-commit-audit
description: Post-commit self-healing audit loop. Runs automatically after development workflows. Accepts --scope (required).
---

# Workflow: Post-Commit Audit (Self-Healing Loop)

Automatically audits the most recent commit and fixes Critical findings in a bounded loop.
This workflow is invoked by other development workflows after their final commit — do not run it standalone unless testing.

**Required:** `--scope="area"` — the scope from the parent workflow's Analyst step.
**Internal:** `--round=N` (default 1) — current iteration. Do not set manually.

## Iteration Guard

1. Parse `--round` (default: 1)
2. If `--round > 2`: jump directly to **Max Iterations Reached** (Step 5)

## Step 1: Scoped Audit

Invoke the `reviewer` subagent in **Post-Commit Audit Mode**:
1. Identify changed files: `git diff HEAD~1 --name-only`
2. Cross-reference with `--scope` — only review files that match both
3. The reviewer focuses on Correctness, Security, and Technical Debt
4. The reviewer produces a report ending with `AUDIT-VERDICT: clean` or `AUDIT-VERDICT: requires-fix`

Save the report to: `docs/audits/post-commit-audit-[scope]-r[round]-[date].md`
(Replace spaces/slashes in scope with hyphens. Date format: YYYY-MM-DD.)

## Step 2: Parse Verdict

Read the saved audit report. Find the `AUDIT-VERDICT:` line.

- **If `AUDIT-VERDICT: clean`:**
  - Report to user: "Post-commit audit passed (round [round]/2). No Critical findings."
  - Clean up `docs/.workflow/` temporary files
  - **STOP** — workflow complete

- **If `AUDIT-VERDICT: requires-fix`:**
  - Report to user: "Post-commit audit found Critical findings (round [round]/2). Starting auto-fix cycle."
  - Proceed to Step 3

## Step 3: Reduced Bugfix Cycle

### 3a: Analyst
Invoke the `analyst` subagent:
1. Read the audit report from Step 1
2. Treat each **Critical finding** as a bug to fix (ignore Minor findings)
3. Generate fix requirements with IDs and acceptance criteria
4. Save to `specs/bugfixes/post-commit-fix-r[round].md`

### 3b: Developer
Invoke the `developer` subagent:
1. Read the fix requirements from 3a
2. Fix each Critical finding — scoped to the affected files only
3. Run existing tests to verify no regressions
4. Commit with message: `fix(audit-r[round]): resolve post-commit audit findings in [scope]`

### 3c: QA
Invoke the `qa` subagent:
1. Validate that each Critical finding from the audit is resolved
2. Verify existing tests still pass
3. Generate report at `docs/qa/post-commit-fix-r[round]-qa.md`
4. If QA finds the fix broke something:
   - Developer gets **one retry** to fix the regression
   - If still broken after retry, **escalate to user**: "Auto-fix introduced a regression that could not be resolved. Manual intervention needed. See: [QA report path]"
   - **STOP** — do not continue to re-audit

## Step 4: Re-audit (Loop Back)

Invoke this workflow again with incremented round:
`/workflow:post-commit-audit --scope="[same scope]" --round=[round + 1]`

## Step 5: Max Iterations Reached

When `--round > 2`:
1. Report to user:
   > Post-commit audit loop reached maximum iterations (2/2).
   > Remaining findings in: `docs/audits/post-commit-audit-[scope]-r2-[date].md`
   > These may require a full `/workflow:bugfix` or manual intervention.
2. Clean up `docs/.workflow/` temporary files
3. **STOP**
