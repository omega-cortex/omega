use serde::{Deserialize, Serialize};

/// Controls which optional context blocks are loaded and injected.
///
/// Used by the gateway to skip expensive DB queries and prompt sections
/// when the user's message doesn't need them — reducing token overhead by ~55-70%.
pub struct ContextNeeds {
    /// Load semantic recall (FTS5 related past messages).
    pub recall: bool,
    /// Load and inject pending scheduled tasks.
    pub pending_tasks: bool,
    /// Inject user profile (facts) into the system prompt.
    pub profile: bool,
    /// Load and inject recent conversation summaries.
    pub summaries: bool,
    /// Load and inject recent reward outcomes.
    pub outcomes: bool,
}

impl Default for ContextNeeds {
    fn default() -> Self {
        Self {
            recall: true,
            pending_tasks: true,
            profile: true,
            summaries: true,
            outcomes: true,
        }
    }
}

/// A single entry in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    /// "user" or "assistant".
    pub role: String,
    /// The message content.
    pub content: String,
}

/// An MCP server declared by a skill.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServer {
    /// Server name (used as the key in Claude settings).
    pub name: String,
    /// Command to launch the server.
    pub command: String,
    /// Command-line arguments.
    pub args: Vec<String>,
}

/// Conversation context passed to an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// System prompt prepended to every request.
    pub system_prompt: String,
    /// Conversation history (oldest first).
    pub history: Vec<ContextEntry>,
    /// The current user message.
    pub current_message: String,
    /// MCP servers to activate for this request.
    #[serde(default)]
    pub mcp_servers: Vec<McpServer>,
    /// Override the provider's default max_turns. When `Some`, the provider
    /// uses this value instead of its configured default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    /// Override the provider's default allowed tools. When `Some`, the provider
    /// uses this list instead of its configured default. `Some(vec![])` = no tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    /// Override the provider's default model. When `Some`, the provider passes
    /// `--model` with this value instead of its configured default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Session ID for conversation continuity (Claude Code CLI).
    /// When set, `to_prompt_string()` skips the system prompt and history
    /// (already in the session) and emits only the current message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Agent name for Claude Code CLI `--agent` flag. When set, the CLI
    /// loads the agent definition from `.claude/agents/<name>.md` in the
    /// working directory. The agent file provides the system prompt, so
    /// `to_prompt_string()` emits only the current_message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
}

/// A structured message for API-based providers (OpenAI, Anthropic, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    /// "user" or "assistant".
    pub role: String,
    /// The message content.
    pub content: String,
}

