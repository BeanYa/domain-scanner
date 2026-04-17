use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::async_runtime;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::db::init;
use crate::db::log_repo::{LogRepo, LogType};
use crate::db::proxy_repo::ProxyRepo;
use crate::db::scan_batch_repo::ScanBatchRepo;
use crate::db::task_repo::TaskRepo;
use crate::db::task_run_repo::TaskRunRepo;
use crate::models::proxy::ProxyConfig;
use crate::models::task::{TaskRun, TaskStatus};
use crate::proxy::manager::ProxyManager;
use crate::scanner::batch_planner::{default_scan_batch_size, plan_scan_batches};
use crate::scanner::domain_checker::{CheckConfig, DomainChecker};
use crate::scanner::engine::{CancelIntent, EngineConfig, ScanEngine};
use crate::scanner::list_generator::ListGenerator;
use uuid::Uuid;

/// Manages running scan tasks — tracks spawned tokio tasks and their cancellation tokens
pub struct TaskRunner {
    running: Arc<Mutex<HashMap<String, RunningTaskControl>>>,
}

#[derive(Clone)]
struct RunningTaskControl {
    token: CancellationToken,
    intent: Arc<Mutex<CancelIntent>>,
}

impl TaskRunner {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start (or resume) a scan task in the background
    pub fn start(&self, task_id: String, app: AppHandle) -> Result<(), String> {
        // Check if already running
        {
            let map = self.running.lock().map_err(|e| e.to_string())?;
            if map.contains_key(&task_id) {
                return Err(format!("Task {} is already running", task_id));
            }
        }

        // Load task from DB
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&conn);
        let run_repo = TaskRunRepo::new(&conn);
        let task = task_repo
            .get_by_id(&task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        if !matches!(task.status, TaskStatus::Pending | TaskStatus::Paused) {
            return Err(format!("Cannot start task in {:?} state", task.status));
        }

        let total_count =
            ListGenerator::new(task.scan_mode.clone(), task.tlds.clone()).total_count();
        let run = match task.status {
            TaskStatus::Paused => run_repo
                .get_latest_by_task(&task_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("No paused run found for task {}", task_id))?,
            TaskStatus::Pending => self.create_new_run(&run_repo, &task_id, total_count)?,
            _ => unreachable!(),
        };
        if matches!(task.status, TaskStatus::Pending) {
            let scan_batch_repo = ScanBatchRepo::new(&conn);
            let batches =
                plan_scan_batches(&task_id, &run.id, total_count, default_scan_batch_size());
            scan_batch_repo
                .create_many(&batches)
                .map_err(|e| format!("Failed to create scan batches: {}", e))?;
        }

        task_repo
            .update_status(&task_id, &TaskStatus::Running)
            .map_err(|e| e.to_string())?;
        task_repo
            .update_total_count(&task_id, total_count)
            .map_err(|e| e.to_string())?;
        if matches!(task.status, TaskStatus::Paused) {
            run_repo
                .update_status(&run.id, &TaskStatus::Running, false)
                .map_err(|e| e.to_string())?;
        }
        let proxy_config = if let Some(pid) = task.proxy_id {
            let proxy_repo = ProxyRepo::new(&conn);
            proxy_repo.get_by_id(pid).map_err(|e| e.to_string())?
        } else {
            None
        };
        let task_name = task.name.clone();
        let proxy_label = describe_proxy(proxy_config.as_ref());
        let start_message = if matches!(task.status, TaskStatus::Paused) {
            format!("Run #{} resumed via {}", run.run_number, proxy_label)
        } else {
            format!(
                "Run #{} started: {} candidates, concurrency {}, proxy {}",
                run.run_number, total_count, task.concurrency, proxy_label,
            )
        };
        write_log_and_emit_direct(
            &conn,
            Some(&app),
            &task_id,
            Some(&run.id),
            "info",
            &start_message,
        );
        let _ = app.emit(
            "task-status-change",
            serde_json::json!({
                "task_id": task_id.clone(),
                "task_name": task_name.clone(),
                "status": "running",
                "message": "任务已开始运行",
            }),
        );

        // Build engine config from task settings
        let engine_config = EngineConfig {
            max_concurrency: task.concurrency as usize,
            ..EngineConfig::default()
        };
        let cancel_token = CancellationToken::new();
        let cancel_intent = Arc::new(Mutex::new(CancelIntent::Pause));
        let scan_token = cancel_token.clone();
        let scan_intent = cancel_intent.clone();
        let status_token = cancel_token.clone();

        // Store token
        {
            let mut map = self.running.lock().map_err(|e| e.to_string())?;
            map.insert(
                task_id.clone(),
                RunningTaskControl {
                    token: cancel_token,
                    intent: cancel_intent,
                },
            );
        }

        let running_ref = self.running.clone();
        let tid = task_id.clone();
        let run_id = run.id.clone();
        let run_number = run.run_number;
        let task_name_for_spawn = task_name.clone();
        let task_concurrency = task.concurrency;
        let proxy_config_for_spawn = proxy_config.clone();
        let engine_config_for_spawn = engine_config.clone();

        // Spawn background task
        async_runtime::spawn(async move {
            // Open a dedicated connection for this scan (long-running)
            let conn = match init::open_db() {
                Ok(c) => Arc::new(std::sync::Mutex::new(c)),
                Err(e) => {
                    tracing::error!("Failed to open DB for scan: {}", e);
                    if let Ok(c) = init::open_db() {
                        let repo = TaskRepo::new(&c);
                        let _ = repo.update_status(&tid, &TaskStatus::Paused);
                        let run_repo = TaskRunRepo::new(&c);
                        let _ = run_repo.update_status(&run_id, &TaskStatus::Paused, false);
                        write_log_and_emit_direct(
                            &c,
                            Some(&app),
                            &tid,
                            Some(&run_id),
                            "error",
                            &format!("Failed to open DB for scan: {}", e),
                        );
                    }
                    let _ = app.emit(
                        "task-status-change",
                        serde_json::json!({
                            "task_id": tid.clone(),
                            "task_name": task_name_for_spawn.clone(),
                            "status": "paused",
                            "reason": "scan_error",
                            "message": "任务已暂停：无法打开扫描数据库",
                        }),
                    );
                    let _ = app.emit(
                        "scan-error",
                        serde_json::json!({
                            "task_id": tid, "error": e.to_string()
                        }),
                    );
                    let mut map = running_ref.lock().unwrap();
                    map.remove(&tid);
                    return;
                }
            };

            let log_conn = conn.clone();
            let log_app = app.clone();
            let log_task_id = tid.clone();
            let log_run_id = run_id.clone();
            let request_logger = Arc::new(move |level: String, message: String| {
                write_request_log_and_emit_locked(
                    &log_conn,
                    &log_app,
                    &log_task_id,
                    Some(&log_run_id),
                    &level,
                    &message,
                );
            });

            let check_config = CheckConfig::default();
            let checker = match proxy_config_for_spawn.as_ref() {
                Some(proxy_cfg) => match ProxyManager::build_reqwest_proxy(proxy_cfg) {
                    Ok(proxy) => DomainChecker::with_proxy(
                        check_config.clone(),
                        proxy,
                        describe_proxy(Some(proxy_cfg)),
                    )
                    .with_log_hook(request_logger.clone()),
                    Err(err) => {
                        write_log_and_emit_locked(
                            &conn,
                            &app,
                            &tid,
                            Some(&run_id),
                            "warn",
                            &format!(
                                "Proxy {} is invalid for this run: {}. Falling back to direct connection",
                                describe_proxy(Some(proxy_cfg)),
                                err
                            ),
                        );
                        DomainChecker::new(check_config.clone())
                            .with_log_hook(request_logger.clone())
                    }
                },
                None => {
                    DomainChecker::new(check_config.clone()).with_log_hook(request_logger.clone())
                }
            };
            write_log_and_emit_locked(
                &conn,
                &app,
                &tid,
                Some(&run_id),
                "info",
                &format!(
                    "Run #{} preparing scan: {} candidates, concurrency {}, proxy {}",
                    run_number,
                    total_count,
                    task_concurrency,
                    describe_proxy(proxy_config_for_spawn.as_ref())
                ),
            );
            let engine = ScanEngine::from_parts(engine_config_for_spawn, checker);
            let result = engine
                .run_scan(&tid, &run_id, conn, scan_token, scan_intent, &app)
                .await;

            // Clean up
            {
                let mut map = running_ref.lock().unwrap();
                map.remove(&tid);
            }

            match result {
                Ok(progress) => {
                    let pause_reason = progress.pause_reason.clone();
                    if !status_token.is_cancelled() {
                        if let Ok(c) = init::open_db() {
                            let run_repo = TaskRunRepo::new(&c);
                            if pause_reason.as_deref() == Some("proxy_error_threshold") {
                                write_log_and_emit_direct(
                                    &c,
                                    Some(&app),
                                    &tid,
                                    Some(&run_id),
                                    "error",
                                    "Task paused because the selected proxy produced too many request errors. Proxy status was set to error.",
                                );
                            } else {
                                let _ =
                                    run_repo.update_status(&run_id, &TaskStatus::Completed, true);
                                write_log_and_emit_direct(
                                    &c,
                                    Some(&app),
                                    &tid,
                                    Some(&run_id),
                                    "info",
                                    &format!(
                                        "Run #{} completed: {} checked, {} available, {} errors",
                                        run_number,
                                        progress.completed_count,
                                        progress.available_count,
                                        progress.error_count
                                    ),
                                );
                            }
                        }
                    }
                    let status = match pause_reason.as_deref() {
                        Some("manual_stop") => "stopped",
                        _ if status_token.is_cancelled() || pause_reason.is_some() => "paused",
                        _ => "completed",
                    };
                    let message = match status {
                        "completed" => "任务已完成",
                        "stopped" => "任务已停止；重新开始会创建新的运行记录",
                        _ if pause_reason.as_deref() == Some("proxy_error_threshold") => {
                            "任务已暂停：代理错误请求过多"
                        }
                        _ => "任务已暂停",
                    };
                    let manual_user_cancel = matches!(
                        pause_reason.as_deref(),
                        Some("manual_pause" | "manual_stop")
                    );
                    if !manual_user_cancel {
                        let _ = app.emit(
                            "task-status-change",
                            serde_json::json!({
                                "task_id": tid.clone(),
                                "task_name": task_name_for_spawn.clone(),
                                "status": status,
                                "reason": pause_reason,
                                "message": message,
                            }),
                        );
                    }
                    let _ = app.emit("scan-complete", &progress);
                }
                Err(e) => {
                    tracing::error!("Scan failed: {}", e);
                    if let Ok(c) = init::open_db() {
                        let repo = TaskRepo::new(&c);
                        let _ = repo.update_status(&tid, &TaskStatus::Paused);
                        let run_repo = TaskRunRepo::new(&c);
                        let _ = run_repo.update_status(&run_id, &TaskStatus::Paused, false);
                        write_log_and_emit_direct(
                            &c,
                            Some(&app),
                            &tid,
                            Some(&run_id),
                            "error",
                            &format!("Run #{} failed: {}", run_number, e),
                        );
                    }
                    let _ = app.emit(
                        "task-status-change",
                        serde_json::json!({
                            "task_id": tid.clone(),
                            "task_name": task_name_for_spawn.clone(),
                            "status": "paused",
                            "reason": "scan_error",
                            "message": "任务已暂停：扫描失败",
                        }),
                    );
                    let _ = app.emit(
                        "scan-error",
                        serde_json::json!({
                            "task_id": tid, "error": e
                        }),
                    );
                }
            }
        });

        Ok(())
    }

