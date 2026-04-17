use crate::models::proxy::{ProxyConfig, ProxyStatus, ProxyType};

pub struct ProxyRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> ProxyRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, proxy: &ProxyConfig) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO proxies (name, url, proxy_type, username, password, is_active, status, last_checked_at, last_error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                proxy.name, proxy.url,
                serde_json::to_string(&proxy.proxy_type).unwrap(),
                proxy.username, proxy.password,
                proxy.is_active as i32,
                serde_json::to_string(&proxy.status).unwrap(),
                proxy.last_checked_at,
                proxy.last_error
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<ProxyConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, proxy_type, username, password, is_active, status, last_checked_at, last_error FROM proxies WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_proxy(row)?)),
            None => Ok(None),
        }
    }

    pub fn list(&self, active_only: bool) -> Result<Vec<ProxyConfig>, rusqlite::Error> {
        let sql = if active_only {
            "SELECT id, name, url, proxy_type, username, password, is_active, status, last_checked_at, last_error FROM proxies WHERE is_active = 1 ORDER BY id"
        } else {
            "SELECT id, name, url, proxy_type, username, password, is_active, status, last_checked_at, last_error FROM proxies ORDER BY id"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let proxies = stmt
            .query_map([], |row| self.row_to_proxy(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(proxies)
    }

    pub fn update(&self, proxy: &ProxyConfig) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE proxies SET name = ?1, url = ?2, proxy_type = ?3, username = ?4, password = ?5, is_active = ?6, status = ?7, last_checked_at = ?8, last_error = ?9 WHERE id = ?10",
            rusqlite::params![
                proxy.name, proxy.url,
                serde_json::to_string(&proxy.proxy_type).unwrap(),
                proxy.username, proxy.password,
                proxy.is_active as i32,
                serde_json::to_string(&proxy.status).unwrap(),
                proxy.last_checked_at,
                proxy.last_error,
                proxy.id
            ],
        )?;
        Ok(())
    }

    pub fn update_health(
        &self,
        id: i64,
        status: &ProxyStatus,
        active: bool,
        last_checked_at: Option<&str>,
        last_error: Option<&str>,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE proxies SET is_active = ?1, status = ?2, last_checked_at = ?3, last_error = ?4 WHERE id = ?5",
            rusqlite::params![
                active as i32,
                serde_json::to_string(status).unwrap(),
                last_checked_at,
                last_error,
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM proxies WHERE id = ?1", [id])?;
        Ok(())
    }

    fn row_to_proxy(&self, row: &rusqlite::Row) -> Result<ProxyConfig, rusqlite::Error> {
        let is_active: i32 = row.get(6)?;
        Ok(ProxyConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            proxy_type: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(ProxyType::Http),
            username: row.get(4)?,
            password: row.get(5)?,
            is_active: is_active != 0,
            status: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or(ProxyStatus::Pending),
            last_checked_at: row.get(8)?,
            last_error: row.get(9)?,
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

    fn make_proxy(id: i64, url: &str, ptype: ProxyType, active: bool) -> ProxyConfig {
        ProxyConfig {
            id,
            name: Some(format!("Proxy {}", id)),
            url: url.to_string(),
            proxy_type: ptype,
            username: None,
            password: None,
            is_active: active,
            status: if active {
                ProxyStatus::Available
            } else {
                ProxyStatus::Pending
            },
            last_checked_at: None,
            last_error: None,
        }
    }

    #[test]
    fn test_create_and_get_proxy() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        let proxy = make_proxy(0, "socks5://127.0.0.1:1080", ProxyType::Socks5, true);
        let id = repo.create(&proxy).unwrap();
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        assert_eq!(fetched.url, "socks5://127.0.0.1:1080");
        assert_eq!(fetched.proxy_type, ProxyType::Socks5);
        assert!(fetched.is_active);
    }

    #[test]
    fn test_list_all_proxies() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        repo.create(&make_proxy(0, "http://p1:8080", ProxyType::Http, true))
            .unwrap();
        repo.create(&make_proxy(0, "https://p2:8443", ProxyType::Https, false))
            .unwrap();
        let all = repo.list(false).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_active_proxies() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        repo.create(&make_proxy(0, "http://p1:8080", ProxyType::Http, true))
            .unwrap();
        repo.create(&make_proxy(0, "https://p2:8443", ProxyType::Https, false))
            .unwrap();
        let active = repo.list(true).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].url, "http://p1:8080");
    }

    #[test]
    fn test_update_proxy() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        let id = repo
            .create(&make_proxy(0, "http://p1:8080", ProxyType::Http, true))
            .unwrap();
        let mut proxy = repo.get_by_id(id).unwrap().unwrap();
        proxy.url = "http://updated:9999".to_string();
        repo.update(&proxy).unwrap();
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        assert_eq!(fetched.url, "http://updated:9999");
    }

    #[test]
    fn test_set_active() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        let id = repo
            .create(&make_proxy(0, "http://p1:8080", ProxyType::Http, true))
            .unwrap();
        repo.update_health(
            id,
            &ProxyStatus::Unavailable,
            false,
            Some("2026-04-16T00:00:00Z"),
            Some("timeout"),
        )
        .unwrap();
        let fetched = repo.get_by_id(id).unwrap().unwrap();
        assert!(!fetched.is_active);
        assert_eq!(fetched.status, ProxyStatus::Unavailable);
        assert_eq!(fetched.last_error.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_delete_proxy() {
        let (conn, _temp) = setup();
        let repo = ProxyRepo::new(&conn);
        let id = repo
            .create(&make_proxy(0, "http://p1:8080", ProxyType::Http, true))
            .unwrap();
        repo.delete(id).unwrap();
        assert!(repo.get_by_id(id).unwrap().is_none());
    }
}
