# Spec: src/selfcheck.rs

## File Path
`/Users/isudoajl/ownCloud/Projects/omega/src/selfcheck.rs`

## Purpose
Startup self-check module that verifies all components of the Omega system are operational before the gateway event loop begins. It performs a series of validation checks and produces diagnostic output to confirm system readiness.

## Module Type
Internal async module providing pre-flight verification for the main gateway.

## Exported Functions

### `pub async fn run(config: &Config, store: &Store) -> bool`
- **Purpose**: Execute all startup checks and report results
- **Parameters**:
  - `config: &Config` — System configuration (provider settings, channel settings)
  - `store: &Store` — Database connection for memory access
- **Returns**: `bool` — `true` if all checks passed, `false` if any check failed
- **Output**: Prints formatted check results to stdout with pass/fail indicators

## Internal Structures

### `struct CheckResult`
Encapsulates the result of a single check.

**Fields:**
- `name: String` — Display name of the check (e.g., "Database", "Provider", "Channel")
- `detail: String` — Descriptive detail about the result (e.g., "accessible (2.5 MB)", "NOT FOUND")
- `ok: bool` — Pass/fail status

## Validation Checks

### 1. Database Check (`check_database`)
**Name**: "Database"

**What it validates:**
- Database is accessible and functional
- Database query operations work correctly
- Database file size can be determined

**Implementation:**
- Calls `store.db_size().await` to retrieve the database file size
- Formats size as human-readable (bytes/KB/MB):
  - Less than 1024 bytes: displayed as "X B"
  - 1024 bytes to 1 MB: displayed as "X.X KB"
  - 1 MB and above: displayed as "X.X MB"

**Diagnostic Output:**
- **Success**: "Database — accessible (size)"
- **Failure**: "Database — FAILED: {error}"

**Failure Conditions:**
- `store.db_size()` returns an error
- Database file cannot be read or queried

---

### 2. Provider Check (`check_provider`)
**Name**: "Provider"

**What it validates:**
- Configured AI provider is operational and available
- For `claude-code` provider: Claude CLI is installed and executable
- Provider configuration is valid

**Implementation:**
- Reads `config.provider.default` to determine which provider to check
- For `"claude-code"` provider:
  - Calls `ClaudeCodeProvider::check_cli().await` to verify CLI availability
  - Checks if `claude` command can be invoked
- For other providers:
  - Performs no validation (marked as "unchecked")

**Diagnostic Output:**
- **claude-code (available)**: "Provider — claude-code (available)"
- **claude-code (unavailable)**: "Provider — claude-code (NOT FOUND — install claude CLI)"
- **Other providers**: "Provider — {provider_name} (unchecked)"

**Failure Conditions:**
- Provider is `claude-code` AND `ClaudeCodeProvider::check_cli()` returns `false`
- This indicates the `claude` CLI is not installed or not in PATH

**Pass Condition:**
- Provider is available, or provider is non-claude-code type (unchecked but considered passing)

---

### 3. Telegram Channel Check (`check_telegram`)
**Name**: "Channel"

**What it validates:**
- Telegram bot is configured
- Telegram bot token is present and non-empty
- Telegram bot token is valid by making API call to Telegram servers
- Bot credentials allow successful authentication with Telegram API

**Implementation:**
- Checks if `tg.bot_token` is empty — if so, fails immediately
- Constructs Telegram API endpoint: `https://api.telegram.org/bot{token}/getMe`
- Uses `reqwest::Client` to make HTTP GET request
- Checks HTTP response status code
- On success, parses JSON response and extracts bot username from `result.username`
- Falls back to "unknown" if username cannot be extracted

**Diagnostic Output:**
- **Success with username**: "Channel — telegram (@username)"
- **Missing token**: "Channel — telegram (missing bot_token)"
- **Invalid token**: "Channel — telegram (token invalid — HTTP {status_code})"
- **Network error**: "Channel — telegram (network error: {error})"

**Failure Conditions:**
- `bot_token` is empty or not configured
- HTTP request to Telegram API fails with non-2xx status code
- Network connectivity issue prevents reaching Telegram API servers
- Request timeout or connection refused

**Conditional Execution:**
- Only runs if `config.channel.telegram` is `Some` AND `tg.enabled` is `true`
- Skipped entirely if Telegram channel is disabled or not configured

---

## Diagnostic Output Format

### Console Output
```
Omega Self-Check
================
  + Database — accessible (2.5 MB)
  + Provider — claude-code (available)
  + Channel — telegram (@omega_bot)

```

### Pass Indicator
- `+` — Check passed

