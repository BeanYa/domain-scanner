/// Local ONNX model for embedding generation
/// Uses ort (ONNX Runtime) for inference with optional GPU acceleration
use crate::models::gpu::GpuBackend;

const EMBEDDING_DIM: usize = 384;

/// Local embedding model using ONNX Runtime
pub struct LocalEmbeddingModel {
    model_path: String,
    backend: GpuBackend,
    batch_size: usize,
}

/// Result of embedding generation
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    pub embeddings: Vec<Vec<f32>>,
    pub dim: usize,
    pub backend_used: GpuBackend,
}

impl LocalEmbeddingModel {
    pub fn new(model_path: String, backend: GpuBackend, batch_size: usize) -> Self {
        Self {
            model_path,
            backend,
            batch_size,
        }
    }

    /// Get the embedding dimension
    pub fn dim(&self) -> usize {
        EMBEDDING_DIM
    }

    /// Check if the model file exists
    pub fn model_exists(&self) -> bool {
        std::path::Path::new(&self.model_path).exists()
    }

    /// Generate embeddings for a batch of texts
    /// In real implementation, this would load the ONNX model and run inference
    pub fn embed(&self, texts: &[String]) -> Result<EmbeddingResult, String> {
        if !self.model_exists() {
            return Err(format!("Model file not found: {}", self.model_path));
        }

        // In real implementation:
        // 1. Load ONNX model with ort::Session
        // 2. Tokenize input texts
        // 3. Run inference
        // 4. Extract embeddings from output tensor
        // 5. Normalize embeddings

        // Placeholder: generate deterministic pseudo-embeddings for testing
        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let seed = Self::text_to_seed(text, i);
                Self::generate_pseudo_embedding(seed)
            })
            .collect();

        Ok(EmbeddingResult {
            embeddings,
            dim: EMBEDDING_DIM,
            backend_used: self.backend.clone(),
        })
    }

    /// Generate embeddings for a large batch, splitting into smaller chunks
    pub fn embed_batch(&self, texts: &[String]) -> Result<EmbeddingResult, String> {
        let mut all_embeddings = Vec::new();
        for chunk in texts.chunks(self.batch_size) {
            let result = self.embed(chunk)?;
            all_embeddings.extend(result.embeddings);
        }
        Ok(EmbeddingResult {
            embeddings: all_embeddings,
            dim: EMBEDDING_DIM,
            backend_used: self.backend.clone(),
        })
    }

    /// Convert text to a seed for deterministic pseudo-embedding generation
    fn text_to_seed(text: &str, index: usize) -> u64 {
        let mut hash: u64 = 5381;
        for byte in text.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash.wrapping_add(index as u64)
    }

    /// Generate a deterministic pseudo-embedding from a seed
    fn generate_pseudo_embedding(seed: u64) -> Vec<f32> {
        let mut rng_state = seed;
        let mut embedding = Vec::with_capacity(EMBEDDING_DIM);
        for _ in 0..EMBEDDING_DIM {
            // Simple LCG pseudo-random number generator
            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let val = ((rng_state >> 33) as i32) as f32 / i32::MAX as f32;
            embedding.push(val);
        }

        // Normalize to unit length
        let norm: f32 = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in embedding.iter_mut() {
                *v /= norm;
            }
        }

        embedding
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_model() -> LocalEmbeddingModel {
        LocalEmbeddingModel::new("/nonexistent/model.onnx".to_string(), GpuBackend::Cpu, 500)
    }

    fn make_model_with_path(path: &str) -> LocalEmbeddingModel {
        LocalEmbeddingModel::new(path.to_string(), GpuBackend::Cpu, 10)
    }

    #[test]
    fn test_model_creation() {
        let model = make_test_model();
        assert_eq!(model.dim(), EMBEDDING_DIM);
        assert_eq!(model.backend, GpuBackend::Cpu);
    }

    #[test]
    fn test_model_not_exists() {
        let model = make_test_model();
        assert!(!model.model_exists());
    }

    #[test]
    fn test_embed_fails_without_model() {
        let model = make_test_model();
        let result = model.embed(&["test".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_embed_with_temp_model() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.onnx");
        std::fs::write(&model_path, b"fake model data").unwrap();

        let model = make_model_with_path(model_path.to_str().unwrap());
        assert!(model.model_exists());

        let result = model
            .embed(&["hello".to_string(), "world".to_string()])
            .unwrap();
        assert_eq!(result.embeddings.len(), 2);
        assert_eq!(result.dim, EMBEDDING_DIM);
        assert_eq!(result.embeddings[0].len(), EMBEDDING_DIM);
    }

    #[test]
    fn test_embedding_normalization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.onnx");
        std::fs::write(&model_path, b"fake").unwrap();

        let model = make_model_with_path(model_path.to_str().unwrap());
        let result = model.embed(&["test text".to_string()]).unwrap();

        // Check unit length normalization
        let norm: f32 = result.embeddings[0]
            .iter()
            .map(|v| v * v)
            .sum::<f32>()
            .sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Embedding should be normalized, got norm = {}",
            norm
        );
    }

    #[test]
    fn test_embedding_deterministic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.onnx");
        std::fs::write(&model_path, b"fake").unwrap();

        let model = make_model_with_path(model_path.to_str().unwrap());
        let r1 = model.embed(&["test".to_string()]).unwrap();
        let r2 = model.embed(&["test".to_string()]).unwrap();
        assert_eq!(r1.embeddings[0], r2.embeddings[0]);
    }

    #[test]
    fn test_embedding_different_texts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.onnx");
        std::fs::write(&model_path, b"fake").unwrap();

        let model = make_model_with_path(model_path.to_str().unwrap());
        let result = model
            .embed(&["hello".to_string(), "world".to_string()])
            .unwrap();
        assert_ne!(result.embeddings[0], result.embeddings[1]);
    }

    #[test]
    fn test_embed_batch_splitting() {
        let temp_dir = tempfile::tempdir().unwrap();
        let model_path = temp_dir.path().join("model.onnx");
        std::fs::write(&model_path, b"fake").unwrap();

        let model = LocalEmbeddingModel::new(
            model_path.to_str().unwrap().to_string(),
            GpuBackend::Cpu,
            3, // batch size of 3
        );

        let texts: Vec<String> = (0..10).map(|i| format!("text{}", i)).collect();
        let result = model.embed_batch(&texts).unwrap();
        assert_eq!(result.embeddings.len(), 10);
    }
}
