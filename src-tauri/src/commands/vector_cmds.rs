use crate::db::init;
use crate::embedding::gpu_detector::GpuDetector;
use crate::models::gpu::{GpuBackend, GpuConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorizeProgress {
    pub task_id: String,
    pub total: i64,
    pub processed: i64,
    pub percentage: f64,
    pub backend: GpuBackend,
    pub estimated_remaining_secs: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct StartVectorizeRequest {
    pub task_id: String,
    pub backend: Option<String>,
    pub batch_size: Option<usize>,
}

#[tauri::command]
pub fn start_vectorize(request: StartVectorizeRequest) -> Result<(), String> {
    let _conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;

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

    // Check backend availability
    let status = GpuDetector::select_backend(&GpuConfig {
        backend: backend.clone(),
        ..gpu_config
    });

    if !status.available {
        return Err(format!("Selected backend {:?} is not available", backend));
    }

    // In full implementation:
    // 1. Load scan items for the task
    // 2. Initialize embedding model with selected backend
    // 3. Process items in batches
    // 4. Store vectors in sqlite-vec
    // 5. Emit progress events via Tauri

    Ok(())
}

#[tauri::command]
pub fn get_vectorize_progress(task_id: String) -> Result<String, String> {
    // In full implementation, this would query actual progress from the running vectorization
    let progress = VectorizeProgress {
        task_id,
        total: 0,
        processed: 0,
        percentage: 0.0,
        backend: GpuBackend::Cpu,
        estimated_remaining_secs: None,
    };

    serde_json::to_string(&progress).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_vectorize_cpu() {
        let req = StartVectorizeRequest {
            task_id: "test-task".to_string(),
            backend: Some("cpu".to_string()),
            batch_size: Some(100),
        };
        // This will work since CPU is always available
        let result = start_vectorize(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_vectorize_progress() {
        let result = get_vectorize_progress("test-task".to_string()).unwrap();
        let progress: VectorizeProgress = serde_json::from_str(&result).unwrap();
        assert_eq!(progress.task_id, "test-task");
        assert_eq!(progress.percentage, 0.0);
    }
}
