# src/task_confirmation.rs — Task Scheduling Confirmation

## Purpose

Anti-hallucination layer for task scheduling. When the AI emits SCHEDULE/SCHEDULE_ACTION markers, the gateway processes them and collects results into `MarkerResult` values. This module formats those results into a localized confirmation message sent AFTER the AI's response, ensuring users see what was actually created in the database — not what the AI claimed to create.

## Types

### `MarkerResult` (enum)

| Variant | Fields | When |
|---------|--------|------|
| `TaskCreated` | `description`, `due_at`, `repeat`, `task_type` | Task saved to DB successfully |
| `TaskFailed` | `description`, `reason` | DB write failed |
| `TaskParseError` | `raw_line` | Marker line could not be parsed |

## Functions

### `descriptions_are_similar(a: &str, b: &str) -> bool`

Word-overlap similarity check for duplicate detection. Extracts significant words (3+ chars, excluding stop words), computes overlap between smaller and larger sets. Returns true if >= 50% of the smaller set overlaps.

**Stop words**: the, and, for, that, this, with, from, are, was, were, been, have, has, had, will, would, could, should, may, might, can, about, into, over, after, before, between, under, again, then, once, daily, weekly, monthly.

### `format_task_confirmation(results, similar_warnings, lang) -> Option<String>`

Formats a human-readable confirmation message from marker results. Returns `None` if no results to report.

**Output format:**
- Single task: `✓ Scheduled: {desc} — {due_at} ({repeat})`
- Multiple tasks: `✓ Scheduled {n} tasks:` + bulleted list
- Similar warning: `⚠ Similar task exists: "{desc}" — {due_at}`
- Failure: `✗ Failed to save {n} task(s). Please try again.`

All strings are localized via `i18n::t()` and `i18n::tasks_confirmed()` / `i18n::task_save_failed()`.

## Integration

- `gateway.rs::process_markers()` returns `Vec<MarkerResult>` (previously returned `()`)
- `gateway.rs::send_task_confirmation()` calls `get_tasks_for_sender()` to check for similar existing tasks, then calls `format_task_confirmation()` to build the message
- Called from both `handle_message()` (direct responses) and `execute_steps()` (multi-step planning)

## Tests

| Test | Verifies |
|------|----------|
| `test_descriptions_are_similar_exact_match` | Identical descriptions match |
| `test_descriptions_are_similar_reworded` | Semantically similar descriptions match |
| `test_descriptions_are_similar_different` | Unrelated descriptions don't match |
| `test_descriptions_are_similar_empty` | Empty strings don't match |
| `test_descriptions_are_similar_short_words_ignored` | Stop words and short words excluded |
| `test_descriptions_are_similar_case_insensitive` | Case doesn't affect similarity |
| `test_format_task_confirmation_single_created` | Single task formats correctly |
| `test_format_task_confirmation_multiple_created` | Multiple tasks show count + list |
| `test_format_task_confirmation_with_failure` | Failure message shown |
| `test_format_task_confirmation_with_similar_warning` | Warning about similar task shown |
| `test_format_task_confirmation_empty` | Empty results return None |
| `test_significant_words` | Word extraction filters correctly |
