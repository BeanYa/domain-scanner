#[tauri::command]
pub fn filter_exact() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn filter_fuzzy() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn filter_regex() -> Result<String, String> { Ok("[]".to_string()) }

#[tauri::command]
pub fn filter_semantic() -> Result<String, String> { Ok("[]".to_string()) }
