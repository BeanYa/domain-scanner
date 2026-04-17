#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorizeRun {
    pub id: String,
    pub task_id: String,
    pub status: String,
    pub backend: String,
    pub total_count: i64,
    pub processed_count: i64,
    pub skipped_existing: i64,
    pub batch_size: i64,
    pub embedding_dim: i64,
    pub error_message: Option<String>,
    pub started_at: String,
    pub updated_at: String,
    pub finished_at: Option<String>,
}

pub struct VectorizeRunRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> VectorizeRunRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create_running(
        &self,
        task_id: &str,
        backend: &str,
        total_count: i64,
        processed_count: i64,
        skipped_existing: i64,
        batch_size: i64,
        embedding_dim: i64,
    ) -> Result<VectorizeRun, rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        let run = VectorizeRun {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            status: "running".to_string(),
            backend: backend.to_string(),
            total_count,
            processed_count,
            skipped_existing,
            batch_size,
            embedding_dim,
            error_message: None,
            started_at: now.clone(),
            updated_at: now,
            finished_at: None,
        };

        self.conn.execute(
            "INSERT INTO vectorize_runs
             (id, task_id, status, backend, total_count, processed_count, skipped_existing, batch_size, embedding_dim, error_message, started_at, updated_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                run.id,
                run.task_id,
                run.status,
                run.backend,
                run.total_count,
                run.processed_count,
                run.skipped_existing,
                run.batch_size,
                run.embedding_dim,
                run.error_message,
                run.started_at,
                run.updated_at,
                run.finished_at,
            ],
        )?;

        Ok(run)
    }

    pub fn update_progress(&self, id: &str, processed_count: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE vectorize_runs
             SET processed_count = ?1, updated_at = ?2
             WHERE id = ?3",
            rusqlite::params![processed_count, chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn finish(
        &self,
        id: &str,
        status: &str,
        processed_count: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE vectorize_runs
             SET status = ?1,
                 processed_count = COALESCE(?2, processed_count),
                 error_message = ?3,
                 updated_at = ?4,
                 finished_at = ?4
             WHERE id = ?5",
            rusqlite::params![status, processed_count, error_message, now, id],
        )?;
        Ok(())
    }

    pub fn get_latest_by_task(
        &self,
        task_id: &str,
    ) -> Result<Option<VectorizeRun>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, status, backend, total_count, processed_count, skipped_existing, batch_size, embedding_dim, error_message, started_at, updated_at, finished_at
             FROM vectorize_runs
             WHERE task_id = ?1
             ORDER BY updated_at DESC, started_at DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query([task_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_run(row)?)),
            None => Ok(None),
        }
    }

    pub fn mark_running_interrupted(&self) -> Result<usize, rusqlite::Error> {
        self.conn.execute(
            "UPDATE vectorize_runs
             SET status = 'interrupted',
                 error_message = COALESCE(error_message, '应用重启或进程退出，运行状态已中断。'),
                 updated_at = ?1,
                 finished_at = COALESCE(finished_at, ?1)
             WHERE status = 'running'",
            [chrono::Utc::now().to_rfc3339()],
        )
    }
}

fn row_to_run(row: &rusqlite::Row) -> Result<VectorizeRun, rusqlite::Error> {
    Ok(VectorizeRun {
        id: row.get(0)?,
        task_id: row.get(1)?,
        status: row.get(2)?,
        backend: row.get(3)?,
        total_count: row.get(4)?,
        processed_count: row.get(5)?,
        skipped_existing: row.get(6)?,
        batch_size: row.get(7)?,
        embedding_dim: row.get(8)?,
        error_message: row.get(9)?,
        started_at: row.get(10)?,
        updated_at: row.get(11)?,
        finished_at: row.get(12)?,
    })
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
        create_test_task(&conn);
        (conn, temp)
    }

    fn create_test_task(conn: &rusqlite::Connection) {
        conn.execute(
            "INSERT INTO tasks (id, name, signature, status, scan_mode, config_json, tlds, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at)
             VALUES ('task1', 'Test', 'sig1', '\"completed\"', '{}', '{}', '[\".com\"]', 10, 10, 10, 10, 0, '2026-01-01T00:00:00', '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn test_create_and_finish_run() {
        let (conn, _temp) = setup();
        let repo = VectorizeRunRepo::new(&conn);
        let run = repo
            .create_running("task1", "remote", 10, 2, 2, 5, 384)
            .unwrap();
        repo.update_progress(&run.id, 7).unwrap();
        repo.finish(&run.id, "completed", Some(10), None).unwrap();

        let latest = repo.get_latest_by_task("task1").unwrap().unwrap();
        assert_eq!(latest.status, "completed");
        assert_eq!(latest.processed_count, 10);
        assert!(latest.finished_at.is_some());
    }

    #[test]
    fn test_mark_running_interrupted() {
        let (conn, _temp) = setup();
        let repo = VectorizeRunRepo::new(&conn);
        repo.create_running("task1", "remote", 10, 0, 0, 5, 384)
            .unwrap();
        let count = repo.mark_running_interrupted().unwrap();
        assert_eq!(count, 1);
        let latest = repo.get_latest_by_task("task1").unwrap().unwrap();
        assert_eq!(latest.status, "interrupted");
    }
}
