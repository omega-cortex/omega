//! Provider factory — builds the configured AI provider from config.

use omega_core::{config, traits::Provider};
use omega_providers::{
    anthropic::AnthropicProvider, claude_code::ClaudeCodeProvider, gemini::GeminiProvider,
    ollama::OllamaProvider, openai::OpenAiProvider, openrouter::OpenRouterProvider,
};

/// Build the configured provider, returning `(provider, model_fast, model_complex)`.
///
/// For Claude Code, `model_fast` and `model_complex` come from its config.
/// For all other providers, both are set to the provider's single `model` field.
pub fn build_provider(
    cfg: &config::Config,
    workspace_path: &std::path::Path,
) -> anyhow::Result<(Box<dyn Provider>, String, String)> {
    let ws = Some(workspace_path.to_path_buf());

    match cfg.provider.default.as_str() {
        "claude-code" => {
            let cc = cfg
                .provider
                .claude_code
                .as_ref()
                .cloned()
                .unwrap_or_default();
            let model_fast = cc.model.clone();
            let model_complex = cc.model_complex.clone();
            Ok((
                Box::new(ClaudeCodeProvider::from_config(
                    cc.max_turns,
                    cc.allowed_tools,
                    cc.timeout_secs,
                    ws,
                    cc.max_resume_attempts,
                    cc.model,
                )),
                model_fast,
                model_complex,
            ))
        }
        "ollama" => {
            let oc = cfg
                .provider
                .ollama
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("provider.ollama section missing in config"))?;
            let m = oc.model.clone();
            Ok((
                Box::new(OllamaProvider::from_config(
                    oc.base_url.clone(),
                    oc.model.clone(),
                    ws,
                )?),
                m.clone(),
                m,
            ))
        }
        "openai" => {
            let oc = cfg
                .provider
                .openai
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("provider.openai section missing in config"))?;
            let m = oc.model.clone();
            Ok((
                Box::new(OpenAiProvider::from_config(
                    oc.base_url.clone(),
                    oc.api_key.clone(),
                    oc.model.clone(),
                    ws,
                )?),
                m.clone(),
                m,
            ))
        }
        "anthropic" => {
            let ac =
                cfg.provider.anthropic.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("provider.anthropic section missing in config")
                })?;
            let m = ac.model.clone();
            Ok((
                Box::new(AnthropicProvider::from_config(
                    ac.api_key.clone(),
                    ac.model.clone(),
                    ac.max_tokens,
                    ws,
                )?),
                m.clone(),
                m,
            ))
        }
        "openrouter" => {
            let oc =
                cfg.provider.openrouter.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("provider.openrouter section missing in config")
                })?;
            let m = oc.model.clone();
            Ok((
                Box::new(OpenRouterProvider::from_config(
                    oc.api_key.clone(),
                    oc.model.clone(),
                    ws,
                )?),
                m.clone(),
                m,
            ))
        }
        "gemini" => {
            let gc = cfg
                .provider
                .gemini
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("provider.gemini section missing in config"))?;
            let m = gc.model.clone();
            Ok((
                Box::new(GeminiProvider::from_config(
                    gc.api_key.clone(),
                    gc.model.clone(),
                    ws,
                )?),
                m.clone(),
                m,
            ))
        }
        other => anyhow::bail!("unsupported provider: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omega_core::config::*;
    use std::path::PathBuf;

    /// Build a minimal Config with all defaults and the given provider name.
    fn test_config(provider_name: &str) -> Config {
        Config {
            omega: OmegaConfig::default(),
            auth: AuthConfig::default(),
            provider: ProviderConfig {
                default: provider_name.to_string(),
                ..Default::default()
            },
            channel: ChannelConfig::default(),
            memory: MemoryConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            scheduler: SchedulerConfig::default(),
            api: ApiConfig::default(),
        }
    }

    #[test]
    fn test_unsupported_provider_returns_error() {
        let cfg = test_config("nonexistent");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result.err().expect("should fail with unsupported provider");
        assert!(
            err.to_string().contains("unsupported provider"),
            "error should mention unsupported provider, got: {err}"
        );
    }

    #[test]
    fn test_claude_code_defaults_succeeds() {
        let cfg = test_config("claude-code");
        let ws = PathBuf::from("/tmp");
        let (provider, model_fast, model_complex) = build_provider(&cfg, &ws).unwrap();
        assert_eq!(provider.name(), "claude-code");
        // Default fast model is Sonnet, complex is Opus.
        assert!(
            model_fast.contains("sonnet"),
            "fast model should be sonnet, got: {model_fast}"
        );
        assert!(
            model_complex.contains("opus"),
            "complex model should be opus, got: {model_complex}"
        );
    }

    #[test]
    fn test_claude_code_custom_models() {
        let mut cfg = test_config("claude-code");
        cfg.provider.claude_code = Some(ClaudeCodeConfig {
            model: "custom-fast-model".to_string(),
            model_complex: "custom-complex-model".to_string(),
            ..Default::default()
        });
        let ws = PathBuf::from("/tmp");
        let (_provider, model_fast, model_complex) = build_provider(&cfg, &ws).unwrap();
        assert_eq!(model_fast, "custom-fast-model");
        assert_eq!(model_complex, "custom-complex-model");
    }

    #[test]
    fn test_ollama_missing_config_returns_error() {
        let cfg = test_config("ollama");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result
            .err()
            .expect("should fail with missing ollama config");
        assert!(
            err.to_string().contains("provider.ollama section missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_openai_missing_config_returns_error() {
        let cfg = test_config("openai");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result
            .err()
            .expect("should fail with missing openai config");
        assert!(
            err.to_string().contains("provider.openai section missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_anthropic_missing_config_returns_error() {
        let cfg = test_config("anthropic");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result
            .err()
            .expect("should fail with missing anthropic config");
        assert!(
            err.to_string()
                .contains("provider.anthropic section missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_openrouter_missing_config_returns_error() {
        let cfg = test_config("openrouter");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result
            .err()
            .expect("should fail with missing openrouter config");
        assert!(
            err.to_string()
                .contains("provider.openrouter section missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_gemini_missing_config_returns_error() {
        let cfg = test_config("gemini");
        let ws = PathBuf::from("/tmp");
        let result = build_provider(&cfg, &ws);
        let err = result
            .err()
            .expect("should fail with missing gemini config");
        assert!(
            err.to_string().contains("provider.gemini section missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_ollama_with_config_returns_same_model_for_both() {
        let mut cfg = test_config("ollama");
        cfg.provider.ollama = Some(OllamaConfig {
            enabled: true,
            base_url: "http://localhost:11434".to_string(),
            model: "llama3".to_string(),
        });
        let ws = PathBuf::from("/tmp");
        let (provider, model_fast, model_complex) = build_provider(&cfg, &ws).unwrap();
        assert_eq!(provider.name(), "ollama");
        assert_eq!(model_fast, "llama3");
        assert_eq!(model_complex, "llama3");
    }

    #[test]
    fn test_anthropic_with_config_succeeds() {
        let mut cfg = test_config("anthropic");
        cfg.provider.anthropic = Some(AnthropicConfig {
            enabled: true,
            api_key: "test-key".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
        });
        let ws = PathBuf::from("/tmp");
        let (provider, model_fast, model_complex) = build_provider(&cfg, &ws).unwrap();
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(model_fast, "claude-sonnet-4-20250514");
        assert_eq!(model_complex, "claude-sonnet-4-20250514");
    }
}
