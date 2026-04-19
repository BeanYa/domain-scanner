use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;

use crate::db::cluster_worker_repo::ClusterWorkerRepo;
use crate::db::log_repo::{LogRepo, LogType};
use crate::db::proxy_repo::ProxyRepo;
use crate::db::scan_batch_repo::ScanBatchRepo;
use crate::db::scan_item_repo::ScanItemRepo;
use crate::db::task_repo::TaskRepo;
use crate::db::task_run_repo::TaskRunRepo;
use crate::models::cluster_worker::{ClusterWorker, ClusterWorkerStatus, ClusterWorkerType};
use crate::models::proxy::ProxyConfig;
use crate::models::scan_batch::ScanBatchStatus;
use crate::models::scan_item::{ScanItem, ScanItemStatus};
use crate::models::task::Task;
use crate::models::task::TaskStatus;
use crate::scanner::batch::{BatchLog, BatchPlan, BatchResult, BatchStatusSnapshot};
use crate::scanner::engine::{CancelIntent, ScanProgress};
use crate::scanner::local_worker::LocalEmbeddedWorker;
use crate::scanner::remote_worker::RemoteHttpWorker;

pub struct BatchCoordinator {
    local_worker: LocalEmbeddedWorker,
}

impl BatchCoordinator {
    pub fn new(local_worker: LocalEmbeddedWorker) -> Self {
        Self { local_worker }
    }

    pub async fn run_local_task(
        &self,
        task_id: &str,
        run_id: &str,
        concurrency: i64,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        let plans = self.load_remaining_local_plans(&conn, task_id, run_id, concurrency)?;
        if plans.is_empty() {
            self.mark_completed(&conn, task_id, run_id)?;
            return self.current_progress(&conn, task_id, run_id, None);
        }
        let task = self.load_task(&conn, task_id)?;
        let proxy = self.load_proxy(&conn, task.proxy_id)?;
        let remote_workers = self.load_available_remote_workers(&conn, concurrency)?;
        let mut remote_cursor = 0usize;

        for plan in plans {
            if cancel_token.is_cancelled() {
                return self.persist_cancel_before_batch(
                    &conn,
                    task_id,
                    run_id,
                    &cancel_intent,
                );
            }

            let progress = if remote_workers.is_empty() {
                self.write_task_log(
                    &conn,
                    task_id,
                    Some(run_id),
                    "info",
                    &format!("Executing batch {} on local worker", plan.batch_id),
                );
                self.execute_local_batch(
                    &plan,
                    conn.clone(),
                    cancel_token.clone(),
                    cancel_intent.clone(),
                    app,
                )
                .await?
            } else {
                let worker = &remote_workers[remote_cursor % remote_workers.len()];
                remote_cursor += 1;
                self.write_task_log(
                    &conn,
                    task_id,
                    Some(run_id),
                    "info",
                    &format!("Submitting batch {} to remote worker {}", plan.batch_id, worker.id),
                );
                match self
                    .execute_remote_batch(
                        worker,
                        &plan,
                        &task,
                        proxy.as_ref(),
                        conn.clone(),
                        cancel_token.clone(),
                        cancel_intent.clone(),
                        app,
                    )
                    .await
                {
                    Ok(progress) => progress,
                    Err(err) => {
                        self.write_task_log(
                            &conn,
                            task_id,
                            Some(run_id),
                            "warn",
                            &format!(
                                "Remote worker {} failed for batch {}: {}. Falling back to local worker",
                                worker.id, plan.batch_id, err
                            ),
                        );
                        self.execute_local_batch(
                            &plan,
                            conn.clone(),
                            cancel_token.clone(),
                            cancel_intent.clone(),
                            app,
                        )
                        .await?
                    }
                }
            };

            if progress.pause_reason.is_some() || cancel_token.is_cancelled() {
                return Ok(progress);
            }
        }

        self.mark_completed(&conn, task_id, run_id)?;
        Ok(self.current_progress(&conn, task_id, run_id, None)?)
    }

    async fn execute_local_batch(
        &self,
        plan: &BatchPlan,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        self.local_worker
            .execute_batch(plan, conn, cancel_token, cancel_intent, app)
            .await
    }

