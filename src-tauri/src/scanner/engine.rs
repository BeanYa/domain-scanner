use crate::db::proxy_repo::ProxyRepo;
use crate::db::scan_batch_repo::ScanBatchRepo;
use crate::db::scan_item_repo::ScanItemRepo;
use crate::db::task_repo::TaskRepo;
use crate::db::task_run_repo::TaskRunRepo;
use crate::models::proxy::ProxyStatus;
use crate::models::scan_batch::{make_scan_batch_id, ScanBatchStatus, LOCAL_WORKER_ID};
use crate::models::scan_item::{ScanItem, ScanItemStatus};
use crate::models::task::TaskStatus;
use crate::scanner::batch_planner::batch_index_for_item;
use crate::scanner::domain_checker::{CheckConfig, DomainChecker};
use crate::scanner::list_generator::{DomainCandidate, ListGenerator};
use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// Scan engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_concurrency: usize,
    pub request_delay_ms: u64,
    pub batch_write_size: usize,
    pub progress_report_interval_ms: u64,
    pub proxy_error_pause_threshold: usize,
    pub scan_batch_size: i64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 50,
            request_delay_ms: 100,
            batch_write_size: 500,
            progress_report_interval_ms: 200,
            proxy_error_pause_threshold: 10,
            scan_batch_size: crate::models::scan_batch::DEFAULT_SCAN_BATCH_SIZE,
        }
    }
}

