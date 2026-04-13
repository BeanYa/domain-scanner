use crate::models::llm::LlmConfig;

pub struct LlmRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> LlmRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create(&self, config: &LlmConfig) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO llm_configs (id, name, base_url, api_key, model, embedding_model, embedding_dim, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                config.id, config.name, config.base_url, config.api_key,
                config.model, config.embedding_model, config.embedding_dim,
                config.is_default as i32
            ],
        )?;
        Ok(())
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<LlmConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_url, api_key, model, embedding_model, embedding_dim, is_default FROM llm_configs WHERE id = ?1"
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_config(row)?)),
            None => Ok(None),
        }
    }

    pub fn list(&self) -> Result<Vec<LlmConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_url, api_key, model, embedding_model, embedding_dim, is_default FROM llm_configs ORDER BY name"
        )?;
        let configs = stmt
            .query_map([], |row| self.row_to_config(row))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(configs)
    }

    pub fn update(&self, config: &LlmConfig) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE llm_configs SET name = ?1, base_url = ?2, api_key = ?3, model = ?4, embedding_model = ?5, embedding_dim = ?6, is_default = ?7 WHERE id = ?8",
            rusqlite::params![
                config.name, config.base_url, config.api_key,
                config.model, config.embedding_model, config.embedding_dim,
                config.is_default as i32, config.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM llm_configs WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_default(&self) -> Result<Option<LlmConfig>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, base_url, api_key, model, embedding_model, embedding_dim, is_default FROM llm_configs WHERE is_default = 1 LIMIT 1"
        )?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok(Some(self.row_to_config(row)?)),
            None => Ok(None),
        }
    }

    pub fn set_default(&self, id: &str) -> Result<(), rusqlite::Error> {
        // Unset all defaults first
        self.conn.execute("UPDATE llm_configs SET is_default = 0", [])?;
        // Set the new default
        self.conn.execute(
            "UPDATE llm_configs SET is_default = 1 WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    fn row_to_config(&self, row: &rusqlite::Row) -> Result<LlmConfig, rusqlite::Error> {
        let is_default: i32 = row.get(7)?;
        Ok(LlmConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            base_url: row.get(2)?,
            api_key: row.get(3)?,
            model: row.get(4)?,
            embedding_model: row.get(5)?,
            embedding_dim: row.get(6)?,
            is_default: is_default != 0,
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

    fn make_config(id: &str, name: &str, is_default: bool) -> LlmConfig {
        LlmConfig {
            id: id.to_string(),
            name: name.to_string(),
            base_url: "https://api.example.com/v1/".to_string(),
            api_key: "sk-test".to_string(),
            model: "gpt-4".to_string(),
            embedding_model: Some("text-embedding-3-small".to_string()),
            embedding_dim: 1536,
            is_default,
        }
    }

    #[test]
    fn test_create_and_get_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        let config = make_config("glm4", "GLM-4", true);
        repo.create(&config).unwrap();
        let fetched = repo.get_by_id("glm4").unwrap().unwrap();
        assert_eq!(fetched.name, "GLM-4");
        assert_eq!(fetched.embedding_dim, 1536);
        assert!(fetched.is_default);
    }

    #[test]
    fn test_get_nonexistent_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        assert!(repo.get_by_id("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_list_configs() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        repo.create(&make_config("glm4", "GLM-4", true)).unwrap();
        repo.create(&make_config("minimax", "MiniMax", false)).unwrap();
        let configs = repo.list().unwrap();
        assert_eq!(configs.len(), 2);
    }

    #[test]
    fn test_update_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        repo.create(&make_config("glm4", "GLM-4", false)).unwrap();
        let mut config = repo.get_by_id("glm4").unwrap().unwrap();
        config.name = "GLM-4-Plus".to_string();
        config.embedding_dim = 2048;
        repo.update(&config).unwrap();
        let fetched = repo.get_by_id("glm4").unwrap().unwrap();
        assert_eq!(fetched.name, "GLM-4-Plus");
        assert_eq!(fetched.embedding_dim, 2048);
    }

    #[test]
    fn test_delete_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        repo.create(&make_config("glm4", "GLM-4", false)).unwrap();
        repo.delete("glm4").unwrap();
        assert!(repo.get_by_id("glm4").unwrap().is_none());
    }

    #[test]
    fn test_get_default_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        assert!(repo.get_default().unwrap().is_none());
        repo.create(&make_config("glm4", "GLM-4", true)).unwrap();
        repo.create(&make_config("minimax", "MiniMax", false)).unwrap();
        let default = repo.get_default().unwrap().unwrap();
        assert_eq!(default.id, "glm4");
    }

    #[test]
    fn test_set_default_config() {
        let (conn, _temp) = setup();
        let repo = LlmRepo::new(&conn);
        repo.create(&make_config("glm4", "GLM-4", true)).unwrap();
        repo.create(&make_config("minimax", "MiniMax", false)).unwrap();
        repo.set_default("minimax").unwrap();
        let default = repo.get_default().unwrap().unwrap();
        assert_eq!(default.id, "minimax");
        let glm = repo.get_by_id("glm4").unwrap().unwrap();
        assert!(!glm.is_default);
    }
}
