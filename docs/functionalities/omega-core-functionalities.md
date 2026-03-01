# Functionalities: omega-core

## Overview

Core types, traits, configuration, error handling, and prompt sanitization for the Omega agent. This crate defines the foundational abstractions that all other crates depend on.

## Functionalities

| # | Name | Type | Location | Description | Dependencies |
|---|------|------|----------|-------------|--------------|
| 1 | Provider trait | Trait | `backend/crates/omega-core/src/traits.rs:14` | Async trait for AI backends: name(), requires_api_key(), complete(), is_available() | Context, OmegaError, OutgoingMessage |
| 2 | Channel trait | Trait | `backend/crates/omega-core/src/traits.rs:33` | Async trait for messaging platforms: start(), send(), send_typing(), send_photo(), stop(), as_any() | IncomingMessage, OutgoingMessage, OmegaError |
| 3 | OmegaError | Model | `backend/crates/omega-core/src/error.rs:5` | Top-level error enum with 7 variants: Provider, Channel, Config, Memory, Sandbox, Io, Serialization | thiserror |
| 4 | IncomingMessage | Model | `backend/crates/omega-core/src/message.rs:7` | Incoming message struct with channel, sender, text, attachments, reply_target, is_group, source fields | chrono, uuid, serde |
| 5 | OutgoingMessage | Model | `backend/crates/omega-core/src/message.rs:34` | Outgoing message struct with text, metadata, reply_target | MessageMetadata |
| 6 | MessageMetadata | Model | `backend/crates/omega-core/src/message.rs:44` | Metadata for AI responses: provider_used, tokens_used, processing_time_ms, model, session_id | -- |
| 7 | Attachment / AttachmentType | Model | `backend/crates/omega-core/src/message.rs:60` | File attachment struct with type (Image/Document/Audio/Video/Other), url, data, filename | -- |
| 8 | sanitize() | Service | `backend/crates/omega-core/src/sanitize.rs:49` | Neutralizes prompt injection attacks: role tag replacement, override phrase detection (with zero-width/double-space/case bypass protection), code block warning | -- |
| 9 | SanitizeResult | Model | `backend/crates/omega-core/src/sanitize.rs:10` | Result of sanitization: cleaned text, was_modified flag, warnings list | -- |
| 10 | Context | Model | `backend/crates/omega-core/src/context.rs:54` | Conversation context for providers: system_prompt, history, current_message, mcp_servers, max_turns, allowed_tools, model, session_id, agent_name | McpServer, ContextEntry |
| 11 | Context::to_prompt_string() | Service | `backend/crates/omega-core/src/context.rs:120` | Flattens context into a single prompt string for CLI providers. Session mode skips system+history. Agent mode returns only current_message | -- |
| 12 | Context::to_api_messages() | Service | `backend/crates/omega-core/src/context.rs:165` | Converts context to structured API messages for HTTP providers (returns system_prompt separately) | ApiMessage |
| 13 | ContextNeeds | Model | `backend/crates/omega-core/src/context.rs:7` | Selective context loading flags: recall, pending_tasks, profile, summaries, outcomes | -- |
| 14 | McpServer | Model | `backend/crates/omega-core/src/context.rs:42` | MCP server declaration: name, command, args | -- |
| 15 | ContextEntry | Model | `backend/crates/omega-core/src/context.rs:33` | Single conversation history entry: role, content | -- |
| 16 | ApiMessage | Model | `backend/crates/omega-core/src/context.rs:91` | Structured message for API-based providers: role, content | -- |
| 17 | Config | Model | `backend/crates/omega-core/src/config/mod.rs:22` | Top-level config: omega, auth, provider, channel, memory, heartbeat, scheduler, api sections | All sub-configs |
| 18 | AuthConfig | Model | `backend/crates/omega-core/src/config/mod.rs:42` | Auth settings: enabled flag, deny_message | -- |
| 19 | OmegaConfig | Model | `backend/crates/omega-core/src/config/mod.rs:54` | General settings: name, data_dir, log_level | -- |
| 20 | MemoryConfig | Model | `backend/crates/omega-core/src/config/mod.rs:75` | Memory settings: backend, db_path, max_context_messages | -- |
| 21 | HeartbeatConfig | Model | `backend/crates/omega-core/src/config/mod.rs:96` | Heartbeat settings: enabled, interval_minutes, active_start/end, channel, reply_target | -- |
| 22 | SchedulerConfig | Model | `backend/crates/omega-core/src/config/mod.rs:130` | Scheduler settings: enabled, poll_interval_secs | -- |
| 23 | ApiConfig | Model | `backend/crates/omega-core/src/config/mod.rs:148` | API settings: enabled, host, port, api_key | -- |
| 24 | SYSTEM_FACT_KEYS | Constant | `backend/crates/omega-core/src/config/mod.rs:176` | System-managed fact keys protected from user modification: welcomed, preferred_language, active_project, personality, onboarding_stage, pending_build_request, pending_discovery, pending_setup | -- |
| 25 | shellexpand() | Utility | `backend/crates/omega-core/src/config/mod.rs:188` | Expands ~ to home directory in paths | -- |
| 26 | migrate_layout() | Service | `backend/crates/omega-core/src/config/mod.rs:203` | Migrates flat ~/.omega/ layout to structured subdirectories (data/, logs/, prompts/) | -- |
| 27 | patch_heartbeat_interval() | Service | `backend/crates/omega-core/src/config/mod.rs:261` | Text-based patching of heartbeat interval in config.toml (preserves comments/formatting) | -- |
| 28 | load() | Service | `backend/crates/omega-core/src/config/mod.rs:323` | Loads configuration from TOML file with defaults fallback | -- |
| 29 | ProviderConfig | Model | `backend/crates/omega-core/src/config/providers.rs` | Provider configurations for 6 backends: ClaudeCodeConfig, AnthropicConfig, OpenAiConfig, OllamaConfig, OpenRouterConfig, GeminiConfig | -- |
| 30 | ChannelConfig | Model | `backend/crates/omega-core/src/config/channels.rs` | Channel configurations: TelegramConfig (bot_token, allowed_users, whisper_api_key), WhatsAppConfig (allowed_users, whisper_api_key) | -- |
| 31 | Prompts | Model | `backend/crates/omega-core/src/config/prompts.rs:11` | Externalized prompt templates with 14 sections: identity, soul, system, scheduling, projects_rules, builds, meta, summarize, facts, heartbeat, heartbeat_checklist, welcome | -- |
| 32 | Prompts::load() | Service | `backend/crates/omega-core/src/config/prompts.rs:167` | Loads prompts from SYSTEM_PROMPT.md (markdown sections) and WELCOME.toml (welcome messages) | parse_markdown_sections |
| 33 | install_bundled_prompts() | Service | `backend/crates/omega-core/src/config/prompts.rs:140` | Deploys bundled SYSTEM_PROMPT.md and WELCOME.toml to runtime directory (never overwrites) | -- |
| 34 | bundled_workspace_claude() | Service | `backend/crates/omega-core/src/config/prompts.rs:133` | Returns the bundled WORKSPACE_CLAUDE.md template for the Claude Code subprocess | -- |
| 35 | Default config values | Utility | `backend/crates/omega-core/src/config/defaults.rs` | 19 default value functions for all config fields (provider=claude-code, model=sonnet, model_complex=opus, max_turns=25, etc.) | -- |

## Internal Dependencies

- Provider trait depends on Context, OmegaError, OutgoingMessage
- Channel trait depends on IncomingMessage, OutgoingMessage, OmegaError
- Config loads from TOML and produces all sub-config structs
- Context::to_prompt_string() uses session_id and agent_name to control output format
- Prompts::load() uses parse_markdown_sections() to parse SYSTEM_PROMPT.md

## Dead Code / Unused

- None detected in this crate. All public items are consumed by downstream crates.
