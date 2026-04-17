use crate::models::cluster_worker::{
    ClusterWorker, ClusterWorkerStatus, ClusterWorkerType, WorkerCapabilities,
};

pub struct ClusterWorkerRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> ClusterWorkerRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_local_worker(&self) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO cluster_workers (
                id, name, base_url, worker_type, status, version,
                max_running_batches, max_total_concurrency, max_batch_concurrency,
                current_running_batches, current_concurrency, enabled, created_at, updated_at
             ) VALUES ('local', '本地内置 Worker', NULL, 'local', 'available', NULL, 1, 500, 500, 0, 0, 1, ?1, ?1)
             ON CONFLICT(id) DO UPDATE SET
                worker_type = 'local',
                status = CASE WHEN enabled = 0 THEN 'disabled' ELSE 'available' END,
                max_running_batches = COALESCE(max_running_batches, 1),
                max_total_concurrency = COALESCE(max_total_concurrency, 500),
                max_batch_concurrency = COALESCE(max_batch_concurrency, 500),
                updated_at = ?1",
            [now],
        )?;
        Ok(())
    }

    pub fn create_pending(&self, worker: &ClusterWorker) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO cluster_workers (
                id, name, base_url, worker_type, status, registration_token_hash,
                auth_token_ref, version, max_running_batches, max_total_concurrency,
                max_batch_concurrency, current_running_batches, current_concurrency,
                install_command, expires_at, last_heartbeat_at, last_checked_at,
                last_error, enabled, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
            rusqlite::params![
                worker.id,
                worker.name,
                worker.base_url,
                worker.worker_type.as_str(),
                worker.status.as_str(),
                worker.registration_token_hash,
                worker.auth_token_ref,
                worker.version,
                worker.max_running_batches,
                worker.max_total_concurrency,
                worker.max_batch_concurrency,
                worker.current_running_batches,
                worker.current_concurrency,
                worker.install_command,
                worker.expires_at,
                worker.last_heartbeat_at,
                worker.last_checked_at,
                worker.last_error,
                worker.enabled as i64,
                worker.created_at,
                worker.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<ClusterWorker>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_url, worker_type, status, registration_token_hash,
                    auth_token_ref, version, max_running_batches, max_total_concurrency,
                    max_batch_concurrency, current_running_batches, current_concurrency,
                    install_command, expires_at, last_heartbeat_at, last_checked_at,
                    last_error, enabled, created_at, updated_at
             FROM cluster_workers WHERE id = ?1",
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::row_to_worker(row)?)),
            None => Ok(None),
        }
    }

    pub fn list(&self) -> Result<Vec<ClusterWorker>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_url, worker_type, status, registration_token_hash,
                    auth_token_ref, version, max_running_batches, max_total_concurrency,
                    max_batch_concurrency, current_running_batches, current_concurrency,
                    install_command, expires_at, last_heartbeat_at, last_checked_at,
                    last_error, enabled, created_at, updated_at
             FROM cluster_workers
             ORDER BY CASE worker_type WHEN 'local' THEN 0 ELSE 1 END, created_at DESC",
        )?;
        let workers = stmt
            .query_map([], |row| Self::row_to_worker(row))?
            .filter_map(|row| row.ok())
            .collect();
        Ok(workers)
    }

    pub fn update_health(
        &self,
        id: &str,
        status: &ClusterWorkerStatus,
        checked_at: &str,
        heartbeat_at: Option<&str>,
        last_error: Option<&str>,
        version: Option<&str>,
        capabilities: Option<&WorkerCapabilities>,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE cluster_workers
             SET status = ?1,
                 last_checked_at = ?2,
                 last_heartbeat_at = COALESCE(?3, last_heartbeat_at),
                 last_error = ?4,
                 version = COALESCE(?5, version),
                 max_running_batches = COALESCE(?6, max_running_batches),
                 max_total_concurrency = COALESCE(?7, max_total_concurrency),
                 max_batch_concurrency = COALESCE(?8, max_batch_concurrency),
                 updated_at = ?2
             WHERE id = ?9",
            rusqlite::params![
                status.as_str(),
                checked_at,
                heartbeat_at,
                last_error,
                version,
                capabilities.map(|c| c.max_running_batches),
                capabilities.map(|c| c.max_total_concurrency),
                capabilities.map(|c| c.max_batch_concurrency),
                id,
            ],
        )?;
        Ok(())
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        let status = if enabled { "unavailable" } else { "disabled" };
        self.conn.execute(
            "UPDATE cluster_workers SET enabled = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![enabled as i64, status, now, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM cluster_workers WHERE id = ?1 AND worker_type <> 'local'",
            [id],
        )?;
        Ok(())
    }

    fn row_to_worker(row: &rusqlite::Row) -> Result<ClusterWorker, rusqlite::Error> {
        let worker_type: String = row.get(3)?;
        let status: String = row.get(4)?;
        Ok(ClusterWorker {
            id: row.get(0)?,
            name: row.get(1)?,
            base_url: row.get(2)?,
            worker_type: ClusterWorkerType::from_db(&worker_type),
            status: ClusterWorkerStatus::from_db(&status),
            registration_token_hash: row.get(5)?,
            auth_token_ref: row.get(6)?,
            version: row.get(7)?,
            max_running_batches: row.get(8)?,
            max_total_concurrency: row.get(9)?,
            max_batch_concurrency: row.get(10)?,
            current_running_batches: row.get(11)?,
            current_concurrency: row.get(12)?,
            install_command: row.get(13)?,
            expires_at: row.get(14)?,
            last_heartbeat_at: row.get(15)?,
            last_checked_at: row.get(16)?,
            last_error: row.get(17)?,
            enabled: row.get::<_, i64>(18).unwrap_or(1) != 0,
            created_at: row.get(19)?,
            updated_at: row.get(20)?,
        })
    }
}
