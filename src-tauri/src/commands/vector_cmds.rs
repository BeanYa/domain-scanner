use crate::db::init;
use crate::db::llm_repo::LlmRepo;
use crate::db::log_repo::{LogRepo, LogType};
use crate::db::scan_item_repo::ScanItemRepo;
use crate::db::vector_repo::{VectorRecord, VectorRepo, EMBEDDING_DIM};
use crate::db::vectorize_run_repo::{VectorizeRun, VectorizeRunRepo};
use crate::embedding::gpu_detector::GpuDetector;
use crate::embedding::remote_api::RemoteEmbeddingClient;
use crate::models::gpu::GpuBackend;
use crate::models::scan_item::ScanItemStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{async_runtime, AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Default)]
pub struct VectorizeRunner {
    running: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl VectorizeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    fn start(&self, task_id: &str) -> Result<CancellationToken, String> {
        let mut running = self
            .running
            .lock()
            .map_err(|_| "向量化运行状态锁已损坏。".to_string())?;
        if running.contains_key(task_id) {
            return Err("该任务正在向量化，请等待完成或先取消。".to_string());
        }
        let token = CancellationToken::new();
        running.insert(task_id.to_string(), token.clone());
        Ok(token)
    }

    fn cancel(&self, task_id: &str) -> bool {
        match self.running.lock() {
            Ok(running) => {
                if let Some(token) = running.get(task_id) {
                    token.cancel();
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    fn finish(&self, task_id: &str) {
        if let Ok(mut running) = self.running.lock() {
            running.remove(task_id);
        }
    }

    fn is_running(&self, task_id: &str) -> bool {
        self.running
            .lock()
            .map(|running| running.contains_key(task_id))
            .unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorizeProgress {
    pub run_id: Option<String>,
    pub task_id: String,
    pub total: i64,
    pub processed: i64,
    pub percentage: f64,
    pub backend: GpuBackend,
    pub speed_per_sec: Option<f64>,
    pub estimated_remaining_secs: Option<f64>,
    pub status: String,
    pub message: Option<String>,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartVectorizeRequest {
    pub task_id: String,
    pub backend: Option<String>,
    pub batch_size: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartVectorizeResponse {
    pub run_id: String,
    pub task_id: String,
    pub backend: GpuBackend,
    pub processed: usize,
    pub skipped_existing: usize,
    pub pending: usize,
    pub total: i64,
    pub embedding_dim: usize,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopVectorizeResponse {
    pub task_id: String,
    pub cancelled: bool,
}

#[derive(Debug, Deserialize)]
pub struct VectorTaskRequest {
    pub task_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ListVectorsRequest {
    pub task_id: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct VectorItemRequest {
    pub domain_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorStats {
    pub task_id: String,
    pub table_name: String,
    pub embedding_dim: usize,
    pub total_available: i64,
    pub vector_count: i64,
    pub missing_count: i64,
    pub coverage: f64,
    pub running: bool,
    pub last_run: Option<VectorizeRun>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorListResponse {
    pub items: Vec<VectorRecord>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteVectorResponse {
    pub deleted: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevectorizeItemResponse {
    pub domain_id: i64,
    pub domain: String,
    pub vector_dim: usize,
}

#[derive(Debug)]
struct VectorizeJob {
    run_id: String,
    task_id: String,
    backend: GpuBackend,
    batch_size: usize,
    config: crate::models::llm::LlmConfig,
    pending: Vec<crate::models::scan_item::ScanItem>,
    skipped_existing: usize,
    total: i64,
}

#[tauri::command]
pub async fn start_vectorize(
    app: AppHandle,
    runner: State<'_, VectorizeRunner>,
    request: StartVectorizeRequest,
) -> Result<String, String> {
    let mut job = prepare_vectorize_job(request)?;
    let task_id = job.task_id.clone();
    let token = runner.start(&task_id)?;
    let run = match create_vectorize_run(&job) {
        Ok(run) => run,
        Err(err) => {
            runner.finish(&task_id);
            return Err(err);
        }
    };
    job.run_id = run.id.clone();
    let runner_handle = runner.inner().clone();
    let response = StartVectorizeResponse {
        run_id: run.id.clone(),
        task_id: task_id.clone(),
        backend: job.backend.clone(),
        processed: 0,
        skipped_existing: job.skipped_existing,
        pending: job.pending.len(),
        total: job.total,
        embedding_dim: EMBEDDING_DIM,
        status: "running".to_string(),
    };

    async_runtime::spawn(async move {
        let result = run_vectorize_job(Some(&app), job, token).await;
        if let Err(err) = result {
            if let Ok(conn) = init::open_db() {
                let _ = VectorizeRunRepo::new(&conn).finish(&run.id, "failed", None, Some(&err));
                write_vector_log_and_emit(
                    &conn,
                    Some(&app),
                    &task_id,
                    LogType::Task,
                    "error",
                    &format!("向量化失败：{}", err),
                );
            }
            let progress = VectorizeProgress {
                task_id: task_id.clone(),
                total: 0,
                processed: 0,
                percentage: 0.0,
                backend: GpuBackend::Remote,
                speed_per_sec: None,
                estimated_remaining_secs: None,
                status: "failed".to_string(),
                message: Some(err),
                updated_at: chrono::Utc::now().to_rfc3339(),
                run_id: Some(run.id.clone()),
                started_at: Some(run.started_at.clone()),
                finished_at: Some(chrono::Utc::now().to_rfc3339()),
            };
            emit_vector_progress(Some(&app), &progress);
        }
        runner_handle.finish(&task_id);
    });

    serde_json::to_string(&response).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_vectorize(
    app: AppHandle,
    runner: State<'_, VectorizeRunner>,
    request: VectorTaskRequest,
) -> Result<String, String> {
    let cancelled = runner.cancel(&request.task_id);
    if cancelled {
        if let Ok(conn) = init::open_db() {
            write_vector_log_and_emit(
                &conn,
                Some(&app),
                &request.task_id,
                LogType::Task,
                "warn",
                "已请求取消向量化，正在等待当前请求结束。",
            );
        }
    }
    serde_json::to_string(&StopVectorizeResponse {
        task_id: request.task_id,
        cancelled,
    })
    .map_err(|e| e.to_string())
}

fn prepare_vectorize_job(request: StartVectorizeRequest) -> Result<VectorizeJob, String> {
    // Determine backend
    let gpu_config = GpuDetector::default_config();
    let backend = match request.backend.as_deref() {
        Some("cpu") => GpuBackend::Cpu,
        Some("remote") => GpuBackend::Remote,
        Some("cuda") => GpuBackend::Cuda,
        Some("directml") => GpuBackend::DirectML,
        Some("rocm") => GpuBackend::ROCm,
        Some("coreml") => GpuBackend::CoreML,
        _ => gpu_config.backend,
    };

    if !matches!(backend, GpuBackend::Remote) {
        return Err(
            "当前版本使用 OpenAI 兼容远程 embedding；请在设置中配置默认 Embedding API，并使用 remote 后端。"
                .to_string(),
        );
    }

    let (config, pending, skipped_existing, total) = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = LlmRepo::new(&conn);
        let config = repo
            .get_default()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "未配置默认 Embedding API。请先在设置中保存 OpenAI 兼容 embedding 配置并设为默认。"
                    .to_string()
            })?;
        let scan_repo = ScanItemRepo::new(&conn);
        let vector_repo = VectorRepo::new(&conn);
        let items = scan_repo
            .list_by_task(
                &request.task_id,
                None,
                Some(&ScanItemStatus::Available),
                100_000,
                0,
            )
            .map_err(|e| e.to_string())?;
        if items.is_empty() {
            return Err("该任务没有可向量化的可用域名结果。".to_string());
        }

        let mut pending = Vec::new();
        let mut skipped_existing = 0usize;
        for item in items {
            if vector_repo.exists(item.id).map_err(|e| e.to_string())? {
                skipped_existing += 1;
            } else {
                pending.push(item);
            }
        }

        let total = (pending.len() + skipped_existing) as i64;

        (config, pending, skipped_existing, total)
    };

    if config.embedding_dim as usize != EMBEDDING_DIM {
        return Err(format!(
            "当前向量表固定为 {} 维，但默认配置为 {} 维。请使用支持 dimensions={} 的 embedding 模型，或将配置维度设为 {}。",
            EMBEDDING_DIM, config.embedding_dim, EMBEDDING_DIM, EMBEDDING_DIM
        ));
    }
    if config.embedding_model.is_none() {
        return Err("默认 Embedding 配置缺少 embedding_model，无法向量化。".to_string());
    }

    let batch_size = request.batch_size.unwrap_or(100).clamp(1, 500);

    Ok(VectorizeJob {
        run_id: String::new(),
        task_id: request.task_id,
        backend,
        batch_size,
        config,
        pending,
        skipped_existing,
        total,
    })
}

fn create_vectorize_run(job: &VectorizeJob) -> Result<VectorizeRun, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = VectorizeRunRepo::new(&conn);
    repo.create_running(
        &job.task_id,
        gpu_backend_label(&job.backend),
        job.total,
        job.skipped_existing as i64,
        job.skipped_existing as i64,
        job.batch_size as i64,
        EMBEDDING_DIM as i64,
    )
    .map_err(|e| e.to_string())
}

async fn run_vectorize_job(
    app: Option<&AppHandle>,
    job: VectorizeJob,
    token: CancellationToken,
) -> Result<StartVectorizeResponse, String> {
    let run_id = job.run_id;
    let task_id = job.task_id;
    let backend = job.backend;
    let batch_size = job.batch_size;
    let pending = job.pending;
    let skipped_existing = job.skipped_existing;
    let total = job.total;
    let batch_count = pending.len().div_ceil(batch_size);
    let client = RemoteEmbeddingClient::new(job.config);
    let mut processed = 0usize;
    let started_at = Instant::now();
    let initial_processed = job.skipped_existing as i64;

    {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        write_vector_log_and_emit(
            &conn,
            app,
            &task_id,
            LogType::Task,
            "info",
            &format!(
                "向量化开始：后端 remote，批大小 {}，总计 {}，已存在 {}，待处理 {}",
                batch_size,
                total,
                skipped_existing,
                pending.len()
            ),
        );
    }

    let mut progress = build_vector_progress(
        &task_id,
        total,
        initial_processed,
        backend.clone(),
        None,
        "running",
        Some("向量化已开始"),
        Some(&run_id),
    );
    emit_vector_progress(app, &progress);

    if pending.is_empty() {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        write_vector_log_and_emit(
            &conn,
            app,
            &task_id,
            LogType::Task,
            "info",
            "所有可用域名已存在向量，跳过本次向量化。",
        );
        progress = build_vector_progress(
            &task_id,
            total,
            total,
            backend.clone(),
            Some((started_at, initial_processed)),
            "completed",
            Some("所有域名已完成向量化"),
            Some(&run_id),
        );
        emit_vector_progress(app, &progress);
        finish_vectorize_run(&run_id, "completed", total, None);
        return Ok(StartVectorizeResponse {
            run_id,
            task_id,
            backend,
            processed,
            skipped_existing,
            pending: 0,
            total,
            embedding_dim: EMBEDDING_DIM,
            status: "completed".to_string(),
        });
    }

    for (index, chunk) in pending.chunks(batch_size).enumerate() {
        if token.is_cancelled() {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_vector_log_and_emit(
                &conn,
                app,
                &task_id,
                LogType::Task,
                "warn",
                "向量化已取消。",
            );
            progress = build_vector_progress(
                &task_id,
                total,
                initial_processed + processed as i64,
                backend.clone(),
                Some((started_at, initial_processed)),
                "cancelled",
                Some("向量化已取消"),
                Some(&run_id),
            );
            emit_vector_progress(app, &progress);
            finish_vectorize_run(
                &run_id,
                "cancelled",
                initial_processed + processed as i64,
                None,
            );
            return Ok(StartVectorizeResponse {
                run_id,
                task_id,
                backend,
                processed,
                skipped_existing,
                pending: pending.len().saturating_sub(processed),
                total,
                embedding_dim: EMBEDDING_DIM,
                status: "cancelled".to_string(),
            });
        }
        let texts: Vec<String> = chunk.iter().map(|item| item.domain.clone()).collect();
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_vector_log_and_emit(
                &conn,
                app,
                &task_id,
                LogType::Request,
                "info",
                &format!(
                    "Embedding 请求：批次 {}/{}，{} 个域名，未使用代理",
                    index + 1,
                    batch_count,
                    chunk.len()
                ),
            );
        }

        let result = client.embed(&texts).await?;
        if token.is_cancelled() {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_vector_log_and_emit(
                &conn,
                app,
                &task_id,
                LogType::Task,
                "warn",
                "向量化已取消，当前批次结果未写入。",
            );
            progress = build_vector_progress(
                &task_id,
                total,
                initial_processed + processed as i64,
                backend.clone(),
                Some((started_at, initial_processed)),
                "cancelled",
                Some("向量化已取消"),
                Some(&run_id),
            );
            emit_vector_progress(app, &progress);
            finish_vectorize_run(
                &run_id,
                "cancelled",
                initial_processed + processed as i64,
                None,
            );
            return Ok(StartVectorizeResponse {
                run_id,
                task_id,
                backend,
                processed,
                skipped_existing,
                pending: pending.len().saturating_sub(processed),
                total,
                embedding_dim: EMBEDDING_DIM,
                status: "cancelled".to_string(),
            });
        }
        if result.dim != EMBEDDING_DIM {
            return Err(format!(
                "Embedding API returned {} dimensions, expected {}",
                result.dim, EMBEDDING_DIM
            ));
        }
        if result.embeddings.len() != chunk.len() {
            return Err(format!(
                "Embedding API returned {} vectors for {} inputs",
                result.embeddings.len(),
                chunk.len()
            ));
        }
        let inserted_count;
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            let vector_repo = VectorRepo::new(&conn);
            let insert_items: Vec<(i64, &[f32])> = chunk
                .iter()
                .zip(result.embeddings.iter())
                .map(|(item, embedding)| (item.id, embedding.as_slice()))
                .collect();
            inserted_count = vector_repo
                .batch_insert(&insert_items)
                .map_err(|e| e.to_string())?;
            processed += inserted_count;
        }

        let current_processed = initial_processed + processed as i64;
        let message = format!(
            "批次 {}/{} 完成：已处理 {}/{}",
            index + 1,
            batch_count,
            current_processed,
            total
        );
        {
            let conn = init::open_db().map_err(|e| e.to_string())?;
            write_vector_log_and_emit(
                &conn,
                app,
                &task_id,
                LogType::Request,
                "info",
                &format!(
                    "Embedding 响应：批次 {}/{}，写入 {} 条向量，tokens {}",
                    index + 1,
                    batch_count,
                    inserted_count,
                    result
                        .tokens_used
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "未知".to_string())
                ),
            );
            write_vector_log_and_emit(&conn, app, &task_id, LogType::Task, "info", &message);
        }

        progress = build_vector_progress(
            &task_id,
            total,
            current_processed,
            backend.clone(),
            Some((started_at, initial_processed)),
            "running",
            Some(&message),
            Some(&run_id),
        );
        emit_vector_progress(app, &progress);
        update_vectorize_run_progress(&run_id, current_processed);
    }

    let conn = init::open_db().map_err(|e| e.to_string())?;
    write_vector_log_and_emit(
        &conn,
        app,
        &task_id,
        LogType::Task,
        "info",
        &format!(
            "向量化完成：新增 {}，已存在 {}，总计 {}",
            processed, skipped_existing, total
        ),
    );
    progress = build_vector_progress(
        &task_id,
        total,
        initial_processed + processed as i64,
        backend.clone(),
        Some((started_at, initial_processed)),
        "completed",
        Some("向量化完成"),
        Some(&run_id),
    );
    emit_vector_progress(app, &progress);
    finish_vectorize_run(
        &run_id,
        "completed",
        initial_processed + processed as i64,
        None,
    );

    Ok(StartVectorizeResponse {
        run_id,
        task_id,
        backend,
        processed,
        skipped_existing,
        pending: 0,
        total,
        embedding_dim: EMBEDDING_DIM,
        status: "completed".to_string(),
    })
}

#[tauri::command]
pub fn get_vectorize_progress(
    runner: State<'_, VectorizeRunner>,
    task_id: String,
) -> Result<String, String> {
    let is_running = runner.is_running(&task_id);
    let progress = get_vectorize_progress_value(task_id, is_running)?;
    serde_json::to_string(&progress).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_vector_stats(
    runner: State<'_, VectorizeRunner>,
    request: VectorTaskRequest,
) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let scan_repo = ScanItemRepo::new(&conn);
    let vector_repo = VectorRepo::new(&conn);
    let total_available = scan_repo
        .count_by_task(&request.task_id, None, Some(&ScanItemStatus::Available))
        .map_err(|e| e.to_string())?;
    let vector_count = vector_repo
        .count_by_task(&request.task_id)
        .map_err(|e| e.to_string())?
        .min(total_available);
    let missing_count = (total_available - vector_count).max(0);
    let coverage = calculate_percentage(vector_count, total_available);
    let last_run = VectorizeRunRepo::new(&conn)
        .get_latest_by_task(&request.task_id)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&VectorStats {
        task_id: request.task_id.clone(),
        table_name: "domain_vectors".to_string(),
        embedding_dim: EMBEDDING_DIM,
        total_available,
        vector_count,
        missing_count,
        coverage,
        running: runner.is_running(&request.task_id),
        last_run,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_vectors(request: ListVectorsRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let vector_repo = VectorRepo::new(&conn);
    let limit = request.limit.unwrap_or(50).clamp(1, 500);
    let offset = request.offset.unwrap_or(0).max(0);
    let items = vector_repo
        .list_by_task(&request.task_id, limit, offset)
        .map_err(|e| e.to_string())?;
    let total = vector_repo
        .count_by_task(&request.task_id)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&VectorListResponse { items, total }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_vector(app: AppHandle, request: VectorItemRequest) -> Result<String, String> {
    let (task_id, deleted) = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let scan_repo = ScanItemRepo::new(&conn);
        let item = scan_repo
            .get_by_id(request.domain_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "未找到对应域名记录。".to_string())?;
        let vector_repo = VectorRepo::new(&conn);
        let existed = vector_repo
            .exists(request.domain_id)
            .map_err(|e| e.to_string())?;
        vector_repo
            .delete(request.domain_id)
            .map_err(|e| e.to_string())?;
        write_vector_log_and_emit(
            &conn,
            Some(&app),
            &item.task_id,
            LogType::Task,
            "warn",
            &format!("已删除向量：{} ({})", item.domain, request.domain_id),
        );
        (item.task_id, if existed { 1 } else { 0 })
    };

    if let Ok(progress) = get_vectorize_progress_value(task_id, false) {
        emit_vector_progress(Some(&app), &progress);
    }

    serde_json::to_string(&DeleteVectorResponse { deleted }).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task_vectors(app: AppHandle, request: VectorTaskRequest) -> Result<String, String> {
    let deleted = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let vector_repo = VectorRepo::new(&conn);
        let deleted = vector_repo
            .delete_by_task(&request.task_id)
            .map_err(|e| e.to_string())?;
        write_vector_log_and_emit(
            &conn,
            Some(&app),
            &request.task_id,
            LogType::Task,
            "warn",
            &format!("已清空该任务的向量库记录：{} 条", deleted),
        );
        deleted
    };

    if let Ok(progress) = get_vectorize_progress_value(request.task_id, false) {
        emit_vector_progress(Some(&app), &progress);
    }

    serde_json::to_string(&DeleteVectorResponse { deleted }).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn revectorize_item(
    app: AppHandle,
    request: VectorItemRequest,
) -> Result<String, String> {
    let (item, config) = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let scan_repo = ScanItemRepo::new(&conn);
        let item = scan_repo
            .get_by_id(request.domain_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "未找到对应域名记录。".to_string())?;
        let repo = LlmRepo::new(&conn);
        let config = repo
            .get_default()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "未配置默认 Embedding API。请先在设置中保存 OpenAI 兼容 embedding 配置并设为默认。"
                    .to_string()
            })?;
        (item, config)
    };

    if config.embedding_dim as usize != EMBEDDING_DIM {
        return Err(format!(
            "当前向量表固定为 {} 维，但默认配置为 {} 维。",
            EMBEDDING_DIM, config.embedding_dim
        ));
    }
    if config.embedding_model.is_none() {
        return Err("默认 Embedding 配置缺少 embedding_model，无法重建向量。".to_string());
    }

    let client = RemoteEmbeddingClient::new(config);
    let result = client.embed(std::slice::from_ref(&item.domain)).await?;
    let embedding = result
        .embeddings
        .first()
        .ok_or_else(|| "Embedding API 没有返回向量。".to_string())?;
    if embedding.len() != EMBEDDING_DIM {
        return Err(format!(
            "Embedding API returned {} dimensions, expected {}",
            embedding.len(),
            EMBEDDING_DIM
        ));
    }

    {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let vector_repo = VectorRepo::new(&conn);
        vector_repo
            .upsert(item.id, embedding)
            .map_err(|e| e.to_string())?;
        write_vector_log_and_emit(
            &conn,
            Some(&app),
            &item.task_id,
            LogType::Task,
            "info",
            &format!("已重建向量：{} ({})", item.domain, item.id),
        );
        write_vector_log_and_emit(
            &conn,
            Some(&app),
            &item.task_id,
            LogType::Request,
            "info",
            &format!("Embedding 请求：单条重建 {}，未使用代理", item.domain),
        );
    }

    if let Ok(progress) = get_vectorize_progress_value(item.task_id.clone(), false) {
        emit_vector_progress(Some(&app), &progress);
    }

    serde_json::to_string(&RevectorizeItemResponse {
        domain_id: item.id,
        domain: item.domain,
        vector_dim: EMBEDDING_DIM,
    })
    .map_err(|e| e.to_string())
}

fn get_vectorize_progress_value(
    task_id: String,
    is_running: bool,
) -> Result<VectorizeProgress, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let scan_repo = ScanItemRepo::new(&conn);
    let vector_repo = VectorRepo::new(&conn);
    let run_repo = VectorizeRunRepo::new(&conn);
    let total = scan_repo
        .count_by_task(&task_id, None, Some(&ScanItemStatus::Available))
        .map_err(|e| e.to_string())?;
    let processed = vector_repo
        .count_by_task(&task_id)
        .map_err(|e| e.to_string())?
        .min(total);
    let percentage = calculate_percentage(processed, total);
    let latest_run = run_repo
        .get_latest_by_task(&task_id)
        .map_err(|e| e.to_string())?;
    let status = progress_status_from_run(total, processed, latest_run.as_ref(), is_running);
    let message = latest_run
        .as_ref()
        .and_then(|run| run.error_message.clone())
        .or_else(|| progress_message_for_status(&status).map(|value| value.to_string()));
    let updated_at = latest_run
        .as_ref()
        .map(|run| run.updated_at.clone())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    Ok(VectorizeProgress {
        run_id: latest_run.as_ref().map(|run| run.id.clone()),
        task_id,
        total,
        processed,
        percentage,
        backend: GpuBackend::Remote,
        speed_per_sec: None,
        estimated_remaining_secs: None,
        status,
        message,
        updated_at,
        started_at: latest_run.as_ref().map(|run| run.started_at.clone()),
        finished_at: latest_run.as_ref().and_then(|run| run.finished_at.clone()),
    })
}

fn build_vector_progress(
    task_id: &str,
    total: i64,
    processed: i64,
    backend: GpuBackend,
    timing: Option<(Instant, i64)>,
    status: &str,
    message: Option<&str>,
    run_id: Option<&str>,
) -> VectorizeProgress {
    let speed_per_sec = timing.and_then(|(started_at, initial_processed)| {
        let elapsed = started_at.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return None;
        }
        let newly_processed = (processed - initial_processed).max(0) as f64;
        if newly_processed <= 0.0 {
            None
        } else {
            Some(newly_processed / elapsed)
        }
    });
    let estimated_remaining_secs = speed_per_sec.and_then(|speed| {
        if speed <= 0.0 {
            None
        } else {
            Some(((total - processed).max(0) as f64) / speed)
        }
    });

    VectorizeProgress {
        run_id: run_id.map(|value| value.to_string()),
        task_id: task_id.to_string(),
        total,
        processed,
        percentage: calculate_percentage(processed, total),
        backend,
        speed_per_sec,
        estimated_remaining_secs,
        status: status.to_string(),
        message: message.map(|value| value.to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        started_at: None,
        finished_at: None,
    }
}

fn update_vectorize_run_progress(run_id: &str, processed_count: i64) {
    if let Ok(conn) = init::open_db() {
        let _ = VectorizeRunRepo::new(&conn).update_progress(run_id, processed_count);
    }
}

fn finish_vectorize_run(
    run_id: &str,
    status: &str,
    processed_count: i64,
    error_message: Option<&str>,
) {
    if let Ok(conn) = init::open_db() {
        let _ = VectorizeRunRepo::new(&conn).finish(
            run_id,
            status,
            Some(processed_count),
            error_message,
        );
    }
}

fn progress_status_from_run(
    total: i64,
    processed: i64,
    latest_run: Option<&VectorizeRun>,
    is_running: bool,
) -> String {
    if is_running {
        return "running".to_string();
    }
    if let Some(run) = latest_run {
        match run.status.as_str() {
            "running" => return "interrupted".to_string(),
            "failed" | "cancelled" | "interrupted" => return run.status.clone(),
            _ => {}
        }
    }
    if total > 0 && processed >= total {
        "completed".to_string()
    } else {
        "idle".to_string()
    }
}

fn progress_message_for_status(status: &str) -> Option<&'static str> {
    match status {
        "completed" => Some("向量化已完成"),
        "cancelled" => Some("向量化已取消"),
        "interrupted" => Some("向量化运行已中断"),
        "failed" => Some("向量化失败"),
        _ => None,
    }
}

fn calculate_percentage(processed: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        ((processed as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    }
}

fn gpu_backend_label(backend: &GpuBackend) -> &'static str {
    match backend {
        GpuBackend::Auto => "auto",
        GpuBackend::Cuda => "cuda",
        GpuBackend::DirectML => "directml",
        GpuBackend::ROCm => "rocm",
        GpuBackend::CoreML => "coreml",
        GpuBackend::Cpu => "cpu",
        GpuBackend::Remote => "remote",
    }
}

fn emit_vector_progress(app: Option<&AppHandle>, progress: &VectorizeProgress) {
    let Some(app) = app else {
        return;
    };
    let _ = app.emit("vector-progress", progress);
    let _ = app.emit(&format!("vector-progress-{}", progress.task_id), progress);
}

fn write_vector_log_and_emit(
    conn: &rusqlite::Connection,
    app: Option<&AppHandle>,
    task_id: &str,
    log_type: LogType,
    level: &str,
    message: &str,
) {
    let repo = LogRepo::new(conn);
    match repo.create_entry_with_type(task_id, None, log_type, level, message) {
        Ok(entry) => {
            if let Some(app) = app {
                let _ = app.emit("task-log-created", &entry);
                let _ = app.emit(&format!("task-log-{}", task_id), &entry);
            }
        }
        Err(err) => tracing::warn!("Failed to write vectorize log: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_vectorize_requires_default_llm_config() {
        let req = StartVectorizeRequest {
            task_id: "test-task".to_string(),
            backend: Some("remote".to_string()),
            batch_size: Some(100),
        };
        let result = prepare_vectorize_job(req);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未配置默认 Embedding API"));
    }

    #[test]
    fn test_vectorize_runner_cancel() {
        let runner = VectorizeRunner::new();
        let _token = runner.start("task-1").unwrap();
        assert!(runner.is_running("task-1"));
        assert!(runner.cancel("task-1"));
        runner.finish("task-1");
        assert!(!runner.is_running("task-1"));
    }

    #[test]
    fn test_get_vectorize_progress() {
        let progress = get_vectorize_progress_value("test-task".to_string(), false).unwrap();
        assert_eq!(progress.task_id, "test-task");
        assert_eq!(progress.percentage, 0.0);
        assert_eq!(progress.status, "idle");
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(0, 0), 0.0);
        assert_eq!(calculate_percentage(5, 10), 50.0);
        assert_eq!(calculate_percentage(15, 10), 100.0);
    }
}
