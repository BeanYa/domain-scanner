use crate::models::scan_item::{ScanItem, ScanItemStatus};

pub struct ScanItemRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> ScanItemRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Insert a single scan item
    pub fn create(&self, item: &ScanItem) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO scan_items (task_id, domain, tld, item_index, status, is_available, query_method, response_time_ms, error_message, checked_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                item.task_id, item.domain, item.tld, item.item_index,
                serde_json::to_string(&item.status).unwrap(),
                item.is_available, item.query_method, item.response_time_ms,
                item.error_message, item.checked_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Batch insert scan items within a transaction
    pub fn batch_insert(&self, items: &[ScanItem]) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for item in items {
            tx.execute(
                "INSERT INTO scan_items (task_id, domain, tld, item_index, status, is_available, query_method, response_time_ms, error_message, checked_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    item.task_id, item.domain, item.tld, item.item_index,
                    serde_json::to_string(&item.status).unwrap(),
                    item.is_available, item.query_method, item.response_time_ms,
                    item.error_message, item.checked_at
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Update scan item status and result
    pub fn update_status(
        &self,
        id: i64,
        status: &ScanItemStatus,
        is_available: Option<bool>,
        query_method: Option<&str>,
        response_time_ms: Option<i64>,
        error_message: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        let status_str = serde_json::to_string(status).unwrap();
        self.conn.execute(
            "UPDATE scan_items SET status = ?1, is_available = ?2, query_method = ?3, response_time_ms = ?4, error_message = ?5, checked_at = CURRENT_TIMESTAMP WHERE id = ?6",
            rusqlite::params![status_str, is_available, query_method, response_time_ms, error_message, id],
        )?;
        Ok(())
    }

    /// Get scan item by ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<ScanItem>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, domain, tld, item_index, status, is_available, query_method, response_time_ms, error_message, checked_at FROM scan_items WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_item(row)?)),
            None => Ok(None),
        }
    }

    /// List scan items for a task with pagination
    pub fn list_by_task(
        &self,
        task_id: &str,
        status: Option<&ScanItemStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ScanItem>, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT id, task_id, domain, tld, item_index, status, is_available, query_method, response_time_ms, error_message, checked_at FROM scan_items WHERE task_id = ?1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(task_id.to_string())];

        if let Some(s) = status {
            sql.push_str(" AND status = ?");
            param_values.push(Box::new(serde_json::to_string(s).unwrap()));
        }
        sql.push_str(" ORDER BY item_index ASC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let items = stmt
            .query_map(params.as_slice(), |row| self.row_to_item(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    /// Count scan items for a task, optionally filtered by status
    pub fn count_by_task(&self, task_id: &str, status: Option<&ScanItemStatus>) -> Result<i64, rusqlite::Error> {
        match status {
            Some(s) => {
                let status_str = serde_json::to_string(s).unwrap();
                self.conn.query_row(
                    "SELECT COUNT(*) FROM scan_items WHERE task_id = ?1 AND status = ?2",
                    rusqlite::params![task_id, status_str],
                    |row| row.get(0),
                )
            }
            None => {
                self.conn.query_row(
                    "SELECT COUNT(*) FROM scan_items WHERE task_id = ?1",
                    [task_id],
                    |row| row.get(0),
                )
            }
        }
    }

    /// Get scan items by index range (for checkpoint resume)
    pub fn get_by_index_range(
        &self,
        task_id: &str,
        from_index: i64,
        limit: i64,
    ) -> Result<Vec<ScanItem>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, domain, tld, item_index, status, is_available, query_method, response_time_ms, error_message, checked_at FROM scan_items WHERE task_id = ?1 AND item_index >= ?2 ORDER BY item_index ASC LIMIT ?3"
        )?;
        let items = stmt
            .query_map(rusqlite::params![task_id, from_index, limit], |row| self.row_to_item(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    fn row_to_item(&self, row: &rusqlite::Row) -> Result<ScanItem, rusqlite::Error> {
        Ok(ScanItem {
            id: row.get(0)?,
            task_id: row.get(1)?,
            domain: row.get(2)?,
            tld: row.get(3)?,
            item_index: row.get(4)?,
            status: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or(ScanItemStatus::Pending),
            is_available: row.get(6)?,
            query_method: row.get(7)?,
            response_time_ms: row.get(8)?,
            error_message: row.get(9)?,
            checked_at: row.get(10)?,
        })
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
            scan_mode: ScanMode::Regex { pattern: "^[a-z]{3}$".to_string() },
            config_json: "{}".to_string(),
            tlds: vec![".com".to_string()],
            prefix_pattern: None,
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

    fn make_scan_item(index: i64, domain: &str) -> ScanItem {
        ScanItem {
            id: 0,
            task_id: "task1".to_string(),
            domain: domain.to_string(),
            tld: ".com".to_string(),
            item_index: index,
            status: ScanItemStatus::Pending,
            is_available: None,
            query_method: None,
            response_time_ms: None,
            error_message: None,
            checked_at: None,
        }
    }

    #[test]
    fn test_create_and_get_scan_item() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let item = make_scan_item(0, "abc.com");
        let id = repo.create(&item).unwrap();
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        assert_eq!(fetched.domain, "abc.com");
        assert_eq!(fetched.item_index, 0);
    }

    #[test]
    fn test_batch_insert() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let items: Vec<ScanItem> = (0..10)
            .map(|i| make_scan_item(i, &format!("domain{}.com", i)))
            .collect();
        let count = repo.batch_insert(&items).unwrap();
        assert_eq!(count, 10);
        assert_eq!(repo.count_by_task("task1", None).unwrap(), 10);
    }

    #[test]
    fn test_update_status() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let id = repo.create(&make_scan_item(0, "abc.com")).unwrap();
        repo.update_status(id, &ScanItemStatus::Available, Some(true), Some("rdap"), Some(150), None).unwrap();
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        assert_eq!(fetched.status, ScanItemStatus::Available);
        assert_eq!(fetched.is_available, Some(true));
        assert_eq!(fetched.query_method, Some("rdap".to_string()));
        assert_eq!(fetched.response_time_ms, Some(150));
        assert!(fetched.checked_at.is_some());
    }

    #[test]
    fn test_list_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let items: Vec<ScanItem> = (0..5)
            .map(|i| make_scan_item(i, &format!("d{}.com", i)))
            .collect();
        repo.batch_insert(&items).unwrap();
        let listed = repo.list_by_task("task1", None, 100, 0).unwrap();
        assert_eq!(listed.len(), 5);
    }

    #[test]
    fn test_list_by_task_with_status_filter() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let id = repo.create(&make_scan_item(0, "abc.com")).unwrap();
        repo.create(&make_scan_item(1, "def.com")).unwrap();
        repo.update_status(id, &ScanItemStatus::Available, Some(true), Some("rdap"), None, None).unwrap();
        let available = repo.list_by_task("task1", Some(&ScanItemStatus::Available), 100, 0).unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].domain, "abc.com");
    }

    #[test]
    fn test_list_by_task_pagination() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let items: Vec<ScanItem> = (0..10)
            .map(|i| make_scan_item(i, &format!("d{}.com", i)))
            .collect();
        repo.batch_insert(&items).unwrap();
        let page1 = repo.list_by_task("task1", None, 3, 0).unwrap();
        let page2 = repo.list_by_task("task1", None, 3, 3).unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page2.len(), 3);
    }

    #[test]
    fn test_count_by_task() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let items: Vec<ScanItem> = (0..5)
            .map(|i| make_scan_item(i, &format!("d{}.com", i)))
            .collect();
        repo.batch_insert(&items).unwrap();
        assert_eq!(repo.count_by_task("task1", None).unwrap(), 5);
        assert_eq!(repo.count_by_task("task1", Some(&ScanItemStatus::Pending)).unwrap(), 5);
    }

    #[test]
    fn test_get_by_index_range() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let items: Vec<ScanItem> = (0..10)
            .map(|i| make_scan_item(i, &format!("d{}.com", i)))
            .collect();
        repo.batch_insert(&items).unwrap();
        let from_5 = repo.get_by_index_range("task1", 5, 10).unwrap();
        assert_eq!(from_5.len(), 5);
        assert_eq!(from_5[0].item_index, 5);
    }

    #[test]
    fn test_list_empty_result() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = ScanItemRepo::new(&conn);
        let result = repo.list_by_task("task1", None, 100, 0).unwrap();
        assert!(result.is_empty());
    }
}
