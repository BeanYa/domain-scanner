#[tauri::command]
pub fn get_gpu_status() -> Result<String, String> { Ok("{}".to_string()) }

#[tauri::command]
pub fn update_gpu_config() -> Result<(), String> { Ok(()) }
