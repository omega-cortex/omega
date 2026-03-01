# Technical Specification: `providers-mcp-client.md`

## File

| Property | Value |
|----------|-------|
| **Path** | `backend/crates/omega-providers/src/mcp_client.rs` |
| **Crate** | `omega-providers` |
| **Module** | `pub mod mcp_client` |
| **Status** | Implemented |

## Purpose

Minimal MCP (Model Context Protocol) client over stdio transport. Implements JSON-RPC 2.0 over newline-delimited JSON on stdin/stdout. No external MCP crate — raw protocol using tokio async I/O and serde_json. Used by `ToolExecutor` to connect to MCP servers declared in skills and inject their tools into agentic provider loops.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MCP_REQUEST_TIMEOUT_SECS` | `120` | Maximum seconds to wait for a JSON-RPC response before aborting |

## Struct: `McpClient`

| Field | Type | Description |
|-------|------|-------------|
| `child` | `Child` | Spawned MCP server process handle |
| `stdin` | `BufWriter<ChildStdin>` | Buffered writer to the server's stdin |
| `stdout` | `Lines<BufReader<ChildStdout>>` | Line-by-line reader from the server's stdout |
| `next_id` | `u64` | Monotonically increasing JSON-RPC request ID counter |
| `server_name` | `String` | Human-readable server name (from skill config) |
| `tools` | `Vec<McpToolDef>` | Tools discovered via `tools/list` during `connect()` |

## Public Types

### `McpToolDef`

A tool definition discovered from an MCP server.

| Field | Type | Serde | Description |
|-------|------|-------|-------------|
| `name` | `String` | default | Tool name (e.g. `"browser_navigate"`) |
| `description` | `String` | `#[serde(default)]` | Human-readable description |
| `input_schema` | `Value` | `#[serde(default, rename = "inputSchema")]` | JSON Schema for the tool's parameters |

Derives: `Debug`, `Clone`, `Serialize`, `Deserialize`.

### `McpToolResult`

Result of calling a tool on the MCP server.

| Field | Type | Description |
|-------|------|-------------|
| `content` | `String` | Concatenated text content blocks from the tool response |
| `is_error` | `bool` | Whether the tool reported an error (`isError` field in response) |

Derives: `Debug`, `Clone`.

## Lifecycle Methods

### `connect(name, command, args) -> Result<Self, anyhow::Error>`

Spawns the MCP server process and performs the full initialization handshake:

1. Spawn process with `Stdio::piped()` for stdin/stdout; stderr is discarded (`Stdio::null()`).
2. Send `initialize` request with `protocolVersion: "2025-11-25"`, empty capabilities, and `clientInfo: { name: "omega", version: "0.1.0" }`.
3. Send `notifications/initialized` notification (no ID, no response expected).
4. Send `tools/list` request; populate `self.tools` via `parse_tools_list()`.

Returns a fully initialized `McpClient` ready for tool calls.

```rust
pub async fn connect(name: &str, command: &str, args: &[String]) -> Result<Self, anyhow::Error>
```

### `call_tool(tool_name, arguments) -> Result<McpToolResult, anyhow::Error>`

Sends a `tools/call` JSON-RPC request with `{ name, arguments }` params and returns the parsed result.

```rust
pub async fn call_tool(&mut self, tool_name: &str, arguments: &Value) -> Result<McpToolResult, anyhow::Error>
```

### `shutdown(self)`

Gracefully terminates the server process. Calls `start_kill()` then waits up to 5 seconds via `tokio::select!`. Logs a warning if the process does not exit within the timeout. Consumes `self`.

```rust
pub async fn shutdown(mut self)
```

## Protocol

