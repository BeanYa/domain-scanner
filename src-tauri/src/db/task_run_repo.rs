use crate::models::task::{TaskRun, TaskStatus};

pub struct TaskRunRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> TaskRunRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, run: &TaskRun) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO task_runs (id, task_id, run_number, status, total_count, completed_count, available_count, error_count, started_at, finished_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                run.id,
                run.task_id,
                run.run_number,
                serde_json::to_string(&run.status).unwrap(),
                run.total_count,
                run.completed_count,
                run.available_count,
                run.error_count,
                run.started_at,
                run.finished_at,
            ],
        )?;
        Ok(())
    }

    pub fn list_by_task(&self, task_id: &str) -> Result<Vec<TaskRun>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, run_number, status, total_count, completed_count, available_count, error_count, started_at, finished_at
             FROM task_runs WHERE task_id = ?1 ORDER BY run_number DESC"
        )?;
        let runs = stmt
            .query_map([task_id], |row| self.row_to_run(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(runs)
    }

    pub fn get_latest_by_task(&self, task_id: &str) -> Result<Option<TaskRun>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, task_id, run_number, status, total_count, completed_count, available_count, error_count, started_at, finished_at
             FROM task_runs WHERE task_id = ?1 ORDER BY run_number DESC LIMIT 1"
        )?;
        let mut rows = stmt.query([task_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_run(row)?)),
            None => Ok(None),
        }
    }

    pub fn next_run_number(&self, task_id: &str) -> Result<i64, rusqlite::Error> {
        let latest: Option<i64> = self.conn.query_row(
            "SELECT MAX(run_number) FROM task_runs WHERE task_id = ?1",
            [task_id],
            |row| row.get(0),
        )?;
        Ok(latest.unwrap_or(0) + 1)
    }

    pub fn update_status(
        &self,
        run_id: &str,
        status: &TaskStatus,
        finish_now: bool,
    ) -> Result<(), rusqlite::Error> {
        let sql = if finish_now {
            "UPDATE task_runs SET status = ?1, finished_at = CURRENT_TIMESTAMP WHERE id = ?2"
        } else {
            "UPDATE task_runs SET status = ?1 WHERE id = ?2"
        };
        self.conn.execute(
            sql,
            rusqlite::params![serde_json::to_string(status).unwrap(), run_id],
        )?;
        Ok(())
    }

    pub fn update_progress(
        &self,
        run_id: &str,
        completed_count: i64,
        available_count: i64,
        error_count: i64,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE task_runs SET completed_count = ?1, available_count = ?2, error_count = ?3 WHERE id = ?4",
            rusqlite::params![completed_count, available_count, error_count, run_id],
        )?;
        Ok(())
    }

    fn row_to_run(&self, row: &rusqlite::Row) -> Result<TaskRun, rusqlite::Error> {
        Ok(TaskRun {
            id: row.get(0)?,
            task_id: row.get(1)?,
            run_number: row.get(2)?,
            status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(TaskStatus::Pending),
            total_count: row.get(4)?,
            completed_count: row.get(5)?,
            available_count: row.get(6)?,
            error_count: row.get(7)?,
            started_at: row.get(8)?,
            finished_at: row.get(9)?,
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
        crate::db::task_repo::TaskRepo::new(conn)
            .create(&task)
            .unwrap();
    }

    fn make_run(id: &str, n: i64) -> TaskRun {
        TaskRun {
            id: id.to_string(),
            task_id: "task1".to_string(),
            run_number: n,
            status: TaskStatus::Pending,
            total_count: 100,
            completed_count: 0,
            available_count: 0,
            error_count: 0,
            started_at: "2026-01-01T00:00:00".to_string(),
            finished_at: None,
        }
    }

    #[test]
    fn test_create_and_list_runs() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = TaskRunRepo::new(&conn);
        repo.create(&make_run("run1", 1)).unwrap();
        repo.create(&make_run("run2", 2)).unwrap();
        let runs = repo.list_by_task("task1").unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].run_number, 2);
    }

    #[test]
    fn test_next_run_number() {
        let (conn, _temp) = setup();
        create_test_task(&conn);
        let repo = TaskRunRepo::new(&conn);
        assert_eq!(repo.next_run_number("task1").unwrap(), 1);
        repo.create(&make_run("run1", 1)).unwrap();
        assert_eq!(repo.next_run_number("task1").unwrap(), 2);
    }
}
