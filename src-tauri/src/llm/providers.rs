use crate::models::llm::LlmConfig;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub region: &'static str,
    pub category: &'static str,
    pub base_url: &'static str,
    pub model: &'static str,
    pub embedding_model: Option<&'static str>,
    pub embedding_dim: i64,
    pub vector_ready: bool,
    pub notes: &'static str,
}

/// Pre-configured LLM provider templates
pub struct LlmProviders;

impl LlmProviders {
    /// GLM (Zhipu AI) configuration
    pub fn glm4(api_key: &str) -> LlmConfig {
        LlmConfig {
            id: "zhipu-api".to_string(),
            name: "Zhipu API".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4/".to_string(),
            api_key: api_key.to_string(),
            model: "glm-5.1".to_string(),
            embedding_model: Some("embedding-3".to_string()),
            embedding_dim: 384,
            is_default: false,
        }
    }

    /// MiniMax configuration
    pub fn minimax(api_key: &str) -> LlmConfig {
        LlmConfig {
            id: "minimax-global".to_string(),
            name: "MiniMax Global".to_string(),
            base_url: "https://api.minimax.io/v1/".to_string(),
            api_key: api_key.to_string(),
            model: "MiniMax-M2.7".to_string(),
            embedding_model: None,
            embedding_dim: 384,
            is_default: false,
        }
    }

    /// Zhipu AI configuration (alias for GLM)
    pub fn zhipu(api_key: &str) -> LlmConfig {
        Self::glm4(api_key)
    }

