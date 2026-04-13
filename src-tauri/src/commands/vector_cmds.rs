#[tauri::command]
pub fn start_vectorize() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn get_vectorize_progress() -> Result<String, String> { Ok("{}".to_string()) }
