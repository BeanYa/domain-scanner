use rusqlite::Connection;

/// Initialize all database tables and indexes
pub fn init_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS task_batches (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            task_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            batch_id TEXT REFERENCES task_batches(id),
            name TEXT NOT NULL,
            signature TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'pending',
            scan_mode TEXT NOT NULL,
            config_json TEXT NOT NULL,
            tld TEXT NOT NULL,
            prefix_pattern TEXT,
            total_count INTEGER DEFAULT 0,
            completed_count INTEGER DEFAULT 0,
            completed_index INTEGER DEFAULT 0,
            available_count INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_signature ON tasks(signature);
        CREATE INDEX IF NOT EXISTS idx_tasks_batch ON tasks(batch_id);
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

        CREATE TABLE IF NOT EXISTS scan_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            domain TEXT NOT NULL,
            tld TEXT NOT NULL,
            item_index INTEGER NOT NULL,
            status TEXT DEFAULT 'pending',
            is_available INTEGER,
            query_method TEXT,
            response_time_ms INTEGER,
            error_message TEXT,
            checked_at DATETIME,
            UNIQUE(task_id, domain)
        );
        CREATE INDEX IF NOT EXISTS idx_scan_items_task_status ON scan_items(task_id, status);

        CREATE TABLE IF NOT EXISTS task_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            level TEXT NOT NULL DEFAULT 'info',
            message TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_task_logs_task ON task_logs(task_id);

        CREATE TABLE IF NOT EXISTS proxies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            url TEXT NOT NULL,
            proxy_type TEXT NOT NULL,
            username TEXT,
            password TEXT,
            is_active INTEGER DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS llm_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL,
            api_key TEXT NOT NULL,
            model TEXT NOT NULL,
            embedding_model TEXT,
            embedding_dim INTEGER DEFAULT 768,
            is_default INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS gpu_configs (
            id INTEGER PRIMARY KEY DEFAULT 1,
            backend TEXT DEFAULT 'auto',
            device_id INTEGER DEFAULT 0,
            batch_size INTEGER DEFAULT 500,
            model_path TEXT
        );

        CREATE TABLE IF NOT EXISTS filtered_results (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            domain TEXT NOT NULL,
            filter_type TEXT NOT NULL,
            filter_pattern TEXT,
            is_matched INTEGER NOT NULL,
            score REAL,
            embedding_id INTEGER
        );
        "
    )?;

    // Initialize default GPU config
    conn.execute(
        "INSERT OR IGNORE INTO gpu_configs (id, backend, device_id, batch_size) VALUES (1, 'auto', 0, 500)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> (Connection, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        init_database(&conn).unwrap();
        (conn, temp_file)
    }

    #[test]
    fn test_init_database_creates_tables() {
        let (conn, _temp) = setup_test_db();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"task_batches".to_string()));
        assert!(tables.contains(&"scan_items".to_string()));
        assert!(tables.contains(&"task_logs".to_string()));
        assert!(tables.contains(&"proxies".to_string()));
        assert!(tables.contains(&"llm_configs".to_string()));
        assert!(tables.contains(&"gpu_configs".to_string()));
        assert!(tables.contains(&"filtered_results".to_string()));
    }

    #[test]
    fn test_init_database_creates_indexes() {
        let (conn, _temp) = setup_test_db();

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(indexes.contains(&"idx_tasks_signature".to_string()));
        assert!(indexes.contains(&"idx_tasks_batch".to_string()));
        assert!(indexes.contains(&"idx_tasks_status".to_string()));
        assert!(indexes.contains(&"idx_scan_items_task_status".to_string()));
    }

    #[test]
    fn test_wal_mode_enabled() {
        let (conn, _temp) = setup_test_db();

        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "wal");
    }

    #[test]
    fn test_default_gpu_config() {
        let (conn, _temp) = setup_test_db();

        let backend: String = conn
            .query_row(
                "SELECT backend FROM gpu_configs WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(backend, "auto");
    }
}
