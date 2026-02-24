---
name: workflow:new
description: Start a new project from scratch with the full workflow
---

# Workflow: New Project

The user wants to create something new from scratch. Execute the full chain.

**This is a greenfield project.** There may be no existing code, no `specs/`, and no `docs/`. Each agent must handle this gracefully — creating structure instead of reading it.

## Step 1: Analyst
Invoke the `analyst` subagent with the user's description.
1. If `specs/SPECS.md` exists, read it to understand any existing project layout
2. If `specs/SPECS.md` does NOT exist, skip codebase reading — this is a new project
3. Focus entirely on questioning the idea, clarifying requirements, and identifying risks
4. Generate the requirements document with:
   - Unique requirement IDs (REQ-[DOMAIN]-[NNN])
   - MoSCoW priorities (Must/Should/Could/Won't)
   - Acceptance criteria for every requirement
   - User stories where applicable
   - Impact analysis (if any existing code)
   - Initial traceability matrix
5. Create `specs/` directory if it doesn't exist
6. Save output to `specs/[domain]-requirements.md` and create/update `specs/SPECS.md` index

## Step 2: Architect
Once the analyst completes, invoke the `architect` subagent passing the requirements document.
1. If this is a new project (no existing code), design the full project structure:
   - Create `backend/` (and `frontend/` if needed) directory layout
   - Define module structure, interfaces, dependencies
2. Design failure modes and recovery strategies per module
3. Define security considerations and trust boundaries
4. Set performance budgets and complexity targets
5. Plan graceful degradation behavior
6. Create `specs/` and `docs/` scaffolding if they don't exist
7. Save specs to `specs/[domain].md` and create/update `specs/SPECS.md`
8. Save docs to `docs/[topic].md` and create/update `docs/DOCS.md`
9. Update the traceability matrix with architecture sections

## Step 3: Test Writer
Once the architect completes, invoke the `test-writer` subagent passing the architecture.
The test-writer works one module at a time, saving tests to disk after each.
1. Tests Must requirements first (exhaustive coverage + edge cases)
2. Tests Should requirements second (solid coverage)
3. Tests Could requirements last (basic happy path)
4. Every test references a requirement ID for traceability
5. Tests cover failure modes and security from the architect's design
6. Updates the traceability matrix with test IDs
7. All tests must fail initially (red phase)

## Step 4: Developer
Once tests are written, invoke the `developer` subagent.
The developer works one module at a time: read tests → implement → run tests → commit → next.
Must implement module by module until all tests pass.
If context gets heavy mid-implementation, commit progress and continue.

## Step 5: QA
Once all code passes the tests, invoke the `qa` subagent.
1. Verify the traceability matrix is complete (every Must/Should has tests and code)
2. Verify acceptance criteria for every Must requirement
3. Verify acceptance criteria for every Should requirement
4. Run end-to-end flows that cross module boundaries
5. Perform exploratory testing — try what no test anticipated
6. Validate failure modes and recovery strategies
7. Validate security considerations
8. Generate QA report at `docs/qa/[name]-qa-report.md`

## Step 6: QA Iteration
If QA finds blocking issues (Must requirements failing, broken flows):
- Return to the developer with the QA findings
- The developer fixes them (scoped to the affected area only)
- QA re-validates (scoped to the fix only)
- Repeat until QA approves

## Step 7: Reviewer
Once QA approves, invoke the `reviewer` subagent.
The reviewer works module by module, saving findings incrementally.
Wait for the review report, including specs/docs drift check.
Save output to `docs/reviews/[name]-review.md`.

## Step 8: Review Iteration
If the reviewer finds critical issues:
- Return to the developer with the findings
- The developer fixes them (scoped to the affected module only)
- The reviewer reviews again (scoped to the fix only)
- Repeat until approved

## Step 9: Versioning
Once approved, create the final commit and version tag.
Clean up `docs/.workflow/` temporary files.
