use crate::models::scan_batch::{ScanBatch, ScanBatchStatus, ScanBatchSummary};

pub struct ScanBatchRepo<'a> {
    pub conn: &'a rusqlite::Connection,
}

impl<'a> ScanBatchRepo<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self { conn }
    }

    pub fn create_many(&self, batches: &[ScanBatch]) -> Result<usize, rusqlite::Error> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0usize;
        for batch in batches {
            tx.execute(
                "INSERT OR IGNORE INTO scan_batches (
                    id, task_id, run_id, batch_index, start_index, end_index, request_count,
                    status, worker_id, attempt, completed_count, available_count, error_count,
                    result_cursor, log_cursor, lease_expires_at, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                rusqlite::params![
                    batch.id,
                    batch.task_id,
                    batch.run_id,
                    batch.batch_index,
                    batch.start_index,
                    batch.end_index,
                    batch.request_count,
                    batch.status.as_str(),
                    batch.worker_id,
                    batch.attempt,
                    batch.completed_count,
                    batch.available_count,
                    batch.error_count,
                    batch.result_cursor,
                    batch.log_cursor,
                    batch.lease_expires_at,
                    batch.created_at,
                    batch.updated_at,
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn list_by_run(
        &self,
        task_id: &str,
        run_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ScanBatch>, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT id, task_id, run_id, batch_index, start_index, end_index, request_count,
                    status, worker_id, attempt, completed_count, available_count, error_count,
                    result_cursor, log_cursor, lease_expires_at, created_at, updated_at
             FROM scan_batches WHERE task_id = ?1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(task_id.to_string())];
        if let Some(run_id) = run_id {
            sql.push_str(" AND run_id = ?");
            params.push(Box::new(run_id.to_string()));
        }
        sql.push_str(" ORDER BY batch_index ASC LIMIT ? OFFSET ?");
        params.push(Box::new(limit));
        params.push(Box::new(offset));
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let batches = stmt
            .query_map(refs.as_slice(), |row| Self::row_to_batch(row))?
            .filter_map(|row| row.ok())
            .collect();
        Ok(batches)
    }

    pub fn summarize(
        &self,
        task_id: &str,
        run_id: Option<&str>,
    ) -> Result<ScanBatchSummary, rusqlite::Error> {
        let mut sql = String::from(
            "SELECT
                COUNT(*),
                SUM(CASE WHEN status = 'queued' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'assigned' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'running' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'succeeded' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'retrying' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'paused' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'cancelled' THEN 1 ELSE 0 END),
                SUM(CASE WHEN status = 'expired' THEN 1 ELSE 0 END),
                COALESCE(SUM(completed_count), 0),
                COALESCE(SUM(available_count), 0),
                COALESCE(SUM(error_count), 0),
                COUNT(DISTINCT worker_id)
             FROM scan_batches WHERE task_id = ?1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(task_id.to_string())];
        if let Some(run_id) = run_id {
            sql.push_str(" AND run_id = ?");
            params.push(Box::new(run_id.to_string()));
        }
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        self.conn.query_row(&sql, refs.as_slice(), |row| {
            Ok(ScanBatchSummary {
                total: row.get(0)?,
                queued: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                assigned: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                running: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                succeeded: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                failed: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                retrying: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                paused: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                cancelled: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                expired: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                completed_count: row.get(10)?,
                available_count: row.get(11)?,
                error_count: row.get(12)?,
                worker_count: row.get(13)?,
            })
        })
    }

    pub fn update_local_progress(
        &self,
        task_id: &str,
        run_id: &str,
        completed_index: i64,
        terminal_status: Option<ScanBatchStatus>,
    ) -> Result<(), rusqlite::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        let terminal_status = terminal_status.map(|s| s.as_str().to_string());
        self.conn.execute(
            "UPDATE scan_batches
             SET completed_count =
                   CASE
                     WHEN ?3 >= end_index THEN request_count
                     WHEN ?3 <= start_index THEN 0
                     ELSE ?3 - start_index
                   END,
                 available_count = (
                   SELECT COUNT(*) FROM scan_items
                   WHERE scan_items.task_id = scan_batches.task_id
                     AND scan_items.run_id = scan_batches.run_id
                     AND scan_items.item_index >= scan_batches.start_index
                     AND scan_items.item_index < scan_batches.end_index
                     AND scan_items.status = '\"available\"'
                 ),
                 error_count = (
                   SELECT COUNT(*) FROM scan_items
                   WHERE scan_items.task_id = scan_batches.task_id
                     AND scan_items.run_id = scan_batches.run_id
                     AND scan_items.item_index >= scan_batches.start_index
                     AND scan_items.item_index < scan_batches.end_index
                     AND scan_items.status = '\"error\"'
                 ),
                 status =
                   CASE
                     WHEN ?4 IS NOT NULL AND ?3 < end_index AND ?3 > start_index THEN ?4
                     WHEN ?4 IS NOT NULL AND ?3 <= start_index THEN ?4
                     WHEN ?3 >= end_index THEN 'succeeded'
                     WHEN ?3 > start_index THEN 'running'
                     ELSE status
                   END,
                 updated_at = ?5
             WHERE task_id = ?1 AND run_id = ?2",
            rusqlite::params![task_id, run_id, completed_index, terminal_status, now],
        )?;
        Ok(())
    }

    pub fn active_count_for_worker(&self, worker_id: &str) -> Result<i64, rusqlite::Error> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM scan_batches
             WHERE worker_id = ?1 AND status IN ('assigned', 'running', 'retrying')",
            [worker_id],
            |row| row.get(0),
        )
    }

    pub fn delete_by_task(&self, task_id: &str) -> Result<(), rusqlite::Error> {
        self.conn
            .execute("DELETE FROM scan_batches WHERE task_id = ?1", [task_id])?;
        Ok(())
    }

    fn row_to_batch(row: &rusqlite::Row) -> Result<ScanBatch, rusqlite::Error> {
        let status: String = row.get(7)?;
        Ok(ScanBatch {
            id: row.get(0)?,
            task_id: row.get(1)?,
            run_id: row.get(2)?,
            batch_index: row.get(3)?,
            start_index: row.get(4)?,
            end_index: row.get(5)?,
            request_count: row.get(6)?,
            status: ScanBatchStatus::from_db(&status),
            worker_id: row.get(8)?,
            attempt: row.get(9)?,
            completed_count: row.get(10)?,
            available_count: row.get(11)?,
            error_count: row.get(12)?,
            result_cursor: row.get(13)?,
            log_cursor: row.get(14)?,
            lease_expires_at: row.get(15)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        })
    }
}
