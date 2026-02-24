---
name: workflow:feature
description: Add a feature to an existing project. Accepts optional --scope to limit context.
---

# Workflow: New Feature

The user wants to add functionality to existing code.
Optional: `--scope="area"` to limit which part of the codebase is analyzed.

## Step 1: Analyst
Invoke the `analyst` subagent. It MUST:
1. Read `specs/SPECS.md` index (not all files)
2. If `--scope` provided, read only that area's specs and code
3. If no `--scope`, determine minimal scope from the task description
4. Flag any drift between code and specs/docs
5. Perform impact analysis — what existing code/behavior is affected
6. Ask questions considering the current architecture
7. Generate requirements with IDs, MoSCoW priorities, acceptance criteria, and user stories
8. Build the traceability matrix
9. Explicitly state the scope in the requirements document

Save output to `specs/[domain]-requirements.md` and update `specs/SPECS.md`.

## Step 2: Architect
Invoke the `architect` subagent.
1. Read the Analyst's requirements (scope, priorities, and IDs are defined there)
2. Read only the scoped codebase and specs
3. Design the architecture including failure modes, security, and performance budgets
4. Update existing spec files or create new ones in `specs/`
5. Define how the new feature integrates with what already exists
6. Plan graceful degradation where applicable
7. Update `docs/` with new documentation
8. Update both master indexes (SPECS.md and DOCS.md)
9. Update the traceability matrix with architecture sections

## Step 3: Test Writer
Invoke the `test-writer` subagent.
1. Read requirements with IDs, priorities, and acceptance criteria
2. Test Must requirements first, then Should, then Could
3. Every test references a requirement ID
4. Cover failure modes and security from the architect's design
5. Update the traceability matrix with test IDs
6. All previous tests must continue passing (regression)

## Step 4: Developer
Invoke the `developer` subagent.
Work within the scope defined by the analyst.
Module by module: read tests → implement → run tests → commit → next.

## Step 5: QA
Invoke the `qa` subagent.
1. Verify traceability matrix completeness
2. Verify acceptance criteria for Must and Should requirements
3. Run end-to-end flows including integration with existing functionality
4. Perform exploratory testing
5. Validate failure modes and security
6. Generate QA report

## Step 6: QA Iteration
If QA finds blocking issues:
- Developer fixes → QA re-validates (scoped to fix only)
- Repeat until QA approves

## Step 7: Reviewer
Invoke the `reviewer` subagent.
The reviewer specifically checks that specs/ and docs/ were updated for the new feature.
All work within the scope defined by the analyst.

## Step 8: Review Iteration
If the reviewer finds critical issues:
- Developer fixes → reviewer re-reviews (scoped to fix only)
- Repeat until approved

## Step 9: Versioning
Once approved, create the final commit and version tag.
Clean up `docs/.workflow/` temporary files.
