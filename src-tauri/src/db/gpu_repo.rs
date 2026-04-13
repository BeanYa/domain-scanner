use crate::models::gpu::{GpuBackend, GpuConfig};

pub struct GpuRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> GpuRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn get_config(&self) -> Result<GpuConfig, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, backend, device_id, batch_size, model_path FROM gpu_configs WHERE id = 1"
        )?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok(GpuConfig {
                id: row.get(0)?,
                backend: serde_json::from_str(&format!("\"{}\"", row.get::<_, String>(1)?))
                    .unwrap_or(GpuBackend::Auto),
                device_id: row.get(2)?,
                batch_size: row.get(3)?,
                model_path: row.get(4)?,
            }),
            None => Ok(GpuConfig {
                id: 1,
                backend: GpuBackend::Auto,
                device_id: 0,
                batch_size: 500,
                model_path: None,
            }),
        }
    }

    pub fn update_config(&self, config: &GpuConfig) -> Result<(), rusqlite::Error> {
        let backend_str = serde_json::to_string(&config.backend)
            .unwrap_or_else(|_| "\"auto\"".to_string())
            .trim_matches('"')
            .to_string();
        self.conn.execute(
            "UPDATE gpu_configs SET backend = ?1, device_id = ?2, batch_size = ?3, model_path = ?4 WHERE id = 1",
            rusqlite::params![backend_str, config.device_id, config.batch_size, config.model_path],
        )?;
        Ok(())
    }

    pub fn set_backend(&self, backend: &GpuBackend) -> Result<(), rusqlite::Error> {
        let backend_str = serde_json::to_string(backend)
            .unwrap_or_else(|_| "\"auto\"".to_string())
            .trim_matches('"')
            .to_string();
        self.conn.execute(
            "UPDATE gpu_configs SET backend = ?1 WHERE id = 1",
            [backend_str],
        )?;
        Ok(())
    }

    pub fn set_batch_size(&self, batch_size: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE gpu_configs SET batch_size = ?1 WHERE id = 1",
            [batch_size],
        )?;
        Ok(())
    }

    pub fn set_model_path(&self, path: Option<&str>) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE gpu_configs SET model_path = ?1 WHERE id = 1",
            [path],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init;
    use tempfile::NamedTempFile;

    fn setup() -> (rusqlite::Connection, NamedTempFile) {
        crate::db::init::register_vec_extension();
        let temp = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(temp.path()).unwrap();
        init::init_database(&conn).unwrap();
        (conn, temp)
    }

    #[test]
    fn test_get_default_config() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        let config = repo.get_config().unwrap();
        assert_eq!(config.backend, GpuBackend::Auto);
        assert_eq!(config.device_id, 0);
        assert_eq!(config.batch_size, 500);
        assert!(config.model_path.is_none());
    }

    #[test]
    fn test_update_config() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        let mut config = repo.get_config().unwrap();
        config.backend = GpuBackend::Cuda;
        config.device_id = 1;
        config.batch_size = 256;
        config.model_path = Some("/models/model.onnx".to_string());
        repo.update_config(&config).unwrap();
        let updated = repo.get_config().unwrap();
        assert_eq!(updated.backend, GpuBackend::Cuda);
        assert_eq!(updated.device_id, 1);
        assert_eq!(updated.batch_size, 256);
        assert_eq!(updated.model_path, Some("/models/model.onnx".to_string()));
    }

    #[test]
    fn test_set_backend() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        repo.set_backend(&GpuBackend::DirectML).unwrap();
        let config = repo.get_config().unwrap();
        assert_eq!(config.backend, GpuBackend::DirectML);
    }

    #[test]
    fn test_set_batch_size() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        repo.set_batch_size(1024).unwrap();
        let config = repo.get_config().unwrap();
        assert_eq!(config.batch_size, 1024);
    }

    #[test]
    fn test_set_model_path() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        repo.set_model_path(Some("/custom/model.onnx")).unwrap();
        let config = repo.get_config().unwrap();
        assert_eq!(config.model_path, Some("/custom/model.onnx".to_string()));
        repo.set_model_path(None).unwrap();
        let config = repo.get_config().unwrap();
        assert!(config.model_path.is_none());
    }

    #[test]
    fn test_all_backend_types() {
        let (conn, _temp) = setup();
        let repo = GpuRepo::new(&conn);
        let backends = [
            GpuBackend::Auto, GpuBackend::Cuda, GpuBackend::DirectML,
            GpuBackend::ROCm, GpuBackend::CoreML, GpuBackend::Cpu, GpuBackend::Remote,
        ];
        for backend in &backends {
            repo.set_backend(backend).unwrap();
            let config = repo.get_config().unwrap();
            assert_eq!(&config.backend, backend);
        }
    }
}
