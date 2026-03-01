# Functionalities: Keywords

## Overview

Keyword matching system that gates conditional prompt sections to reduce token usage by ~55-70%. Nine keyword categories determine which system prompt sections and context blocks are loaded.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | kw_match() | Utility | `backend/src/gateway/keywords.rs` | Matches message text against a keyword array (case-insensitive substring matching) | -- |
| 2 | SCHEDULING_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for scheduling section: remind, schedule, alarm, timer, etc. | -- |
| 3 | RECALL_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for recall section: remember, recall, last time, previously, etc. | -- |
| 4 | TASKS_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for tasks section: task, pending, upcoming, etc. | -- |
| 5 | PROJECTS_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for projects section: project, role, domain, etc. | -- |
| 6 | BUILDS_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for builds section: build, create, scaffold, develop, etc. | -- |
| 7 | META_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for meta section: skill, bug, whatsapp, heartbeat, etc. | -- |
| 8 | PROFILE_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for profile section: name, timezone, preference, etc. | -- |
| 9 | OUTCOMES_KW | Constant | `backend/src/gateway/keywords_data.rs` | Keywords for outcomes section: outcome, reward, learn, lesson, etc. | -- |
| 10 | is_build_confirmed() | Utility | `backend/src/gateway/keywords.rs` | Checks if message confirms a build in 8 languages | -- |
| 11 | is_build_cancelled() | Utility | `backend/src/gateway/keywords.rs` | Checks if message cancels a build in 8 languages | -- |
| 12 | is_valid_fact() | Utility | `backend/src/gateway/keywords.rs` | Validates facts: rejects system keys, too long values, non-personal data | SYSTEM_FACT_KEYS |
| 13 | SYSTEM_FACT_KEYS | Re-export | `backend/src/gateway/keywords.rs` | Re-exported from omega-core config | -- |

## Internal Dependencies

- Pipeline uses kw_match() with 9 keyword arrays to determine prompt sections and context needs
- is_build_confirmed()/is_build_cancelled() used by pipeline_builds for build confirmation flow
- is_valid_fact() used by summarizer for fact extraction validation

## Dead Code / Unused

- None detected.
