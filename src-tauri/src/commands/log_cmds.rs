use crate::db::init;
use crate::db::log_repo::{LogRepo, LogType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetLogsRequest {
    pub task_id: String,
    pub run_id: Option<String>,
    pub log_type: Option<String>,
    pub level: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub fn get_logs(request: GetLogsRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = LogRepo::new(&conn);
    let log_type = request.log_type.as_deref().and_then(LogType::from_str);

    let logs = repo
        .list_by_task(
            &request.task_id,
            request.run_id.as_deref(),
            log_type,
            request.level.as_deref(),
            request.limit.unwrap_or(100),
            request.offset.unwrap_or(0),
        )
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&logs).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_logs_empty() {
        let req = GetLogsRequest {
            task_id: "nonexistent".to_string(),
            run_id: None,
            log_type: None,
            level: None,
            limit: Some(100),
            offset: Some(0),
        };
        let result = get_logs(req).unwrap();
        let logs: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(logs.is_empty());
    }
}
