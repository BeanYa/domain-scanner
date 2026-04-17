/// Vector repository for sqlite-vec integration
/// Provides CRUD operations for domain embedding vectors and similarity search

pub const EMBEDDING_DIM: usize = 384;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorRecord {
    pub domain_id: i64,
    pub task_id: String,
    pub domain: String,
    pub tld: String,
    pub vector_dim: i64,
}

pub struct VectorRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> VectorRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// Insert a single embedding vector for a domain
    pub fn insert(&self, domain_id: i64, embedding: &[f32]) -> Result<(), rusqlite::Error> {
        assert_eq!(
            embedding.len(),
            EMBEDDING_DIM,
            "Embedding dimension must be {}",
            EMBEDDING_DIM
        );
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        self.conn.execute(
            "INSERT INTO domain_vectors(domain_id, domain_embedding) VALUES (?1, ?2)",
            rusqlite::params![domain_id, embedding_bytes],
        )?;
        Ok(())
    }

    /// Replace an existing vector or insert a new one for the given scan item.
    pub fn upsert(&self, domain_id: i64, embedding: &[f32]) -> Result<(), rusqlite::Error> {
        assert_eq!(
            embedding.len(),
            EMBEDDING_DIM,
            "Embedding dimension must be {}",
            EMBEDDING_DIM
        );
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM domain_vectors WHERE domain_id = ?1",
            [domain_id],
        )?;
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        tx.execute(
            "INSERT INTO domain_vectors(domain_id, domain_embedding) VALUES (?1, ?2)",
            rusqlite::params![domain_id, embedding_bytes],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Batch insert embedding vectors
    pub fn batch_insert(&self, items: &[(i64, &[f32])]) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for (domain_id, embedding) in items {
            assert_eq!(embedding.len(), EMBEDDING_DIM);
            let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
            tx.execute(
                "INSERT INTO domain_vectors(domain_id, domain_embedding) VALUES (?1, ?2)",
                rusqlite::params![domain_id, embedding_bytes],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    /// Search for similar domain vectors by cosine distance
    /// Returns (domain_id, distance) pairs sorted by similarity
    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: i64,
    ) -> Result<Vec<(i64, f32)>, rusqlite::Error> {
        assert_eq!(query_embedding.len(), EMBEDDING_DIM);
        let query_bytes: Vec<u8> = query_embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let mut stmt = self.conn.prepare(
            "SELECT domain_id, distance FROM domain_vectors WHERE domain_embedding MATCH ?1 ORDER BY distance LIMIT ?2"
        )?;

        let results = stmt
            .query_map(rusqlite::params![query_bytes, limit], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f32>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Search similar vectors scoped to a scan task.
    /// Returns (scan_item_id, domain, distance) rows sorted by vector distance.
    pub fn search_similar_by_task(
        &self,
        task_id: &str,
        query_embedding: &[f32],
        limit: i64,
    ) -> Result<Vec<(i64, String, f32)>, rusqlite::Error> {
        assert_eq!(query_embedding.len(), EMBEDDING_DIM);
        let query_bytes: Vec<u8> = query_embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let mut stmt = self.conn.prepare(
            "SELECT v.domain_id, s.domain, v.distance
             FROM domain_vectors v
             JOIN scan_items s ON s.id = v.domain_id
             WHERE s.task_id = ?1
               AND v.domain_embedding MATCH ?2
             ORDER BY v.distance
             LIMIT ?3",
        )?;

        let results = stmt
            .query_map(rusqlite::params![task_id, query_bytes, limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, f32>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Delete a vector by domain_id
    pub fn delete(&self, domain_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM domain_vectors WHERE domain_id = ?1",
            [domain_id],
        )?;
        Ok(())
    }

    /// Delete all vectors for a scan task.
    pub fn delete_by_task(&self, task_id: &str) -> Result<usize, rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM domain_vectors
             WHERE domain_id IN (
                SELECT id FROM scan_items WHERE task_id = ?1
             )",
            [task_id],
        )
    }

    /// Count total vectors
    pub fn count(&self) -> Result<i64, rusqlite::Error> {
        self.conn
            .query_row("SELECT COUNT(*) FROM domain_vectors", [], |row| row.get(0))
    }

    /// Count vectors that belong to a specific scan task.
    pub fn count_by_task(&self, task_id: &str) -> Result<i64, rusqlite::Error> {
        self.conn.query_row(
            "SELECT COUNT(*)
             FROM domain_vectors v
             JOIN scan_items s ON s.id = v.domain_id
             WHERE s.task_id = ?1",
            [task_id],
            |row| row.get(0),
        )
    }

    /// List vector metadata for a scan task without exposing large raw embeddings.
    pub fn list_by_task(
        &self,
        task_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<VectorRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT v.domain_id, s.task_id, s.domain, s.tld, length(v.domain_embedding) / 4 AS vector_dim
             FROM domain_vectors v
             JOIN scan_items s ON s.id = v.domain_id
             WHERE s.task_id = ?1
             ORDER BY s.domain ASC
             LIMIT ?2 OFFSET ?3",
        )?;

        let rows = stmt.query_map(
            rusqlite::params![task_id, limit.max(0), offset.max(0)],
            |row| {
                Ok(VectorRecord {
                    domain_id: row.get(0)?,
                    task_id: row.get(1)?,
                    domain: row.get(2)?,
                    tld: row.get(3)?,
                    vector_dim: row.get(4)?,
                })
            },
        )?;
        Ok(rows.filter_map(|row| row.ok()).collect())
    }

    /// Check if a domain has a vector
    pub fn exists(&self, domain_id: i64) -> Result<bool, rusqlite::Error> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM domain_vectors WHERE domain_id = ?1",
            [domain_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
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

    fn make_embedding(seed: f32) -> Vec<f32> {
        (0..EMBEDDING_DIM)
            .map(|i| seed + i as f32 * 0.001)
            .collect()
    }

    #[test]
    fn test_insert_and_count() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        let embedding = make_embedding(0.0);
        assert_eq!(repo.count().unwrap(), 0);
        repo.insert(1, &embedding).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }

    #[test]
    fn test_batch_insert() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        let e1 = make_embedding(0.0);
        let e2 = make_embedding(1.0);
        let e3 = make_embedding(2.0);
        let items: Vec<(i64, &[f32])> = vec![(1, &e1), (2, &e2), (3, &e3)];
        let count = repo.batch_insert(&items).unwrap();
        assert_eq!(count, 3);
        assert_eq!(repo.count().unwrap(), 3);
    }

    #[test]
    fn test_exists() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        let embedding = make_embedding(0.0);
        assert!(!repo.exists(1).unwrap());
        repo.insert(1, &embedding).unwrap();
        assert!(repo.exists(1).unwrap());
    }

    #[test]
    fn test_count_by_task() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        conn.execute(
            "INSERT INTO tasks (id, name, signature, status, scan_mode, config_json, tlds, total_count, completed_count, completed_index, available_count, error_count, created_at, updated_at)
             VALUES ('task1', 'Test', 'sig1', '\"pending\"', '{}', '{}', '[\".com\"]', 1, 0, 0, 0, 0, '2026-01-01T00:00:00', '2026-01-01T00:00:00')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO scan_items (id, task_id, run_id, domain, tld, item_index, status)
             VALUES (1, 'task1', 'run1', 'abc.com', '.com', 0, '\"available\"')",
            [],
        )
        .unwrap();
        repo.insert(1, &make_embedding(0.0)).unwrap();

        assert_eq!(repo.count_by_task("task1").unwrap(), 1);
        assert_eq!(repo.count_by_task("other").unwrap(), 0);
    }

    #[test]
    fn test_delete() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        let embedding = make_embedding(0.0);
        repo.insert(1, &embedding).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
        repo.delete(1).unwrap();
        assert_eq!(repo.count().unwrap(), 0);
        assert!(!repo.exists(1).unwrap());
    }

    #[test]
    fn test_upsert_replaces_vector() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        repo.insert(1, &make_embedding(0.0)).unwrap();
        repo.upsert(1, &make_embedding(1.0)).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
        assert!(repo.exists(1).unwrap());
    }

    #[test]
    fn test_search_similar() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);

        // Insert vectors with different seeds
        let e1 = make_embedding(0.0);
        let e2 = make_embedding(0.5);
        let e3 = make_embedding(5.0);
        repo.insert(1, &e1).unwrap();
        repo.insert(2, &e2).unwrap();
        repo.insert(3, &e3).unwrap();

        // Search with a query close to e1
        let query = make_embedding(0.01);
        let results = repo.search_similar(&query, 3).unwrap();
        assert!(!results.is_empty());
        // The closest should be domain_id 1 (seed 0.0 is closest to 0.01)
        assert_eq!(results[0].0, 1);
    }

    #[test]
    fn test_search_similar_with_limit() {
        let (conn, _temp) = setup();
        let repo = VectorRepo::new(&conn);
        for i in 0..5 {
            let embedding = make_embedding(i as f32);
            repo.insert(i + 1, &embedding).unwrap();
        }
        let query = make_embedding(0.01);
        let results = repo.search_similar(&query, 2).unwrap();
        assert_eq!(results.len(), 2);
    }
}
