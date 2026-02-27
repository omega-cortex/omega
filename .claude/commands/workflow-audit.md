---
name: workflow:audit
description: Audit existing code without modifying it. Accepts optional --scope to limit to a milestone/module.
---

# Workflow: Audit

Invoke ONLY the `reviewer` subagent in full audit mode.
Optional: `--scope="milestone or module"` to audit a specific area.

## Prerequisite Fallback
If `specs/SPECS.md` does not exist:
- **Don't fail.** Proceed with a code-only audit.
- Note in the audit report: "No specs/SPECS.md found. Audit is based on codebase analysis only. Specs drift checks were skipped."
- The reviewer should still check for the existence of docs and report their absence as a finding.

## Without scope (full audit)
The reviewer MUST work in chunks to avoid context limits:
1. Read `specs/SPECS.md` to get the list of milestones/domains (if it exists; if not, derive structure from directory layout)
2. For each milestone:
   a. Review the code for that milestone
   b. Review corresponding specs and docs
   c. Save findings to `docs/.workflow/audit-[milestone].md`
   d. Clear mental context before next milestone
3. Compile all milestone findings into the final report
4. Save to `docs/audits/audit-[date].md`
5. Clean up `docs/.workflow/audit-*.md` temporary files

## With scope (targeted audit)
The reviewer works only within the specified area:
1. Read only the scoped code, specs, and docs
2. Generate the audit report for that area
3. Save to `docs/audits/audit-[scope]-[date].md`

## Audit Covers
- Security vulnerabilities
- Performance issues
- Technical debt
- Dead code
- Missing tests
- Suggested improvements
- Specs/docs drift (specs that don't match code, missing specs, orphaned docs)