    /// Pause a running task by cancelling its token
    pub fn pause(&self, task_id: &str, app: Option<&AppHandle>) -> Result<(), String> {
        let mut cancelled = false;
        {
            let map = self.running.lock().map_err(|e| e.to_string())?;
            if let Some(control) = map.get(task_id) {
                if let Ok(mut intent) = control.intent.lock() {
                    *intent = CancelIntent::Pause;
                }
                control.token.cancel();
                cancelled = true;
            }
        }

        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = TaskRepo::new(&conn);
        let run_repo = TaskRunRepo::new(&conn);
        let task = repo
            .get_by_id(task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        let latest_run = run_repo
            .get_latest_by_task(task_id)
            .map_err(|e| e.to_string())?;

        match task.status {
            TaskStatus::Running => {
                repo.update_status(task_id, &TaskStatus::Paused)
                    .map_err(|e| e.to_string())?;
                if let Some(run) = latest_run.as_ref() {
                    run_repo
                        .update_status(&run.id, &TaskStatus::Paused, false)
                        .map_err(|e| e.to_string())?;
                }
                write_log_and_emit_direct(
                    &conn,
                    app,
                    task_id,
                    latest_run.as_ref().map(|r| r.id.as_str()),
                    "info",
                    if cancelled {
                        "Task paused by user"
                    } else {
                        "Task marked as paused"
                    },
                );
                if let Some(app) = app {
                    let _ = app.emit(
                        "task-status-change",
                        serde_json::json!({
                            "task_id": task_id,
                            "task_name": task.name,
                            "status": "paused",
                            "reason": "manual_pause",
                            "message": "任务已暂停",
                        }),
                    );
                }
                Ok(())
            }
            TaskStatus::Paused => Ok(()),
            _ => Err(format!("Task {} is not running", task_id)),
        }
    }

    /// Stop a task so it cannot be checkpoint-resumed. Use rerun to start over.
    pub fn stop(&self, task_id: &str, app: Option<&AppHandle>) -> Result<(), String> {
        let mut cancelled = false;
        {
            let map = self.running.lock().map_err(|e| e.to_string())?;
            if let Some(control) = map.get(task_id) {
                if let Ok(mut intent) = control.intent.lock() {
                    *intent = CancelIntent::Stop;
                }
                control.token.cancel();
                cancelled = true;
            }
        }

        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = TaskRepo::new(&conn);
        let run_repo = TaskRunRepo::new(&conn);
        let task = repo
            .get_by_id(task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        let latest_run = run_repo
            .get_latest_by_task(task_id)
            .map_err(|e| e.to_string())?;

        match task.status {
            TaskStatus::Running | TaskStatus::Paused => {
                repo.update_status(task_id, &TaskStatus::Stopped)
                    .map_err(|e| e.to_string())?;
                if let Some(run) = latest_run.as_ref() {
                    run_repo
                        .update_status(&run.id, &TaskStatus::Stopped, true)
                        .map_err(|e| e.to_string())?;
                }
                write_log_and_emit_direct(
                    &conn,
                    app,
                    task_id,
                    latest_run.as_ref().map(|r| r.id.as_str()),
                    "info",
                    if cancelled {
                        "Task stopped by user"
                    } else {
                        "Paused task marked as stopped"
                    },
                );
                if let Some(app) = app {
                    let _ = app.emit(
                        "task-status-change",
                        serde_json::json!({
                            "task_id": task_id,
                            "task_name": task.name,
                            "status": "stopped",
                            "reason": "manual_stop",
                            "message": "任务已停止；重新开始会创建新的运行记录",
                        }),
                    );
                }
                Ok(())
            }
            TaskStatus::Stopped => Ok(()),
            _ => Err(format!(
                "Task {} cannot be stopped from {:?} state",
                task_id, task.status
            )),
        }
    }

    pub fn rerun(&self, task_id: String, app: AppHandle) -> Result<String, String> {
        {
            let map = self.running.lock().map_err(|e| e.to_string())?;
            if map.contains_key(&task_id) {
                return Err(format!("Task {} is already running", task_id));
            }
        }

        let conn = init::open_db().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&conn);
        let task = task_repo
            .get_by_id(&task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;

        if task.status == TaskStatus::Running {
            return Err(format!("Task {} is already running", task_id));
        }

        task_repo
            .reset_for_rerun(&task_id)
            .map_err(|e| e.to_string())?;
        self.start(task_id.clone(), app)?;

        let run_repo = TaskRunRepo::new(&conn);
        let latest_run = run_repo
            .get_latest_by_task(&task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Failed to create rerun for task {}", task_id))?;
        Ok(latest_run.id)
    }

    /// Check if a task is currently running
    pub fn is_running(&self, task_id: &str) -> bool {
        self.running
            .lock()
            .map(|m| m.contains_key(task_id))
            .unwrap_or(false)
    }

    /// Cancel an in-memory running task without touching persisted state
    pub fn cancel(&self, task_id: &str) -> Result<bool, String> {
        let map = self.running.lock().map_err(|e| e.to_string())?;
        if let Some(control) = map.get(task_id) {
            control.token.cancel();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_new_run(
        &self,
        run_repo: &TaskRunRepo<'_>,
        task_id: &str,
        total_count: i64,
    ) -> Result<TaskRun, String> {
        let run_number = run_repo
            .next_run_number(task_id)
            .map_err(|e| e.to_string())?;
        let run = TaskRun {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            run_number,
            status: TaskStatus::Running,
            total_count,
            completed_count: 0,
            available_count: 0,
            error_count: 0,
            started_at: chrono::Utc::now().to_rfc3339(),
            finished_at: None,
        };
        run_repo.create(&run).map_err(|e| e.to_string())?;
        Ok(run)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct TaskLogEventPayload {
    id: i64,
    task_id: String,
    run_id: Option<String>,
    log_type: String,
    level: String,
    message: String,
    created_at: String,
}

fn describe_proxy(proxy: Option<&ProxyConfig>) -> String {
    match proxy {
        Some(proxy) => {
            let label = proxy.name.as_deref().unwrap_or(proxy.url.as_str());
            format!("{} [{}]", label, proxy.proxy_type.to_url_scheme())
        }
        None => "direct".to_string(),
    }
}

fn write_log_and_emit_direct(
    conn: &rusqlite::Connection,
    app: Option<&AppHandle>,
    task_id: &str,
    run_id: Option<&str>,
    level: &str,
    message: &str,
) {
    match create_log_event(conn, task_id, run_id, LogType::Task, level, message) {
        Ok(payload) => {
            if let Some(app) = app {
                emit_task_log_events(app, &payload);
            }
        }
        Err(err) => tracing::warn!("Failed to write task log: {}", err),
    }
}

fn write_request_log_and_emit_locked(
    conn: &Mutex<rusqlite::Connection>,
    app: &AppHandle,
    task_id: &str,
    run_id: Option<&str>,
    level: &str,
    message: &str,
) {
    match conn.lock() {
        Ok(c) => match create_log_event(&c, task_id, run_id, LogType::Request, level, message) {
            Ok(payload) => emit_task_log_events(app, &payload),
            Err(err) => tracing::warn!("Failed to write request log: {}", err),
        },
        Err(err) => tracing::warn!("Failed to acquire request log DB lock: {}", err),
    }
}

fn write_log_and_emit_locked(
    conn: &Mutex<rusqlite::Connection>,
    app: &AppHandle,
    task_id: &str,
    run_id: Option<&str>,
    level: &str,
    message: &str,
) {
    match conn.lock() {
        Ok(c) => write_log_and_emit_direct(&c, Some(app), task_id, run_id, level, message),
        Err(err) => tracing::warn!("Failed to acquire log DB lock: {}", err),
    }
}

fn create_log_event(
    conn: &rusqlite::Connection,
    task_id: &str,
    run_id: Option<&str>,
    log_type: LogType,
    level: &str,
    message: &str,
) -> Result<TaskLogEventPayload, rusqlite::Error> {
    let repo = LogRepo::new(conn);
    let entry = repo.create_entry_with_type(task_id, run_id, log_type, level, message)?;
    Ok(TaskLogEventPayload {
        id: entry.id,
        task_id: entry.task_id,
        run_id: entry.run_id,
        log_type: entry.log_type,
        level: entry.level,
        message: entry.message,
        created_at: entry.created_at,
    })
}

fn emit_task_log_events(app: &AppHandle, payload: &TaskLogEventPayload) {
    if payload.level == "info" {
        return;
    }
    let _ = app.emit("task-log-created", payload);
    let _ = app.emit(&format!("task-log-{}", payload.task_id), payload);
}
