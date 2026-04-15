use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;

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

    pub fn create_entry(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: &str,
        message: &str,
    ) -> Result<TaskLog, rusqlite::Error> {
        let entry = TaskLog {
            id: next_log_id(),
            task_id: task_id.to_string(),
            run_id: run_id.map(|value| value.to_string()),
            level: level.to_string(),
            message: message.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let path = self.log_file_path(task_id, run_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io_to_rusqlite)?;
        }

        let line = serde_json::to_string(&entry)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?
            + "\n";
        self.rotate_if_needed(&path, line.as_bytes().len() as u64)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(io_to_rusqlite)?;
        file.write_all(line.as_bytes()).map_err(io_to_rusqlite)?;

        Ok(entry)
    }

    /// Insert a single log entry
    pub fn create(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: &str,
        message: &str,
    ) -> Result<i64, rusqlite::Error> {
        Ok(self.create_entry(task_id, run_id, level, message)?.id)
    }

    /// Batch insert log entries
    pub fn batch_insert(
        &self,
        logs: &[(&str, Option<&str>, &str, &str)],
    ) -> Result<usize, rusqlite::Error> {
        let mut count = 0;
        for (task_id, run_id, level, message) in logs {
            self.create(task_id, *run_id, level, message)?;
            count += 1;
        }
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
        let mut logs = self.read_logs(task_id, run_id)?;
        if let Some(level) = level {
            logs.retain(|entry| entry.level == level);
        }
        logs.sort_by(|a, b| b.id.cmp(&a.id));
        Ok(logs
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect())
    }

    /// Count logs for a task
    pub fn count_by_task(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        level: Option<&str>,
    ) -> Result<i64, rusqlite::Error> {
        let mut logs = self.read_logs(task_id, run_id)?;
        if let Some(level) = level {
            logs.retain(|entry| entry.level == level);
        }
        Ok(logs.len() as i64)
    }

    /// Delete all logs for a task
    pub fn delete_by_task(&self, task_id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM task_logs WHERE task_id = ?1", [task_id])?;
        let task_dir = self.logs_root().join(task_id);
        if task_dir.exists() {
            fs::remove_dir_all(task_dir).map_err(io_to_rusqlite)?;
        }
        Ok(())
    }

    fn read_logs(
        &self,
        task_id: &str,
        run_id: Option<&str>,
    ) -> Result<Vec<TaskLog>, rusqlite::Error> {
        let paths = self.log_paths(task_id, run_id).map_err(io_to_rusqlite)?;
        let mut logs = Vec::new();
        for path in paths {
            if !path.exists() {
                continue;
            }
            let file = OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(io_to_rusqlite)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.map_err(io_to_rusqlite)?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(entry) = serde_json::from_str::<TaskLog>(&line) {
                    logs.push(entry);
                }
            }
        }
        Ok(logs)
    }

    fn rotate_if_needed(&self, path: &Path, incoming_bytes: u64) -> Result<(), rusqlite::Error> {
        let current_bytes = path.metadata().map(|meta| meta.len()).unwrap_or(0);
        if current_bytes + incoming_bytes <= MAX_LOG_BYTES {
            return Ok(());
        }

        let backup = self.backup_path(path);
        if backup.exists() {
            fs::remove_file(&backup).map_err(io_to_rusqlite)?;
        }
        if path.exists() {
            fs::rename(path, backup).map_err(io_to_rusqlite)?;
        }
        Ok(())
    }

    fn logs_root(&self) -> PathBuf {
        match database_file_path(self.conn) {
            Some(path) => {
                let folder_name = path
                    .file_stem()
                    .map(|stem| format!("task-logs-{}", stem.to_string_lossy()))
                    .unwrap_or_else(|| "task-logs".to_string());
                path.parent()
                    .map(|parent| parent.join(folder_name))
                    .unwrap_or_else(|| PathBuf::from("task-logs"))
            }
            None => std::env::temp_dir().join("domain-scanner-task-logs"),
        }
    }

    fn log_file_path(&self, task_id: &str, run_id: Option<&str>) -> PathBuf {
        let filename = match run_id {
            Some(run_id) => format!("run-{}.log", run_id),
            None => "task.log".to_string(),
        };
        self.logs_root().join(task_id).join(filename)
    }

    fn backup_path(&self, path: &Path) -> PathBuf {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "task.log".to_string());
        path.with_file_name(format!("{}.1", file_name))
    }

    fn log_paths(
        &self,
        task_id: &str,
        run_id: Option<&str>,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        if let Some(run_id) = run_id {
            let current = self.log_file_path(task_id, Some(run_id));
            return Ok(vec![self.backup_path(&current), current]);
        }

        let task_dir = self.logs_root().join(task_id);
        if !task_dir.exists() {
            return Ok(Vec::new());
        }

        let mut paths = Vec::new();
        for entry in fs::read_dir(task_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                paths.push(path);
            }
        }
        Ok(paths)
    }
}

fn database_file_path(conn: &rusqlite::Connection) -> Option<PathBuf> {
    conn.query_row("PRAGMA database_list", [], |row| row.get::<_, String>(2))
        .ok()
        .and_then(|path| {
            if path.is_empty() || path == ":memory:" {
                None
            } else {
                Some(PathBuf::from(path))
            }
        })
}

fn io_to_rusqlite(err: std::io::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(err))
}

fn next_log_id() -> i64 {
    chrono::Utc::now()
        .timestamp_micros()
        .saturating_mul(1000)
        .saturating_add((rand::random::<u16>() % 1000) as i64)
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
