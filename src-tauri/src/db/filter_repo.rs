#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilteredResult {
    pub id: i64,
    pub task_id: String,
    pub domain: String,
    pub filter_type: String,
    pub filter_pattern: Option<String>,
    pub is_matched: bool,
    pub score: Option<f64>,
    pub embedding_id: Option<i64>,
}

pub struct FilterRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> FilterRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, result: &FilteredResult) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO filtered_results (task_id, domain, filter_type, filter_pattern, is_matched, score, embedding_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                result.task_id, result.domain, result.filter_type,
                result.filter_pattern, result.is_matched as i32,
                result.score, result.embedding_id
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn batch_insert(&self, results: &[FilteredResult]) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for result in results {
            tx.execute(
                "INSERT INTO filtered_results (task_id, domain, filter_type, filter_pattern, is_matched, score, embedding_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    result.task_id, result.domain, result.filter_type,
                    result.filter_pattern, result.is_matched as i32,
                    result.score, result.embedding_id
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn list_by_task(
        &self,
        task_id: &str,
        filter_type: Option<&str>,
        matched_only: bool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<FilteredResult>, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT id, task_id, domain, filter_type, filter_pattern, is_matched, score, embedding_id FROM filtered_results WHERE task_id = ?1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(task_id.to_string())];

        if let Some(ft) = filter_type {
            sql.push_str(" AND filter_type = ?");
            param_values.push(Box::new(ft.to_string()));
        }
        if matched_only {
            sql.push_str(" AND is_matched = 1");
        }
        sql.push_str(" ORDER BY id DESC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let results = stmt
            .query_map(params.as_slice(), |row| {
                let is_matched: i32 = row.get(5)?;
                Ok(FilteredResult {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    domain: row.get(2)?,
                    filter_type: row.get(3)?,
                    filter_pattern: row.get(4)?,
                    is_matched: is_matched != 0,
                    score: row.get(6)?,
                    embedding_id: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    pub fn count_by_task(&self, task_id: &str, matched_only: bool) -> Result<i64, rusqlite::Error> {
        if matched_only {
            self.conn.query_row(
                "SELECT COUNT(*) FROM filtered_results WHERE task_id = ?1 AND is_matched = 1",
                [task_id],
                |row| row.get(0),
            )
        } else {
            self.conn.query_row(
                "SELECT COUNT(*) FROM filtered_results WHERE task_id = ?1",
                [task_id],
                |row| row.get(0),
            )
        }
    }

    pub fn delete_by_task(&self, task_id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM filtered_results WHERE task_id = ?1", [task_id])?;
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

    fn make_result(
        task_id: &str,
        domain: &str,
        ftype: &str,
        matched: bool,
        score: Option<f64>,
    ) -> FilteredResult {
        FilteredResult {
            id: 0,
            task_id: task_id.to_string(),
            domain: domain.to_string(),
            filter_type: ftype.to_string(),
            filter_pattern: Some("test".to_string()),
            is_matched: matched,
            score,
            embedding_id: None,
        }
    }

    #[test]
    fn test_create_and_list() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        repo.create(&make_result("task1", "abc.com", "exact", true, None))
            .unwrap();
        repo.create(&make_result("task1", "def.com", "exact", false, None))
            .unwrap();
        let results = repo.list_by_task("task1", None, false, 100, 0).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_batch_insert() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        let results: Vec<FilteredResult> = (0..5)
            .map(|i| make_result("task1", &format!("d{}.com", i), "regex", i % 2 == 0, None))
            .collect();
        let count = repo.batch_insert(&results).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_list_matched_only() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        repo.create(&make_result("task1", "abc.com", "exact", true, None))
            .unwrap();
        repo.create(&make_result("task1", "def.com", "exact", false, None))
            .unwrap();
        let matched = repo.list_by_task("task1", None, true, 100, 0).unwrap();
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].domain, "abc.com");
    }

    #[test]
    fn test_list_by_filter_type() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        repo.create(&make_result("task1", "abc.com", "exact", true, None))
            .unwrap();
        repo.create(&make_result(
            "task1",
            "def.com",
            "semantic",
            true,
            Some(0.95),
        ))
        .unwrap();
        let exact = repo
            .list_by_task("task1", Some("exact"), false, 100, 0)
            .unwrap();
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].filter_type, "exact");
    }

    #[test]
    fn test_count_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        repo.create(&make_result("task1", "abc.com", "exact", true, None))
            .unwrap();
        repo.create(&make_result("task1", "def.com", "exact", false, None))
            .unwrap();
        assert_eq!(repo.count_by_task("task1", false).unwrap(), 2);
        assert_eq!(repo.count_by_task("task1", true).unwrap(), 1);
    }

    #[test]
    fn test_delete_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = FilterRepo::new(&conn);
        repo.create(&make_result("task1", "abc.com", "exact", true, None))
            .unwrap();
        repo.delete_by_task("task1").unwrap();
        assert_eq!(repo.count_by_task("task1", false).unwrap(), 0);
    }
}
