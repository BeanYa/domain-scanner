use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub embedding_model: Option<String>,
    pub embedding_dim: i64,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub model: String,
    pub dim: i64,
    pub backend: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_roundtrip() {
        let config = LlmConfig {
            id: "glm-4".to_string(),
            name: "GLM-4".to_string(),
            base_url: "https://open.bigmodel.cn/api/paas/v4/".to_string(),
            api_key: "sk-test".to_string(),
            model: "glm-4".to_string(),
            embedding_model: Some("embedding-2".to_string()),
            embedding_dim: 1024,
            is_default: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LlmConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.base_url, deserialized.base_url);
        assert_eq!(config.embedding_dim, deserialized.embedding_dim);
    }
}
