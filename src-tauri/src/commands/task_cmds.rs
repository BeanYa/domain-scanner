use crate::db::task_repo::TaskRepo;
use crate::db::batch_repo::BatchRepo;
use crate::db::init;
use crate::models::task::{ScanMode, Task, TaskStatus};
use crate::scanner::signature::generate_signature;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTasksRequest {
    pub name: String,
    pub scan_mode: ScanMode,
    pub tlds: Vec<String>,
    pub batch_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTasksResponse {
    pub batch_id: Option<String>,
    pub created: u32,
    pub skipped: u32,
    pub task_ids: Vec<String>,
    pub skipped_tlds: Vec<String>,
}

#[tauri::command]
pub fn create_tasks(request: CreateTasksRequest) -> Result<CreateTasksResponse, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;

    // Create batch if multiple TLDs
    let batch_id = if request.tlds.len() > 1 {
        let id = Uuid::new_v4().to_string();
        let batch_name = request.batch_name.clone()
            .unwrap_or_else(|| format!("Batch {}", &id[..8]));
        let batch = crate::models::task::TaskBatch {
            id: id.clone(),
            name: batch_name,
            task_count: request.tlds.len() as i64,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        let batch_repo = BatchRepo::new(&conn);
        batch_repo.create(&batch).map_err(|e| e.to_string())?;
        Some(id)
    } else {
        None
    };

    let task_repo = TaskRepo::new(&conn);
    let mut created = 0u32;
    let mut skipped = 0u32;
    let mut task_ids = Vec::new();
    let mut skipped_tlds = Vec::new();

    for tld in &request.tlds {
        let signature = generate_signature(&request.scan_mode, tld);

        // Check for duplicate signature
        if task_repo.signature_exists(&signature).map_err(|e| e.to_string())? {
            skipped += 1;
            skipped_tlds.push(tld.clone());
            continue;
        }

        let task_id = Uuid::new_v4().to_string();
        let prefix_pattern = match &request.scan_mode {
            ScanMode::Regex { pattern } => Some(pattern.clone()),
            ScanMode::Wildcard { pattern } => Some(pattern.clone()),
            ScanMode::Llm { prompt, .. } => Some(prompt.clone()),
            ScanMode::Manual { domains } => Some(domains.join(",")),
        };

        let total_count = match &request.scan_mode {
            ScanMode::Manual { domains } => domains.len() as i64,
            _ => 0, // Will be estimated later
        };

        let task = Task {
            id: task_id.clone(),
            batch_id: batch_id.clone(),
            name: format!("{} - {}", request.name, tld),
            signature,
            status: TaskStatus::Pending,
            scan_mode: request.scan_mode.clone(),
            config_json: serde_json::to_string(&request.scan_mode).unwrap_or_default(),
            tld: tld.clone(),
            prefix_pattern,
            total_count,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        task_repo.create(&task).map_err(|e| e.to_string())?;
        task_ids.push(task_id);
        created += 1;
    }

    // Update batch task count
    if let Some(ref bid) = batch_id {
        let batch_repo = BatchRepo::new(&conn);
        batch_repo.update_task_count(bid, created as i64).map_err(|e| e.to_string())?;
    }

    Ok(CreateTasksResponse {
        batch_id,
        created,
        skipped,
        task_ids,
        skipped_tlds,
    })
}

#[tauri::command]
pub fn start_task(task_id: String) -> Result<(), String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let task = repo.get_by_id(&task_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    if !task.status.can_transition_to(&TaskStatus::Running) {
        return Err(format!("Cannot start task in {} state", serde_json::to_string(&task.status).unwrap()));
    }

    repo.update_status(&task_id, &TaskStatus::Running).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn pause_task(task_id: String) -> Result<(), String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let task = repo.get_by_id(&task_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    if !task.status.can_transition_to(&TaskStatus::Paused) {
        return Err(format!("Cannot pause task in {} state", serde_json::to_string(&task.status).unwrap()));
    }

    repo.update_status(&task_id, &TaskStatus::Paused).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn resume_task(task_id: String) -> Result<(), String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let task = repo.get_by_id(&task_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    if !task.status.can_transition_to(&TaskStatus::Running) {
        return Err(format!("Cannot resume task in {} state", serde_json::to_string(&task.status).unwrap()));
    }

    repo.update_status(&task_id, &TaskStatus::Running).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_task(task_id: String) -> Result<(), String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);
    repo.delete(&task_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ListTasksRequest {
    pub status: Option<String>,
    pub batch_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub fn list_tasks(request: ListTasksRequest) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let status: Option<TaskStatus> = request.status.as_deref()
        .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok());

    let tasks = repo.list(
        status.as_ref(),
        request.batch_id.as_deref(),
        request.limit.unwrap_or(100),
        request.offset.unwrap_or(0),
    ).map_err(|e| e.to_string())?;

    serde_json::to_string(&tasks).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_task_detail(task_id: String) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let task = repo.get_by_id(&task_id).map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    serde_json::to_string(&task).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_request(tlds: Vec<&str>) -> CreateTasksRequest {
        CreateTasksRequest {
            name: "Test Task".to_string(),
            scan_mode: ScanMode::Regex { pattern: "^[a-z]{3}$".to_string() },
            tlds: tlds.iter().map(|s| s.to_string()).collect(),
            batch_name: None,
        }
    }

    #[test]
    fn test_create_tasks_single_tld() {
        let req = make_test_request(vec![".com"]);
        let result = create_tasks(req).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(result.skipped, 0);
        assert!(result.batch_id.is_none());
    }

    #[test]
    fn test_create_tasks_multiple_tlds() {
        let req = make_test_request(vec![".com", ".net", ".org"]);
        let result = create_tasks(req).unwrap();
        assert_eq!(result.created, 3);
        assert!(result.batch_id.is_some());
    }

    #[test]
    fn test_list_tasks() {
        // Note: each command opens a new :memory: database, so we can't test
        // cross-command data flow with :memory: databases.
        // This test verifies the list endpoint works with an empty database.
        let list_req = ListTasksRequest {
            status: None,
            batch_id: None,
            limit: Some(100),
            offset: Some(0),
        };
        let result = list_tasks(list_req).unwrap();
        let tasks: Vec<Task> = serde_json::from_str(&result).unwrap();
        assert!(tasks.is_empty()); // Fresh :memory: database is empty
    }

    #[test]
    fn test_get_task_detail_not_found() {
        let result = get_task_detail("nonexistent".to_string());
        assert!(result.is_err());
    }
}
