use serde::{Deserialize, Serialize};

pub const DEFAULT_SCAN_BATCH_SIZE: i64 = 5_000;
pub const LOCAL_WORKER_ID: &str = "local";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanBatchStatus {
    Queued,
    Assigned,
    Running,
    Succeeded,
    Failed,
    Retrying,
    Paused,
    Cancelled,
    Expired,
}

impl ScanBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScanBatchStatus::Queued => "queued",
            ScanBatchStatus::Assigned => "assigned",
            ScanBatchStatus::Running => "running",
            ScanBatchStatus::Succeeded => "succeeded",
            ScanBatchStatus::Failed => "failed",
            ScanBatchStatus::Retrying => "retrying",
            ScanBatchStatus::Paused => "paused",
            ScanBatchStatus::Cancelled => "cancelled",
            ScanBatchStatus::Expired => "expired",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.trim_matches('"') {
            "assigned" => ScanBatchStatus::Assigned,
            "running" => ScanBatchStatus::Running,
            "succeeded" => ScanBatchStatus::Succeeded,
            "failed" => ScanBatchStatus::Failed,
            "retrying" => ScanBatchStatus::Retrying,
            "paused" => ScanBatchStatus::Paused,
            "cancelled" => ScanBatchStatus::Cancelled,
            "expired" => ScanBatchStatus::Expired,
            _ => ScanBatchStatus::Queued,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanBatch {
    pub id: String,
    pub task_id: String,
    pub run_id: String,
    pub batch_index: i64,
    pub start_index: i64,
    pub end_index: i64,
    pub request_count: i64,
    pub status: ScanBatchStatus,
    pub worker_id: Option<String>,
    pub attempt: i64,
    pub completed_count: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub result_cursor: i64,
    pub log_cursor: i64,
    pub lease_expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanBatchSummary {
    pub total: i64,
    pub queued: i64,
    pub assigned: i64,
    pub running: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub retrying: i64,
    pub paused: i64,
    pub cancelled: i64,
    pub expired: i64,
    pub completed_count: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub worker_count: i64,
}

impl Default for ScanBatchSummary {
    fn default() -> Self {
        Self {
            total: 0,
            queued: 0,
            assigned: 0,
            running: 0,
            succeeded: 0,
            failed: 0,
            retrying: 0,
            paused: 0,
            cancelled: 0,
            expired: 0,
            completed_count: 0,
            available_count: 0,
            error_count: 0,
            worker_count: 0,
        }
    }
}

pub fn make_scan_batch_id(run_id: &str, batch_index: i64) -> String {
    format!("{}-batch-{:06}", run_id, batch_index)
}
