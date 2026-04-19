use std::time::Duration;

use reqwest::StatusCode;
use serde::Serialize;

use crate::models::cluster_worker::{ClusterWorker, WorkerCapabilities};
use crate::models::proxy::ProxyConfig;
use crate::models::task::{ScanMode, Task};
use crate::scanner::batch::{
    BatchLogPage, BatchPlan, BatchResultPage, BatchStatusSnapshot, BatchSubmitAck,
};

pub struct RemoteHttpWorker {
    worker: ClusterWorker,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SubmitBatchRequest<'a> {
    batch_id: &'a str,
    task_id: &'a str,
    run_id: &'a str,
    batch_index: i64,
    start_index: i64,
    end_index: i64,
    scan_mode: &'a ScanMode,
    tlds: &'a [String],
    concurrency: i64,
    proxy: Option<&'a ProxyConfig>,
    attempt: i64,
}

impl RemoteHttpWorker {
    pub fn new(worker: ClusterWorker) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|err| format!("Failed to create remote worker client: {}", err))?;
        Ok(Self { worker, client })
    }

    pub fn worker_id(&self) -> &str {
        &self.worker.id
    }

    pub async fn get_health(&self) -> Result<serde_json::Value, String> {
        self.get_json("/health").await
    }

    pub async fn get_capabilities(&self) -> Result<WorkerCapabilities, String> {
        self.get_json("/capabilities").await
    }

    pub async fn submit_batch(
        &self,
        plan: &BatchPlan,
        task: &Task,
        proxy: Option<&ProxyConfig>,
    ) -> Result<BatchSubmitAck, String> {
        let request = SubmitBatchRequest {
            batch_id: &plan.batch_id,
            task_id: &plan.task_id,
            run_id: &plan.run_id,
            batch_index: plan.batch_index,
            start_index: plan.start_index,
            end_index: plan.end_index,
            scan_mode: &task.scan_mode,
            tlds: &task.tlds,
            concurrency: plan.concurrency,
            proxy,
            attempt: plan.attempt,
        };
        self.post_json("/batches", &request).await
    }

    pub async fn get_status(&self, batch_id: &str) -> Result<BatchStatusSnapshot, String> {
        self.get_json(&format!("/batches/{}/status", batch_id))
            .await
    }

    pub async fn get_results(
        &self,
        batch_id: &str,
        after_seq: i64,
        limit: usize,
    ) -> Result<BatchResultPage, String> {
        self.get_json(&format!(
            "/batches/{}/results?after_seq={}&limit={}",
            batch_id, after_seq, limit
        ))
        .await
    }

    pub async fn get_logs(
        &self,
        batch_id: &str,
        after_seq: i64,
        limit: usize,
    ) -> Result<BatchLogPage, String> {
        self.get_json(&format!(
            "/batches/{}/logs?after_seq={}&limit={}",
            batch_id, after_seq, limit
        ))
        .await
    }

    pub async fn pause_batch(&self, batch_id: &str) -> Result<(), String> {
        self.post_empty(&format!("/batches/{}/pause", batch_id)).await
    }

    pub async fn cancel_batch(&self, batch_id: &str) -> Result<(), String> {
        self.post_empty(&format!("/batches/{}/cancel", batch_id)).await
    }

    async fn get_json<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        self.request(self.client.get(self.url(path))).await
    }

    async fn post_json<T, B>(&self, path: &str, body: &B) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(self.client.post(self.url(path)).json(body)).await
    }

    async fn post_empty(&self, path: &str) -> Result<(), String> {
        let response = self
            .authorized(self.client.post(self.url(path)))
            .send()
            .await
            .map_err(|err| format!("Remote worker request failed: {}", err))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Remote worker {} returned HTTP {}",
                self.worker.id,
                response.status().as_u16()
            ))
        }
    }

    async fn request<T>(&self, builder: reqwest::RequestBuilder) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .authorized(builder)
            .send()
            .await
            .map_err(|err| format!("Remote worker request failed: {}", err))?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(format!("Remote worker {} rejected token", self.worker.id));
        }
        if !status.is_success() {
            return Err(format!(
                "Remote worker {} returned HTTP {}",
                self.worker.id,
                status.as_u16()
            ));
        }
        response
            .json::<T>()
            .await
            .map_err(|err| format!("Remote worker JSON parse failed: {}", err))
    }

    fn authorized(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.worker.auth_token_ref.as_deref() {
            Some(token) => builder.bearer_auth(token),
            None => builder,
        }
    }

    fn url(&self, path: &str) -> String {
        let base_url = self.worker.base_url.as_deref().unwrap_or_default();
        format!("{}{}", base_url.trim_end_matches('/'), path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::cluster_worker::{ClusterWorkerStatus, ClusterWorkerType};

    fn make_worker(base_url: &str) -> ClusterWorker {
        ClusterWorker {
            id: "worker-1".to_string(),
            name: Some("worker".to_string()),
            base_url: Some(base_url.to_string()),
            worker_type: ClusterWorkerType::Remote,
            status: ClusterWorkerStatus::Available,
            registration_token_hash: None,
            auth_token_ref: Some("token".to_string()),
            version: None,
            max_running_batches: Some(1),
            max_total_concurrency: Some(50),
            max_batch_concurrency: Some(50),
            current_running_batches: 0,
            current_concurrency: 0,
            install_command: None,
            expires_at: None,
            last_heartbeat_at: None,
            last_checked_at: None,
            last_error: None,
            enabled: true,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_remote_worker_url_trims_base_slash() {
        let worker = RemoteHttpWorker::new(make_worker("http://127.0.0.1:8731/")).unwrap();
        assert_eq!(
            worker.url("/health"),
            "http://127.0.0.1:8731/health".to_string()
        );
    }
}
