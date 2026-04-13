/// GPU detector for automatic backend selection
/// Detects available GPU hardware and selects the best ONNX Runtime backend

use crate::models::gpu::{GpuBackend, GpuConfig, GpuStatus};

/// GPU detector for selecting the best available backend
pub struct GpuDetector;

impl GpuDetector {
    /// Detect available GPU backends based on compile-time features and runtime availability
    pub fn detect() -> GpuStatus {
        let (backend, available, device_name) = Self::detect_backend();

        GpuStatus {
            backend,
            available,
            device_name,
            vram_total_mb: None,
            vram_used_mb: None,
        }
    }

    /// Detect the best available backend
    fn detect_backend() -> (GpuBackend, bool, Option<String>) {
        // Priority chain: CUDA > DirectML > ROCm > CoreML > CPU
        // The actual GPU availability depends on compile-time features

        #[cfg(feature = "gpu-cuda")]
        {
            if let Some(info) = Self::check_cuda() {
                return (GpuBackend::Cuda, true, Some(info));
            }
        }

        #[cfg(feature = "gpu-directml")]
        {
            if let Some(info) = Self::check_directml() {
                return (GpuBackend::DirectML, true, Some(info));
            }
        }

        #[cfg(feature = "gpu-rocm")]
        {
            if let Some(info) = Self::check_rocm() {
                return (GpuBackend::ROCm, true, Some(info));
            }
        }

        #[cfg(feature = "gpu-coreml")]
        {
            if let Some(info) = Self::check_coreml() {
                return (GpuBackend::CoreML, true, Some(info));
            }
        }

        // No GPU available, fall back to CPU
        (GpuBackend::Cpu, true, Some("CPU".to_string()))
    }

    /// Check CUDA availability
    #[cfg(feature = "gpu-cuda")]
    fn check_cuda() -> Option<String> {
        // In real implementation, would check for CUDA driver and device
        // For now, return None to simulate no CUDA device
        None
    }

    /// Check DirectML availability (Windows)
    #[cfg(feature = "gpu-directml")]
    fn check_directml() -> Option<String> {
        // In real implementation, would check for DirectX 12 and GPU device
        None
    }

    /// Check ROCm availability (Linux)
    #[cfg(feature = "gpu-rocm")]
    fn check_rocm() -> Option<String> {
        // In real implementation, would check for ROCm installation and AMD GPU
        None
    }

    /// Check CoreML availability (macOS)
    #[cfg(feature = "gpu-coreml")]
    fn check_coreml() -> Option<String> {
        // In real implementation, would check for Apple Silicon
        None
    }

    /// Select the best backend based on config and availability
    pub fn select_backend(config: &GpuConfig) -> GpuStatus {
        match config.backend {
            GpuBackend::Auto => Self::detect(),
            GpuBackend::Cuda => {
                #[cfg(feature = "gpu-cuda")]
                {
                    if let Some(info) = Self::check_cuda() {
                        return GpuStatus {
                            backend: GpuBackend::Cuda,
                            available: true,
                            device_name: Some(info),
                            vram_total_mb: None,
                            vram_used_mb: None,
                        };
                    }
                }
                GpuStatus {
                    backend: GpuBackend::Cuda,
                    available: false,
                    device_name: None,
                    vram_total_mb: None,
                    vram_used_mb: None,
                }
            }
            GpuBackend::DirectML => {
                #[cfg(feature = "gpu-directml")]
                {
                    if let Some(info) = Self::check_directml() {
                        return GpuStatus {
                            backend: GpuBackend::DirectML,
                            available: true,
                            device_name: Some(info),
                            vram_total_mb: None,
                            vram_used_mb: None,
                        };
                    }
                }
                GpuStatus {
                    backend: GpuBackend::DirectML,
                    available: false,
                    device_name: None,
                    vram_total_mb: None,
                    vram_used_mb: None,
                }
            }
            GpuBackend::ROCm => {
                #[cfg(feature = "gpu-rocm")]
                {
                    if let Some(info) = Self::check_rocm() {
                        return GpuStatus {
                            backend: GpuBackend::ROCm,
                            available: true,
                            device_name: Some(info),
                            vram_total_mb: None,
                            vram_used_mb: None,
                        };
                    }
                }
                GpuStatus {
                    backend: GpuBackend::ROCm,
                    available: false,
                    device_name: None,
                    vram_total_mb: None,
                    vram_used_mb: None,
                }
            }
            GpuBackend::CoreML => {
                #[cfg(feature = "gpu-coreml")]
                {
                    if let Some(info) = Self::check_coreml() {
                        return GpuStatus {
                            backend: GpuBackend::CoreML,
                            available: true,
                            device_name: Some(info),
                            vram_total_mb: None,
                            vram_used_mb: None,
                        };
                    }
                }
                GpuStatus {
                    backend: GpuBackend::CoreML,
                    available: false,
                    device_name: None,
                    vram_total_mb: None,
                    vram_used_mb: None,
                }
            }
            GpuBackend::Cpu => GpuStatus {
                backend: GpuBackend::Cpu,
                available: true,
                device_name: Some("CPU".to_string()),
                vram_total_mb: None,
                vram_used_mb: None,
            },
            GpuBackend::Remote => GpuStatus {
                backend: GpuBackend::Remote,
                available: true,
                device_name: Some("Remote API".to_string()),
                vram_total_mb: None,
                vram_used_mb: None,
            },
        }
    }

