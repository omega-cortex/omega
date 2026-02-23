---
name: workflow:new
description: Start a new project from scratch with the full workflow
---

# Workflow: New Project

The user wants to create something new from scratch. Execute the full chain:

## Step 1: Analyst
Invoke the `analyst` subagent with the user's description.
The analyst reads `specs/SPECS.md` index first, then scopes to relevant areas.
Wait for it to generate the requirements document with confirmed assumptions.
Save output to `specs/[domain]-requirements.md` and update `specs/SPECS.md` index.

## Step 2: Architect
Once the analyst completes, invoke the `architect` subagent passing the requirements document.
The architect reads only the scoped codebase (defined by analyst's scope).
Wait for it to generate the architecture document.
Save specs to `specs/[domain].md` and update `specs/SPECS.md`.
Save docs to `docs/[topic].md` and update `docs/DOCS.md`.

## Step 3: Test Writer
Once the architect completes, invoke the `test-writer` subagent passing the architecture.
The test-writer works one module at a time, saving tests to disk after each.
Wait for it to generate all tests (they must fail initially).

## Step 4: Developer
Once tests are written, invoke the `developer` subagent.
The developer works one module at a time: read tests → implement → run tests → commit → next.
Must implement module by module until all tests pass.
If context gets heavy mid-implementation, commit progress and continue.

## Step 5: Reviewer
Once all code passes the tests, invoke the `reviewer` subagent.
The reviewer works module by module, saving findings incrementally.
Wait for the review report, including specs/docs drift check.
Save output to `docs/reviews/[name]-review.md`.

## Step 6: Iteration
If the reviewer finds critical issues:
- Return to the developer with the findings
- The developer fixes them (scoped to the affected module only)
- The reviewer reviews again (scoped to the fix only)
- Repeat until approved

## Step 7: Versioning
Once approved, create the final commit and version tag.
Clean up `docs/.workflow/` temporary files.
