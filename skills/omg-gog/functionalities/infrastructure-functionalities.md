# Functionalities: Infrastructure

## Overview
Shared infrastructure modules: HTTP client with retry/circuit-breaker, output formatting (JSON/Plain/CSV/Text), error handling with stable exit codes, time parsing, UI (color, progress, prompts), pagination, export format mapping, and the ServiceContext bootstrap pipeline.

## HTTP Module (`src/http/`)

### Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | RetryConfig | Struct | src/http/mod.rs | Retry settings (max_retries_429=3, max_retries_5xx=1, base_delay=1s) |
| 2 | CircuitBreaker | Struct | src/http/circuit_breaker.rs | Thread-safe circuit breaker (Closed/Open/HalfOpen), threshold=5, reset=30s |
| 3 | RetryableRequest | Struct | src/http/middleware.rs | Request metadata for retry middleware |

### Constants

| # | Name | Location | Value |
|---|------|----------|-------|
| 1 | DEFAULT_TIMEOUT | src/http/client.rs | 300 seconds |
| 2 | USER_AGENT | src/http/client.rs | `omega-google/{version}` |
| 3 | CIRCUIT_BREAKER_THRESHOLD | src/http/circuit_breaker.rs | 5 consecutive failures |
| 4 | CIRCUIT_BREAKER_RESET_TIME | src/http/circuit_breaker.rs | 30 seconds |

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `build_client` | src/http/client.rs | Build unauthenticated reqwest client (rustls TLS 1.2+) |
| 2 | `build_authenticated_client` | src/http/client.rs | Build client with Bearer token |
| 3 | `api_get` | src/http/api.rs | Generic GET with deserialization |
| 4 | `api_post` | src/http/api.rs | Generic POST with JSON body |
| 5 | `api_post_empty` | src/http/api.rs | POST with no response body |
| 6 | `api_patch` | src/http/api.rs | Generic PATCH with JSON body |
| 7 | `api_delete` | src/http/api.rs | Generic DELETE |
| 8 | `api_put_bytes` | src/http/api.rs | PUT raw bytes |
| 9 | `api_post_bytes` | src/http/api.rs | POST raw bytes |
| 10 | `api_get_raw` | src/http/api.rs | GET returning raw bytes |
| 11 | `check_response_status` | src/http/api.rs | Check HTTP status, format error |
| 12 | `redact_auth_header` | src/http/api.rs | Redact Bearer token for verbose logging |
| 13 | `calculate_backoff` | src/http/retry.rs | Exponential backoff with jitter |
| 14 | `parse_retry_after` | src/http/retry.rs | Parse Retry-After header |
| 15 | `is_retryable` | src/http/retry.rs | Check if status is retryable (429 or 5xx) |
| 16 | `is_rate_limited` | src/http/retry.rs | Check if status is 429 |
| 17 | `is_server_error` | src/http/retry.rs | Check if status is 5xx |
| 18 | `execute_with_retry` | src/http/middleware.rs | Execute request with retry + circuit breaker |

---

## Output Module (`src/output/`)

### Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | OutputMode | Enum | src/output/mod.rs | Json, Plain, Text, Csv |
| 2 | JsonTransform | Struct | src/output/mod.rs | Transform config (results_only, select fields) |

### Constants

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | META_KEYS | src/output/transform.rs | Keys considered metadata (kind, etag, nextPageToken, etc) |
| 2 | KNOWN_RESULT_KEYS | src/output/transform.rs | Keys containing result arrays (messages, files, events, etc) |
| 3 | ANSI codes | src/output/text.rs | RESET, BOLD, RED, GREEN, YELLOW, BLUE, CYAN, DIM |

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `resolve_mode` | src/output/mod.rs | Resolve output mode from --json/--plain flags |
| 2 | `resolve_mode_full` | src/output/mod.rs | Resolve mode with --csv support |
| 3 | `write_json` | src/output/mod.rs | Write value as pretty JSON |
| 4 | `write_plain` | src/output/mod.rs | Write value as tab-separated rows |
| 5 | `write_csv` | src/output/mod.rs | Write value as CSV (RFC 4180) |
| 6 | `csv_escape` | src/output/mod.rs | Escape CSV field |
| 7 | `json_to_plain_rows` | src/output/plain.rs | Convert JSON to plain row vectors |
| 8 | `write_json_raw` | src/output/json.rs | Write raw JSON string |
| 9 | `to_pretty_json` | src/output/json.rs | Serialize to pretty JSON string |
| 10 | `unwrap_primary` | src/output/transform.rs | Strip metadata envelope, extract result array |
| 11 | `select_fields` | src/output/transform.rs | Project selected fields from JSON |
| 12 | `get_at_path` | src/output/transform.rs | Navigate dot-path in JSON value |
| 13 | `colored` | src/output/text.rs | Wrap text in ANSI color |
| 14 | `bold` | src/output/text.rs | Wrap text in bold |
| 15 | `format_error` | src/output/text.rs | Format error message (red) |
| 16 | `format_warning` | src/output/text.rs | Format warning message (yellow) |
| 17 | `format_hint` | src/output/text.rs | Format hint message (cyan) |

---

## Error Module (`src/error/`)

### Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | OmegaError | Enum | src/error/exit.rs | 12 error variants mapping to exit codes |

### Exit Codes

