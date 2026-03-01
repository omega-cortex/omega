# Functionalities: Supporting Modules

## Overview

Supporting modules that handle workspace maintenance, task confirmation anti-hallucination, and styled CLI output.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | ensure_claudemd() | Service | `backend/src/claudemd.rs:35` | Deploys bundled CLAUDE.md template to workspace, enriches with dynamic content via claude -p | bundled_workspace_claude, protected_command |
| 2 | refresh_claudemd() | Service | `backend/src/claudemd.rs:73` | Refreshes workspace CLAUDE.md: re-deploys template (preserving dynamic content), updates dynamic sections | extract_dynamic_content |
| 3 | claudemd_loop() | Background Task | `backend/src/claudemd.rs:143` | Background loop refreshing CLAUDE.md every 24h | refresh_claudemd |
| 4 | extract_dynamic_content() | Utility | `backend/src/claudemd.rs:126` | Extracts content below the dynamic content marker from CLAUDE.md | -- |
| 5 | MarkerResult | Model | `backend/src/task_confirmation.rs` | Enum with 13 variants tracking marker processing outcomes (TaskCreated, TaskFailed, TaskCancelled, etc.) | -- |
| 6 | descriptions_are_similar() | Utility | `backend/src/task_confirmation.rs` | Word overlap similarity check between task descriptions | -- |
| 7 | format_task_confirmation() | Service | `backend/src/task_confirmation.rs` | Formats localized task confirmation with similar task warnings; suppresses implicit replacement cancels | i18n |
| 8 | init_style module | Utility | `backend/src/init_style.rs` | Branded CLI helpers: omega_intro, omega_outro, typewrite animation, etc. Cyan accent palette | cliclack |

## Internal Dependencies

- claudemd_loop() spawned by Gateway::run() for Claude Code provider only
- format_task_confirmation() used by send_task_confirmation() in process_markers
- init_style used by init module for interactive setup wizard

## Dead Code / Unused

- None detected.
