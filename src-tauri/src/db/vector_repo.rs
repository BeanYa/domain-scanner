/// Vector repository for sqlite-vec integration
/// Provides CRUD operations for domain embedding vectors and similarity search

const EMBEDDING_DIM: usize = 384;

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

    /// Delete a vector by domain_id
    pub fn delete(&self, domain_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM domain_vectors WHERE domain_id = ?1",
            [domain_id],
        )?;
        Ok(())
    }

    /// Count total vectors
    pub fn count(&self) -> Result<i64, rusqlite::Error> {
        self.conn
            .query_row("SELECT COUNT(*) FROM domain_vectors", [], |row| row.get(0))
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
