use crate::models::scan_batch::{ScanBatch, ScanBatchStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPlan {
    pub batch_id: String,
    pub task_id: String,
    pub run_id: String,
    pub batch_index: i64,
    pub start_index: i64,
    pub end_index: i64,
    pub request_count: i64,
    pub worker_id: Option<String>,
    pub attempt: i64,
    pub concurrency: i64,
    pub result_cursor: i64,
    pub log_cursor: i64,
}

impl BatchPlan {
    pub fn from_scan_batch(batch: &ScanBatch, concurrency: i64) -> Self {
        Self {
            batch_id: batch.id.clone(),
            task_id: batch.task_id.clone(),
            run_id: batch.run_id.clone(),
            batch_index: batch.batch_index,
            start_index: batch.start_index,
            end_index: batch.end_index,
            request_count: batch.request_count,
            worker_id: batch.worker_id.clone(),
            attempt: batch.attempt,
            concurrency,
            result_cursor: batch.result_cursor,
            log_cursor: batch.log_cursor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStatusSnapshot {
    pub batch_id: String,
    pub status: ScanBatchStatus,
    pub attempt: i64,
    pub request_count: i64,
    pub completed_count: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub result_cursor: i64,
    pub log_cursor: i64,
    pub started_at: Option<String>,
    pub updated_at: String,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub seq: i64,
    pub item_index: i64,
    pub domain: String,
    pub tld: String,
    pub status: String,
    pub is_available: Option<bool>,
    pub query_method: Option<String>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLog {
    pub seq: i64,
    pub level: String,
    pub log_type: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSubmitAck {
    pub accepted: bool,
    pub batch_id: String,
    pub status: ScanBatchStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultPage {
    pub batch_id: String,
    pub items: Vec<BatchResult>,
    pub next_seq: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLogPage {
    pub batch_id: String,
    pub items: Vec<BatchLog>,
    pub next_seq: i64,
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::scan_batch::ScanBatchStatus;

    #[test]
    fn test_batch_plan_from_scan_batch() {
        let now = "2026-01-01T00:00:00Z".to_string();
        let batch = ScanBatch {
            id: "batch-1".to_string(),
            task_id: "task-1".to_string(),
            run_id: "run-1".to_string(),
            batch_index: 2,
            start_index: 100,
            end_index: 200,
            request_count: 100,
            status: ScanBatchStatus::Queued,
            worker_id: Some("local".to_string()),
            attempt: 1,
            completed_count: 0,
            available_count: 0,
            error_count: 0,
            result_cursor: 0,
            log_cursor: 0,
            lease_expires_at: None,
            created_at: now.clone(),
            updated_at: now,
        };

        let plan = BatchPlan::from_scan_batch(&batch, 50);
        assert_eq!(plan.batch_id, "batch-1");
        assert_eq!(plan.start_index, 100);
        assert_eq!(plan.end_index, 200);
        assert_eq!(plan.concurrency, 50);
    }
}