| Aspect | Detail |
|--------|--------|
| Transport | Child process stdio (stdin write, stdout read) |
| Framing | Newline-delimited JSON (one JSON object per line) |
| Encoding | UTF-8 |
| Server stderr | Discarded (`Stdio::null()`) |
| Request format | `{ jsonrpc, id, method, params }` |
| Notification format | Same as request but `id` omitted (`skip_serializing_if = "Option::is_none"`) |
| Response matching | Reads lines until `resp.id == request.id`; skips non-JSON lines and mismatched IDs |
| Error handling | JSON-RPC `error` field mapped to `anyhow::Error`; deserialization failures skip the line |

## Private Types

### `JsonRpcRequest`

| Field | Type | Serde |
|-------|------|-------|
| `jsonrpc` | `&'static str` | always `"2.0"` |
| `id` | `Option<u64>` | `skip_serializing_if = "Option::is_none"` |
| `method` | `String` | — |
| `params` | `Option<Value>` | `skip_serializing_if = "Option::is_none"` |

### `JsonRpcResponse`

| Field | Type | Description |
|-------|------|-------------|
| `jsonrpc` | `Option<String>` | Protocol version (informational, `#[allow(dead_code)]`) |
| `id` | `Option<Value>` | Matched against request ID (`#[allow(dead_code)]`) |
| `result` | `Option<Value>` | Success payload |
| `error` | `Option<JsonRpcError>` | Error payload |

### `JsonRpcError`

| Field | Type | Description |
|-------|------|-------------|
| `code` | `i64` | JSON-RPC error code (`#[allow(dead_code)]`) |
| `message` | `String` | Human-readable error message (used in `anyhow::Error`) |

## Private Methods

### `request(method, params) -> Result<Value, anyhow::Error>`

Assigns the next monotonic ID, serializes a `JsonRpcRequest` to a newline-terminated string, writes it to stdin, flushes, then reads stdout lines until a response with the matching ID is received. The entire read loop is wrapped in `tokio::time::timeout(MCP_REQUEST_TIMEOUT_SECS)` — if the server does not respond within the timeout, returns an error instead of hanging indefinitely. Returns `resp.result` (or `Value::Null` if absent).

### `read_response(stdout, id, server_name, method) -> Result<Value, anyhow::Error>`

Static async helper extracted from `request()`. Reads lines from `stdout` until a JSON-RPC response with the matching `id` is found. Skips non-JSON lines and mismatched IDs. Returns the `result` field or maps the `error` field to `anyhow::Error`. Called by `request()` inside a timeout guard.

### `notify(method, params) -> Result<(), anyhow::Error>`

Sends a JSON-RPC notification (no `id` field) to stdin. Does not read a response.

## Helper Functions

### `parse_tools_list(result: &Value) -> Vec<McpToolDef>`

Extracts the `tools` array from a `tools/list` result value. Deserializes each element into `McpToolDef`; silently skips malformed entries. Returns an empty vec if the field is absent.

### `parse_tool_result(result: &Value) -> McpToolResult`

Extracts `isError` (bool, defaults `false`) and concatenates all `text` fields from the `content` array of `{ type, text }` blocks. Returns a `McpToolResult` with the joined text and the error flag.

## Tests

| Test | Description |
|------|-------------|
| `test_jsonrpc_request_serialization` | Verifies `JsonRpcRequest` with ID serializes correctly; `params` omitted when `None` |
| `test_jsonrpc_request_with_params` | Verifies `params` field is included and accessible when provided |
| `test_jsonrpc_notification_no_id` | Verifies notification (no ID) serializes without `id` field |
| `test_jsonrpc_response_parsing` | Verifies success response deserialization with `tools` array in result |
| `test_jsonrpc_error_response_parsing` | Verifies error response deserialization with code and message |
| `test_tools_list_parsing` | Verifies `parse_tools_list` extracts two tools with correct names and descriptions |
| `test_tool_result_parsing` | Verifies `parse_tool_result` joins two text blocks with newline; `is_error` false |
| `test_tool_result_error` | Verifies `parse_tool_result` sets `is_error` true when `isError: true` |
| `test_tool_result_empty` | Verifies `parse_tool_result` returns empty content and no error for empty result |