    /// OpenAI-compatible configuration
    pub fn openai_compatible(
        id: &str,
        name: &str,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> LlmConfig {
        LlmConfig {
            id: id.to_string(),
            name: name.to_string(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            embedding_model: None,
            embedding_dim: 384,
            is_default: false,
        }
    }

    /// Get all predefined providers (with placeholder API keys)
    pub fn all_providers() -> Vec<ProviderTemplate> {
        vec![
            ProviderTemplate {
                id: "openrouter-free-embedding",
                name: "OpenRouter Free Embedding",
                region: "Global",
                category: "Embedding API",
                base_url: "https://openrouter.ai/api/v1/",
                model: "nvidia/llama-nemotron-embed-vl-1b-v2:free",
                embedding_model: Some("nvidia/llama-nemotron-embed-vl-1b-v2:free"),
                embedding_dim: 384,
                vector_ready: true,
                notes: "Free OpenRouter embedding preset. The app requests 384 dimensions because the current sqlite-vec index is fixed to 384.",
            },
            ProviderTemplate {
                id: "openai-compatible",
                name: "OpenAI Compatible",
                region: "Global",
                category: "API",
                base_url: "https://api.openai.com/v1/",
                model: "gpt-4o-mini",
                embedding_model: Some("text-embedding-3-small"),
                embedding_dim: 384,
                vector_ready: true,
                notes: "Generic OpenAI-compatible API preset. Uses 384-dimensional embeddings for the current sqlite-vec index.",
            },
            ProviderTemplate {
                id: "kimi-china",
                name: "Kimi / Moonshot China",
                region: "China",
                category: "API",
                base_url: "https://api.moonshot.cn/v1/",
                model: "kimi-k2.5",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "China Moonshot endpoint. Chat/coding preset only; configure a separate embeddings model before vectorization.",
            },
            ProviderTemplate {
                id: "kimi-global",
                name: "Kimi / Moonshot Global",
                region: "Global",
                category: "API",
                base_url: "https://api.moonshot.ai/v1/",
                model: "kimi-k2.5",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Global Moonshot endpoint. Chat/coding preset only; configure a separate embeddings model before vectorization.",
            },
            ProviderTemplate {
                id: "zhipu-api",
                name: "Zhipu / BigModel API",
                region: "China",
                category: "API",
                base_url: "https://open.bigmodel.cn/api/paas/v4/",
                model: "glm-5.1",
                embedding_model: Some("embedding-3"),
                embedding_dim: 384,
                vector_ready: true,
                notes: "General BigModel API endpoint. Embedding-3 supports configurable dimensions; this app stores 384-dimensional vectors.",
            },
            ProviderTemplate {
                id: "zhipu-coding-plan",
                name: "Zhipu GLM Coding Plan",
                region: "China",
                category: "Coding Plan",
                base_url: "https://open.bigmodel.cn/api/coding/paas/v4/",
                model: "GLM-5.1",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Dedicated Coding Plan endpoint for coding agents. It is not intended for general embeddings/vectorization calls.",
            },
            ProviderTemplate {
                id: "zai-api",
                name: "Z.ai API",
                region: "Global",
                category: "API",
                base_url: "https://api.z.ai/api/paas/v4/",
                model: "glm-5.1",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "General Z.ai OpenAI-compatible endpoint. Add an embeddings model if your account exposes one.",
            },
            ProviderTemplate {
                id: "zai-coding-plan",
                name: "Z.ai GLM Coding Plan",
                region: "Global",
                category: "Coding Plan",
                base_url: "https://api.z.ai/api/coding/paas/v4/",
                model: "GLM-5.1",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Dedicated Coding Plan endpoint for coding agents. Use the general API endpoint for non-coding API workloads.",
            },
            ProviderTemplate {
                id: "minimax-china",
                name: "MiniMax China",
                region: "China",
                category: "Token Plan / API",
                base_url: "https://api.minimaxi.com/v1/",
                model: "MiniMax-M2.7",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "China endpoint for MiniMax M2.7 chat/coding. This preset does not configure embeddings.",
            },
            ProviderTemplate {
                id: "minimax-global",
                name: "MiniMax Global",
                region: "Global",
                category: "Token Plan / API",
                base_url: "https://api.minimax.io/v1/",
                model: "MiniMax-M2.7",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "International endpoint for MiniMax M2.7 chat/coding. This preset does not configure embeddings.",
            },
            ProviderTemplate {
                id: "openrouter",
                name: "OpenRouter",
                region: "Global",
                category: "Router API",
                base_url: "https://openrouter.ai/api/v1/",
                model: "openrouter/auto",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Unified OpenAI-compatible router. Pick a concrete model if you do not want OpenRouter automatic routing.",
            },
            ProviderTemplate {
                id: "alibaba-cloud-model-studio-china",
                name: "Alibaba Cloud Model Studio China",
                region: "China",
                category: "API",
                base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1/",
                model: "qwen3-coder-plus",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "China Beijing endpoint for Alibaba Cloud Model Studio. Uses Qwen3 Coder by default; configure embeddings separately for vectorization.",
            },
            ProviderTemplate {
                id: "alibaba-cloud-model-studio-global",
                name: "Alibaba Cloud Model Studio Global",
                region: "Global",
                category: "API",
                base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1/",
                model: "qwen3-coder-plus",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Singapore endpoint for Alibaba Cloud Model Studio. Regional API keys differ from the China endpoint.",
            },
            ProviderTemplate {
                id: "volcengine-ark-api",
                name: "Volcengine Ark API",
                region: "China",
                category: "API",
                base_url: "https://ark.cn-beijing.volces.com/api/v3/",
                model: "ep-xxxxxxxx",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "General Ark OpenAI-compatible endpoint. Replace the model placeholder with your Ark endpoint ID.",
            },
            ProviderTemplate {
                id: "volcengine-ark-coding-plan",
                name: "Volcengine Ark Coding Plan",
                region: "China",
                category: "Coding Plan",
                base_url: "https://ark.cn-beijing.volces.com/api/coding/v3/",
                model: "ark-code-latest",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "OpenAI-compatible Coding Plan gateway for Ark. The default model follows the Ark console selection.",
            },
            ProviderTemplate {
                id: "tencent-cloud-codebuddy-code",
                name: "Tencent Cloud CodeBuddy Code",
                region: "China",
                category: "Coding Plan",
                base_url: "https://api.lkeap.cloud.tencent.com/coding/v3/",
                model: "tc-code-latest",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Tencent Cloud CodeBuddy Code OpenAI-compatible endpoint. Use a Coding Plan API key in sk-sp-* format.",
            },
            ProviderTemplate {
                id: "tencent-cloud-coding-plan",
                name: "Tencent Cloud Coding Plan",
                region: "China",
                category: "Coding Plan",
                base_url: "https://api.lkeap.cloud.tencent.com/plan/v3/",
                model: "tc-code-latest",
                embedding_model: None,
                embedding_dim: 384,
                vector_ready: false,
                notes: "Tencent Cloud Coding Plan endpoint used by CodeBuddy plan integrations. Use a Coding Plan API key in sk-sp-* format.",
            },
        ]
    }

    /// Providers that can be used by this app's vectorization workflow.
    pub fn embedding_providers() -> Vec<ProviderTemplate> {
        Self::all_providers()
            .into_iter()
            .filter(|provider| {
                provider
                    .embedding_model
                    .is_some_and(|model| !model.trim().is_empty())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glm4_config() {
        let config = LlmProviders::glm4("sk-test");
        assert_eq!(config.id, "zhipu-api");
        assert_eq!(config.model, "glm-5.1");
        assert!(config.base_url.contains("bigmodel"));
        assert!(config.embedding_model.is_some());
        assert_eq!(config.embedding_dim, 384);
    }

    #[test]
    fn test_minimax_config() {
        let config = LlmProviders::minimax("sk-test");
        assert_eq!(config.id, "minimax-global");
        assert!(config.base_url.contains("minimax"));
        assert_eq!(config.model, "MiniMax-M2.7");
    }

    #[test]
    fn test_openai_compatible_config() {
        let config = LlmProviders::openai_compatible(
            "custom",
            "Custom",
            "https://api.custom.com/v1/",
            "sk-test",
            "gpt-4",
        );
        assert_eq!(config.id, "custom");
        assert_eq!(config.model, "gpt-4");
        assert!(config.embedding_model.is_none());
        assert_eq!(config.embedding_dim, 384);
    }

    #[test]
    fn test_all_providers_not_empty() {
        let providers = LlmProviders::all_providers();
        assert!(!providers.is_empty());
        assert!(providers.len() >= 17);
    }

    #[test]
    fn test_provider_templates_include_common_coding_apis() {
        let providers = LlmProviders::all_providers();
        let ids: Vec<&str> = providers.iter().map(|provider| provider.id).collect();
        assert!(ids.contains(&"kimi-china"));
        assert!(ids.contains(&"kimi-global"));
        assert!(ids.contains(&"zhipu-api"));
        assert!(ids.contains(&"zai-api"));
        assert!(ids.contains(&"zai-coding-plan"));
        assert!(ids.contains(&"minimax-china"));
        assert!(ids.contains(&"minimax-global"));
        assert!(ids.contains(&"openrouter"));
        assert!(ids.contains(&"alibaba-cloud-model-studio-china"));
        assert!(ids.contains(&"alibaba-cloud-model-studio-global"));
        assert!(ids.contains(&"volcengine-ark-api"));
        assert!(ids.contains(&"volcengine-ark-coding-plan"));
        assert!(ids.contains(&"tencent-cloud-codebuddy-code"));
        assert!(ids.contains(&"tencent-cloud-coding-plan"));

        let vector_ready = providers
            .iter()
            .filter(|provider| provider.vector_ready)
            .count();
        assert!(vector_ready >= 2);
    }

    #[test]
    fn test_embedding_providers_exclude_chat_only_presets() {
        let providers = LlmProviders::embedding_providers();
        let ids: Vec<&str> = providers.iter().map(|provider| provider.id).collect();
        assert!(ids.contains(&"openrouter-free-embedding"));
        assert!(ids.contains(&"openai-compatible"));
        assert!(ids.contains(&"zhipu-api"));
        assert!(!ids.contains(&"minimax-china"));
        assert!(!ids.contains(&"zai-coding-plan"));
        assert!(providers
            .iter()
            .all(|provider| provider.embedding_model.is_some()));
    }
}
