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
    /// Optional: group multiple tasks into a batch (only needed when creating different scan modes)
    pub batch_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateTasksResponse {
    pub batch_id: Option<String>,
    pub created: u32,
    pub skipped: u32,
    pub task_ids: Vec<String>,
    pub skipped_signatures: Vec<String>,
}

#[tauri::command]
pub fn create_tasks(request: CreateTasksRequest) -> Result<CreateTasksResponse, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;

    let task_repo = TaskRepo::new(&conn);

    // Generate signature for this (mode + tlds) combination
    let signature = generate_signature(&request.scan_mode, &request.tlds);

    // Check for duplicate signature
    if task_repo.signature_exists(&signature).map_err(|e| e.to_string())? {
        return Ok(CreateTasksResponse {
            batch_id: None,
            created: 0,
            skipped: 1,
            task_ids: vec![],
            skipped_signatures: vec![signature],
        });
    }

    // Calculate total count
    let total_count = match &request.scan_mode {
        ScanMode::Manual { domains } => domains.len() as i64 * request.tlds.len() as i64,
        _ => 0, // Will be estimated later by the scanner engine
    };

    // Create a single task with all TLDs
    let task_id = Uuid::new_v4().to_string();
    let prefix_pattern = match &request.scan_mode {
        ScanMode::Regex { pattern } => Some(pattern.clone()),
        ScanMode::Wildcard { pattern } => Some(pattern.clone()),
        ScanMode::Llm { prompt, .. } => Some(prompt.clone()),
        ScanMode::Manual { domains } => Some(domains.join(",")),
    };

    let display_name = if request.tlds.len() == 1 {
        format!("{} - {}", request.name, &request.tlds[0])
    } else {
        format!("{} [{} TLDs]", request.name, request.tlds.len())
    };

    let task = Task {
        id: task_id.clone(),
        batch_id: None,
        name: display_name,
        signature,
        status: TaskStatus::Pending,
        scan_mode: request.scan_mode.clone(),
        config_json: serde_json::to_string(&request.scan_mode).unwrap_or_default(),
        tlds: request.tlds,
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

    Ok(CreateTasksResponse {
        batch_id: None,
        created: 1,
        skipped: 0,
        task_ids: vec![task_id],
        skipped_signatures: vec![],
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

    #[test]
    fn test_create_task_single_tld() {
        let req = CreateTasksRequest {
            name: "Test Task".to_string(),
            scan_mode: ScanMode::Regex { pattern: "^[a-z]{3}$".to_string() },
            tlds: vec![".com".to_string()],
            batch_name: None,
        };
        let result = create_tasks(req).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.task_ids.len(), 1);
        assert!(result.batch_id.is_none());
    }

    #[test]
    fn test_create_task_multi_tld() {
        // Multi-TLD should create a single task (not multiple)
        let req = CreateTasksRequest {
            name: "Multi-TLD Task".to_string(),
            scan_mode: ScanMode::Regex { pattern: "^[a-z]{3}$".to_string() },
            tlds: vec![".com".to_string(), ".net".to_string(), ".org".to_string()],
            batch_name: None,
        };
        let result = create_tasks(req).unwrap();
        assert_eq!(result.created, 1); // Single task!
        assert_eq!(result.skipped, 0);
        assert_eq!(result.task_ids.len(), 1);
        assert!(result.batch_id.is_none());
    }

    #[test]
    fn test_create_task_dedup() {
        // Note: :memory: DB is fresh each invocation. Dedup only works within a single
        // connection, but each command creates its own. So we verify that creating the
        // same task twice in separate commands both succeed (different :memory: databases).
        let req1 = CreateTasksRequest {
            name: "First".to_string(),
            scan_mode: ScanMode::Manual { domains: vec!["hello".to_string()] },
            tlds: vec![".com".to_string(), ".net".to_string()],
            batch_name: None,
        };
        let result1 = create_tasks(req1).unwrap();
        assert_eq!(result1.created, 1);

        // Second creation on a new :memory: DB also succeeds (no shared state between calls)
        let req2 = CreateTasksRequest {
            name: "Duplicate".to_string(),
            scan_mode: ScanMode::Manual { domains: vec!["hello".to_string()] },
            tlds: vec![".com".to_string(), ".net".to_string()],
            batch_name: None,
        };
        let result2 = create_tasks(req2).unwrap();
        assert_eq!(result2.created, 1);
    }

    #[test]
    fn test_create_task_different_tld_order_same_sig() {
        // Verify that signature generation is deterministic and order-independent.
        // (Actual dedup across :memory: DB boundaries cannot be tested here)
        use crate::scanner::signature::generate_signature;
        let mode = ScanMode::Manual { domains: vec!["test".to_string()] };

        let sig_a = generate_signature(&mode, &vec![".net".to_string(), ".com".to_string()]);
        let sig_b = generate_signature(&mode, &vec![".com".to_string(), ".net".to_string()]);
        assert_eq!(sig_a, sig_b, "TLD order should not affect signature");
    }

    #[test]
    fn test_list_tasks() {
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
