use crate::db::batch_repo::BatchRepo;
use crate::db::init;
use crate::db::task_repo::TaskRepo;
use crate::models::task::TaskStatus;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListBatchesRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub fn list_batches(request: ListBatchesRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = BatchRepo::new(&conn);

    let batches = repo
        .list(request.limit.unwrap_or(100), request.offset.unwrap_or(0))
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&batches).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_pause(batch_id: String) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let task_repo = TaskRepo::new(&conn);

    let tasks = task_repo
        .list(None, Some(&batch_id), 10000, 0)
        .map_err(|e| e.to_string())?;

    for task in tasks {
        if task.status == TaskStatus::Running {
            task_repo
                .update_status(&task.id, &TaskStatus::Paused)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub fn batch_resume(batch_id: String) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let task_repo = TaskRepo::new(&conn);

    let tasks = task_repo
        .list(None, Some(&batch_id), 10000, 0)
        .map_err(|e| e.to_string())?;

    for task in tasks {
        if task.status == TaskStatus::Paused {
            task_repo
                .update_status(&task.id, &TaskStatus::Running)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_batches() {
        let req = ListBatchesRequest {
            limit: Some(100),
            offset: Some(0),
        };
        let result = list_batches(req).unwrap();
        let batches: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(batches.is_array());
    }
}
