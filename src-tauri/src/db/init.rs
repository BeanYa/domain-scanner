use rusqlite::Connection;
use std::sync::{Once, OnceLock};

static VEC_EXT_INIT: Once = Once::new();
static DB_PATH: OnceLock<String> = OnceLock::new();

/// Set the persistent database file path (call once during app setup).
pub fn set_db_path(path: String) {
    DB_PATH.set(path).expect("Database path already set");
}

/// Open a connection to the persistent database.
/// Falls back to `:memory:` if `set_db_path` was never called (e.g. in tests).
pub fn open_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let path = DB_PATH.get().map(|s| s.as_str()).unwrap_or(":memory:");
    open_and_init(path)
}

/// Register the sqlite-vec extension for all new connections
pub fn register_vec_extension() {
    VEC_EXT_INIT.call_once(|| unsafe {
        rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(
                *mut rusqlite::ffi::sqlite3,
                *mut *const std::ffi::c_char,
                *const rusqlite::ffi::sqlite3_api_routines,
            ) -> i32,
        >(sqlite_vec::sqlite3_vec_init)));
    });
}

/// Initialize all database tables, indexes, and extensions
pub fn init_database(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Create the vec0 virtual table for domain embeddings
    conn.execute_batch("CREATE VIRTUAL TABLE IF NOT EXISTS domain_vectors USING vec0(domain_id INTEGER PRIMARY KEY, domain_embedding float[384]);")?;

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
            tlds TEXT NOT NULL,
            prefix_pattern TEXT,
            concurrency INTEGER DEFAULT 50,
            proxy_id INTEGER,
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

        CREATE TABLE IF NOT EXISTS task_runs (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            run_number INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            total_count INTEGER DEFAULT 0,
            completed_count INTEGER DEFAULT 0,
            available_count INTEGER DEFAULT 0,
            error_count INTEGER DEFAULT 0,
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            finished_at DATETIME,
            UNIQUE(task_id, run_number)
        );
        CREATE INDEX IF NOT EXISTS idx_task_runs_task_started ON task_runs(task_id, started_at DESC);

        CREATE TABLE IF NOT EXISTS scan_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            run_id TEXT NOT NULL,
            domain TEXT NOT NULL,
            tld TEXT NOT NULL,
            item_index INTEGER NOT NULL,
            status TEXT DEFAULT 'pending',
            is_available INTEGER,
            query_method TEXT,
            response_time_ms INTEGER,
            error_message TEXT,
            checked_at DATETIME,
            UNIQUE(task_id, run_id, domain)
        );
        CREATE INDEX IF NOT EXISTS idx_scan_items_task_status ON scan_items(task_id, run_id, status);

        CREATE TABLE IF NOT EXISTS task_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id),
            run_id TEXT,
            level TEXT NOT NULL DEFAULT 'info',
            message TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_task_logs_task ON task_logs(task_id, run_id);

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
        CREATE INDEX IF NOT EXISTS idx_filtered_results_task ON filtered_results(task_id);
        "
    )?;

    // Initialize default GPU config
    conn.execute(
        "INSERT OR IGNORE INTO gpu_configs (id, backend, device_id, batch_size) VALUES (1, 'auto', 0, 500)",
        [],
    )?;

    // Migrations: add columns that may be missing from older schema
    migrate_add_column(conn, "tasks", "concurrency", "INTEGER DEFAULT 50");
    migrate_add_column(conn, "tasks", "proxy_id", "INTEGER");
    migrate_add_column(conn, "scan_items", "run_id", "TEXT");
    migrate_add_column(conn, "task_logs", "run_id", "TEXT");

    Ok(())
}

/// Safely add a column if it doesn't already exist (idempotent migration)
fn migrate_add_column(conn: &Connection, table: &str, column: &str, col_type: &str) {
    let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, col_type);
    // "duplicate column name" error means it already exists — that's fine
    let _ = conn.execute(&sql, []);
}

/// Open a database connection and initialize schema
pub fn open_and_init(path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    register_vec_extension();
    let conn = Connection::open(path)?;
    init_database(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    pub fn setup_test_db() -> (Connection, NamedTempFile) {
        register_vec_extension();
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        init_database(&conn).unwrap();
        (conn, temp_file)
    }

    #[test]
    fn test_init_database_creates_tables() {
        let (conn, _temp) = setup_test_db();

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
        assert!(tables.contains(&"task_runs".to_string()));
        assert!(tables.contains(&"proxies".to_string()));
        assert!(tables.contains(&"llm_configs".to_string()));
        assert!(tables.contains(&"gpu_configs".to_string()));
        assert!(tables.contains(&"filtered_results".to_string()));
        assert!(tables.contains(&"domain_vectors".to_string()));
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
        assert!(indexes.contains(&"idx_task_logs_task".to_string()));
        assert!(indexes.contains(&"idx_task_runs_task_started".to_string()));
        assert!(indexes.contains(&"idx_filtered_results_task".to_string()));
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
            .query_row("SELECT backend FROM gpu_configs WHERE id = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(backend, "auto");
    }

    #[test]
    fn test_domain_vectors_virtual_table_exists() {
        let (conn, _temp) = setup_test_db();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='domain_vectors'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"domain_vectors".to_string()));
    }

    #[test]
    fn test_idempotent_init() {
        register_vec_extension();
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        init_database(&conn).unwrap();
        // Second init should not fail
        init_database(&conn).unwrap();
    }
}