    async fn execute_remote_batch(
        &self,
        worker: &ClusterWorker,
        plan: &BatchPlan,
        task: &Task,
        proxy: Option<&ProxyConfig>,
        conn: Arc<Mutex<rusqlite::Connection>>,
        cancel_token: CancellationToken,
        cancel_intent: Arc<Mutex<CancelIntent>>,
        app: &AppHandle,
    ) -> Result<ScanProgress, String> {
        let remote = RemoteHttpWorker::new(worker.clone())?;
        {
            let c = conn.lock().map_err(|e| e.to_string())?;
            let repo = ScanBatchRepo::new(&c);
            repo.assign_worker(&plan.batch_id, remote.worker_id(), &ScanBatchStatus::Assigned)
                .map_err(|e| e.to_string())?;
        }

        remote.submit_batch(plan, task, proxy).await?;
        let mut result_cursor = plan.result_cursor;
        let mut log_cursor = plan.log_cursor;

        loop {
            if cancel_token.is_cancelled() {
                let intent = cancel_intent
                    .lock()
                    .map(|guard| *guard)
                    .unwrap_or(CancelIntent::Pause);
                let _ = match intent {
                    CancelIntent::Pause => remote.pause_batch(&plan.batch_id).await,
                    CancelIntent::Stop => remote.cancel_batch(&plan.batch_id).await,
                };
                return self.persist_cancel_before_batch(
                    &conn,
                    &plan.task_id,
                    &plan.run_id,
                    &cancel_intent,
                );
            }

            let status = remote.get_status(&plan.batch_id).await?;
            let flushed_count = self
                .pull_remote_results(&remote, plan, remote.worker_id(), &mut result_cursor, &conn)
                .await?;
            self.pull_remote_logs(&remote, plan, &mut log_cursor, &conn)
                .await?;
            self.persist_remote_status(
                &conn,
                plan,
                &status,
                result_cursor,
                log_cursor,
                app,
                flushed_count,
            )?;

            match status.status {
                ScanBatchStatus::Succeeded => {
                    return self.current_progress(&conn, &plan.task_id, &plan.run_id, None);
                }
                ScanBatchStatus::Failed | ScanBatchStatus::Expired => {
                    return Err(format!(
                        "Remote batch {} ended with status {}",
                        plan.batch_id,
                        status.status.as_str()
                    ));
                }
                ScanBatchStatus::Paused | ScanBatchStatus::Cancelled => {
                    return self.current_progress(
                        &conn,
                        &plan.task_id,
                        &plan.run_id,
                        Some(status.status.as_str().to_string()),
                    );
                }
                _ => {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }

    fn load_remaining_local_plans(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
        concurrency: i64,
    ) -> Result<Vec<BatchPlan>, String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = ScanBatchRepo::new(&c);
        let batches = repo
            .list_by_run(task_id, Some(run_id), i64::MAX, 0)
            .map_err(|e| e.to_string())?;

        Ok(batches
            .into_iter()
            .filter(|batch| {
                !matches!(
                    batch.status,
                    ScanBatchStatus::Succeeded | ScanBatchStatus::Cancelled
                )
            })
            .map(|batch| BatchPlan::from_scan_batch(&batch, concurrency))
            .collect())
    }

    fn load_task(&self, conn: &Mutex<rusqlite::Connection>, task_id: &str) -> Result<Task, String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = TaskRepo::new(&c);
        repo.get_by_id(task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))
    }

    fn load_proxy(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        proxy_id: Option<i64>,
    ) -> Result<Option<ProxyConfig>, String> {
        let Some(proxy_id) = proxy_id else {
            return Ok(None);
        };
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = ProxyRepo::new(&c);
        repo.get_by_id(proxy_id).map_err(|e| e.to_string())
    }