| # | Name | Value | Description |
|---|------|-------|-------------|
| 1 | SUCCESS | 0 | Successful execution |
| 2 | GENERIC_ERROR | 1 | Unclassified error |
| 3 | USAGE_ERROR | 2 | Invalid arguments |
| 4 | EMPTY_RESULTS | 3 | Query returned no results |
| 5 | AUTH_REQUIRED | 4 | Authentication needed |
| 6 | NOT_FOUND | 5 | Resource not found |
| 7 | PERMISSION_DENIED | 6 | Insufficient permissions |
| 8 | RATE_LIMITED | 7 | API rate limit hit |
| 9 | RETRYABLE | 8 | Transient error, retry possible |
| 10 | CONFIG_ERROR | 10 | Configuration problem |
| 11 | CANCELLED | 130 | User cancelled (Ctrl+C / SIGINT) |

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `exit_code_for` | src/error/exit.rs | Map OmegaError to exit code |
| 2 | `format_api_error` | src/error/api_error.rs | Format API error for display |
| 3 | `parse_google_error` | src/error/api_error.rs | Extract message from Google API error JSON |

---

## Time Module (`src/time/`)

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `parse_datetime` | src/time/parse.rs | Parse datetime string (ISO, relative, weekday) |
| 2 | `parse_date` | src/time/parse.rs | Parse date string to NaiveDate |
| 3 | `parse_duration` | src/time/parse.rs | Parse duration string (e.g., "2h", "30m", "1d") |
| 4 | `is_relative` | src/time/parse.rs | Check if input is relative time ("tomorrow", "next monday") |
| 5 | `is_weekday_ref` | src/time/parse.rs | Check if input is weekday reference |
| 6 | `is_duration` | src/time/parse.rs | Check if input is duration string |

---

## UI Module (`src/ui/`)

### Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | ColorMode | Enum | src/ui/mod.rs | Auto, Always, Never |
| 2 | UiOptions | Struct | src/ui/mod.rs | UI configuration options |
| 3 | Ui | Struct | src/ui/mod.rs | UI context (hint, warn, status, progress methods) |

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `confirm` | src/ui/prompt.rs | Interactive yes/no prompt (respects --force, --no-input) |
| 2 | `prompt_input` | src/ui/prompt.rs | Interactive text input prompt |
| 3 | `progress` | src/ui/progress.rs | Print progress message to stderr |
| 4 | `progress_ln` | src/ui/progress.rs | Print progress with newline |
| 5 | `status` | src/ui/progress.rs | Print operation status to stderr |
| 6 | `done` | src/ui/progress.rs | Print completion message |
| 7 | `terminal_supports_color` | src/ui/color.rs | Check terminal color support |
| 8 | `should_use_color` | src/ui/color.rs | Resolve color mode string to boolean |

---

## Services Common (`src/services/`)

### Types

| # | Name | Kind | Location | Description |
|---|------|------|----------|-------------|
| 1 | ServiceContext | Struct | src/services/mod.rs | Context for all handlers (client, output_mode, json_transform, ui, flags, circuit_breaker, retry_config, email) |
| 2 | PaginationParams | Struct | src/services/common.rs | Common pagination (max_results, page_token) |
| 3 | ListResponse\<T\> | Struct | src/services/common.rs | Generic list with items + nextPageToken |

### Traits

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | HasNextPageToken | src/services/pagination.rs | Trait for paginated response types |

### Functions

| # | Name | Location | Description |
|---|------|----------|-------------|
| 1 | `bootstrap_service_context` | src/services/mod.rs | 10-step auth bootstrap: account -> config -> credentials -> token -> refresh -> client -> output mode -> ServiceContext |
| 2 | `ServiceContext::write_output` | src/services/mod.rs | Format-aware output with transforms |
| 3 | `ServiceContext::write_paginated` | src/services/mod.rs | Output with next-page-token hint |
| 4 | `paginate` | src/services/pagination.rs | Generic pagination loop (max 1000 pages) |
| 5 | `paginate_with_progress` | src/services/pagination.rs | Pagination with progress indicator |
| 6 | `check_fail_empty` | src/services/pagination.rs | Error if results empty + --fail-empty |
| 7 | `fetch_page` | src/services/pagination.rs | Fetch single page |
| 8 | `format_size` | src/services/common.rs | Human-readable byte size (KB/MB/GB) |
| 9 | `format_datetime` | src/services/common.rs | Format ISO datetime for display |
| 10 | `format_to_mime` | src/services/export.rs | Map export format name to MIME type |
| 11 | `export_formats` | src/services/export.rs | List available export formats for a Google MIME type |
| 12 | `default_export_format` | src/services/export.rs | Default export format for a Google MIME type |
| 13 | `guess_content_type_from_path` | src/services/export.rs | Guess MIME type from file extension |

---

## Dependency Chain

```
bootstrap_service_context()
  -> config::read_config()
  -> config::read_client_credentials()
  -> auth::credential_store_factory()
  -> auth::resolve_account()
  -> auth::needs_refresh() -> auth::refresh_access_token()
  -> http::build_authenticated_client()
  -> output::resolve_mode_full()
  -> ui::Ui::new()
  -> ServiceContext { client, output_mode, json_transform, ui, ... }

All api_* calls:
  -> execute_with_retry()
    -> CircuitBreaker::check()
    -> reqwest send
    -> calculate_backoff() on retry
    -> CircuitBreaker::record()

ServiceContext::write_output()
  -> unwrap_primary() (if --results-only)
  -> select_fields() (if --select)
  -> write_json() | write_plain() | write_csv()
```
