use crate::db::gpu_repo::GpuRepo;
use crate::db::init;
use crate::embedding::gpu_detector::GpuDetector;
use crate::models::gpu::{GpuBackend, GpuConfig, GpuStatus};
use serde::Deserialize;

#[tauri::command]
pub fn get_gpu_status() -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = GpuRepo::new(&conn);

    // Get stored config or default
    let config = repo.get_config().map_err(|e| e.to_string())?;

    // Detect current status
    let status = GpuDetector::select_backend(&config);

    // Enrich with stored config info
    let result = serde_json::json!({
        "status": status,
        "config": config,
        "fallback_chain": GpuDetector::fallback_chain(&config.backend),
    });

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct UpdateGpuConfigRequest {
    pub backend: Option<String>,
    pub device_id: Option<i64>,
    pub batch_size: Option<i64>,
    pub model_path: Option<String>,
}

#[tauri::command]
pub fn update_gpu_config(request: UpdateGpuConfigRequest) -> Result<(), String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = GpuRepo::new(&conn);

    let mut config = repo.get_config().map_err(|e| e.to_string())?;

    if let Some(backend) = request.backend {
        config.backend = match backend.to_lowercase().as_str() {
            "auto" => GpuBackend::Auto,
            "cuda" => GpuBackend::Cuda,
            "directml" => GpuBackend::DirectML,
            "rocm" => GpuBackend::ROCm,
            "coreml" => GpuBackend::CoreML,
            "cpu" => GpuBackend::Cpu,
            "remote" => GpuBackend::Remote,
            _ => return Err(format!("Unknown GPU backend: {}", backend)),
        };
    }

    if let Some(device_id) = request.device_id {
        config.device_id = device_id;
    }

    if let Some(batch_size) = request.batch_size {
        config.batch_size = batch_size;
    }

    if let Some(model_path) = request.model_path {
        config.model_path = Some(model_path);
    }

    // Verify the selected backend is available
    let status = GpuDetector::select_backend(&config);
    if !status.available && config.backend != GpuBackend::Auto {
        return Err(format!("GPU backend {:?} is not available on this system", config.backend));
    }

    repo.update_config(&config).map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_gpu_status() {
        let result = get_gpu_status().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["status"].is_object());
        assert!(parsed["config"].is_object());
        assert!(parsed["fallback_chain"].is_array());
    }

    #[test]
    fn test_update_gpu_config_cpu() {
        let req = UpdateGpuConfigRequest {
            backend: Some("cpu".to_string()),
            device_id: Some(0),
            batch_size: Some(500),
            model_path: None,
        };
        let result = update_gpu_config(req);
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_gpu_config_invalid_backend() {
        let req = UpdateGpuConfigRequest {
            backend: Some("invalid".to_string()),
            device_id: None,
            batch_size: None,
            model_path: None,
        };
        let result = update_gpu_config(req);
        assert!(result.is_err());
    }
}