    fn load_available_remote_workers(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        concurrency: i64,
    ) -> Result<Vec<ClusterWorker>, String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = ClusterWorkerRepo::new(&c);
        let workers = repo.list().map_err(|e| e.to_string())?;
        Ok(workers
            .into_iter()
            .filter(|worker| {
                worker.enabled
                    && worker.worker_type == ClusterWorkerType::Remote
                    && worker.status == ClusterWorkerStatus::Available
                    && worker.base_url.is_some()
                    && worker.auth_token_ref.is_some()
                    && worker.current_running_batches
                        < worker.max_running_batches.unwrap_or(1).max(1)
                    && worker.current_concurrency
                        < worker.max_total_concurrency.unwrap_or(concurrency).max(1)
            })
            .collect())
    }

    async fn pull_remote_results(
        &self,
        remote: &RemoteHttpWorker,
        plan: &BatchPlan,
        worker_id: &str,
        result_cursor: &mut i64,
        conn: &Mutex<rusqlite::Connection>,
    ) -> Result<usize, String> {
        let mut flushed_count = 0usize;
        loop {
            let page = remote
                .get_results(&plan.batch_id, *result_cursor, 500)
                .await?;
            if page.items.is_empty() {
                *result_cursor = page.next_seq.max(*result_cursor);
                break;
            }
            flushed_count += self.persist_remote_results(conn, plan, worker_id, &page.items)?;
            *result_cursor = page.next_seq.max(*result_cursor);
            if !page.has_more {
                break;
            }
        }
        Ok(flushed_count)
    }

    async fn pull_remote_logs(
        &self,
        remote: &RemoteHttpWorker,
        plan: &BatchPlan,
        log_cursor: &mut i64,
        conn: &Mutex<rusqlite::Connection>,
    ) -> Result<(), String> {
        loop {
            let page = remote.get_logs(&plan.batch_id, *log_cursor, 500).await?;
            if page.items.is_empty() {
                *log_cursor = page.next_seq.max(*log_cursor);
                break;
            }
            self.persist_remote_logs(conn, plan, &page.items)?;
            *log_cursor = page.next_seq.max(*log_cursor);
            if !page.has_more {
                break;
            }
        }
        Ok(())
    }

    fn persist_remote_results(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        plan: &BatchPlan,
        worker_id: &str,
        results: &[BatchResult],
    ) -> Result<usize, String> {
        if results.is_empty() {
            return Ok(0);
        }
        let items: Vec<ScanItem> = results
            .iter()
            .map(|result| ScanItem {
                id: 0,
                task_id: plan.task_id.clone(),
                run_id: plan.run_id.clone(),
                batch_id: Some(plan.batch_id.clone()),
                worker_id: Some(worker_id.to_string()),
                domain: result.domain.clone(),
                tld: result.tld.clone(),
                item_index: result.item_index,
                status: parse_scan_item_status(&result.status),
                is_available: result.is_available,
                query_method: result.query_method.clone(),
                response_time_ms: result.response_time_ms,
                error_message: result.error_message.clone(),
                checked_at: Some(result.checked_at.clone()),
            })
            .collect();

        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = ScanItemRepo::new(&c);
        repo.batch_insert(&items).map_err(|e| e.to_string())
    }

    fn persist_remote_logs(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        plan: &BatchPlan,
        logs: &[BatchLog],
    ) -> Result<(), String> {
        if logs.is_empty() {
            return Ok(());
        }
        let c = conn.lock().map_err(|e| e.to_string())?;
        let repo = LogRepo::new(&c);
        for log in logs {
            let log_type = LogType::from_str(&log.log_type).unwrap_or(LogType::Request);
            repo.create_entry_with_type(
                &plan.task_id,
                Some(&plan.run_id),
                log_type,
                &log.level,
                &log.message,
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn persist_remote_status(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        plan: &BatchPlan,
        status: &BatchStatusSnapshot,
        result_cursor: i64,
        log_cursor: i64,
        app: &AppHandle,
        flushed_count: usize,
    ) -> Result<(), String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let batch_repo = ScanBatchRepo::new(&c);
        batch_repo
            .update_remote_progress(
                &plan.batch_id,
                &status.status,
                status.completed_count,
                status.available_count,
                status.error_count,
                result_cursor,
                log_cursor,
            )
            .map_err(|e| e.to_string())?;
        let summary = batch_repo
            .summarize(&plan.task_id, Some(&plan.run_id))
            .map_err(|e| e.to_string())?;
        let completed_index = (plan.start_index + status.completed_count).min(plan.end_index);
        let task_repo = TaskRepo::new(&c);
        let total_count = task_repo
            .get_by_id(&plan.task_id)
            .map_err(|e| e.to_string())?
            .map(|task| task.total_count)
            .unwrap_or(summary.completed_count);
        task_repo
            .update_progress(
                &plan.task_id,
                summary.completed_count,
                completed_index,
                summary.available_count,
                summary.error_count,
            )
            .map_err(|e| e.to_string())?;
        let run_repo = TaskRunRepo::new(&c);
        run_repo
            .update_progress(
                &plan.run_id,
                summary.completed_count,
                summary.available_count,
                summary.error_count,
            )
            .map_err(|e| e.to_string())?;

        if flushed_count > 0 {
            let _ = app.emit(
                "scan-results-updated",
                serde_json::json!({
                    "task_id": plan.task_id,
                    "run_id": plan.run_id,
                    "flushed_count": flushed_count,
                    "completed_count": summary.completed_count,
                }),
            );
        }
        let percent = if total_count > 0 {
            (summary.completed_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };
        let progress = ScanProgress {
            task_id: plan.task_id.clone(),
            run_id: plan.run_id.clone(),
            completed_count: summary.completed_count,
            total_count,
            available_count: summary.available_count,
            error_count: summary.error_count,
            percent,
            pause_reason: None,
        };
        let _ = app.emit("scan-progress", &progress);
        Ok(())
    }

    fn write_task_log(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: Option<&str>,
        level: &str,
        message: &str,
    ) {
        if let Ok(c) = conn.lock() {
            let repo = LogRepo::new(&c);
            let _ = repo.create_entry(task_id, run_id, level, message);
        }
    }

    fn persist_cancel_before_batch(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
        cancel_intent: &Arc<Mutex<CancelIntent>>,
    ) -> Result<ScanProgress, String> {
        let intent = cancel_intent
            .lock()
            .map(|guard| *guard)
            .unwrap_or(CancelIntent::Pause);
        let status = match intent {
            CancelIntent::Pause => TaskStatus::Paused,
            CancelIntent::Stop => TaskStatus::Stopped,
        };
        let batch_status = match intent {
            CancelIntent::Pause => ScanBatchStatus::Paused,
            CancelIntent::Stop => ScanBatchStatus::Cancelled,
        };
        let reason = match intent {
            CancelIntent::Pause => "manual_pause",
            CancelIntent::Stop => "manual_stop",
        };

        let c = conn.lock().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&c);
        let task = task_repo
            .get_by_id(task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        task_repo
            .update_status(task_id, &status)
            .map_err(|e| e.to_string())?;
        let run_repo = TaskRunRepo::new(&c);
        run_repo
            .update_status(run_id, &status, intent == CancelIntent::Stop)
            .map_err(|e| e.to_string())?;
        let batch_repo = ScanBatchRepo::new(&c);
        batch_repo
            .update_local_progress(task_id, run_id, task.completed_index, Some(batch_status))
            .map_err(|e| e.to_string())?;

        Ok(self.make_progress(&task, run_id, Some(reason.to_string())))
    }

    fn mark_completed(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
    ) -> Result<(), String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&c);
        task_repo
            .update_status(task_id, &TaskStatus::Completed)
            .map_err(|e| e.to_string())?;
        let run_repo = TaskRunRepo::new(&c);
        run_repo
            .update_status(run_id, &TaskStatus::Completed, true)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn current_progress(
        &self,
        conn: &Mutex<rusqlite::Connection>,
        task_id: &str,
        run_id: &str,
        pause_reason: Option<String>,
    ) -> Result<ScanProgress, String> {
        let c = conn.lock().map_err(|e| e.to_string())?;
        let task_repo = TaskRepo::new(&c);
        let task = task_repo
            .get_by_id(task_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Task not found: {}", task_id))?;
        Ok(self.make_progress(&task, run_id, pause_reason))
    }

    fn make_progress(
        &self,
        task: &crate::models::task::Task,
        run_id: &str,
        pause_reason: Option<String>,
    ) -> ScanProgress {
        let percent = if task.total_count > 0 {
            (task.completed_count as f64 / task.total_count as f64) * 100.0
        } else {
            0.0
        };
        ScanProgress {
            task_id: task.id.clone(),
            run_id: run_id.to_string(),
            completed_count: task.completed_count,
            total_count: task.total_count,
            available_count: task.available_count,
            error_count: task.error_count,
            percent,
            pause_reason,
        }
    }
}

fn parse_scan_item_status(value: &str) -> ScanItemStatus {
    serde_json::from_str::<ScanItemStatus>(&format!("\"{}\"", value.trim_matches('"')))
        .unwrap_or(ScanItemStatus::Error)
}
