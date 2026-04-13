/// Remote embedding API fallback when local GPU inference is unavailable
/// Uses OpenAI-compatible embedding API

use crate::models::llm::LlmConfig;
use crate::models::gpu::GpuBackend;

const EMBEDDING_DIM: usize = 384;

/// Remote embedding client
pub struct RemoteEmbeddingClient {
    config: LlmConfig,
    http_client: reqwest::Client,
}

/// Result from remote embedding API
#[derive(Debug, Clone)]
pub struct RemoteEmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub dim: usize,
    pub tokens_used: Option<i64>,
}

impl RemoteEmbeddingClient {
    pub fn new(config: LlmConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self { config, http_client }
    }

    /// Check if this client has an embedding model configured
    pub fn has_embedding_model(&self) -> bool {
        self.config.embedding_model.is_some()
    }

    /// Get embeddings for a list of texts via remote API
    pub async fn embed(&self, texts: &[String]) -> Result<RemoteEmbeddingResult, String> {
        let embedding_model = self.config.embedding_model.as_ref()
            .ok_or_else(|| "No embedding model configured".to_string())?;

        let url = format!("{}embeddings", self.config.base_url);

        #[derive(serde::Serialize)]
        struct EmbeddingRequest {
            model: String,
            input: Vec<String>,
        }

        let request = EmbeddingRequest {
            model: embedding_model.clone(),
            input: texts.to_vec(),
        };

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Remote embedding request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Remote embedding API error {}: {}", status, body));
        }

        #[derive(serde::Deserialize)]
        struct EmbeddingResponse {
            data: Vec<EmbeddingData>,
            usage: Option<Usage>,
        }

        #[derive(serde::Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        #[derive(serde::Deserialize)]
        struct Usage {
            total_tokens: i64,
        }

        let embed_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse remote embedding response: {}", e))?;

        let embeddings: Vec<Vec<f32>> = embed_response.data.into_iter()
            .map(|d| d.embedding)
            .collect();

        Ok(RemoteEmbeddingResult {
            dim: embeddings.first().map(|e| e.len()).unwrap_or(EMBEDDING_DIM),
            embeddings,
            tokens_used: embed_response.usage.map(|u| u.total_tokens),
        })
    }

    /// Get embeddings in batches to avoid API limits
    pub async fn embed_batch(&self, texts: &[String], batch_size: usize) -> Result<RemoteEmbeddingResult, String> {
        let mut all_embeddings = Vec::new();
        let mut total_tokens = 0i64;

        for chunk in texts.chunks(batch_size) {
            let result = self.embed(chunk).await?;
            all_embeddings.extend(result.embeddings);
            if let Some(tokens) = result.tokens_used {
                total_tokens += tokens;
            }
        }

        Ok(RemoteEmbeddingResult {
            dim: all_embeddings.first().map(|e| e.len()).unwrap_or(EMBEDDING_DIM),
            embeddings: all_embeddings,
            tokens_used: Some(total_tokens),
        })
    }

    /// Get the configured embedding dimension
    pub fn embedding_dim(&self) -> i64 {
        self.config.embedding_dim
    }

    /// Determine the fallback backend based on availability
    pub fn determine_fallback_backend() -> GpuBackend {
        GpuBackend::Remote
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> LlmConfig {
        LlmConfig {
            id: "test-remote".to_string(),
            name: "Test Remote".to_string(),
            base_url: "https://api.example.com/v1/".to_string(),
            api_key: "sk-test-key".to_string(),
            model: "test-model".to_string(),
            embedding_model: Some("text-embedding-3-small".to_string()),
            embedding_dim: 384,
            is_default: false,
        }
    }

    fn make_config_no_embedding() -> LlmConfig {
        LlmConfig {
            id: "no-embed".to_string(),
            name: "No Embedding".to_string(),
            base_url: "https://api.example.com/v1/".to_string(),
            api_key: "sk-test".to_string(),
            model: "test-model".to_string(),
            embedding_model: None,
            embedding_dim: 1536,
            is_default: false,
        }
    }

    #[test]
    fn test_client_creation() {
        let config = make_test_config();
        let client = RemoteEmbeddingClient::new(config);
        assert!(client.has_embedding_model());
    }

    #[test]
    fn test_no_embedding_model() {
        let config = make_config_no_embedding();
        let client = RemoteEmbeddingClient::new(config);
        assert!(!client.has_embedding_model());
    }

    #[test]
    fn test_embedding_dim() {
        let config = make_test_config();
        let client = RemoteEmbeddingClient::new(config);
        assert_eq!(client.embedding_dim(), 384);
    }

    #[test]
    fn test_fallback_backend() {
        assert_eq!(RemoteEmbeddingClient::determine_fallback_backend(), GpuBackend::Remote);
    }

    #[tokio::test]
    async fn test_embed_fails_without_model() {
        let config = make_config_no_embedding();
        let client = RemoteEmbeddingClient::new(config);
        let result = client.embed(&["test".to_string()]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No embedding model"));
    }
}