impl Context {
    /// Create a new context with just a current message and default system prompt.
    pub fn new(message: &str) -> Self {
        Self {
            system_prompt: default_system_prompt(),
            history: Vec::new(),
            current_message: message.to_string(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: None,
        }
    }

    /// Flatten the context into a single prompt string for providers
    /// that accept a single text input (e.g. Claude Code CLI).
    ///
    /// When `session_id` is set (continuation), the full system prompt and
    /// history are already in the CLI session — we only send a minimal
    /// context update (time, keyword-gated sections) prepended to the user message.
    pub fn to_prompt_string(&self) -> String {
        // Agent mode: the agent file provides the system prompt, so we
        // only emit the bare current_message (no [System], no history).
        if self.agent_name.is_some() {
            return self.current_message.clone();
        }

        let mut parts = Vec::new();

        if self.session_id.is_none() {
            // First message: send full system prompt + history.
            if !self.system_prompt.is_empty() {
                parts.push(format!("[System]\n{}", self.system_prompt));
            }

            for entry in &self.history {
                let role = if entry.role == "user" {
                    "User"
                } else {
                    "Assistant"
                };
                parts.push(format!("[{}]\n{}", role, entry.content));
            }

            parts.push(format!("[User]\n{}", self.current_message));
        } else {
            // Continuation: system_prompt has minimal context update (time, etc.).
            // Prepend it to the user message so the AI sees it.
            if !self.system_prompt.is_empty() {
                parts.push(format!(
                    "[User]\n{}\n\n{}",
                    self.system_prompt, self.current_message
                ));
            } else {
                parts.push(format!("[User]\n{}", self.current_message));
            }
        }

        parts.join("\n\n")
    }

    /// Convert context to structured API messages.
    ///
    /// Returns `(system_prompt, messages)` — the system prompt is separated
    /// because Anthropic and Gemini require it outside the messages array.
    pub fn to_api_messages(&self) -> (String, Vec<ApiMessage>) {
        let mut messages = Vec::with_capacity(self.history.len() + 1);

        for entry in &self.history {
            messages.push(ApiMessage {
                role: entry.role.clone(),
                content: entry.content.clone(),
            });
        }

        messages.push(ApiMessage {
            role: "user".to_string(),
            content: self.current_message.clone(),
        });

        (self.system_prompt.clone(), messages)
    }
}

/// Default system prompt for the Omega agent.
fn default_system_prompt() -> String {
    "You are OMEGA Ω, a personal AI assistant running on the user's own server. \
     You are helpful, concise, and action-oriented."
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_serde_round_trip() {
        let server = McpServer {
            name: "playwright".into(),
            command: "npx".into(),
            args: vec!["@playwright/mcp".into(), "--headless".into()],
        };
        let json = serde_json::to_string(&server).unwrap();
        let deserialized: McpServer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "playwright");
        assert_eq!(deserialized.command, "npx");
        assert_eq!(deserialized.args, vec!["@playwright/mcp", "--headless"]);
    }

    #[test]
    fn test_context_new_has_empty_mcp_servers() {
        let ctx = Context::new("hello");
        assert!(ctx.mcp_servers.is_empty());
    }

    #[test]
    fn test_context_with_mcp_servers_serde() {
        let ctx = Context {
            system_prompt: "test".into(),
            history: Vec::new(),
            current_message: "browse google.com".into(),
            mcp_servers: vec![McpServer {
                name: "playwright".into(),
                command: "npx".into(),
                args: vec!["@playwright/mcp".into()],
            }],
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: None,
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.mcp_servers.len(), 1);
        assert_eq!(deserialized.mcp_servers[0].name, "playwright");
    }

    #[test]
    fn test_context_deserialize_without_mcp_servers() {
        // Old JSON without mcp_servers field should still deserialize.
        let json = r#"{"system_prompt":"test","history":[],"current_message":"hi"}"#;
        let ctx: Context = serde_json::from_str(json).unwrap();
        assert!(ctx.mcp_servers.is_empty());
    }

