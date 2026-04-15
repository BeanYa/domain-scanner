use domain_scanner_app_lib::db::filter_repo::{FilterRepo, FilteredResult};
use domain_scanner_app_lib::db::init;
use domain_scanner_app_lib::db::log_repo::LogRepo;
use domain_scanner_app_lib::db::scan_item_repo::ScanItemRepo;
use domain_scanner_app_lib::db::task_repo::TaskRepo;
use domain_scanner_app_lib::db::task_run_repo::TaskRunRepo;
use domain_scanner_app_lib::models::scan_item::{ScanItem, ScanItemStatus};
use domain_scanner_app_lib::models::task::{ScanMode, Task, TaskRun, TaskStatus};
use tempfile::NamedTempFile;

fn setup() -> (rusqlite::Connection, NamedTempFile) {
    init::register_vec_extension();
    let temp = NamedTempFile::new().unwrap();
    let conn = rusqlite::Connection::open(temp.path()).unwrap();
    init::init_database(&conn).unwrap();
    (conn, temp)
}

fn make_task() -> Task {
    Task {
        id: "task-1".to_string(),
        batch_id: None,
        name: "Large Task".to_string(),
        signature: "sig-1".to_string(),
        status: TaskStatus::Running,
        scan_mode: ScanMode::Regex {
            pattern: "^[a-z]{5}$".to_string(),
        },
        config_json: "{}".to_string(),
        tlds: vec![".com".to_string(), ".net".to_string()],
        prefix_pattern: Some("^[a-z]{5}$".to_string()),
        concurrency: 200,
        proxy_id: None,
        total_count: 3_000_000,
        completed_count: 0,
        completed_index: 0,
        available_count: 0,
        error_count: 0,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn make_run(id: &str, run_number: i64) -> TaskRun {
    TaskRun {
        id: id.to_string(),
        task_id: "task-1".to_string(),
        run_number,
        status: TaskStatus::Running,
        total_count: 3_000_000,
        completed_count: 0,
        available_count: 0,
        error_count: 0,
        started_at: "2026-01-01T00:00:00Z".to_string(),
        finished_at: None,
    }
}

fn make_item(run_id: &str, item_index: i64) -> ScanItem {
    ScanItem {
        id: 0,
        task_id: "task-1".to_string(),
        run_id: run_id.to_string(),
        domain: format!("domain-{:04}.com", item_index),
        tld: ".com".to_string(),
        item_index,
        status: ScanItemStatus::Available,
        is_available: Some(true),
        query_method: Some("rdap".to_string()),
        response_time_ms: Some(25),
        error_message: None,
        checked_at: Some(format!("2026-01-01T00:00:{:02}Z", item_index % 60)),
    }
}

#[test]
fn large_result_sets_stay_paginated_per_run() {
    let (conn, _temp) = setup();
    let task_repo = TaskRepo::new(&conn);
    let run_repo = TaskRunRepo::new(&conn);
    let scan_repo = ScanItemRepo::new(&conn);

    task_repo.create(&make_task()).unwrap();
    run_repo.create(&make_run("run-1", 1)).unwrap();
    run_repo.create(&make_run("run-2", 2)).unwrap();

    let first_run_items: Vec<ScanItem> = (0..250).map(|index| make_item("run-1", index)).collect();
    let second_run_items: Vec<ScanItem> =
        (250..500).map(|index| make_item("run-2", index)).collect();

    scan_repo.batch_insert(&first_run_items).unwrap();
    scan_repo.batch_insert(&second_run_items).unwrap();

    let page = scan_repo
        .list_by_task("task-1", Some("run-1"), None, 10, 0)
        .unwrap();
    assert_eq!(page.len(), 10);
    assert!(page.iter().all(|item| item.run_id == "run-1"));

    let second_page = scan_repo
        .list_by_task("task-1", Some("run-1"), None, 10, 10)
        .unwrap();
    assert_eq!(second_page.len(), 10);
    assert_ne!(page[0].domain, second_page[0].domain);
}

#[test]
fn deleting_all_task_children_allows_parent_delete() {
    let (conn, _temp) = setup();
    let task_repo = TaskRepo::new(&conn);
    let run_repo = TaskRunRepo::new(&conn);
    let scan_repo = ScanItemRepo::new(&conn);
    let log_repo = LogRepo::new(&conn);
    let filter_repo = FilterRepo::new(&conn);

    task_repo.create(&make_task()).unwrap();
    run_repo.create(&make_run("run-1", 1)).unwrap();
    scan_repo.create(&make_item("run-1", 1)).unwrap();
    log_repo
        .create("task-1", Some("run-1"), "error", "network timeout")
        .unwrap();
    filter_repo
        .create(&FilteredResult {
            id: 0,
            task_id: "task-1".to_string(),
            domain: "domain-0001.com".to_string(),
            filter_type: "exact".to_string(),
            filter_pattern: Some("domain".to_string()),
            is_matched: true,
            score: Some(1.0),
            embedding_id: None,
        })
        .unwrap();

    filter_repo.delete_by_task("task-1").unwrap();
    log_repo.delete_by_task("task-1").unwrap();
    scan_repo.delete_by_task("task-1").unwrap();
    run_repo.delete_by_task("task-1").unwrap();
    task_repo.delete("task-1").unwrap();

    assert!(task_repo.get_by_id("task-1").unwrap().is_none());
    assert!(run_repo.list_by_task("task-1").unwrap().is_empty());
    assert!(scan_repo
        .list_by_task("task-1", Some("run-1"), None, 10, 0)
        .unwrap()
        .is_empty());
}
