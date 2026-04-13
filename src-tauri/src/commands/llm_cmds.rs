use crate::db::llm_repo::LlmRepo;
use crate::db::init;
use crate::llm::client::LlmClient;
use crate::llm::providers::LlmProviders;
use crate::models::llm::LlmConfig;
use serde::Deserialize;

#[tauri::command]
pub fn list_llm_configs() -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = LlmRepo::new(&conn);

    let configs = repo.list().map_err(|e| e.to_string())?;

    // Also include predefined providers as templates
    let mut result = serde_json::to_value(&configs).unwrap();
    if let Some(arr) = result.as_array_mut() {
        for (id, name, url) in LlmProviders::all_providers() {
            arr.push(serde_json::json!({
                "id": id,
                "name": name,
                "base_url": url,
                "is_template": true,
            }));
        }
    }

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct SaveLlmConfigRequest {
    pub id: Option<String>,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub embedding_model: Option<String>,
    pub embedding_dim: Option<i64>,
    pub is_default: Option<bool>,
}

#[tauri::command]
pub fn save_llm_config(request: SaveLlmConfigRequest) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = LlmRepo::new(&conn);

    let config = LlmConfig {
        id: request.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        name: request.name,
        base_url: request.base_url,
        api_key: request.api_key,
        model: request.model,
        embedding_model: request.embedding_model,
        embedding_dim: request.embedding_dim.unwrap_or(1536),
        is_default: request.is_default.unwrap_or(false),
    };

    // Try update first, then create
    if repo.get_by_id(&config.id).map_err(|e| e.to_string())?.is_some() {
        repo.update(&config).map_err(|e| e.to_string())?;
    } else {
        repo.create(&config).map_err(|e| e.to_string())?;
    }

    serde_json::to_string(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_llm_config(config_id: String) -> Result<String, String> {
    // Fetch config in a blocking scope to avoid Send issues
    let config_result: Result<Option<LlmConfig>, String> = {
        let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
        let repo = LlmRepo::new(&conn);
        repo.get_by_id(&config_id).map_err(|e| e.to_string())
    };

    let config = config_result?
        .ok_or_else(|| format!("LLM config not found: {}", config_id))?;

    let client = LlmClient::new(config);
    match client.test_connection().await {
        Ok(()) => Ok(serde_json::json!({"success": true, "message": "Connection successful"}).to_string()),
        Err(e) => Ok(serde_json::json!({"success": false, "message": e}).to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_llm_configs() {
        let result = list_llm_configs().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn test_save_llm_config() {
        let req = SaveLlmConfigRequest {
            id: None,
            name: "Test LLM".to_string(),
            base_url: "https://api.example.com/v1/".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4".to_string(),
            embedding_model: Some("text-embedding-3-small".to_string()),
            embedding_dim: Some(1536),
            is_default: Some(false),
        };
        let result = save_llm_config(req).unwrap();
        let config: LlmConfig = serde_json::from_str(&result).unwrap();
        assert_eq!(config.name, "Test LLM");
        assert_eq!(config.model, "gpt-4");
    }
}
