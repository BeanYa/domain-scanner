#[tauri::command]
pub fn list_proxies() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn create_proxy() -> Result<String, String> { Ok("{}".to_string()) }

#[tauri::command]
pub fn test_proxy() -> Result<String, String> { Ok("{}".to_string()) }
