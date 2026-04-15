use domain_scanner_app_lib::db::batch_repo::BatchRepo;
use domain_scanner_app_lib::db::init;
use domain_scanner_app_lib::db::scan_item_repo::ScanItemRepo;
use domain_scanner_app_lib::db::task_repo::TaskRepo;
use domain_scanner_app_lib::models::task::{ScanMode, Task, TaskBatch, TaskStatus};
use domain_scanner_app_lib::scanner::signature::generate_signature;

/// Test the complete task lifecycle: create -> start -> pause -> resume -> complete
#[test]
fn test_task_lifecycle() {
    init::register_vec_extension();
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init::init_database(&conn).unwrap();

    let repo = TaskRepo::new(&conn);

    // Create
    let task = Task {
        id: "task-1".to_string(),
        batch_id: None,
        name: "Test Task".to_string(),
        signature: generate_signature(
            &ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            &vec![".com".to_string()],
        ),
        status: TaskStatus::Pending,
        scan_mode: ScanMode::Regex {
            pattern: "^[a-z]{3}$".to_string(),
        },
        config_json: "{}".to_string(),
        tlds: vec![".com".to_string()],
        prefix_pattern: Some("^[a-z]{3}$".to_string()),
        concurrency: 50,
        proxy_id: None,
        total_count: 17576,
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01T00:00:00".to_string(),
        updated_at: "2026-01-01T00:00:00".to_string(),
    };
    repo.create(&task).unwrap();

    // Verify creation
    let fetched = repo.get_by_id("task-1").unwrap().unwrap();
    assert_eq!(fetched.status, TaskStatus::Pending);
    assert_eq!(fetched.tlds, vec![".com"]);

    // Start
    assert!(fetched.status.can_transition_to(&TaskStatus::Running));
    repo.update_status("task-1", &TaskStatus::Running).unwrap();
    let fetched = repo.get_by_id("task-1").unwrap().unwrap();
    assert_eq!(fetched.status, TaskStatus::Running);

    // Pause
    assert!(fetched.status.can_transition_to(&TaskStatus::Paused));
    repo.update_status("task-1", &TaskStatus::Paused).unwrap();
    let fetched = repo.get_by_id("task-1").unwrap().unwrap();
    assert_eq!(fetched.status, TaskStatus::Paused);

    // Resume
    assert!(fetched.status.can_transition_to(&TaskStatus::Running));
    repo.update_status("task-1", &TaskStatus::Running).unwrap();

    // Complete
    let fetched = repo.get_by_id("task-1").unwrap().unwrap();
    assert!(fetched.status.can_transition_to(&TaskStatus::Completed));
    repo.update_status("task-1", &TaskStatus::Completed)
        .unwrap();
    let fetched = repo.get_by_id("task-1").unwrap().unwrap();
    assert_eq!(fetched.status, TaskStatus::Completed);

    // Cannot restart completed task
    assert!(!fetched.status.can_transition_to(&TaskStatus::Running));
}

