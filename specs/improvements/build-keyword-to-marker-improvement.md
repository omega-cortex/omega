# Build Keyword to Marker Improvement

**Status:** IMPLEMENTED
**Date:** 2026-03-09
**Supersedes:** build-discovery-phase-improvement.md, build-autonomy-improvement.md (partially)

## Summary

Replaced hardcoded build keyword matching (107 entries across 8 languages + typos) with
marker-based intent detection via the existing `BUILD_PROPOSAL:` marker. The AI naturally
understands user intent in any language — no keyword list can compete with that.

## What Changed

### Removed (~500 lines)
- `BUILDS_KW` keyword array (107 entries) from `keywords_data.rs`
- `DISCOVERY_TTL_SECS` constant from `keywords_data.rs`
- `handle_pending_discovery()` (~220 lines) from `pipeline_builds.rs`
- `handle_build_keyword_discovery()` (~150 lines) from `pipeline_builds.rs`
- `build_confirm_message()` from `keywords.rs` (only used in keyword path)
- 5 discovery i18n functions from `keywords.rs`
- `DiscoveryOutput` enum and 4 discovery parse functions from `builds_parse.rs`
- ~25 discovery-related tests
- `"pending_discovery"` from `SYSTEM_FACT_KEYS`
- `needs_builds` keyword detection and early-return in `pipeline.rs`

### Modified
- `prompt_builder.rs`: `prompts.builds` always injected (was conditional on keyword match)
- `config/prompts.rs`: Updated hardcoded builds default — emit BUILD_PROPOSAL immediately, no pre-clarification
- `SYSTEM_PROMPT.md`: Replaced `## Builds` section — emit BUILD_PROPOSAL immediately, pipeline handles clarification
- `context_command.rs`: Builds section always shows as ON
- `pipeline.rs`: Removed `needs_builds` from logging and function calls

### Preserved (unchanged)
- `BUILD_PROPOSAL` marker processing in `process_markers.rs`
- `handle_pending_build_confirmation()` — confirmation gate with TTL
- `handle_build_request()` — topology-driven multi-phase build pipeline
- `BUILD_CONFIRM_KW`, `BUILD_CANCEL_KW`, `BUILD_CONFIRM_TTL_SECS`
- `is_build_confirmed()`, `is_build_cancelled()`, `build_cancelled_message()`

## New Flow

```
User: "Quiero que hagas una GUI de task.py"
  → Normal conversation pipeline (no keyword interception)
  → AI understands intent, emits BUILD_PROPOSAL immediately
  → AI emits: BUILD_PROPOSAL: GUI application for task.py
  → Gateway processes marker → stores pending_build_request fact
  → Next message: user says "sí"
  → handle_pending_build_confirmation() → build pipeline starts
```

## Why

The keyword list was a losing game. Every conjugation, dialect variation, and phrasing
required a code change. "Quiero que hagas una GUI" didn't match because only "hazme"
(imperative+pronoun) was covered, not "hagas" (subjunctive). The AI already understands
intent natively — the keyword gate was preventing it from doing its job.
