# Functionalities: omega-providers

## Overview

AI provider implementations. Six backends share the Provider trait. Claude Code CLI is the default zero-config provider using a local subprocess. HTTP providers include an agentic tool loop with MCP client support.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | ClaudeCodeProvider | Provider | `backend/crates/omega-providers/src/claude_code/mod.rs:25` | Claude Code CLI subprocess provider: auto-resume on max_turns, session_id continuity, agent mode, MCP support, sandboxed execution | omega-sandbox |
| 2 | ClaudeCodeProvider::from_config() | Service | `backend/crates/omega-providers/src/claude_code/mod.rs:76` | Creates provider from config: max_turns, allowed_tools, timeout, working_dir, max_resume_attempts, model | -- |
| 3 | ClaudeCodeProvider::check_cli() | Service | `backend/crates/omega-providers/src/claude_code/mod.rs:95` | Checks if claude CLI is installed and accessible | -- |
| 4 | mcp_tool_patterns() | Utility | `backend/crates/omega-providers/src/claude_code/mcp.rs` | Generates tool patterns for MCP server activation | -- |
| 5 | AnthropicProvider | Provider | `backend/crates/omega-providers/src/anthropic/` | Anthropic HTTP API provider with x-api-key header auth, system prompt as top-level field | -- |
| 6 | OpenAiProvider | Provider | `backend/crates/omega-providers/src/openai/` | OpenAI HTTP API provider with Bearer token auth, compatible with OpenAI-compatible endpoints | -- |
| 7 | OllamaProvider | Provider | `backend/crates/omega-providers/src/ollama/` | Ollama local server provider (no auth required) | -- |
| 8 | OpenRouterProvider | Provider | `backend/crates/omega-providers/src/openrouter/` | OpenRouter proxy provider (reuses OpenAI types with Bearer auth) | -- |
| 9 | GeminiProvider | Provider | `backend/crates/omega-providers/src/gemini/` | Google Gemini HTTP API provider (URL query param auth, role mapping: assistant->model) | -- |
| 10 | MCP client | Library | `backend/crates/omega-providers/src/mcp_client.rs` | MCP (Model Context Protocol) client for tool execution in HTTP providers | -- |
| 11 | Tools module | Library | `backend/crates/omega-providers/src/tools.rs` | Tool execution support for HTTP providers' agentic loop | omega-sandbox |
| 12 | build_provider() | Factory | `backend/src/provider_builder.rs:13` | Factory function building provider from config: Claude Code (fast=Sonnet, complex=Opus), others (both=configured model) | All providers |

## Internal Dependencies

- Claude Code uses omega-sandbox::protected_command() for sandboxed subprocess execution
- HTTP providers use mcp_client and tools modules for agentic tool loops
- build_provider() dispatches to all 6 provider constructors

## Dead Code / Unused

- `#[allow(dead_code)]` on MCP client response fields (mcp_client.rs:64,66,74) -- deserialized but not all fields read
- `#[allow(dead_code)]` on tools module tool result fields (tools.rs:76)
