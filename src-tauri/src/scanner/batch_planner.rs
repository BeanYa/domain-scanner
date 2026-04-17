use crate::models::scan_batch::{
    make_scan_batch_id, ScanBatch, ScanBatchStatus, DEFAULT_SCAN_BATCH_SIZE, LOCAL_WORKER_ID,
};

pub fn plan_scan_batches(
    task_id: &str,
    run_id: &str,
    total_count: i64,
    max_batch_size: i64,
) -> Vec<ScanBatch> {
    if total_count <= 0 {
        return Vec::new();
    }

    let max_batch_size = max_batch_size.max(1);
    let batch_count = (total_count + max_batch_size - 1) / max_batch_size;
    let now = chrono::Utc::now().to_rfc3339();

    (0..batch_count)
        .map(|batch_index| {
            let start_index = batch_index * max_batch_size;
            let end_index = (start_index + max_batch_size).min(total_count);
            ScanBatch {
                id: make_scan_batch_id(run_id, batch_index),
                task_id: task_id.to_string(),
                run_id: run_id.to_string(),
                batch_index,
                start_index,
                end_index,
                request_count: end_index - start_index,
                status: ScanBatchStatus::Queued,
                worker_id: Some(LOCAL_WORKER_ID.to_string()),
                attempt: 0,
                completed_count: 0,
                available_count: 0,
                error_count: 0,
                result_cursor: 0,
                log_cursor: 0,
                lease_expires_at: None,
                created_at: now.clone(),
                updated_at: now.clone(),
            }
        })
        .collect()
}

pub fn batch_index_for_item(item_index: i64, max_batch_size: i64) -> i64 {
    item_index.max(0) / max_batch_size.max(1)
}

pub fn default_scan_batch_size() -> i64 {
    DEFAULT_SCAN_BATCH_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_exact_batches() {
        let batches = plan_scan_batches("task", "run", 1000, 100);
        assert_eq!(batches.len(), 10);
        assert_eq!(batches[0].start_index, 0);
        assert_eq!(batches[9].end_index, 1000);
    }

    #[test]
    fn test_plan_partial_last_batch() {
        let batches = plan_scan_batches("task", "run", 1001, 100);
        assert_eq!(batches.len(), 11);
        assert_eq!(batches[10].start_index, 1000);
        assert_eq!(batches[10].end_index, 1001);
        assert_eq!(batches[10].request_count, 1);
    }

    #[test]
    fn test_plan_small_total() {
        let batches = plan_scan_batches("task", "run", 1, 100);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].end_index, 1);
    }

    #[test]
    fn test_plan_batch_size_one() {
        let batches = plan_scan_batches("task", "run", 3, 1);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[2].start_index, 2);
        assert_eq!(batches[2].end_index, 3);
    }

    #[test]
    fn test_plan_no_empty_batches() {
        let batches = plan_scan_batches("task", "run", 0, 100);
        assert!(batches.is_empty());
    }
}
