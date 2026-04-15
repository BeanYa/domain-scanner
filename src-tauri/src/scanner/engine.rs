use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::models::scan_item::{ScanItem, ScanItemStatus};
use crate::models::task::TaskStatus;
use crate::scanner::domain_checker::{CheckConfig, DomainChecker};
use crate::scanner::list_generator::ListGenerator;
use crate::db::task_repo::TaskRepo;
use crate::db::scan_item_repo::ScanItemRepo;

/// Scan engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub max_concurrency: usize,
    pub request_delay_ms: u64,
    pub batch_write_size: usize,
    pub progress_report_interval_ms: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 50,
            request_delay_ms: 100,
            batch_write_size: 500,
            progress_report_interval_ms: 200,
        }
    }
}

/// Progress report for a running scan
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgress {
    pub task_id: String,
    pub completed_count: i64,
    pub total_count: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub percent: f64,
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

    pub fn with_default_config() -> Self {
        Self::new(EngineConfig::default(), CheckConfig::default())
    }

    /// Run a scan task (to be called in a tokio task)
    /// Returns the final progress when done or paused
    pub async fn run_scan(
        &self,
        task_id: &str,
        conn: Arc<rusqlite::Connection>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<ScanProgress, String> {
        // Get task info
        let task = {
            let repo = TaskRepo::new(&conn);
            repo.get_by_id(task_id)
                .map_err(|e| format!("Failed to get task: {}", e))?
                .ok_or_else(|| format!("Task {} not found", task_id))?
        };

        // Update task status to running
        {
            let repo = TaskRepo::new(&conn);
            repo.update_status(task_id, &TaskStatus::Running)
                .map_err(|e| format!("Failed to update status: {}", e))?;
        }

        // Create generator starting from checkpoint
        let mut generator = ListGenerator::new(task.scan_mode.clone(), task.tlds.clone())
            .with_start_index(task.completed_index);

        let _semaphore = Arc::new(Semaphore::new(self.config.max_concurrency));
        let mut completed_count = task.completed_count;
        let mut available_count = task.available_count;
        let mut error_count = task.error_count;

        while generator.has_more() {
            // Check for cancellation
            if cancel_token.is_cancelled() {
                self.persist_progress(&conn, task_id, completed_count, generator.current_index(), available_count, error_count);
                let repo = TaskRepo::new(&conn);
                repo.update_status(task_id, &TaskStatus::Paused)
                    .map_err(|e| format!("Failed to pause: {}", e))?;
                return Ok(self.make_progress(task_id, completed_count, task.total_count, available_count, error_count));
            }

            // Get next batch
            let batch = generator.next_batch();
            if batch.is_empty() {
                break;
            }

            // Check domains concurrently
            let domains: Vec<String> = batch.iter().map(|c| c.domain.clone()).collect();
            let results = self.checker.check_domains(&domains, self.config.max_concurrency).await;

            // Process results
            let mut scan_items = Vec::new();
            for (candidate, result) in batch.iter().zip(results.iter()) {
                completed_count += 1;

                match result.status {
                    ScanItemStatus::Available => available_count += 1,
                    ScanItemStatus::Error => error_count += 1,
                    _ => {}
                }

                scan_items.push(ScanItem {
                    id: 0,
                    task_id: task_id.to_string(),
                    domain: result.domain.clone(),
                    tld: candidate.tld.clone(),
                    item_index: candidate.index,
                    status: result.status.clone(),
                    is_available: result.is_available,
                    query_method: result.query_method.clone(),
                    response_time_ms: result.response_time_ms,
                    error_message: result.error_message.clone(),
                    checked_at: None,
                });

                // Batch write to database
                if scan_items.len() >= self.config.batch_write_size {
                    self.write_scan_items(&conn, &scan_items);
                    scan_items.clear();
                }
            }

            // Write remaining items
            if !scan_items.is_empty() {
                self.write_scan_items(&conn, &scan_items);
            }

            // Update progress in database
            self.persist_progress(&conn, task_id, completed_count, generator.current_index(), available_count, error_count);

            // Random delay between requests
            if self.config.request_delay_ms > 0 {
                let jitter = rand::Rng::gen_range(&mut rand::thread_rng(), 0..self.config.request_delay_ms);
                tokio::time::sleep(std::time::Duration::from_millis(jitter)).await;
            }
        }

        // Mark task as completed
        {
            let repo = TaskRepo::new(&conn);
            repo.update_status(task_id, &TaskStatus::Completed)
                .map_err(|e| format!("Failed to complete: {}", e))?;
        }

        Ok(self.make_progress(task_id, completed_count, task.total_count, available_count, error_count))
    }

    fn write_scan_items(&self, conn: &rusqlite::Connection, items: &[ScanItem]) {
        let repo = ScanItemRepo::new(conn);
        if let Err(e) = repo.batch_insert(items) {
            tracing::error!("Failed to batch insert scan items: {}", e);
        }
    }

    fn persist_progress(
        &self,
        conn: &rusqlite::Connection,
        task_id: &str,
        completed_count: i64,
        completed_index: i64,
        available_count: i64,
        error_count: i64,
    ) {
        let repo = TaskRepo::new(conn);
        if let Err(e) = repo.update_progress(task_id, completed_count, completed_index, available_count, error_count) {
            tracing::error!("Failed to update progress: {}", e);
        }
    }

    fn make_progress(
        &self,
        task_id: &str,
        completed_count: i64,
        total_count: i64,
        available_count: i64,
        error_count: i64,
    ) -> ScanProgress {
        let percent = if total_count > 0 {
            (completed_count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };
        ScanProgress {
            task_id: task_id.to_string(),
            completed_count,
            total_count,
            available_count,
            error_count,
            percent,
        }
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
        let progress = engine.make_progress("t1", 50, 100, 30, 2);
        assert_eq!(progress.completed_count, 50);
        assert_eq!(progress.total_count, 100);
        assert_eq!(progress.available_count, 30);
        assert_eq!(progress.error_count, 2);
        assert!((progress.percent - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_scan_progress_zero_total() {
        let engine = ScanEngine::with_default_config();
        let progress = engine.make_progress("t1", 0, 0, 0, 0);
        assert_eq!(progress.percent, 0.0);
    }
}
