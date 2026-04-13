use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GpuBackend {
    Auto,
    Cuda,
    DirectML,
    ROCm,
    CoreML,
    Cpu,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub id: i64,
    pub backend: GpuBackend,
    pub device_id: i64,
    pub batch_size: i64,
    pub model_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub backend: GpuBackend,
    pub available: bool,
    pub device_name: Option<String>,
    pub vram_total_mb: Option<i64>,
    pub vram_used_mb: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_backend_serialization() {
        assert_eq!(
            serde_json::to_string(&GpuBackend::DirectML).unwrap(),
            "\"directml\""
        );
        assert_eq!(
            serde_json::to_string(&GpuBackend::ROCm).unwrap(),
            "\"rocm\""
        );
        assert_eq!(
            serde_json::to_string(&GpuBackend::Cpu).unwrap(),
            "\"cpu\""
        );
    }

    #[test]
    fn test_gpu_config_roundtrip() {
        let config = GpuConfig {
            id: 1,
            backend: GpuBackend::Auto,
            device_id: 0,
            batch_size: 500,
            model_path: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GpuConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.backend, deserialized.backend);
        assert_eq!(config.batch_size, deserialized.batch_size);
    }
}
