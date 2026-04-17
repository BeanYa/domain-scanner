use crate::db::cluster_worker_repo::ClusterWorkerRepo;
use crate::db::init;
use crate::db::scan_batch_repo::ScanBatchRepo;
use crate::models::cluster_worker::{
    ClusterWorker, ClusterWorkerStatus, ClusterWorkerType, WorkerCapabilities,
    WorkerHealthCheckResult,
};
use crate::models::scan_batch::LOCAL_WORKER_ID;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateWorkerRegistrationRequest {
    pub base_url: String,
    pub name: Option<String>,
    pub script_url: Option<String>,
    pub port: Option<u16>,
    pub timeout_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CreateWorkerRegistrationResponse {
    pub worker_id: String,
    pub status: ClusterWorkerStatus,
    pub install_command: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WorkerIdRequest {
    pub worker_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ListScanBatchesRequest {
    pub task_id: String,
    pub run_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ListScanBatchesResponse {
    pub items: Vec<crate::models::scan_batch::ScanBatch>,
    pub summary: crate::models::scan_batch::ScanBatchSummary,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
struct WorkerHealthPayload {
    worker_id: Option<String>,
    status: Option<String>,
    version: Option<String>,
    capabilities: Option<WorkerCapabilitiesPayload>,
    max_running_batches: Option<i64>,
    max_total_concurrency: Option<i64>,
    max_batch_concurrency: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct WorkerCapabilitiesPayload {
    max_running_batches: Option<i64>,
    max_total_concurrency: Option<i64>,
    max_batch_concurrency: Option<i64>,
}

#[tauri::command]
pub fn create_worker_registration(
    request: CreateWorkerRegistrationRequest,
) -> Result<String, String> {
    let base_url = normalize_base_url(&request.base_url)?;
    let port = request.port.unwrap_or(8731);
    let script_url = request
        .script_url
        .as_deref()
        .unwrap_or("https://example.com/domain-scanner/worker_install.sh")
        .trim()
        .to_string();
    if script_url.is_empty() {
        return Err("script_url cannot be empty".to_string());
    }

    let worker_id = Uuid::new_v4().to_string();
    let token = format!("token-{}", Uuid::new_v4());
    let token_hash = sha256_hex(&token);
    let install_command = format!("bash <(curl -fsSL {}) -t {} -p {}", script_url, token, port);
    let now = chrono::Utc::now();
    let expires_at = (now
        + chrono::Duration::seconds(request.timeout_seconds.unwrap_or(600).max(30)))
    .to_rfc3339();

    let worker = ClusterWorker {
        id: worker_id.clone(),
        name: request.name.filter(|name| !name.trim().is_empty()),
        base_url: Some(base_url),
        worker_type: ClusterWorkerType::Remote,
        status: ClusterWorkerStatus::Pending,
        registration_token_hash: Some(token_hash),
        auth_token_ref: Some(token),
        version: None,
        max_running_batches: None,
        max_total_concurrency: None,
        max_batch_concurrency: None,
        current_running_batches: 0,
        current_concurrency: 0,
        install_command: Some(install_command.clone()),
        expires_at: Some(expires_at.clone()),
        last_heartbeat_at: None,
        last_checked_at: None,
        last_error: None,
        enabled: true,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    };

    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ClusterWorkerRepo::new(&conn);
    repo.create_pending(&worker).map_err(|e| e.to_string())?;

    serde_json::to_string(&CreateWorkerRegistrationResponse {
        worker_id,
        status: ClusterWorkerStatus::Pending,
        install_command,
        expires_at,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_worker_registration(request: WorkerIdRequest) -> Result<String, String> {
    let worker = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = ClusterWorkerRepo::new(&conn);
        repo.get_by_id(&request.worker_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Worker not found: {}", request.worker_id))?
    };

    if let Some(expires_at) = worker.expires_at.as_deref() {
        if is_expired(expires_at) {
            let checked_at = chrono::Utc::now().to_rfc3339();
            let conn = init::open_db().map_err(|e| e.to_string())?;
            let repo = ClusterWorkerRepo::new(&conn);
            repo.update_health(
                &worker.id,
                &ClusterWorkerStatus::Expired,
                &checked_at,
                None,
                Some("Registration token expired"),
                None,
                None,
            )
            .map_err(|e| e.to_string())?;
            return serialize_health_result(WorkerHealthCheckResult {
                worker_id: worker.id,
                success: false,
                status: ClusterWorkerStatus::Expired,
                message: "注册令牌已过期".to_string(),
                checked_at,
                version: worker.version,
                capabilities: None,
            });
        }
    }

    let result = probe_worker(&worker, true).await;
    persist_probe_result(&worker.id, &result)?;
    serialize_health_result(result)
}

#[tauri::command]
pub fn list_workers() -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ClusterWorkerRepo::new(&conn);
    repo.upsert_local_worker().map_err(|e| e.to_string())?;
    let workers = repo.list().map_err(|e| e.to_string())?;
    serde_json::to_string(&workers).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_worker(request: WorkerIdRequest) -> Result<String, String> {
    if request.worker_id == LOCAL_WORKER_ID {
        let checked_at = chrono::Utc::now().to_rfc3339();
        return serialize_health_result(WorkerHealthCheckResult {
            worker_id: LOCAL_WORKER_ID.to_string(),
            success: true,
            status: ClusterWorkerStatus::Available,
            message: "本地内置 worker 可用".to_string(),
            checked_at,
            version: None,
            capabilities: Some(WorkerCapabilities {
                max_running_batches: 1,
                max_total_concurrency: 500,
                max_batch_concurrency: 500,
            }),
        });
    }

    let worker = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = ClusterWorkerRepo::new(&conn);
        repo.get_by_id(&request.worker_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Worker not found: {}", request.worker_id))?
    };
    let result = probe_worker(&worker, false).await;
    persist_probe_result(&worker.id, &result)?;
    serialize_health_result(result)
}

#[tauri::command]
pub fn enable_worker(request: WorkerIdRequest) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ClusterWorkerRepo::new(&conn);
    repo.set_enabled(&request.worker_id, true)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_worker(request: WorkerIdRequest) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ClusterWorkerRepo::new(&conn);
    repo.set_enabled(&request.worker_id, false)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_worker(request: WorkerIdRequest) -> Result<(), String> {
    if request.worker_id == LOCAL_WORKER_ID {
        return Err("本地 worker 不能删除，只能禁用。".to_string());
    }

    let conn = init::open_db().map_err(|e| e.to_string())?;
    let batch_repo = ScanBatchRepo::new(&conn);
    let active_count = batch_repo
        .active_count_for_worker(&request.worker_id)
        .map_err(|e| e.to_string())?;
    if active_count > 0 {
        return Err("该 worker 仍有运行中的 batch，请先停止任务。".to_string());
    }

    let repo = ClusterWorkerRepo::new(&conn);
    repo.delete(&request.worker_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_scan_batches(request: ListScanBatchesRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ScanBatchRepo::new(&conn);
    let summary = repo
        .summarize(&request.task_id, request.run_id.as_deref())
        .map_err(|e| e.to_string())?;
    let items = repo
        .list_by_run(
            &request.task_id,
            request.run_id.as_deref(),
            request.limit.unwrap_or(100).clamp(1, 500),
            request.offset.unwrap_or(0).max(0),
        )
        .map_err(|e| e.to_string())?;

    serde_json::to_string(&ListScanBatchesResponse {
        total: summary.total,
        summary,
        items,
    })
    .map_err(|e| e.to_string())
}

async fn probe_worker(
    worker: &ClusterWorker,
    pending_registration: bool,
) -> WorkerHealthCheckResult {
    let checked_at = chrono::Utc::now().to_rfc3339();
    let Some(base_url) = worker.base_url.as_deref() else {
        return WorkerHealthCheckResult {
            worker_id: worker.id.clone(),
            success: false,
            status: ClusterWorkerStatus::Error,
            message: "worker 缺少 base_url".to_string(),
            checked_at,
            version: worker.version.clone(),
            capabilities: None,
        };
    };
    let Some(token) = worker.auth_token_ref.as_deref() else {
        return WorkerHealthCheckResult {
            worker_id: worker.id.clone(),
            success: false,
            status: ClusterWorkerStatus::Error,
            message: "worker 缺少鉴权 token".to_string(),
            checked_at,
            version: worker.version.clone(),
            capabilities: None,
        };
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return WorkerHealthCheckResult {
                worker_id: worker.id.clone(),
                success: false,
                status: ClusterWorkerStatus::Error,
                message: format!("创建 HTTP client 失败: {}", err),
                checked_at,
                version: worker.version.clone(),
                capabilities: None,
            };
        }
    };

    match client
        .get(format!("{}/health", base_url))
        .bearer_auth(token)
        .send()
        .await
    {
        Ok(response) if response.status() == StatusCode::OK => {
            let payload = response.json::<WorkerHealthPayload>().await.ok();
            let returned_worker_id = payload.as_ref().and_then(|p| p.worker_id.clone());
            let capabilities = payload.as_ref().and_then(payload_to_capabilities);
            let version = payload.as_ref().and_then(|p| p.version.clone());
            let status_ok = payload
                .as_ref()
                .and_then(|p| p.status.as_deref())
                .map(|s| matches!(s, "ok" | "healthy" | "available" | "ready"))
                .unwrap_or(true);

            if !status_ok {
                return WorkerHealthCheckResult {
                    worker_id: worker.id.clone(),
                    success: false,
                    status: ClusterWorkerStatus::Unavailable,
                    message: "worker health 返回非 ready 状态".to_string(),
                    checked_at,
                    version,
                    capabilities,
                };
            }

            if pending_registration && returned_worker_id.as_deref().unwrap_or("").is_empty() {
                return WorkerHealthCheckResult {
                    worker_id: worker.id.clone(),
                    success: false,
                    status: ClusterWorkerStatus::Error,
                    message: "worker 未返回 worker_id".to_string(),
                    checked_at,
                    version,
                    capabilities,
                };
            }

            if pending_registration && capabilities.is_none() {
                return WorkerHealthCheckResult {
                    worker_id: worker.id.clone(),
                    success: false,
                    status: ClusterWorkerStatus::Error,
                    message: "worker 未返回有效 capabilities".to_string(),
                    checked_at,
                    version,
                    capabilities: None,
                };
            }

            WorkerHealthCheckResult {
                worker_id: worker.id.clone(),
                success: true,
                status: ClusterWorkerStatus::Available,
                message: "worker 已上线".to_string(),
                checked_at,
                version,
                capabilities,
            }
        }
        Ok(response) if response.status() == StatusCode::UNAUTHORIZED => WorkerHealthCheckResult {
            worker_id: worker.id.clone(),
            success: false,
            status: ClusterWorkerStatus::Error,
            message: "worker 鉴权失败".to_string(),
            checked_at,
            version: worker.version.clone(),
            capabilities: None,
        },
        Ok(response) => WorkerHealthCheckResult {
            worker_id: worker.id.clone(),
            success: false,
            status: ClusterWorkerStatus::Unavailable,
            message: format!("worker health HTTP {}", response.status().as_u16()),
            checked_at,
            version: worker.version.clone(),
            capabilities: None,
        },
        Err(err) => WorkerHealthCheckResult {
            worker_id: worker.id.clone(),
            success: false,
            status: ClusterWorkerStatus::Unavailable,
            message: format!("worker 不可达: {}", err),
            checked_at,
            version: worker.version.clone(),
            capabilities: None,
        },
    }
}

fn persist_probe_result(worker_id: &str, result: &WorkerHealthCheckResult) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ClusterWorkerRepo::new(&conn);
    repo.update_health(
        worker_id,
        &result.status,
        &result.checked_at,
        result.success.then_some(result.checked_at.as_str()),
        if result.success {
            None
        } else {
            Some(result.message.as_str())
        },
        result.version.as_deref(),
        result.capabilities.as_ref(),
    )
    .map_err(|e| e.to_string())
}

fn payload_to_capabilities(payload: &WorkerHealthPayload) -> Option<WorkerCapabilities> {
    let nested = payload.capabilities.as_ref();
    let capabilities = WorkerCapabilities {
        max_running_batches: nested
            .and_then(|c| c.max_running_batches)
            .or(payload.max_running_batches)
            .unwrap_or(0),
        max_total_concurrency: nested
            .and_then(|c| c.max_total_concurrency)
            .or(payload.max_total_concurrency)
            .unwrap_or(0),
        max_batch_concurrency: nested
            .and_then(|c| c.max_batch_concurrency)
            .or(payload.max_batch_concurrency)
            .unwrap_or(0),
    };
    if capabilities.max_running_batches > 0
        && capabilities.max_total_concurrency > 0
        && capabilities.max_batch_concurrency > 0
    {
        Some(capabilities)
    } else {
        None
    }
}

fn serialize_health_result(result: WorkerHealthCheckResult) -> Result<String, String> {
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn normalize_base_url(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("base_url cannot be empty".to_string());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("base_url must start with http:// or https://".to_string());
    }
    Ok(trimmed.to_string())
}

fn sha256_hex(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn is_expired(value: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|expires_at| expires_at.with_timezone(&chrono::Utc) < chrono::Utc::now())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8731/").unwrap(),
            "http://127.0.0.1:8731"
        );
        assert!(normalize_base_url("127.0.0.1:8731").is_err());
    }

    #[test]
    fn test_hash_token() {
        assert_eq!(sha256_hex("abc").len(), 64);
        assert_ne!(sha256_hex("abc"), sha256_hex("abcd"));
    }
}
