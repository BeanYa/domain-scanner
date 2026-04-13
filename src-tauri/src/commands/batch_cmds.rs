#[tauri::command]
pub fn list_batches() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn batch_pause() -> Result<(), String> { Ok(()) }

#[tauri::command]
pub fn batch_resume() -> Result<(), String> { Ok(()) }
