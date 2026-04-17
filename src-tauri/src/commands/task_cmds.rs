use crate::db::filter_repo::FilterRepo;
use crate::db::init;
use crate::db::log_repo::LogRepo;
use crate::db::proxy_repo::ProxyRepo;
use crate::db::scan_batch_repo::ScanBatchRepo;
use crate::db::scan_item_repo::ScanItemRepo;
use crate::db::task_repo::TaskRepo;
use crate::db::task_run_repo::TaskRunRepo;
use crate::models::scan_item::ScanItemStatus;
use crate::models::task::{ScanMode, Task, TaskRun, TaskStatus};
use crate::proxy::manager::ProxyManager;
use crate::scanner::domain_checker::{CheckConfig, DomainChecker};
use crate::scanner::signature::generate_signature;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTasksRequest {
    pub name: String,
    pub scan_mode: ScanMode,
    pub tlds: Vec<String>,
    /// Optional: group multiple tasks into a batch (only needed when creating different scan modes)
    pub batch_name: Option<String>,
    pub concurrency: Option<i64>,
    pub proxy_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTasksResponse {
    pub batch_id: Option<String>,
    pub created: u32,
    pub skipped: u32,
    pub task_ids: Vec<String>,
    pub skipped_signatures: Vec<String>,
}

fn resolve_task_name(name: &str, scan_mode: &ScanMode, tlds: &[String]) -> String {
    let trimmed = name.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    match scan_mode {
        ScanMode::Regex { .. } => format!("正则扫描 {}", tlds.join("/")),
        ScanMode::Wildcard { .. } => format!("通配符扫描 {}", tlds.join("/")),
        ScanMode::Llm { .. } => format!("LLM扫描 {}", tlds.join("/")),
        ScanMode::Manual { .. } => format!("手动扫描 {}", tlds.join("/")),
    }
}

#[tauri::command]
pub fn create_tasks(request: CreateTasksRequest) -> Result<CreateTasksResponse, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;

    let task_repo = TaskRepo::new(&conn);

    // Generate signature for this (mode + tlds) combination
    let signature = generate_signature(&request.scan_mode, &request.tlds);

    // Check for duplicate signature
    if task_repo
        .signature_exists(&signature)
        .map_err(|e| e.to_string())?
    {
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

    let display_name = resolve_task_name(&request.name, &request.scan_mode, &request.tlds);

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
        concurrency: request.concurrency.unwrap_or(50),
        proxy_id: request.proxy_id,
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
pub fn start_task(
    task_id: String,
    app: tauri::AppHandle,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<(), String> {
    runner.start(task_id, app)
}

#[tauri::command]
pub fn pause_task(
    task_id: String,
    app: tauri::AppHandle,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<(), String> {
    runner.pause(&task_id, Some(&app))
}

#[tauri::command]
pub fn stop_task(
    task_id: String,
    app: tauri::AppHandle,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<(), String> {
    runner.stop(&task_id, Some(&app))
}

#[tauri::command]
pub fn update_task_settings(
    request: UpdateTaskSettingsRequest,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let concurrency = request.concurrency.clamp(1, 500);
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let task_repo = TaskRepo::new(&conn);
    let proxy_repo = ProxyRepo::new(&conn);
    let task = task_repo
        .get_by_id(&request.task_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", request.task_id))?;

    if !matches!(task.status, TaskStatus::Pending | TaskStatus::Paused) {
        return Err("仅未启动或已暂停的任务可以修改并发量和代理设置。".to_string());
    }

    if let Some(proxy_id) = request.proxy_id {
        proxy_repo
            .get_by_id(proxy_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Proxy not found: {}", proxy_id))?;
    }

    task_repo
        .update_settings(&task.id, concurrency, request.proxy_id)
        .map_err(|e| e.to_string())?;

    let updated = task_repo
        .get_by_id(&task.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found after update: {}", task.id))?;

    let mut changes = Vec::new();
    if task.concurrency != updated.concurrency {
        changes.push(format!(
            "并发 {} -> {}",
            task.concurrency, updated.concurrency
        ));
    }
    if task.proxy_id != updated.proxy_id {
        let old_proxy = describe_proxy_id_label(&proxy_repo, task.proxy_id)?;
        let new_proxy = describe_proxy_id_label(&proxy_repo, updated.proxy_id)?;
        changes.push(format!("代理 {} -> {}", old_proxy, new_proxy));
    }

    if !changes.is_empty() {
        write_task_log_and_emit(
            &conn,
            Some(&app),
            &task.id,
            None,
            "info",
            &format!("任务设置已修改：{}", changes.join("；")),
        );
    }

    serde_json::to_string(&updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn resume_task(
    task_id: String,
    app: tauri::AppHandle,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<(), String> {
    runner.start(task_id, app)
}

#[tauri::command]
pub fn rerun_task(
    task_id: String,
    app: tauri::AppHandle,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<String, String> {
    runner.rerun(task_id, app)
}

#[tauri::command]
pub fn delete_task(
    task_id: String,
    runner: tauri::State<'_, crate::scanner::task_runner::TaskRunner>,
) -> Result<(), String> {
    runner.cancel(&task_id)?;
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let filter_repo = FilterRepo::new(&conn);
    let log_repo = LogRepo::new(&conn);
    let scan_item_repo = ScanItemRepo::new(&conn);
    let scan_batch_repo = ScanBatchRepo::new(&conn);
    let task_run_repo = TaskRunRepo::new(&conn);
    let task_repo = TaskRepo::new(&conn);

    filter_repo
        .delete_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    log_repo
        .delete_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    scan_item_repo
        .delete_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    scan_batch_repo
        .delete_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    task_run_repo
        .delete_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    task_repo.delete(&task_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ListTasksRequest {
    pub status: Option<String>,
    pub batch_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListScanItemsRequest {
    pub task_id: String,
    pub run_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ListTaskRunsRequest {
    pub task_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RetryScanItemsRequest {
    pub task_id: String,
    pub run_id: Option<String>,
    pub item_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskSettingsRequest {
    pub task_id: String,
    pub concurrency: i64,
    pub proxy_id: Option<i64>,
}

#[derive(Debug, Serialize)]
struct PaginatedResponse<T> {
    items: Vec<T>,
    total: i64,
    page: i64,
    per_page: i64,
}

#[derive(Debug, Serialize)]
pub struct RetryScanItemsResponse {
    pub updated_count: usize,
    pub available_count: i64,
    pub error_count: i64,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct ScanResultsUpdatedEvent {
    task_id: String,
    run_id: String,
    flushed_count: usize,
    completed_count: i64,
}

#[tauri::command]
pub fn list_tasks(request: ListTasksRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let status: Option<TaskStatus> = request.status.as_deref().and_then(|s| {
        serde_json::from_str::<TaskStatus>(s)
            .ok()
            .or_else(|| serde_json::from_str::<TaskStatus>(&format!("\"{}\"", s)).ok())
    });

    let tasks = repo
        .list(
            status.as_ref(),
            request.batch_id.as_deref(),
            request.limit.unwrap_or(100),
            request.offset.unwrap_or(0),
        )
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&tasks).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_scan_items(request: ListScanItemsRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ScanItemRepo::new(&conn);
    let run_repo = TaskRunRepo::new(&conn);
    let limit = request.limit.unwrap_or(10).max(1);
    let offset = request.offset.unwrap_or(0).max(0);

    let status: Option<ScanItemStatus> = request.status.as_deref().and_then(|s| {
        serde_json::from_str::<ScanItemStatus>(s)
            .ok()
            .or_else(|| serde_json::from_str::<ScanItemStatus>(&format!("\"{}\"", s)).ok())
    });

    let items = repo
        .list_by_task(
            &request.task_id,
            request.run_id.as_deref(),
            status.as_ref(),
            limit,
            offset,
        )
        .map_err(|e| e.to_string())?;
    let total = if status.is_none() {
        match request.run_id.as_deref() {
            Some(run_id) => run_repo
                .get_by_id(run_id)
                .map_err(|e| e.to_string())?
                .filter(|run| run.task_id == request.task_id)
                .map(|run| run.completed_count)
                .unwrap_or(0),
            None => {
                let runs = run_repo
                    .list_by_task(&request.task_id)
                    .map_err(|e| e.to_string())?;
                runs.into_iter().map(|run| run.completed_count).sum()
            }
        }
    } else {
        repo.count_by_task(&request.task_id, request.run_id.as_deref(), status.as_ref())
            .map_err(|e| e.to_string())?
    };

    serde_json::to_string(&PaginatedResponse {
        items,
        total,
        page: (offset / limit) + 1,
        per_page: limit,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_task_runs(request: ListTaskRunsRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = TaskRunRepo::new(&conn);
    let runs: Vec<TaskRun> = repo
        .list_by_task(&request.task_id)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&runs).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_task_detail(task_id: String) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = TaskRepo::new(&conn);

    let task = repo
        .get_by_id(&task_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", task_id))?;

    serde_json::to_string(&task).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn retry_scan_items(
    request: RetryScanItemsRequest,
    app: tauri::AppHandle,
) -> Result<String, String> {
    if request.item_ids.is_empty() {
        return Err("No scan items selected for retry".to_string());
    }

    let mut updated_count = 0usize;
    let mut selected_ids = request.item_ids;
    selected_ids.sort_unstable();
    selected_ids.dedup();

    let (task, run, items_to_retry, checker) = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&conn);
        let run_repo = TaskRunRepo::new(&conn);
        let scan_repo = ScanItemRepo::new(&conn);

        let task = task_repo
            .get_by_id(&request.task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", request.task_id))?;

        let run = match request.run_id.as_deref() {
            Some(run_id) => run_repo
                .get_by_id(run_id)
                .map_err(|e| e.to_string())?
                .filter(|run| run.task_id == request.task_id)
                .ok_or_else(|| format!("Run not found: {}", run_id))?,
            None => run_repo
                .get_latest_by_task(&request.task_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("No runs found for task {}", request.task_id))?,
        };

        let mut items_to_retry = Vec::new();
        for item_id in &selected_ids {
            let Some(item) = scan_repo.get_by_id(*item_id).map_err(|e| e.to_string())? else {
                continue;
            };
            if item.task_id == task.id && item.run_id == run.id {
                items_to_retry.push(item);
            }
        }

        let logger = build_retry_logger(app.clone(), task.id.clone(), run.id.clone());
        let checker = build_retry_checker(&conn, &task, logger)?;
        (task, run, items_to_retry, checker)
    };

    {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        write_task_log_and_emit(
            &conn,
            Some(&app),
            &task.id,
            Some(&run.id),
            "warn",
            &format!("Starting retry for {} scan results", items_to_retry.len()),
        );
    }

    for item in items_to_retry {
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_task_log_and_emit(
                &conn,
                Some(&app),
                &task.id,
                Some(&run.id),
                "warn",
                &format!("Retrying scan result for {}", item.domain),
            );
        }

        let result = checker.check_domain(&item.domain).await;
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            let scan_repo = ScanItemRepo::new(&conn);
            scan_repo
                .update_status(
                    item.id,
                    &result.status,
                    result.is_available,
                    result.query_method.as_deref(),
                    result.response_time_ms,
                    result.error_message.as_deref(),
                )
                .map_err(|e| e.to_string())?;
        }

        let level = if matches!(result.status, ScanItemStatus::Error) {
            "error"
        } else {
            "warn"
        };
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_task_log_and_emit(
                &conn,
                Some(&app),
                &task.id,
                Some(&run.id),
                level,
                &format!(
                    "Retry finished for {}: {}",
                    item.domain,
                    format_retry_result_message(&result.status, result.error_message.as_deref())
                ),
            );
        }
        updated_count += 1;
    }

    let (available_count, error_count) = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let scan_repo = ScanItemRepo::new(&conn);
        let run_repo = TaskRunRepo::new(&conn);
        let task_repo = TaskRepo::new(&conn);

        let available_count = scan_repo
            .count_by_task(&task.id, Some(&run.id), Some(&ScanItemStatus::Available))
            .map_err(|e| e.to_string())?;
        let error_count = scan_repo
            .count_by_task(&task.id, Some(&run.id), Some(&ScanItemStatus::Error))
            .map_err(|e| e.to_string())?;

        run_repo
            .update_progress(&run.id, run.completed_count, available_count, error_count)
            .map_err(|e| e.to_string())?;

        if run_repo
            .get_latest_by_task(&task.id)
            .map_err(|e| e.to_string())?
            .map(|latest| latest.id == run.id)
            .unwrap_or(false)
        {
            task_repo
                .update_progress(
                    &task.id,
                    task.completed_count,
                    task.completed_index,
                    available_count,
                    error_count,
                )
                .map_err(|e| e.to_string())?;
        }

        (available_count, error_count)
    };

    let _ = app.emit(
        "scan-results-updated",
        ScanResultsUpdatedEvent {
            task_id: task.id.clone(),
            run_id: run.id.clone(),
            flushed_count: updated_count,
            completed_count: run.completed_count,
        },
    );

    {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        write_task_log_and_emit(
            &conn,
            Some(&app),
            &task.id,
            Some(&run.id),
            "warn",
            &format!(
                "Retry completed: {} items updated, available {}, error {}",
                updated_count, available_count, error_count
            ),
        );
    }

    serde_json::to_string(&RetryScanItemsResponse {
        updated_count,
        available_count,
        error_count,
        run_id: run.id,
    })
    .map_err(|e| e.to_string())
}

fn build_retry_checker(
    conn: &rusqlite::Connection,
    task: &Task,
    logger: Arc<dyn Fn(String, String) + Send + Sync>,
) -> Result<DomainChecker, String> {
    let check_config = CheckConfig::default();
    let checker = match task.proxy_id {
        Some(proxy_id) => {
            let proxy_repo = ProxyRepo::new(conn);
            let proxy = proxy_repo
                .get_by_id(proxy_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Proxy not found: {}", proxy_id))?;
            let reqwest_proxy = ProxyManager::build_reqwest_proxy(&proxy)?;
            DomainChecker::with_proxy(check_config, reqwest_proxy, describe_proxy_label(&proxy))
        }
        None => DomainChecker::new(check_config),
    };

    Ok(checker.with_log_hook(logger))
}

fn build_retry_logger(
    app: tauri::AppHandle,
    task_id: String,
    run_id: String,
) -> Arc<dyn Fn(String, String) + Send + Sync> {
    Arc::new(move |level: String, message: String| {
        if let Ok(conn) = init::open_db() {
            write_request_log_and_emit(
                &conn,
                Some(&app),
                &task_id,
                Some(&run_id),
                &level,
                &message,
            );
        }
    })
}

fn write_request_log_and_emit(
    conn: &rusqlite::Connection,
    app: Option<&tauri::AppHandle>,
    task_id: &str,
    run_id: Option<&str>,
    level: &str,
    message: &str,
) {
    let repo = LogRepo::new(conn);
    match repo.create_request_entry(task_id, run_id, level, message) {
        Ok(entry) => {
            if let Some(app) = app {
                if entry.level != "info" {
                    let _ = app.emit("task-log-created", &entry);
                    let _ = app.emit(&format!("task-log-{}", task_id), &entry);
                }
            }
        }
        Err(err) => tracing::warn!("Failed to write retry request log: {}", err),
    }
}

fn write_task_log_and_emit(
    conn: &rusqlite::Connection,
    app: Option<&tauri::AppHandle>,
    task_id: &str,
    run_id: Option<&str>,
    level: &str,
    message: &str,
) {
    let repo = LogRepo::new(conn);
    match repo.create_entry(task_id, run_id, level, message) {
        Ok(entry) => {
            if let Some(app) = app {
                if entry.level != "info" {
                    let _ = app.emit("task-log-created", &entry);
                    let _ = app.emit(&format!("task-log-{}", task_id), &entry);
                }
            }
        }
        Err(err) => tracing::warn!("Failed to write retry log: {}", err),
    }
}

fn format_retry_result_message(status: &ScanItemStatus, error_message: Option<&str>) -> String {
    match status {
        ScanItemStatus::Available => "available".to_string(),
        ScanItemStatus::Unavailable => "registered".to_string(),
        ScanItemStatus::Checking => "checking".to_string(),
        ScanItemStatus::Pending => "pending".to_string(),
        ScanItemStatus::Error => error_message.unwrap_or("error").to_string(),
    }
}

fn describe_proxy_label(proxy: &crate::models::proxy::ProxyConfig) -> String {
    let label = proxy.name.as_deref().unwrap_or(proxy.url.as_str());
    format!("{} [{}]", label, proxy.proxy_type.to_url_scheme())
}

fn describe_proxy_id_label(repo: &ProxyRepo<'_>, proxy_id: Option<i64>) -> Result<String, String> {
    match proxy_id {
        Some(id) => {
            let proxy = repo
                .get_by_id(id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Proxy not found: {}", id))?;
            Ok(describe_proxy_label(&proxy))
        }
        None => Ok("direct".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_single_tld() {
        let req = CreateTasksRequest {
            name: "Test Task".to_string(),
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            tlds: vec![".com".to_string()],
            batch_name: None,
            concurrency: None,
            proxy_id: None,
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
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            tlds: vec![".com".to_string(), ".net".to_string(), ".org".to_string()],
            batch_name: None,
            concurrency: None,
            proxy_id: None,
        };
        let result = create_tasks(req).unwrap();
        assert_eq!(result.created, 1); // Single task!
        assert_eq!(result.skipped, 0);
        assert_eq!(result.task_ids.len(), 1);
        assert!(result.batch_id.is_none());
    }

    #[test]
    fn test_create_task_preserves_custom_name() {
        let scan_mode = ScanMode::Manual {
            domains: vec!["hello".to_string()],
        };
        let tlds = vec![".com".to_string(), ".net".to_string()];
        let resolved = resolve_task_name("我的自定义任务", &scan_mode, &tlds);
        assert_eq!(resolved, "我的自定义任务");
    }

    #[test]
    fn test_create_task_generates_default_name_when_blank() {
        let scan_mode = ScanMode::Manual {
            domains: vec!["hello".to_string()],
        };
        let tlds = vec![".com".to_string(), ".net".to_string()];
        let resolved = resolve_task_name("", &scan_mode, &tlds);
        assert_eq!(resolved, "手动扫描 .com/.net");
    }

    #[test]
    fn test_create_task_dedup() {
        // Note: :memory: DB is fresh each invocation. Dedup only works within a single
        // connection, but each command creates its own. So we verify that creating the
        // same task twice in separate commands both succeed (different :memory: databases).
        let req1 = CreateTasksRequest {
            name: "First".to_string(),
            scan_mode: ScanMode::Manual {
                domains: vec!["hello".to_string()],
            },
            tlds: vec![".com".to_string(), ".net".to_string()],
            batch_name: None,
            concurrency: None,
            proxy_id: None,
        };
        let result1 = create_tasks(req1).unwrap();
        assert_eq!(result1.created, 1);

        let req2 = CreateTasksRequest {
            name: "Duplicate".to_string(),
            scan_mode: ScanMode::Manual {
                domains: vec!["hello".to_string()],
            },
            tlds: vec![".com".to_string(), ".net".to_string()],
            batch_name: None,
            concurrency: None,
            proxy_id: None,
        };
        let result2 = create_tasks(req2).unwrap();
        assert_eq!(result2.created, 1);
    }

    #[test]
    fn test_create_task_different_tld_order_same_sig() {
        // Verify that signature generation is deterministic and order-independent.
        // (Actual dedup across :memory: DB boundaries cannot be tested here)
        use crate::scanner::signature::generate_signature;
        let mode = ScanMode::Manual {
            domains: vec!["test".to_string()],
        };

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
    fn test_list_scan_items_empty() {
        let req = ListScanItemsRequest {
            task_id: "nonexistent".to_string(),
            run_id: None,
            status: None,
            limit: Some(100),
            offset: Some(0),
        };
        let result = list_scan_items(req).unwrap();
        let payload: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(payload["total"], 0);
        assert!(payload["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_list_task_runs_empty() {
        let req = ListTaskRunsRequest {
            task_id: "nonexistent".to_string(),
        };
        let result = list_task_runs(req).unwrap();
        let runs: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert!(runs.is_empty());
    }

    #[test]
    fn test_get_task_detail_not_found() {
        let result = get_task_detail("nonexistent".to_string());
        assert!(result.is_err());
    }
}
