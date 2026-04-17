use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClusterWorkerStatus {
    Pending,
    Available,
    Unavailable,
    Error,
    Expired,
    Disabled,
}

impl ClusterWorkerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClusterWorkerStatus::Pending => "pending",
            ClusterWorkerStatus::Available => "available",
            ClusterWorkerStatus::Unavailable => "unavailable",
            ClusterWorkerStatus::Error => "error",
            ClusterWorkerStatus::Expired => "expired",
            ClusterWorkerStatus::Disabled => "disabled",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.trim_matches('"') {
            "available" => ClusterWorkerStatus::Available,
            "unavailable" => ClusterWorkerStatus::Unavailable,
            "error" => ClusterWorkerStatus::Error,
            "expired" => ClusterWorkerStatus::Expired,
            "disabled" => ClusterWorkerStatus::Disabled,
            _ => ClusterWorkerStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClusterWorkerType {
    Local,
    Remote,
}

impl ClusterWorkerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClusterWorkerType::Local => "local",
            ClusterWorkerType::Remote => "remote",
        }
    }

    pub fn from_db(value: &str) -> Self {
        match value.trim_matches('"') {
            "local" => ClusterWorkerType::Local,
            _ => ClusterWorkerType::Remote,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub max_running_batches: i64,
    pub max_total_concurrency: i64,
    pub max_batch_concurrency: i64,
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            max_running_batches: 1,
            max_total_concurrency: 50,
            max_batch_concurrency: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterWorker {
    pub id: String,
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub worker_type: ClusterWorkerType,
    pub status: ClusterWorkerStatus,
    #[serde(skip_serializing)]
    pub registration_token_hash: Option<String>,
    #[serde(skip_serializing)]
    pub auth_token_ref: Option<String>,
    pub version: Option<String>,
    pub max_running_batches: Option<i64>,
    pub max_total_concurrency: Option<i64>,
    pub max_batch_concurrency: Option<i64>,
    pub current_running_batches: i64,
    pub current_concurrency: i64,
    pub install_command: Option<String>,
    pub expires_at: Option<String>,
    pub last_heartbeat_at: Option<String>,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthCheckResult {
    pub worker_id: String,
    pub success: bool,
    pub status: ClusterWorkerStatus,
    pub message: String,
    pub checked_at: String,
    pub version: Option<String>,
    pub capabilities: Option<WorkerCapabilities>,
}
