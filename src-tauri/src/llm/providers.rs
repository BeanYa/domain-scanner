use crate::models::llm::LlmConfig;

/// Pre-configured LLM provider templates
pub struct LlmProviders;

impl LlmProviders {
    /// GLM (Zhipu AI) configuration
    pub fn glm4(api_key: &str) -> LlmConfig {
        LlmConfig {
            id: "glm-4".to_string(),
            name: "GLM-4".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4/".to_string(),
            api_key: api_key.to_string(),
            model: "glm-4".to_string(),
            embedding_model: Some("embedding-2".to_string()),
            embedding_dim: 1024,
            is_default: false,
        }
    }

    /// MiniMax configuration
    pub fn minimax(api_key: &str) -> LlmConfig {
        LlmConfig {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            base_url: "https://api.minimax.chat/v1/".to_string(),
            api_key: api_key.to_string(),
            model: "abab6.5s-chat".to_string(),
            embedding_model: Some("embo-01".to_string()),
            embedding_dim: 1024,
            is_default: false,
        }
    }

    /// Zhipu AI configuration (alias for GLM)
    pub fn zhipu(api_key: &str) -> LlmConfig {
        Self::glm4(api_key)
    }

    /// OpenAI-compatible configuration
    pub fn openai_compatible(id: &str, name: &str, base_url: &str, api_key: &str, model: &str) -> LlmConfig {
        LlmConfig {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            embedding_model: None,
            embedding_dim: 1536,
            is_default: false,
        }
    }

    /// Get all predefined providers (with placeholder API keys)
    pub fn all_providers() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("glm-4", "GLM-4", "https://open.bigmodel.cn/api/paas/v4/"),
            ("minimax", "MiniMax", "https://api.minimax.chat/v1/"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glm4_config() {
        let config = LlmProviders::glm4("sk-test");
        assert_eq!(config.id, "glm-4");
        assert_eq!(config.model, "glm-4");
        assert!(config.base_url.contains("bigmodel"));
        assert!(config.embedding_model.is_some());
    }

    #[test]
    fn test_minimax_config() {
        let config = LlmProviders::minimax("sk-test");
        assert_eq!(config.id, "minimax");
        assert!(config.base_url.contains("minimax"));
    }

    #[test]
    fn test_openai_compatible_config() {
        let config = LlmProviders::openai_compatible(
            "custom", "Custom", "https://api.custom.com/v1/", "sk-test", "gpt-4"
        );
        assert_eq!(config.id, "custom");
        assert_eq!(config.model, "gpt-4");
        assert!(config.embedding_model.is_none());
    }

    #[test]
    fn test_all_providers_not_empty() {
        let providers = LlmProviders::all_providers();
        assert!(!providers.is_empty());
        assert!(providers.len() >= 2);
    }
}