### Fail Indicator
- `x` — Check failed

## Dependency Verification

### External Crates
- `reqwest` — HTTP client for Telegram API calls
- `serde_json` — JSON parsing for Telegram API responses
- `tokio` — Async runtime (implicitly via async functions)

### Internal Crate Dependencies
- `omega_core::config::Config` — System configuration structure
- `omega_core::config::TelegramConfig` — Telegram-specific configuration
- `omega_memory::Store` — Database abstraction for memory operations
- `omega_providers::claude_code::ClaudeCodeProvider` — Claude Code provider availability check

### External Services
- Telegram Bot API (`https://api.telegram.org`)

## Config Validation

### Configuration Fields Checked
1. **Provider Configuration**:
   - `config.provider.default` — Provider type string (reads value, validates if "claude-code")

2. **Telegram Configuration** (conditional):
   - `config.channel.telegram` — Optional Telegram config block
   - `tg.enabled` — Boolean flag determining if Telegram is active
   - `tg.bot_token` — Telegram bot API token (must be non-empty)

### Validation Rules
- Provider type "claude-code" requires CLI installation
- Telegram token must be non-empty string if channel is enabled
- Telegram token format is not validated (API call is definitive test)

## Provider Health Checks

### Claude Code Provider
**Check Method**: `ClaudeCodeProvider::check_cli()`

**What it checks:**
- Claude CLI is installed
- `claude` command exists in system PATH
- CLI is executable

**API Used**: Binary existence check via PATH or direct invocation

**Success Criteria**: Command returns successfully

**Failure Criteria**: Command not found, not executable, or returns error

### Other Providers
- No health checks performed
- Marked as "unchecked" in output
- Do not affect overall self-check pass/fail status

## Error Handling

### Database Check Errors
- Captures and displays error message from `store.db_size()`
- Marked as explicit failure
- Continues to check other components

### Provider Check Errors
- Silent failure for claude-code CLI (returns boolean from `check_cli()`)
- Non-exception based
- Continues to check other components

### Telegram Check Errors
- HTTP errors from `reqwest` are caught and formatted as "network error"
- HTTP non-2xx status codes are captured and displayed with status code
- JSON parsing failures silently fall back to "unknown" for bot username
- Errors do not panic; check completes with failure status

## Overall Flow

1. Initialize results vector
2. Execute database check (always runs)
3. Execute provider check (always runs)
4. Execute Telegram channel check (conditionally, if enabled)
5. Print formatted results to stdout
6. Calculate overall pass/fail status
7. Return boolean status

## Return Value Semantics

- **`true`**: All executed checks passed; system is ready to start
- **`false`**: One or more checks failed; gateway should not start or should start with reduced functionality

## Output Destination
- Stdout only (no file logging)
- Printed before gateway initialization
- Human-readable format

## Security Considerations

- Telegram bot token is included in network request (sent to official Telegram servers)
- Bot token is extracted from config and not logged or displayed in detail
- Bot username (public information) is extracted and displayed
- No sensitive configuration data is logged in error messages beyond HTTP status codes

## Performance Characteristics

- Database check: Fast (single query operation)
- Provider check: Fast (CLI binary existence check)
- Telegram check: Slower (requires network round-trip to Telegram API, ~500ms-2000ms depending on network)
- Overall timeout: No explicit timeout set (inherits from HTTP client defaults)

## Edge Cases

1. **Empty database file**: Returns "0 B", check passes
2. **Database file larger than 1 TB**: Displays as "X.X MB" (formatting cap)
3. **Telegram token is valid but bot is disabled on Telegram side**: getMe succeeds, check passes (actual message delivery would fail later)
4. **Network intermittent during Telegram check**: Marked as network error, check fails
5. **Malformed JSON in Telegram response**: Username extraction fails silently, defaults to "unknown"
6. **Multiple channels**: Only Telegram is currently checked; other channels (WhatsApp) are not validated

## Testing Considerations

- Mock `store.db_size()` for database test
- Mock `ClaudeCodeProvider::check_cli()` for provider test
- Mock `reqwest::Client` for Telegram API test with various HTTP status codes
- Test with disabled Telegram channel to verify skipping behavior
- Test with empty bot_token to verify early exit

## Future Extensions

- Add WhatsApp channel check (similar pattern to Telegram)
- Add configuration file validity checks
- Add permission checks (database file readable/writable)
- Add environment variable validation
- Add timeout handling for Telegram API calls
- Add retry logic for transient network failures
- Add check for CLAUDECODE env var not being set (to prevent nested sessions)
