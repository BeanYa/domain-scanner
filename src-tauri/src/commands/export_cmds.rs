use crate::db::scan_item_repo::ScanItemRepo;
use crate::db::init;
use crate::export::exporter::{self, ExportFormat, ExportOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub task_id: String,
    pub format: String,
    pub output_path: String,
    pub include_unavailable: Option<bool>,
    pub include_errors: Option<bool>,
}

#[tauri::command]
pub fn export_results(request: ExportRequest) -> Result<(), String> {
    let format = ExportFormat::from_str(&request.format)
        .ok_or_else(|| format!("Unsupported export format: {}", request.format))?;

    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = ScanItemRepo::new(&conn);

    // Fetch all items for the task
    let mut all_items = Vec::new();
    let mut offset = 0i64;
    let batch_size = 1000i64;
    loop {
        let items = repo.list_by_task(&request.task_id, None, batch_size, offset)
            .map_err(|e| e.to_string())?;
        if items.is_empty() {
            break;
        }
        all_items.extend(items);
        offset += batch_size;
    }

    let options = ExportOptions {
        format,
        include_unavailable: request.include_unavailable.unwrap_or(false),
        include_errors: request.include_errors.unwrap_or(false),
    };

    let mut file = std::fs::File::create(&request.output_path)
        .map_err(|e| format!("Cannot create output file: {}", e))?;

    exporter::export_items(&mut file, &all_items, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::scan_item::ScanItemStatus;

    fn make_test_item(domain: &str, available: bool) -> ScanItem {
        ScanItem {
            id: 1,
            task_id: "task1".to_string(),
            domain: domain.to_string(),
            tld: ".com".to_string(),
            item_index: 0,
            status: ScanItemStatus::Available,
            is_available: Some(available),
            query_method: Some("rdap".to_string()),
            response_time_ms: Some(100),
            error_message: None,
            checked_at: Some("2026-01-01T00:00:00".to_string()),
        }
    }

    #[test]
    fn test_export_format_validation() {
        assert!(ExportFormat::from_str("json").is_some());
        assert!(ExportFormat::from_str("csv").is_some());
        assert!(ExportFormat::from_str("txt").is_some());
        assert!(ExportFormat::from_str("xml").is_none());
    }

    #[test]
    fn test_export_json_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let output_path = temp_dir.path().join("export.json").to_string_lossy().to_string();

        let items = vec![
            make_test_item("test.com", true),
            make_test_item("demo.com", true),
        ];

        let options = ExportOptions::default();
        let mut file = std::fs::File::create(&output_path).unwrap();
        exporter::export_items(&mut file, &items, &options).unwrap();

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("test.com"));
    }
}
