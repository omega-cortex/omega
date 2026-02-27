# Improvement: Build Pipeline Safety Controls

> Strengthen the runtime build pipeline's safety controls: QA retry loop (3 iterations),
> reviewer loop (2 iterations, fatal), inter-step validation, chain state recovery,
> and agent prompt improvements (MoSCoW, failure modes, report output, specs/docs drift).

## Scope

**Domains affected:** gateway (builds)

**Files affected:**
- `backend/src/gateway/builds_parse.rs` — `ReviewResult` enum, `ChainState` struct, `parse_review_result()`, 6 i18n message functions
- `backend/src/gateway/builds_loop.rs` — NEW: QA loop (3 iter), review loop (2 iter), phase validation, chain state persistence
- `backend/src/gateway/builds.rs` — refactored to use loop methods, added inter-step validation before phases 3/4/5
- `backend/src/gateway/builds_agents.rs` — analyst (MoSCoW), architect (failure modes), QA (opus + report), reviewer (opus + Write + report + drift)
- `backend/src/gateway/mod.rs` — register `builds_loop` module

## Changes

### QA Loop (3 iterations)
- Moved from inline 1-retry to `run_qa_loop()` with 3 iterations
- Each failed iteration: sends localized retry message, re-invokes developer with failure context
- On exhaustion: returns error, caller saves chain state and aborts

### Review Loop (2 iterations, fatal)
- Changed from non-fatal single pass to `run_review_loop()` with 2 iterations
- Review failure now blocks delivery (was previously just a warning)
- Same retry pattern as QA: re-invoke developer with review findings

### Inter-step Validation
- Before test-writer: checks `specs/architecture.md` exists
- Before developer: checks test files exist
- Before QA: checks source files exist
- Validation failure saves chain state for debugging

### Chain State Recovery
- `save_chain_state()` writes `docs/.workflow/chain-state.md` on any pipeline failure
- Records completed phases, failed phase, failure reason
- Best-effort (warns on I/O error, doesn't propagate)

### Agent Prompt Improvements
- **Analyst:** MoSCoW priority prefixes on all requirements (`[Must]`, `[Should]`, `[Could]`)
- **Architect:** failure modes, security boundaries, performance constraints per module
- **QA:** upgraded to `model: opus`, writes `docs/qa-report.md`, exploratory testing
- **Reviewer:** upgraded to `model: opus`, Write tool added, writes `docs/review-report.md`, specs/docs drift check

### i18n
All 6 new message functions support all 8 languages (English, Spanish, Portuguese, French, German, Italian, Dutch, Russian).

## Risk Assessment
- Reviewer becoming fatal could block builds that previously succeeded — mitigated by 2 retry iterations and chain state for partial recovery
- QA/Reviewer model upgrade to Opus increases cost/latency — can be reverted independently
