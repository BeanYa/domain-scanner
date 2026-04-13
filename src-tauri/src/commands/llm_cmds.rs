#[tauri::command]
pub fn list_llm_configs() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn save_llm_config() -> Result<String, String> { Ok("{}".to_string()) }

#[tauri::command]
pub fn test_llm_config() -> Result<String, String> { Ok("{}".to_string()) }
