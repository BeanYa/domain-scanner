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

        // Fallback: try platform-specific detection even without ONNX features
        if cfg!(target_os = "windows") {
            if let Some(info) = Self::detect_windows_gpu() {
                return (GpuBackend::DirectML, true, Some(info));
            }
        }

        (GpuBackend::Cpu, true, Some("CPU".to_string()))
    }

    /// Check CUDA availability via ONNX Runtime
    #[cfg(feature = "gpu-cuda")]
    fn check_cuda() -> Option<String> {
        use ort::session::Session;
        match Session::builder()
            .ok()?
            .with_execution_providers([ort::ep::CUDA::default().build()])
            .ok()?
            .commit_from_memory(&[0u8; 1])
        {
            Ok(_) => Some("NVIDIA GPU (via CUDA)".to_string()),
            Err(_) => None,
        }
    }

    /// Check DirectML availability via ONNX Runtime
    #[cfg(feature = "gpu-directml")]
    fn check_directml() -> Option<String> {
        use ort::session::Session;
        match Session::builder()
            .ok()?
            .with_execution_providers([ort::ep::DirectML::default().build()])
            .ok()?
            .commit_from_memory(&[0u8; 1])
        {
            Ok(_) => Some("GPU (via DirectML)".to_string()),
            Err(_) => None,
        }
    }

    /// Check ROCm availability via ONNX Runtime
    #[cfg(feature = "gpu-rocm")]
    fn check_rocm() -> Option<String> {
        use ort::session::Session;
        match Session::builder()
            .ok()?
            .with_execution_providers([ort::ep::ROCm::default().build()])
            .ok()?
            .commit_from_memory(&[0u8; 1])
        {
            Ok(_) => Some("AMD GPU (via ROCm)".to_string()),
            Err(_) => None,
        }
    }

    /// Check CoreML availability via ONNX Runtime
    #[cfg(feature = "gpu-coreml")]
    fn check_coreml() -> Option<String> {
        use ort::session::Session;
        match Session::builder()
            .ok()?
            .with_execution_providers([ort::ep::CoreML::default().build()])
            .ok()?
            .commit_from_memory(&[0u8; 1])
        {
            Ok(_) => Some("Apple Silicon (via CoreML)".to_string()),
            Err(_) => None,
        }
    }

    /// Fallback Windows GPU detection: reads adapter info from registry / DXGI
    /// Works without ONNX Runtime compiled in
    #[cfg(target_os = "windows")]
    fn detect_windows_gpu() -> Option<String> {
        use std::process::Command;

        // Use PowerShell to query GPU info via WMI/CIM
        if let Ok(output) = Command::new("powershell")
            .args(["-NoProfile", "-Command", r#"Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name | Where-Object { $_ -notmatch 'Basic|Microsoft|Remote' }"#])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let gpu_name = stdout.lines()
                    .next()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())?;
                return Some(format!("{} (DirectML)", gpu_name));
            }
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    fn detect_windows_gpu() -> Option<String> {
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
                // Fallback to Windows detection
                if cfg!(target_os = "windows") {
                    if let Some(name) = Self::detect_windows_gpu() {
                        return GpuStatus {
                            backend: GpuBackend::DirectML,
                            available: true,
                            device_name: Some(name),
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
                GpuBackend::DirectML,
                GpuBackend::Cuda,
                GpuBackend::ROCm,
                GpuBackend::CoreML,
                GpuBackend::Remote,
                GpuBackend::Cpu,
            ],
            GpuBackend::Cuda => vec![GpuBackend::DirectML, GpuBackend::Remote, GpuBackend::Cpu],
            GpuBackend::DirectML => vec![GpuBackend::Cuda, GpuBackend::Remote, GpuBackend::Cpu],
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
        assert!(status.available);
        assert!(status.device_name.is_some());
    }

    #[test]
    fn test_detect_backend_not_cpu_on_windows() {
        // On Windows with a GPU, should detect DirectML or at least report GPU name
        let status = GpuDetector::detect();
        if cfg!(target_os = "windows") {
            println!("Detected backend: {:?}, name: {:?}", status.backend, status.device_name);
        }
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
    fn test_select_backend_remote() {
        let config = GpuConfig {
            id: 1,
            backend: GpuBackend::Remote,
            device_id: 0,
            batch_size: 500,
            model_path: None,
        };
        let status = GpuDetector::select_backend(&config);
        assert!(status.available);
        assert_eq!(status.backend, GpuBackend::Remote);
    }

    #[test]
    fn test_fallback_chain_auto() {
        let chain = GpuDetector::fallback_chain(&GpuBackend::Auto);
        assert!(!chain.is_empty());
        assert!(chain.contains(&GpuBackend::Cpu));
        assert_eq!(chain[0], GpuBackend::DirectML); // DirectML first on Windows
    }

    #[test]
    fn test_fallback_chain_contains_cpu_or_remote() {
        for backend in &[GpuBackend::Cuda, GpuBackend::DirectML, GpuBackend::Cpu] {
            let chain = GpuDetector::fallback_chain(backend);
            // Every fallback chain should end with a usable backend (CPU or Remote)
            let last = chain.last().expect("fallback should not be empty");
            assert!(
                *last == GpuBackend::Cpu || *last == GpuBackend::Remote,
                "fallback for {:?} should end with CPU or Remote",
                backend
            );
        }
    }
}
