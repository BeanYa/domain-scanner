use crate::models::llm::LlmConfig;
use serde::{Deserialize, Serialize};

/// Chat completion request
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<i32>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat completion response
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

/// Embedding request
#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
}

/// Embedding response
#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
}

/// LLM Client - OpenAI-compatible API client
pub struct LlmClient {
    config: LlmConfig,
    http_client: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            config,
            http_client,
        }
    }

    /// Send a chat completion request
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        let url = format!("{}chat/completions", self.config.base_url);
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Chat request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Chat API error {}: {}", status, body));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse chat response: {}", e))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from chat API".to_string())
    }

    /// Get embeddings for a list of texts
    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        let embedding_model = self
            .config
            .embedding_model
            .clone()
            .ok_or_else(|| "No embedding model configured".to_string())?;

        let url = format!("{}embeddings", self.config.base_url);
        let request = EmbeddingRequest {
            model: embedding_model,
            input: texts,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Embedding request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Embedding API error {}: {}", status, body));
        }

        let embed_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse embedding response: {}", e))?;

        Ok(embed_response
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }

    /// Test the connection to the LLM API
    pub async fn test_connection(&self) -> Result<(), String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, respond with OK".to_string(),
        }];
        self.chat(messages).await?;
        Ok(())
    }

    /// Get the config
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_config() -> LlmConfig {
        LlmConfig {
            id: "test".to_string(),
            name: "Test LLM".to_string(),
            base_url: "https://api.example.com/v1/".to_string(),
            api_key: "sk-test-key".to_string(),
            model: "test-model".to_string(),
            embedding_model: Some("test-embedding".to_string()),
            embedding_dim: 384,
            is_default: true,
        }
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "test".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("user"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{"choices":[{"message":{"role":"assistant","content":"OK"}}]}"#;
        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices[0].message.content, "OK");
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{"data":[{"embedding":[0.1,0.2,0.3]}]}"#;
        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data[0].embedding.len(), 3);
        assert!((response.data[0].embedding[0] - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_client_creation() {
        let config = make_test_config();
        let client = LlmClient::new(config);
        assert_eq!(client.config().model, "test-model");
    }
}
