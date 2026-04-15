#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskLog {
    pub id: i64,
    pub task_id: String,
    pub run_id: Option<String>,
    pub level: String,
    pub message: String,
    pub created_at: String,
}

pub struct LogRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> LogRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Insert a single log entry
    pub fn create(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: &str,
        message: &str,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO task_logs (task_id, run_id, level, message) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, run_id, level, message],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Batch insert log entries
    pub fn batch_insert(
        &self,
        logs: &[(&str, Option<&str>, &str, &str)],
    ) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for (task_id, run_id, level, message) in logs {
            tx.execute(
                "INSERT INTO task_logs (task_id, run_id, level, message) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![task_id, run_id, level, message],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    /// List logs for a task with pagination and optional level filter
    pub fn list_by_task(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<TaskLog>, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT id, task_id, run_id, level, message, created_at FROM task_logs WHERE task_id = ?1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(task_id.to_string())];

        if let Some(rid) = run_id {
            sql.push_str(" AND run_id = ?");
            param_values.push(Box::new(rid.to_string()));
        }
        if let Some(lvl) = level {
            sql.push_str(" AND level = ?");
            param_values.push(Box::new(lvl.to_string()));
        }
        sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let logs = stmt
            .query_map(params.as_slice(), |row| {
                Ok(TaskLog {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    run_id: row.get(2)?,
                    level: row.get(3)?,
                    message: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(logs)
    }

    /// Count logs for a task
    pub fn count_by_task(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        let mut sql = String::from("SELECT COUNT(*) FROM task_logs WHERE task_id = ?1");
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(task_id.to_string())];

        if let Some(rid) = run_id {
            sql.push_str(" AND run_id = ?");
            params.push(Box::new(rid.to_string()));
        }
        if let Some(lvl) = level {
            sql.push_str(" AND level = ?");
            params.push(Box::new(lvl.to_string()));
        }

        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.conn.query_row(&sql, refs.as_slice(), |row| row.get(0))
    }

    /// Delete all logs for a task
    pub fn delete_by_task(&self, task_id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM task_logs WHERE task_id = ?1", [task_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init;
    use crate::models::task::{ScanMode, Task, TaskStatus};
    use tempfile::NamedTempFile;

    fn setup() -> (rusqlite::Connection, NamedTempFile) {
        crate::db::init::register_vec_extension();
        let temp = NamedTempFile::new().unwrap();
        let conn = rusqlite::Connection::open(temp.path()).unwrap();
        init::init_database(&conn).unwrap();
        (conn, temp)
    }

    fn create_test_task(conn: &rusqlite::Connection) {
        let task = Task {
            id: "task1".to_string(),
            batch_id: None,
            name: "Test".to_string(),
            signature: "sig1".to_string(),
            status: TaskStatus::Pending,
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            config_json: "{}".to_string(),
            tlds: vec![".com".to_string()],
            prefix_pattern: None,
            concurrency: 50,
            proxy_id: None,
            total_count: 100,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: "2026-01-01T00:00:00".to_string(),
            updated_at: "2026-01-01T00:00:00".to_string(),
        };
        conn.execute(
            "INSERT INTO tasks (id, name, signature, status, scan_mode, config_json, tlds, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            rusqlite::params![
                task.id, task.name, task.signature,
                serde_json::to_string(&task.status).unwrap(),
                serde_json::to_string(&task.scan_mode).unwrap(),
                task.config_json, serde_json::to_string(&task.tlds).unwrap(), task.total_count,
                task.completed_count, task.completed_index,
                task.available_count, task.error_count,
                task.created_at, task.updated_at
            ],
        ).unwrap();
    }

    #[test]
    fn test_create_log() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        let id = repo
            .create("task1", Some("run1"), "info", "Scan started")
            .unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_batch_insert_logs() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        let logs = vec![
            ("task1", Some("run1"), "info", "Started"),
            ("task1", Some("run1"), "warn", "Rate limited"),
            ("task1", Some("run1"), "error", "Connection failed"),
        ];
        let count = repo.batch_insert(&logs).unwrap();
        assert_eq!(count, 3);
        assert_eq!(repo.count_by_task("task1", Some("run1"), None).unwrap(), 3);
    }

    #[test]
    fn test_list_logs_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        repo.batch_insert(&[
            ("task1", Some("run1"), "info", "Msg 1"),
            ("task1", Some("run1"), "warn", "Msg 2"),
            ("task1", Some("run1"), "error", "Msg 3"),
        ])
        .unwrap();
        let logs = repo
            .list_by_task("task1", Some("run1"), None, 100, 0)
            .unwrap();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_list_logs_with_level_filter() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        repo.batch_insert(&[
            ("task1", Some("run1"), "info", "Msg 1"),
            ("task1", Some("run1"), "warn", "Msg 2"),
            ("task1", Some("run1"), "error", "Msg 3"),
        ])
        .unwrap();
        let errors = repo
            .list_by_task("task1", Some("run1"), Some("error"), 100, 0)
            .unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].level, "error");
    }

    #[test]
    fn test_list_logs_pagination() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        for i in 0..10 {
            repo.create("task1", Some("run1"), "info", &format!("Msg {}", i))
                .unwrap();
        }
        let page1 = repo
            .list_by_task("task1", Some("run1"), None, 3, 0)
            .unwrap();
        let page2 = repo
            .list_by_task("task1", Some("run1"), None, 3, 3)
            .unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page2.len(), 3);
    }

    #[test]
    fn test_count_logs_with_level() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        repo.batch_insert(&[
            ("task1", Some("run1"), "info", "Msg 1"),
            ("task1", Some("run1"), "info", "Msg 2"),
            ("task1", Some("run1"), "error", "Msg 3"),
        ])
        .unwrap();
        assert_eq!(repo.count_by_task("task1", Some("run1"), None).unwrap(), 3);
        assert_eq!(
            repo.count_by_task("task1", Some("run1"), Some("info"))
                .unwrap(),
            2
        );
        assert_eq!(
            repo.count_by_task("task1", Some("run1"), Some("error"))
                .unwrap(),
            1
        );
    }

    #[test]
    fn test_delete_logs_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = LogRepo::new(&conn);
        repo.batch_insert(&[
            ("task1", Some("run1"), "info", "Msg 1"),
            ("task1", Some("run1"), "error", "Msg 2"),
        ])
        .unwrap();
        repo.delete_by_task("task1").unwrap();
        assert_eq!(repo.count_by_task("task1", Some("run1"), None).unwrap(), 0);
    }
}