/// Test multi-TLD task creation (single task with multiple TLDs)
#[test]
fn test_multi_tld_task_creation() {
    init::register_vec_extension();
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init::init_database(&conn).unwrap();

    let batch_repo = BatchRepo::new(&conn);
    let task_repo = TaskRepo::new(&conn);

    // Create batch
    let batch = TaskBatch {
        id: "batch-1".to_string(),
        name: "Multi TLD Batch".to_string(),
        task_count: 1,
        created_at: "2026-01-01T00:00:00".to_string(),
    };
    batch_repo.create(&batch).unwrap();

    // Create a single task with multiple TLDs (the new model)
    let scan_mode = ScanMode::Regex {
        pattern: "^[a-z]{3}$".to_string(),
    };
    let tlds = vec![".com".to_string(), ".net".to_string(), ".org".to_string()];
    let sig = generate_signature(&scan_mode, &tlds);

    let task = Task {
        id: "task-multi".to_string(),
        batch_id: Some("batch-1".to_string()),
        name: "3-letter multi-TLD".to_string(),
        signature: sig.clone(),
        status: TaskStatus::Pending,
        scan_mode: scan_mode.clone(),
        config_json: "{}".to_string(),
        tlds: tlds.clone(),
        prefix_pattern: Some("^[a-z]{3}$".to_string()),
        concurrency: 50,
        proxy_id: None,
        total_count: 17576 * 3, // 3 TLDs
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01T00:00:00".to_string(),
        updated_at: "2026-01-01T00:00:00".to_string(),
    };
    task_repo.create(&task).unwrap();

    // Verify: one task with 3 TLDs
    let fetched = task_repo.get_by_id("task-multi").unwrap().unwrap();
    assert_eq!(fetched.tlds.len(), 3);
    assert_eq!(fetched.total_count, 17576 * 3);

    // Verify dedup: same signature should be rejected
    let dup_result = task_repo.create(&task);
    assert!(
        dup_result.is_err(),
        "Duplicate signature should be rejected"
    );

    // Verify order-independence: [.net,.com] produces same signature as [.com,.net]
    let sig_reversed = generate_signature(
        &scan_mode,
        &vec![".net".to_string(), ".com".to_string(), ".org".to_string()],
    );
    assert_eq!(sig, sig_reversed, "TLD order should not affect signature");
}

/// Test checkpoint resume via completed_index
#[test]
fn test_checkpoint_resume() {
    init::register_vec_extension();
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init::init_database(&conn).unwrap();

    let repo = TaskRepo::new(&conn);

    let task = Task {
        id: "task-resume".to_string(),
        batch_id: None,
        name: "Resume Test".to_string(),
        signature: "sig-resume".to_string(),
        status: TaskStatus::Running,
        scan_mode: ScanMode::Regex {
            pattern: "^[a-z]{2}$".to_string(),
        },
        config_json: "{}".to_string(),
        tlds: vec![".com".to_string()],
        prefix_pattern: Some("^[a-z]{2}$".to_string()),
        concurrency: 50,
        proxy_id: None,
        total_count: 676,
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01T00:00:00".to_string(),
        updated_at: "2026-01-01T00:00:00".to_string(),
    };
    repo.create(&task).unwrap();

    // Simulate scanning 100 items
    repo.update_progress("task-resume", 100, 100, 30, 5)
        .unwrap();

    // Pause
    repo.update_status("task-resume", &TaskStatus::Paused)
        .unwrap();

    // Verify checkpoint
    let fetched = repo.get_by_id("task-resume").unwrap().unwrap();
    assert_eq!(fetched.completed_index, 100);
    assert_eq!(fetched.available_count, 30);

    // Resume from checkpoint
    repo.update_status("task-resume", &TaskStatus::Running)
        .unwrap();

    // Continue scanning from index 100
    repo.update_progress("task-resume", 200, 200, 55, 8)
        .unwrap();

    let fetched = repo.get_by_id("task-resume").unwrap().unwrap();
    assert_eq!(fetched.completed_index, 200);
    assert_eq!(fetched.available_count, 55);
}

