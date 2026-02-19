# Technical Specification: `providers-tools.md`

## File

| Property | Value |
|----------|-------|
| **Path** | `crates/omega-providers/src/tools.rs` |
| **Crate** | `omega-providers` |
| **Module** | `pub mod tools` |
| **Status** | Implemented |

## Purpose

Shared tool executor for HTTP-based providers. Provides 4 built-in tools (bash, read, write, edit) with sandbox enforcement, plus dynamic routing to MCP server tools. Designed to be embedded in agentic provider loops that call tools in response to model requests. `ToolExecutor` is the single entry point for all tool execution regardless of origin (built-in or MCP).

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_BASH_OUTPUT` | `30_000` | Maximum characters for bash tool output before truncation |
| `MAX_READ_OUTPUT` | `50_000` | Maximum characters for read tool output before truncation |
| `BASH_TIMEOUT_SECS` | `120` | Default bash command timeout in seconds |

## Public Types

### `ToolDef`

A tool definition in provider-agnostic format, suitable for serialization to any provider's tool schema format.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Tool name |
| `description` | `String` | Human-readable description |
| `parameters` | `Value` | JSON Schema object for parameters |

Derives: `Debug`, `Clone`, `Serialize`, `Deserialize`.

### `ToolResult`

Result of executing a tool call.

| Field | Type | Description |
|-------|------|-------------|
| `content` | `String` | Text output from the tool |
| `is_error` | `bool` | Whether the tool call failed |

Derives: `Debug`, `Clone`.

## Struct: `ToolExecutor`

Executes built-in tools and routes MCP tool calls to the correct server.

| Field | Type | Description |
|-------|------|-------------|
| `workspace_path` | `PathBuf` | Sandbox working directory (`~/.omega/workspace/`) |
| `data_dir` | `PathBuf` | Omega data directory (`~/.omega/`); derived as parent of `workspace_path` |
| `sandbox_mode` | `SandboxMode` | Active sandbox level controlling write permissions |
| `mcp_clients` | `HashMap<String, McpClient>` | Connected MCP servers keyed by server name |
| `mcp_tool_map` | `HashMap<String, String>` | Maps tool name to the server name that provides it |

## Methods

### `new(workspace_path, sandbox_mode) -> Self`

Creates a new `ToolExecutor`. Derives `data_dir` as the parent of `workspace_path` (falls back to `workspace_path` itself if no parent). Both MCP maps start empty.

```rust
pub fn new(workspace_path: PathBuf, sandbox_mode: SandboxMode) -> Self
```

### `connect_mcp_servers(servers: &[McpServer])`

Iterates over the provided `McpServer` configs, calls `McpClient::connect()` for each, and populates `mcp_clients` and `mcp_tool_map`. Failures are logged as warnings and skipped — a single server failure does not abort the others.

```rust
pub async fn connect_mcp_servers(&mut self, servers: &[McpServer])
```

### `all_tool_defs() -> Vec<ToolDef>`

Returns all available tool definitions: the 4 built-in tools (from `builtin_tool_defs()`) followed by all MCP tools from all connected servers. Used to populate the tools array in provider API requests.

```rust
pub fn all_tool_defs(&self) -> Vec<ToolDef>
```

### `execute(tool_name, args) -> ToolResult`

Dispatches a tool call by name. Built-in tools are matched case-insensitively. If the name is not a built-in, looks up the tool in `mcp_tool_map` and routes to the appropriate `McpClient`. Returns an error `ToolResult` for unknown tools or disconnected servers.

```rust
pub async fn execute(&mut self, tool_name: &str, args: &Value) -> ToolResult
```

### `shutdown_mcp()`

Drains `mcp_clients`, calling `McpClient::shutdown()` on each. Also clears `mcp_tool_map`. Safe to call multiple times.

```rust
pub async fn shutdown_mcp(&mut self)
```

## Built-in Tools

| Tool | Required Params | Optional Params | Behavior |
|------|----------------|-----------------|----------|
| `bash` | `command: String` | — | Executes via `bash -c <command>` in `workspace_path` using `sandboxed_command()`. Captures stdout + stderr combined. Returns exit-code string if both are empty. Truncated to `MAX_BASH_OUTPUT`. `is_error` set when exit status is non-zero. Times out after `BASH_TIMEOUT_SECS`. |
| `read` | `file_path: String` | — | Reads file as UTF-8 string via `tokio::fs::read_to_string`. No sandbox restriction on reads. Truncated to `MAX_READ_OUTPUT`. |
| `write` | `file_path: String`, `content: String` | — | Writes (creates or overwrites) file after `is_write_allowed()` check. Creates parent directories with `create_dir_all`. Returns byte count on success. |
| `edit` | `file_path: String`, `old_string: String`, `new_string: String` | — | Reads file, finds first occurrence of `old_string`, replaces it with `new_string`, writes back. Fails if `old_string` not found. `is_write_allowed()` checked before read. Reports occurrence count in success message. |

## Path Validation: `is_write_allowed(path) -> bool`

Controls write access for the `write` and `edit` tools based on the active sandbox mode.

| Sandbox Mode | Allowed Writes |
|--------------|---------------|
| `Rwx` | All paths unrestricted |
| `Sandbox` | Paths under `data_dir` (`~/.omega/`) or under `/tmp` only |
| `Rx` | Same as `Sandbox` (read-execute; writes restricted to `data_dir` and `/tmp`) |

For relative paths in sandbox/rx mode, the path is resolved against `workspace_path` before checking. For absolute paths, checked as-is.

## Helper Functions

### `truncate_output(s: &str, max_chars: usize) -> String`

Returns `s` unchanged if `s.len() <= max_chars`. Otherwise returns the first `max_chars` bytes followed by a newline and a truncation notice: `... (output truncated: N total chars, showing first M)`.

### `builtin_tool_defs() -> Vec<ToolDef>`

Returns the definitions of the 4 built-in tools as a `Vec<ToolDef>` with JSON Schema `parameters` objects. Public so other modules can inspect built-in tool schemas without constructing a `ToolExecutor`.

```rust
pub fn builtin_tool_defs() -> Vec<ToolDef>
```

## Tests

| Test | Description |
|------|-------------|
| `test_builtin_tool_defs_count` | Verifies exactly 4 built-in tools with names bash, read, write, edit |
| `test_tool_def_serialization` | Verifies each `ToolDef` serializes to JSON with name, description, parameters |
| `test_truncate_output_short` | Verifies short string returned unchanged |
| `test_truncate_output_exact` | Verifies string at exactly `max_chars` returned unchanged |
| `test_truncate_output_long` | Verifies long string truncated with truncation notice and total char count |
| `test_is_write_allowed_rwx` | Verifies `Rwx` mode allows all paths including `/etc/passwd` |
| `test_is_write_allowed_sandbox_inside_data_dir` | Verifies `Sandbox` mode allows writes inside `~/.omega/` and `/tmp` |
| `test_is_write_allowed_sandbox_outside` | Verifies `Sandbox` mode denies writes to `/etc/passwd` and user home |
| `test_exec_bash_empty_command` | Verifies bash tool returns error when `command` param is absent |
| `test_exec_bash_echo` | Verifies `echo hello` produces non-error output containing "hello" |
| `test_exec_read_nonexistent` | Verifies read tool returns error for a nonexistent file path |
| `test_exec_write_and_read` | Verifies write creates file and read retrieves the exact content; cleans up |
| `test_exec_edit` | Verifies edit replaces first occurrence of old_string; confirms new content; cleans up |
| `test_exec_write_denied_sandbox` | Verifies write to `/etc/test` is denied in `Sandbox` mode with "denied" in message |
| `test_tool_executor_mcp_tool_map_routing` | Verifies `mcp_tool_map` and `mcp_clients` are empty on construction |
| `test_execute_unknown_tool` | Verifies `execute()` returns error with "Unknown tool" for an unrecognized tool name |
