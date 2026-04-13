#[tauri::command]
pub fn get_logs() -> Result<String, String> { Ok("[]".to_string()) }