/// Progress report for a running scan
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgress {
    pub task_id: String,
    pub run_id: String,
    pub completed_count: i64,
    pub total_count: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub percent: f64,
    pub pause_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelIntent {
    Pause,
    Stop,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ScanResultsUpdated {
    task_id: String,
    run_id: String,
    flushed_count: usize,
    completed_count: i64,
}

/// Scan engine: orchestrates concurrent domain scanning with checkpoint resume
pub struct ScanEngine {
    config: EngineConfig,
    checker: DomainChecker,
}

impl ScanEngine {
    pub fn new(config: EngineConfig, check_config: CheckConfig) -> Self {
        let checker = DomainChecker::new(check_config);
        Self { config, checker }
    }

    pub fn from_parts(config: EngineConfig, checker: DomainChecker) -> Self {
        Self { config, checker }
    }

    pub fn with_default_config() -> Self {
        Self::new(EngineConfig::default(), CheckConfig::default())
    }

    /// Run a scan task (to be called in a tokio task)
    /// Returns the final progress when done or paused
    pub async fn run_scan(
        &self,
        task_id: &str,
        run_id: &str,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: tokio_util::sync::CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        // Get task info
        let task = {
            let c = conn.lock().map_err(|e| e.to_string())?;
            let repo = TaskRepo::new(&c);
            repo.get_by_id(task_id)
                .map_err(|e| format!("Failed to get task: {}", e))?
                .ok_or_else(|| format!("Task {} not found", task_id))?
        };

        // Update task status to running
        {
            let c = conn.lock().map_err(|e| e.to_string())?;
            let repo = TaskRepo::new(&c);
            repo.update_status(task_id, &TaskStatus::Running)
                .map_err(|e| format!("Failed to update status: {}", e))?;
        }

        let mut generator = ListGenerator::new(task.scan_mode.clone(), task.tlds.clone())
            .with_batch_size(1)
            .with_start_index(task.completed_index);
        let total_count = task.total_count.max(generator.total_count());

        let mut completed_count = task.completed_count;
        let mut available_count = task.available_count;
        let mut error_count = task.error_count;
        let mut pending_items = Vec::with_capacity(self.config.batch_write_size.max(1));
        let mut last_persist_at =
            Instant::now() - Duration::from_millis(self.config.progress_report_interval_ms);
        let mut last_progress_emit =
            Instant::now() - Duration::from_millis(self.config.progress_report_interval_ms);
        let initial_progress = self.make_progress(
            task_id,
            run_id,
            completed_count,
            total_count,
            available_count,
            error_count,
            None,
        );
        let _ = app.emit("scan-progress", &initial_progress);

        let mut inflight = FuturesUnordered::new();
        self.fill_inflight(&mut inflight, &mut generator);
        let mut proxy_error_streak = 0usize;

        while let Some((candidate, result)) = inflight.next().await {
            if cancel_token.is_cancelled() {
                let intent = cancel_intent
                    .lock()
                    .map(|guard| *guard)
                    .unwrap_or(CancelIntent::Pause);
                let status = match intent {
                    CancelIntent::Pause => TaskStatus::Paused,
                    CancelIntent::Stop => TaskStatus::Stopped,
                };
                let reason = match intent {
                    CancelIntent::Pause => "manual_pause",
                    CancelIntent::Stop => "manual_stop",
                };
                self.persist_progress_locked(
                    &conn,
                    task_id,
                    run_id,
                    completed_count,
                    generator.current_index(),
                    available_count,
                    error_count,
                    &mut pending_items,
                    app,
                    Some(match intent {
                        CancelIntent::Pause => ScanBatchStatus::Paused,
                        CancelIntent::Stop => ScanBatchStatus::Cancelled,
                    }),
                );
                {
                    let c = conn.lock().map_err(|e| e.to_string())?;
                    let repo = TaskRepo::new(&c);
                    repo.update_status(task_id, &status)
                        .map_err(|e| format!("Failed to update cancelled task: {}", e))?;
                    let run_repo = TaskRunRepo::new(&c);
                    run_repo
                        .update_status(run_id, &status, intent == CancelIntent::Stop)
                        .map_err(|e| format!("Failed to update cancelled run: {}", e))?;
                }
                return Ok(self.make_progress(
                    task_id,
                    run_id,
                    completed_count,
                    total_count,
                    available_count,
                    error_count,
                    Some(reason.to_string()),
                ));
            }

            completed_count += 1;
            let proxy_error = result.proxy_error;
            match &result.status {
                ScanItemStatus::Available => available_count += 1,
                ScanItemStatus::Error => error_count += 1,
                _ => {}
            }

            let item = ScanItem {
                id: 0,
                task_id: task_id.to_string(),
                run_id: run_id.to_string(),
                batch_id: Some(make_scan_batch_id(
                    run_id,
                    batch_index_for_item(candidate.index, self.config.scan_batch_size),
                )),
                worker_id: Some(LOCAL_WORKER_ID.to_string()),
                domain: result.domain,
                tld: candidate.tld,
                item_index: candidate.index,
                status: result.status,
                is_available: result.is_available,
                query_method: result.query_method,
                response_time_ms: result.response_time_ms,
                error_message: result.error_message,
                checked_at: Some(chrono::Utc::now().to_rfc3339()),
            };

            pending_items.push(item);

            if proxy_error {
                proxy_error_streak += 1;
            } else {
                proxy_error_streak = 0;
            }

            if self.should_pause_for_proxy_errors(proxy_error_streak) {
                let reason = format!(
                    "Paused after {} consecutive proxy-related request errors",
                    proxy_error_streak
                );
                self.persist_progress_locked(
                    &conn,
                    task_id,
                    run_id,
                    completed_count,
                    candidate.index + 1,
                    available_count,
                    error_count,
                    &mut pending_items,
                    app,
                    Some(ScanBatchStatus::Paused),
                );
                self.pause_for_proxy_errors_locked(&conn, task_id, run_id, task.proxy_id, &reason)?;
                cancel_token.cancel();
                return Ok(self.make_progress(
                    task_id,
                    run_id,
                    completed_count,
                    total_count,
                    available_count,
                    error_count,
                    Some("proxy_error_threshold".to_string()),
                ));
            }

            if pending_items.len() >= self.config.batch_write_size.max(1)
                || last_persist_at.elapsed()
                    >= Duration::from_millis(self.config.progress_report_interval_ms.max(100))
                || inflight.is_empty()
            {
                self.persist_progress_locked(
                    &conn,
                    task_id,
                    run_id,
                    completed_count,
                    candidate.index + 1,
                    available_count,
                    error_count,
                    &mut pending_items,
                    app,
                    None,
                );
                last_persist_at = Instant::now();
            }

            if last_progress_emit.elapsed()
                >= Duration::from_millis(self.config.progress_report_interval_ms.max(100))
                || inflight.is_empty()
            {
                let progress = self.make_progress(
                    task_id,
                    run_id,
                    completed_count,
                    total_count,
                    available_count,
                    error_count,
                    None,
                );
                let _ = app.emit("scan-progress", &progress);
                last_progress_emit = Instant::now();
            }

            if self.config.request_delay_ms > 0 {
                let jitter =
                    rand::Rng::gen_range(&mut rand::thread_rng(), 0..self.config.request_delay_ms);
                tokio::time::sleep(std::time::Duration::from_millis(jitter)).await;
            }

            self.fill_inflight(&mut inflight, &mut generator);
        }

        // Mark task as completed
        self.persist_progress_locked(
            &conn,
            task_id,
            run_id,
            completed_count,
            generator.current_index(),
            available_count,
            error_count,
            &mut pending_items,
            app,
            Some(ScanBatchStatus::Succeeded),
        );
        {
            let c = conn.lock().map_err(|e| e.to_string())?;
            let repo = TaskRepo::new(&c);
            repo.update_status(task_id, &TaskStatus::Completed)
                .map_err(|e| format!("Failed to complete: {}", e))?;
            let run_repo = TaskRunRepo::new(&c);
            run_repo
                .update_status(run_id, &TaskStatus::Completed, true)
                .map_err(|e| format!("Failed to complete run: {}", e))?;
        }

        Ok(self.make_progress(
            task_id,
            run_id,
            completed_count,
            total_count,
            available_count,
            error_count,
            None,
        ))
    }

    fn should_pause_for_proxy_errors(&self, proxy_error_streak: usize) -> bool {
        self.config.proxy_error_pause_threshold > 0
            && proxy_error_streak >= self.config.proxy_error_pause_threshold
    }

    fn fill_inflight(
        &self,
        inflight: &mut FuturesUnordered<
            BoxFuture<'static, (DomainCandidate, crate::scanner::domain_checker::CheckResult)>,
        >,
        generator: &mut ListGenerator,
    ) {
        while inflight.len() < self.config.max_concurrency {
            let Some(candidate) = next_candidate(generator) else {
                break;
            };
            let checker = self.checker.clone();
            inflight.push(Box::pin(async move {
                let result = checker.check_domain(&candidate.domain).await;
                (candidate, result)
            }));
        }
    }

    fn persist_progress_locked(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
        completed_count: i64,
        completed_index: i64,
        available_count: i64,
        error_count: i64,
        pending_items: &mut Vec<ScanItem>,
        app: &AppHandle,
        terminal_batch_status: Option<ScanBatchStatus>,
    ) {
        if let Ok(c) = conn.lock() {
            let flushed_count = pending_items.len();
            if !pending_items.is_empty() {
                let scan_repo = ScanItemRepo::new(&c);
                if let Err(e) = scan_repo.batch_insert(pending_items) {
                    tracing::error!("Failed to batch insert scan items: {}", e);
                } else {
                    pending_items.clear();
                    let _ = app.emit(
                        "scan-results-updated",
                        ScanResultsUpdated {
                            task_id: task_id.to_string(),
                            run_id: run_id.to_string(),
                            flushed_count,
                            completed_count,
                        },
                    );
                }
            }
            let repo = TaskRepo::new(&c);
            if let Err(e) = repo.update_progress(
                task_id,
                completed_count,
                completed_index,
                available_count,
                error_count,
            ) {
                tracing::error!("Failed to update progress: {}", e);
            }
            let run_repo = TaskRunRepo::new(&c);
            if let Err(e) =
                run_repo.update_progress(run_id, completed_count, available_count, error_count)
            {
                tracing::error!("Failed to update run progress: {}", e);
            }
            let batch_repo = ScanBatchRepo::new(&c);
            if let Err(e) = batch_repo.update_local_progress(
                task_id,
                run_id,
                completed_index,
                terminal_batch_status,
            ) {
                tracing::error!("Failed to update scan batch progress: {}", e);
            }
        }
    }

    fn make_progress(
        &self,
        task_id: &str,
        run_id: &str,
        completed_count: i64,
        total_count: i64,
        available_count: i64,
        error_count: i64,
        pause_reason: Option<String>,
    ) -> ScanProgress {
        let percent = if total_count > 0 {
            (completed_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };
        ScanProgress {
            task_id: task_id.to_string(),
            run_id: run_id.to_string(),
            completed_count,
            total_count,
            available_count,
            error_count,
            percent,
            pause_reason,
        }
    }

    fn pause_for_proxy_errors_locked(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
        proxy_id: Option<i64>,
        reason: &str,
    ) -> Result<(), String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = TaskRepo::new(&c);
        repo.update_status(task_id, &TaskStatus::Paused)
            .map_err(|e| format!("Failed to pause after proxy errors: {}", e))?;
        let run_repo = TaskRunRepo::new(&c);
        run_repo
            .update_status(run_id, &TaskStatus::Paused, false)
            .map_err(|e| format!("Failed to pause run after proxy errors: {}", e))?;

        if let Some(proxy_id) = proxy_id {
            let proxy_repo = ProxyRepo::new(&c);
            proxy_repo
                .update_health(
                    proxy_id,
                    &ProxyStatus::Error,
                    false,
                    Some(&chrono::Utc::now().to_rfc3339()),
                    Some(reason),
                )
                .map_err(|e| format!("Failed to mark proxy as error: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.max_concurrency, 50);
        assert_eq!(config.batch_write_size, 500);
    }

    #[test]
    fn test_scan_progress_calculation() {
        let engine = ScanEngine::with_default_config();
        let progress = engine.make_progress("t1", "run1", 50, 100, 30, 2, None);
        assert_eq!(progress.completed_count, 50);
        assert_eq!(progress.total_count, 100);
        assert_eq!(progress.available_count, 30);
        assert_eq!(progress.error_count, 2);
        assert!((progress.percent - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_scan_progress_zero_total() {
        let engine = ScanEngine::with_default_config();
        let progress = engine.make_progress("t1", "run1", 0, 0, 0, 0, None);
        assert_eq!(progress.percent, 0.0);
    }
}

fn next_candidate(generator: &mut ListGenerator) -> Option<DomainCandidate> {
    let mut batch = generator.next_batch();
    if batch.is_empty() {
        None
    } else {
        Some(batch.remove(0))
    }
}
