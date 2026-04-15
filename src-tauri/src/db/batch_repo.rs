use crate::models::task::TaskBatch;

pub struct BatchRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> BatchRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, batch: &TaskBatch) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO task_batches (id, name, task_count, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![batch.id, batch.name, batch.task_count, batch.created_at],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<TaskBatch>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, task_count, created_at FROM task_batches WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(TaskBatch {
                id: row.get(0)?,
                name: row.get(1)?,
                task_count: row.get(2)?,
                created_at: row.get(3)?,
            })),
            None => Ok(None),
        }
    }

    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<TaskBatch>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, task_count, created_at FROM task_batches ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let batches = stmt
            .query_map(rusqlite::params![limit, offset], |row| {
                Ok(TaskBatch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    task_count: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(batches)
    }

    pub fn update_task_count(&self, id: &str, task_count: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE task_batches SET task_count = ?1 WHERE id = ?2",
            rusqlite::params![task_count, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM task_batches WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn count(&self) -> Result<i64, rusqlite::Error> {
        self.conn
            .query_row("SELECT COUNT(*) FROM task_batches", [], |row| row.get(0))
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

    fn make_batch(id: &str, name: &str, count: i64) -> TaskBatch {
        TaskBatch {
            id: id.to_string(),
            name: name.to_string(),
            task_count: count,
            created_at: "2026-01-01T00:00:00".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_batch() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        let batch = make_batch("b1", "Test Batch", 3);
        repo.create(&batch).unwrap();
        let fetched = repo.get_by_id("b1").unwrap().unwrap();
        assert_eq!(fetched.id, "b1");
        assert_eq!(fetched.name, "Test Batch");
        assert_eq!(fetched.task_count, 3);
    }

    #[test]
    fn test_get_nonexistent_batch() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        assert!(repo.get_by_id("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_batches() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        repo.create(&make_batch("b1", "Batch 1", 2)).unwrap();
        repo.create(&make_batch("b2", "Batch 2", 5)).unwrap();
        let batches = repo.list(100, 0).unwrap();
        assert_eq!(batches.len(), 2);
    }

    #[test]
    fn test_list_batches_pagination() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        for i in 0..5 {
            repo.create(&make_batch(&format!("b{i}"), &format!("Batch {i}"), i))
                .unwrap();
        }
        let page1 = repo.list(2, 0).unwrap();
        let page2 = repo.list(2, 2).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_update_task_count() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        repo.create(&make_batch("b1", "Batch 1", 2)).unwrap();
        repo.update_task_count("b1", 10).unwrap();
        let fetched = repo.get_by_id("b1").unwrap().unwrap();
        assert_eq!(fetched.task_count, 10);
    }

    #[test]
    fn test_delete_batch() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        repo.create(&make_batch("b1", "Batch 1", 2)).unwrap();
        repo.delete("b1").unwrap();
        assert!(repo.get_by_id("b1").unwrap().is_none());
    }

    #[test]
    fn test_count_batches() {
        let (conn, _temp) = setup();
        let repo = BatchRepo::new(&conn);
        assert_eq!(repo.count().unwrap(), 0);
        repo.create(&make_batch("b1", "Batch 1", 2)).unwrap();
        repo.create(&make_batch("b2", "Batch 2", 3)).unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }
}