    /// Get the default GPU config
    pub fn default_config() -> GpuConfig {
        GpuConfig {
            id: 1,
            backend: GpuBackend::Auto,
            device_id: 0,
            batch_size: 500,
            model_path: None,
        }
    }

    /// Determine fallback chain for a given backend
    pub fn fallback_chain(backend: &GpuBackend) -> Vec<GpuBackend> {
        match backend {
            GpuBackend::Auto => vec![
                GpuBackend::Cuda,
                GpuBackend::DirectML,
                GpuBackend::ROCm,
                GpuBackend::CoreML,
                GpuBackend::Remote,
                GpuBackend::Cpu,
            ],
            GpuBackend::Cuda => vec![GpuBackend::DirectML, GpuBackend::Remote, GpuBackend::Cpu],
            GpuBackend::DirectML => vec![GpuBackend::Remote, GpuBackend::Cpu],
            GpuBackend::ROCm => vec![GpuBackend::Remote, GpuBackend::Cpu],
            GpuBackend::CoreML => vec![GpuBackend::Remote, GpuBackend::Cpu],
            GpuBackend::Cpu => vec![GpuBackend::Remote],
            GpuBackend::Remote => vec![GpuBackend::Cpu],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_returns_status() {
        let status = GpuDetector::detect();
        // Without GPU features, should fall back to CPU
        assert!(status.available);
    }

    #[test]
    fn test_default_config() {
        let config = GpuDetector::default_config();
        assert_eq!(config.backend, GpuBackend::Auto);
        assert_eq!(config.batch_size, 500);
        assert_eq!(config.device_id, 0);
    }

    #[test]
    fn test_select_backend_auto() {
        let config = GpuDetector::default_config();
        let status = GpuDetector::select_backend(&config);
        assert!(status.available);
    }

    #[test]
    fn test_select_backend_cpu() {
        let config = GpuConfig {
            id: 1,
            backend: GpuBackend::Cpu,
            device_id: 0,
            batch_size: 500,
            model_path: None,
        };
        let status = GpuDetector::select_backend(&config);
        assert!(status.available);
        assert_eq!(status.backend, GpuBackend::Cpu);
    }

    #[test]
    fn test_select_backend_cuda_unavailable() {
        // Without actual CUDA hardware, this should be unavailable
        let config = GpuConfig {
            id: 1,
            backend: GpuBackend::Cuda,
            device_id: 0,
            batch_size: 500,
            model_path: None,
        };
        let status = GpuDetector::select_backend(&config);
        assert_eq!(status.backend, GpuBackend::Cuda);
        // CUDA is likely not available in test environment
        assert!(!status.available);
    }

    #[test]
    fn test_fallback_chain_auto() {
        let chain = GpuDetector::fallback_chain(&GpuBackend::Auto);
        assert!(!chain.is_empty());
        assert!(chain.contains(&GpuBackend::Cpu));
    }

    #[test]
    fn test_fallback_chain_cuda() {
        let chain = GpuDetector::fallback_chain(&GpuBackend::Cuda);
        assert!(chain.contains(&GpuBackend::Cpu));
        assert!(chain.contains(&GpuBackend::Remote));
    }

    #[test]
    fn test_fallback_chain_cpu() {
        let chain = GpuDetector::fallback_chain(&GpuBackend::Cpu);
        assert!(chain.contains(&GpuBackend::Remote));
    }
}