/// Test batch operations (pause/resume all tasks in a batch)
#[test]
fn test_batch_operations() {
    init::register_vec_extension();
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init::init_database(&conn).unwrap();

    let task_repo = TaskRepo::new(&conn);
    let batch_repo = BatchRepo::new(&conn);

    // Create batch
    let batch = TaskBatch {
        id: "batch-op".to_string(),
        name: "Ops Batch".to_string(),
        task_count: 2,
        created_at: "2026-01-01T00:00:00".to_string(),
    };
    batch_repo.create(&batch).unwrap();

    // Create tasks (each can have different TLD sets)
    for i in 0..2 {
        let task = Task {
            id: format!("task-op-{}", i),
            batch_id: Some("batch-op".to_string()),
            name: format!("Task {}", i),
            signature: format!("sig-op-{}", i),
            status: TaskStatus::Running,
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{2}$".to_string(),
            },
            config_json: "{}".to_string(),
            tlds: vec![format!(".tld{}", i)],
            prefix_pattern: None,
            concurrency: 50,
            proxy_id: None,
            total_count: 676,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: "2026-01-01T00:00:00".to_string(),
            updated_at: "2026-01-01T00:00:00".to_string(),
        };
        task_repo.create(&task).unwrap();
    }

    // Batch pause
    let tasks = task_repo.list(None, Some("batch-op"), 100, 0).unwrap();
    for task in &tasks {
        if task.status == TaskStatus::Running {
            task_repo
                .update_status(&task.id, &TaskStatus::Paused)
                .unwrap();
        }
    }

    // Verify all paused
    let tasks = task_repo.list(None, Some("batch-op"), 100, 0).unwrap();
    assert!(tasks.iter().all(|t| t.status == TaskStatus::Paused));

    // Batch resume
    for task in &tasks {
        if task.status == TaskStatus::Paused {
            task_repo
                .update_status(&task.id, &TaskStatus::Running)
                .unwrap();
        }
    }

    // Verify all running
    let tasks = task_repo.list(None, Some("batch-op"), 100, 0).unwrap();
    assert!(tasks.iter().all(|t| t.status == TaskStatus::Running));
}

/// Test scan items with batch write and pagination
#[test]
fn test_scan_items_batch_and_pagination() {
    init::register_vec_extension();
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    init::init_database(&conn).unwrap();

    // Create task first
    let task_repo = TaskRepo::new(&conn);
    let task = Task {
        id: "task-items".to_string(),
        batch_id: None,
        name: "Items Test".to_string(),
        signature: "sig-items".to_string(),
        status: TaskStatus::Running,
        scan_mode: ScanMode::Regex {
            pattern: "^[a-z]{2}$".to_string(),
        },
        config_json: "{}".to_string(),
        tlds: vec![".com".to_string()],
        prefix_pattern: None,
        concurrency: 50,
        proxy_id: None,
        total_count: 676,
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01T00:00:00".to_string(),
        updated_at: "2026-01-01T00:00:00".to_string(),
    };
    task_repo.create(&task).unwrap();

    // Batch insert scan items
    let item_repo = ScanItemRepo::new(&conn);
    let items: Vec<domain_scanner_app_lib::models::scan_item::ScanItem> = (0..100)
        .map(|i| domain_scanner_app_lib::models::scan_item::ScanItem {
            id: 0,
            task_id: "task-items".to_string(),
            run_id: "run-1".to_string(),
            domain: format!(
                "{}{}.com",
                (b'a' + (i % 26) as u8) as char,
                (b'a' + ((i / 26) % 26) as u8) as char
            ),
            tld: ".com".to_string(),
            item_index: i,
            status: if i % 10 == 0 {
                domain_scanner_app_lib::models::scan_item::ScanItemStatus::Available
            } else {
                domain_scanner_app_lib::models::scan_item::ScanItemStatus::Unavailable
            },
            is_available: Some(i % 10 == 0),
            query_method: Some("rdap".to_string()),
            response_time_ms: Some(100 + i as i64),
            error_message: None,
            checked_at: Some("2026-01-01T00:00:00".to_string()),
        })
        .collect();

    item_repo.batch_insert(&items).unwrap();

    // Pagination
    let page1 = item_repo
        .list_by_task("task-items", Some("run-1"), None, 20, 0)
        .unwrap();
    assert_eq!(page1.len(), 20);

    let page2 = item_repo
        .list_by_task("task-items", Some("run-1"), None, 20, 20)
        .unwrap();
    assert_eq!(page2.len(), 20);

    // Count
    let count = item_repo
        .count_by_task("task-items", Some("run-1"), None)
        .unwrap();
    assert_eq!(count, 100);
}