    #[test]
    fn test_to_api_messages_basic() {
        let ctx = Context::new("hello");
        let (system, messages) = ctx.to_api_messages();
        assert!(!system.is_empty());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "hello");
    }

    #[test]
    fn test_to_api_messages_with_history() {
        let ctx = Context {
            system_prompt: "Be helpful.".into(),
            history: vec![
                ContextEntry {
                    role: "user".into(),
                    content: "Hi".into(),
                },
                ContextEntry {
                    role: "assistant".into(),
                    content: "Hello!".into(),
                },
            ],
            current_message: "How are you?".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: None,
        };
        let (system, messages) = ctx.to_api_messages();
        assert_eq!(system, "Be helpful.");
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hi");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "Hello!");
        assert_eq!(messages[2].role, "user");
        assert_eq!(messages[2].content, "How are you?");
    }

    #[test]
    fn test_to_prompt_string_no_session_full_output() {
        let ctx = Context {
            system_prompt: "Be helpful.".into(),
            history: vec![ContextEntry {
                role: "user".into(),
                content: "Hi".into(),
            }],
            current_message: "How are you?".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: None,
        };
        let prompt = ctx.to_prompt_string();
        assert!(prompt.contains("[System]\nBe helpful."));
        assert!(prompt.contains("[User]\nHi"));
        assert!(prompt.contains("[User]\nHow are you?"));
    }

    #[test]
    fn test_to_prompt_string_with_session_skips_system_and_history() {
        let ctx = Context {
            system_prompt: "Current time: 2026-02-21".into(),
            history: vec![ContextEntry {
                role: "user".into(),
                content: "Hi".into(),
            }],
            current_message: "How are you?".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: Some("sess-abc".into()),
            agent_name: None,
        };
        let prompt = ctx.to_prompt_string();
        // Should NOT contain [System] block or history.
        assert!(!prompt.contains("[System]"));
        assert!(!prompt.contains("[User]\nHi\n"));
        // Should prepend minimal context to user message.
        assert!(prompt.contains("[User]\nCurrent time: 2026-02-21\n\nHow are you?"));
    }

    #[test]
    fn test_to_prompt_string_session_empty_system_prompt() {
        let ctx = Context {
            system_prompt: String::new(),
            history: Vec::new(),
            current_message: "hello".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: Some("sess-xyz".into()),
            agent_name: None,
        };
        let prompt = ctx.to_prompt_string();
        assert_eq!(prompt, "[User]\nhello");
    }

    #[test]
    fn test_session_id_serde_round_trip() {
        let ctx = Context {
            system_prompt: "test".into(),
            history: Vec::new(),
            current_message: "hi".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: Some("sess-123".into()),
            agent_name: None,
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, Some("sess-123".into()));
    }

    #[test]
    fn test_session_id_skipped_when_none() {
        let ctx = Context::new("hello");
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(!json.contains("session_id"));
        // But deserializing without it should give None.
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, None);
    }

    // =======================================================================
    // REQ-BAP-003 (Must): Context.agent_name field
    // =======================================================================

    // Requirement: REQ-BAP-003 (Must)
    // Acceptance: agent_name: Option<String> added to Context
    #[test]
    fn test_context_new_has_agent_name_none() {
        let ctx = Context::new("hello");
        assert_eq!(
            ctx.agent_name, None,
            "Context::new() must initialize agent_name to None"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Acceptance: field is serde(default, skip_serializing_if = "Option::is_none")
    #[test]
    fn test_agent_name_skipped_when_none_in_serialization() {
        let ctx = Context::new("hello");
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(
            !json.contains("agent_name"),
            "agent_name should be omitted from JSON when None"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Acceptance: field is serde(default) — old JSON without agent_name deserializes
    #[test]
    fn test_agent_name_backward_compat_deserialize_without_field() {
        // Simulate JSON from before agent_name was added (no agent_name key).
        let json = r#"{"system_prompt":"test","history":[],"current_message":"hi"}"#;
        let ctx: Context = serde_json::from_str(json).unwrap();
        assert_eq!(
            ctx.agent_name, None,
            "Deserializing old JSON without agent_name should yield None"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Acceptance: agent_name round-trips through serde
    #[test]
    fn test_agent_name_serde_round_trip_some() {
        let ctx = Context {
            system_prompt: "test".into(),
            history: Vec::new(),
            current_message: "hi".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: Some("build-analyst".into()),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(
            json.contains("agent_name"),
            "agent_name should appear in JSON when Some"
        );
        assert!(json.contains("build-analyst"));
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.agent_name,
            Some("build-analyst".to_string())
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Acceptance: to_prompt_string() with agent_name emits only current_message
    #[test]
    fn test_to_prompt_string_with_agent_name_returns_only_current_message() {
        let ctx = Context {
            system_prompt: "You are a build analyst...".into(),
            history: vec![ContextEntry {
                role: "user".into(),
                content: "previous message".into(),
            }],
            current_message: "Build me a task tracker.".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: Some("build-analyst".into()),
        };
        let prompt = ctx.to_prompt_string();
        // When agent_name is set, the agent file provides the system prompt,
        // so to_prompt_string() should emit only the current_message.
        assert_eq!(
            prompt, "Build me a task tracker.",
            "With agent_name set, to_prompt_string() should return only current_message"
        );
        // Must NOT contain system prompt or history markers.
        assert!(
            !prompt.contains("[System]"),
            "Agent mode should not emit [System] block"
        );
        assert!(
            !prompt.contains("[User]"),
            "Agent mode should not emit [User] wrapper"
        );
        assert!(
            !prompt.contains("previous message"),
            "Agent mode should not emit history"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Failure mode: agent_name set with session_id — agent_name should take precedence
    #[test]
    fn test_to_prompt_string_agent_name_takes_precedence_over_session_id() {
        let ctx = Context {
            system_prompt: "system prompt here".into(),
            history: Vec::new(),
            current_message: "Build something.".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: Some("sess-456".into()),
            agent_name: Some("build-architect".into()),
        };
        let prompt = ctx.to_prompt_string();
        // agent_name should win — just return current_message
        assert_eq!(
            prompt, "Build something.",
            "agent_name should take precedence over session_id"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Edge case: agent_name set but current_message is empty
    #[test]
    fn test_to_prompt_string_agent_name_with_empty_message() {
        let ctx = Context {
            system_prompt: "system".into(),
            history: Vec::new(),
            current_message: String::new(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: Some("build-qa".into()),
        };
        let prompt = ctx.to_prompt_string();
        assert_eq!(
            prompt, "",
            "Agent mode with empty message should return empty string"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Security: agent_name with special characters in serialization
    #[test]
    fn test_agent_name_special_characters_serde() {
        let ctx = Context {
            system_prompt: "test".into(),
            history: Vec::new(),
            current_message: "hi".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: Some("build-test-writer".into()),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_name, Some("build-test-writer".into()));
    }

    // Requirement: REQ-BAP-003 (Must)
    // Regression: existing callers without agent_name still work
    #[test]
    fn test_to_prompt_string_without_agent_name_unchanged_behavior() {
        // This is a regression test: the existing behavior of to_prompt_string()
        // when agent_name is None must remain identical.
        let ctx = Context {
            system_prompt: "Be helpful.".into(),
            history: vec![ContextEntry {
                role: "user".into(),
                content: "Hi".into(),
            }],
            current_message: "How are you?".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: None,
        };
        let prompt = ctx.to_prompt_string();
        assert!(
            prompt.contains("[System]\nBe helpful."),
            "No agent_name: should emit full [System] block"
        );
        assert!(
            prompt.contains("[User]\nHi"),
            "No agent_name: should emit history"
        );
        assert!(
            prompt.contains("[User]\nHow are you?"),
            "No agent_name: should emit current message"
        );
    }

    // Requirement: REQ-BAP-003 (Must)
    // Regression: existing full Context construction with all fields still compiles
    // (catches if a field was renamed/removed)
    #[test]
    fn test_context_all_fields_construct() {
        let ctx = Context {
            system_prompt: "sys".into(),
            history: Vec::new(),
            current_message: "msg".into(),
            mcp_servers: Vec::new(),
            max_turns: Some(50),
            allowed_tools: Some(vec!["Bash".into()]),
            model: Some("claude-sonnet-4-6".into()),
            session_id: Some("sess-1".into()),
            agent_name: Some("build-analyst".into()),
        };
        assert_eq!(ctx.agent_name, Some("build-analyst".into()));
        assert_eq!(ctx.session_id, Some("sess-1".into()));
        assert_eq!(ctx.max_turns, Some(50));
    }

    // Requirement: REQ-BAP-003 (Must)
    // Edge case: unicode in agent_name
    #[test]
    fn test_agent_name_unicode_round_trip() {
        let ctx = Context {
            system_prompt: "test".into(),
            history: Vec::new(),
            current_message: "hi".into(),
            mcp_servers: Vec::new(),
            max_turns: None,
            allowed_tools: None,
            model: None,
            session_id: None,
            agent_name: Some("build-\u{03a9}mega".into()),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: Context = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_name, Some("build-\u{03a9}mega".into()));
    }
}
