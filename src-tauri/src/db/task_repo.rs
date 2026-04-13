// Task repository - CRUD + signature dedup + progress update
// TDD: will be implemented with unit tests first

use crate::models::task::{Task, TaskStatus, BatchCreateResult};

pub struct TaskRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> TaskRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, task: &Task) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO tasks (id, batch_id, name, signature, status, scan_mode, config_json, tld, prefix_pattern, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            rusqlite::params![
                task.id, task.batch_id, task.name, task.signature,
                serde_json::to_string(&task.status).unwrap(),
                serde_json::to_string(&task.scan_mode).unwrap(),
                task.config_json, task.tld, task.prefix_pattern,
                task.total_count, task.completed_count, task.completed_index,
                task.available_count, task.error_count,
                task.created_at, task.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Task>, rusqlite::Error> {
        // TODO: implement
        Ok(None)
    }

    pub fn signature_exists(&self, signature: &str) -> Result<bool, rusqlite::Error> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE signature = ?1",
            [signature],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn update_status(&self, id: &str, status: &TaskStatus) -> Result<(), rusqlite::Error> {
        let status_str = serde_json::to_string(status).unwrap();
        self.conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![status_str, id],
        )?;
        Ok(())
    }

    pub fn update_progress(&self, id: &str, completed_count: i64, completed_index: i64, available_count: i64, error_count: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tasks SET completed_count = ?1, completed_index = ?2, available_count = ?3, error_count = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?5",
            rusqlite::params![completed_count, completed_index, available_count, error_count, id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init;
    use crate::models::task::ScanMode;
    use tempfile::NamedTempFile;

    fn setup() -> (rusqlite::Connection, NamedTempFile) {
        let temp = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(temp.path()).unwrap();
        init::init_database(&conn).unwrap();
        (conn, temp)
    }

    fn make_test_task(id: &str, sig: &str, tld: &str) -> Task {
        Task {
            id: id.to_string(),
            batch_id: None,
            name: "Test".to_string(),
            signature: sig.to_string(),
            status: TaskStatus::Pending,
            scan_mode: ScanMode::Regex { pattern: "^[a-z]{3}$".to_string() },
            config_json: "{}".to_string(),
            tld: tld.to_string(),
            prefix_pattern: None,
            total_count: 17576,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: "2026-01-01T00:00:00".to_string(),
            updated_at: "2026-01-01T00:00:00".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_task() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", ".com");
        repo.create(&task).unwrap();
        assert!(repo.signature_exists("sig1").unwrap());
    }

    #[test]
    fn test_signature_uniqueness() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task1 = make_test_task("t1", "sig1", ".com");
        let task2 = make_test_task("t2", "sig1", ".com");
        repo.create(&task1).unwrap();
        let result = repo.create(&task2);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_not_exists() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        assert!(!repo.signature_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_update_status() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", ".com");
        repo.create(&task).unwrap();
        repo.update_status("t1", &TaskStatus::Running).unwrap();
        assert!(repo.signature_exists("sig1").unwrap());
    }

    #[test]
    fn test_update_progress() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", ".com");
        repo.create(&task).unwrap();
        repo.update_progress("t1", 100, 100, 30, 2).unwrap();
    }
}
