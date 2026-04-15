use crate::models::task::{BatchCreateResult, ScanMode, Task, TaskStatus};

pub struct TaskRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> TaskRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, task: &Task) -> Result<(), rusqlite::Error> {
        let tlds_json = serde_json::to_string(&task.tlds).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO tasks (id, batch_id, name, signature, status, scan_mode, config_json, tlds, prefix_pattern, concurrency, proxy_id, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            rusqlite::params![
                task.id, task.batch_id, task.name, task.signature,
                serde_json::to_string(&task.status).unwrap(),
                serde_json::to_string(&task.scan_mode).unwrap(),
                task.config_json, tlds_json, task.prefix_pattern,
                task.concurrency, task.proxy_id,
                task.total_count, task.completed_count, task.completed_index,
                task.available_count, task.error_count,
                task.created_at, task.updated_at
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Task>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, batch_id, name, signature, status, scan_mode, config_json, tlds, prefix_pattern, concurrency, proxy_id, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at FROM tasks WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_task(row)?)),
            None => Ok(None),
        }
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

    pub fn update_progress(
        &self,
        id: &str,
        completed_count: i64,
        completed_index: i64,
        available_count: i64,
        error_count: i64,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tasks SET completed_count = ?1, completed_index = ?2, available_count = ?3, error_count = ?4, updated_at = CURRENT_TIMESTAMP WHERE id = ?5",
            rusqlite::params![completed_count, completed_index, available_count, error_count, id],
        )?;
        Ok(())
    }

    pub fn reset_for_rerun(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE tasks
             SET status = ?1,
                 completed_count = 0,
                 completed_index = 0,
                 available_count = 0,
                 error_count = 0,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = ?2",
            rusqlite::params![serde_json::to_string(&TaskStatus::Pending).unwrap(), id],
        )?;
        Ok(())
    }

    /// List tasks with optional status filter and pagination
    pub fn list(
        &self,
        status: Option<&TaskStatus>,
        batch_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Task>, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT id, batch_id, name, signature, status, scan_mode, config_json, tlds, prefix_pattern, concurrency, proxy_id, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at FROM tasks WHERE 1=1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            param_values.push(Box::new(serde_json::to_string(s).unwrap()));
        }
        if let Some(bid) = batch_id {
            sql.push_str(" AND batch_id = ?");
            param_values.push(Box::new(bid.to_string()));
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let tasks = stmt
            .query_map(params.as_slice(), |row| self.row_to_task(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tasks)
    }

    /// Count tasks with optional status filter
    pub fn count(&self, status: Option<&TaskStatus>) -> Result<i64, rusqlite::Error> {
        match status {
            Some(s) => {
                let status_str = serde_json::to_string(s).unwrap();
                self.conn.query_row(
                    "SELECT COUNT(*) FROM tasks WHERE status = ?1",
                    rusqlite::params![status_str],
                    |row| row.get(0),
                )
            }
            None => self
                .conn
                .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0)),
        }
    }

    /// Delete a task by ID
    pub fn delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Batch create tasks with signature dedup
    pub fn batch_create(&self, tasks: Vec<Task>) -> Result<BatchCreateResult, rusqlite::Error> {
        let mut created = 0u32;
        let mut skipped = 0u32;
        let mut task_ids = Vec::new();
        let mut skipped_signatures = Vec::new();

        for task in &tasks {
            if self.signature_exists(&task.signature)? {
                skipped += 1;
                skipped_signatures.push(task.signature.clone());
            } else {
                self.create(task)?;
                created += 1;
                task_ids.push(task.id.clone());
            }
        }

        Ok(BatchCreateResult {
            created,
            skipped,
            task_ids,
            skipped_signatures,
        })
    }

    fn row_to_task(&self, row: &rusqlite::Row) -> Result<Task, rusqlite::Error> {
        // Deserialize tlds from JSON array string
        let tlds_raw: String = row.get(7)?;
        let tlds: Vec<String> = serde_json::from_str(&tlds_raw).unwrap_or_else(|_| {
            // Backward compat: if it's a single TLD (not JSON), wrap in array
            if tlds_raw.starts_with('[') {
                vec![]
            } else {
                vec![tlds_raw]
            }
        });

        Ok(Task {
            id: row.get(0)?,
            batch_id: row.get(1)?,
            name: row.get(2)?,
            signature: row.get(3)?,
            status: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(TaskStatus::Pending),
            scan_mode: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or(ScanMode::Regex {
                pattern: String::new(),
            }),
            config_json: row.get(6)?,
            tlds,
            prefix_pattern: row.get(8)?,
            concurrency: row.get::<_, i64>(9).unwrap_or(50),
            proxy_id: row.get(10)?,
            total_count: row.get(11)?,
            completed_count: row.get(12)?,
            completed_index: row.get(13)?,
            available_count: row.get(14)?,
            error_count: row.get(15)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
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

    fn make_test_task(id: &str, sig: &str, tlds: Vec<&str>) -> Task {
        Task {
            id: id.to_string(),
            batch_id: None,
            name: "Test".to_string(),
            signature: sig.to_string(),
            status: TaskStatus::Pending,
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            config_json: "{}".to_string(),
            tlds: tlds.iter().map(|s| s.to_string()).collect(),
            prefix_pattern: None,
            concurrency: 50,
            proxy_id: None,
            total_count: 17576 * tlds.len() as i64,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: "2026-01-01T00:00:00".to_string(),
            updated_at: "2026-01-01T00:00:00".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_task_single_tld() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", vec![".com"]);
        repo.create(&task).unwrap();
        let fetched = repo.get_by_id("t1").unwrap().unwrap();
        assert_eq!(fetched.id, "t1");
        assert_eq!(fetched.tld_count(), 1);
        assert_eq!(fetched.primary_tld(), ".com");
        assert_eq!(fetched.tlds, vec![".com"]);
    }

    #[test]
    fn test_create_and_get_task_multi_tld() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", vec![".com", ".net", ".org"]);
        repo.create(&task).unwrap();
        let fetched = repo.get_by_id("t1").unwrap().unwrap();
        assert_eq!(fetched.id, "t1");
        assert_eq!(fetched.tld_count(), 3);
        assert_eq!(fetched.primary_tld(), ".com");
        assert_eq!(fetched.tlds, vec![".com", ".net", ".org"]);
    }

    #[test]
    fn test_get_nonexistent_task() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        assert!(repo.get_by_id("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_signature_uniqueness() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task1 = make_test_task("t1", "sig1", vec![".com"]);
        let task2 = make_test_task("t2", "sig1", vec![".net"]); // same sig, different tld
        repo.create(&task1).unwrap();
        let result = repo.create(&task2);
        assert!(result.is_err()); // UNIQUE constraint on signature
    }

    #[test]
    fn test_signature_exists_and_not_exists() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        assert!(!repo.signature_exists("sig1").unwrap());
        let task = make_test_task("t1", "sig1", vec![".com"]);
        repo.create(&task).unwrap();
        assert!(repo.signature_exists("sig1").unwrap());
    }

    #[test]
    fn test_update_status() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", vec![".com"]);
        repo.create(&task).unwrap();
        repo.update_status("t1", &TaskStatus::Running).unwrap();
        let fetched = repo.get_by_id("t1").unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Running);
    }

    #[test]
    fn test_update_progress() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let task = make_test_task("t1", "sig1", vec![".com"]);
        repo.create(&task).unwrap();
        repo.update_progress("t1", 100, 100, 30, 2).unwrap();
        let fetched = repo.get_by_id("t1").unwrap().unwrap();
        assert_eq!(fetched.completed_count, 100);
        assert_eq!(fetched.completed_index, 100);
        assert_eq!(fetched.available_count, 30);
        assert_eq!(fetched.error_count, 2);
    }

    #[test]
    fn test_reset_for_rerun() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let mut task = make_test_task("t1", "sig1", vec![".com"]);
        task.status = TaskStatus::Completed;
        task.completed_count = 100;
        task.completed_index = 100;
        task.available_count = 25;
        task.error_count = 3;
        repo.create(&task).unwrap();
        repo.reset_for_rerun("t1").unwrap();
        let fetched = repo.get_by_id("t1").unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Pending);
        assert_eq!(fetched.completed_count, 0);
        assert_eq!(fetched.completed_index, 0);
        assert_eq!(fetched.available_count, 0);
        assert_eq!(fetched.error_count, 0);
    }

    #[test]
    fn test_list_all_tasks() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        repo.create(&make_test_task("t1", "sig1", vec![".com"]))
            .unwrap();
        repo.create(&make_test_task("t2", "sig2", vec![".net"]))
            .unwrap();
        let tasks = repo.list(None, None, 100, 0).unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_list_tasks_by_status() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        repo.create(&make_test_task("t1", "sig1", vec![".com"]))
            .unwrap();
        let mut task2 = make_test_task("t2", "sig2", vec![".net"]);
        task2.status = TaskStatus::Running;
        repo.create(&task2).unwrap();
        let running = repo.list(Some(&TaskStatus::Running), None, 100, 0).unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].id, "t2");
    }

    #[test]
    fn test_list_tasks_by_batch() {
        let (conn, _temp) = setup();
        conn.execute(
            "INSERT INTO task_batches (id, name, task_count) VALUES ('batch1', 'Test Batch', 1)",
            [],
        )
        .unwrap();
        let repo = TaskRepo::new(&conn);
        let mut t1 = make_test_task("t1", "sig1", vec![".com"]);
        t1.batch_id = Some("batch1".to_string());
        repo.create(&t1).unwrap();
        repo.create(&make_test_task("t2", "sig2", vec![".net"]))
            .unwrap();
        let batch_tasks = repo.list(None, Some("batch1"), 100, 0).unwrap();
        assert_eq!(batch_tasks.len(), 1);
        assert_eq!(batch_tasks[0].batch_id, Some("batch1".to_string()));
    }

    #[test]
    fn test_list_tasks_pagination() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        for i in 0..5 {
            repo.create(&make_test_task(
                &format!("t{i}"),
                &format!("sig{i}"),
                vec![".com"],
            ))
            .unwrap();
        }
        let page1 = repo.list(None, None, 2, 0).unwrap();
        let page2 = repo.list(None, None, 2, 2).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_count_tasks() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        assert_eq!(repo.count(None).unwrap(), 0);
        repo.create(&make_test_task("t1", "sig1", vec![".com"]))
            .unwrap();
        repo.create(&make_test_task("t2", "sig2", vec![".net"]))
            .unwrap();
        assert_eq!(repo.count(None).unwrap(), 2);
        assert_eq!(repo.count(Some(&TaskStatus::Pending)).unwrap(), 2);
    }

    #[test]
    fn test_delete_task() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        repo.create(&make_test_task("t1", "sig1", vec![".com"]))
            .unwrap();
        repo.delete("t1").unwrap();
        assert!(repo.get_by_id("t1").unwrap().is_none());
    }

    #[test]
    fn test_batch_create_with_dedup() {
        let (conn, _temp) = setup();
        let repo = TaskRepo::new(&conn);
        let tasks = vec![
            make_test_task("t1", "sig1", vec![".com"]),
            make_test_task("t2", "sig2", vec![".net"]),
        ];
        let result = repo.batch_create(tasks).unwrap();
        assert_eq!(result.created, 2);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.task_ids.len(), 2);

        // Try creating again with same signatures
        let tasks2 = vec![
            make_test_task("t3", "sig1", vec![".com"]),
            make_test_task("t4", "sig3", vec![".org"]),
        ];
        let result2 = repo.batch_create(tasks2).unwrap();
        assert_eq!(result2.created, 1);
        assert_eq!(result2.skipped, 1);
        assert!(result2.skipped_signatures.contains(&"sig1".to_string()));
    }
}
