# Functionalities: i18n

## Overview

Internationalization system supporting 8 languages (English, Spanish, Portuguese, French, German, Italian, Dutch, Russian). Provides localized strings for bot commands, confirmations, and format helpers.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | t(key, lang) | Service | `backend/src/i18n/mod.rs:20` | Main localization function: looks up key in labels, confirmations, commands; falls back to English | labels, confirmations, commands modules |
| 2 | Labels module | Library | `backend/src/i18n/labels.rs` | Static label strings in 8 languages | -- |
| 3 | Confirmations module | Library | `backend/src/i18n/confirmations.rs` | Task confirmation strings in 8 languages | -- |
| 4 | Commands module | Library | `backend/src/i18n/commands.rs` | Command-specific strings in 8 languages | -- |
| 5 | Format module | Library | `backend/src/i18n/format.rs` | Format helpers with interpolation (project_activated, etc.) | -- |

## Internal Dependencies

- Commands module uses t() for all user-facing responses
- task_confirmation uses t() for confirmation formatting
- handle_forget() uses t() for response messages
- Pipeline uses i18n for status messages and build confirmations

## Dead Code / Unused

- None detected.
