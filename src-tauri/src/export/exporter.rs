/// Stream exporter for scan results
/// Supports JSON, CSV, and TXT formats with streaming file writes
use crate::models::scan_item::ScanItem;
use std::io::Write;
use std::path::Path;

/// Export format
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "csv" => Some(ExportFormat::Csv),
            "txt" => Some(ExportFormat::Txt),
            _ => None,
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Txt => "txt",
        }
    }
}

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_unavailable: bool,
    pub include_errors: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_unavailable: false,
            include_errors: false,
        }
    }
}

/// Export scan items to a file in streaming fashion
pub fn export_items<W: Write>(
    writer: &mut W,
    items: &[ScanItem],
    options: &ExportOptions,
) -> Result<(), String> {
    let filtered: Vec<&ScanItem> = items
        .iter()
        .filter(|item| {
            if item.is_available.unwrap_or(false) {
                return true;
            }
            if options.include_unavailable {
                return true;
            }
            if options.include_errors && item.error_message.is_some() {
                return true;
            }
            false
        })
        .collect();

    match options.format {
        ExportFormat::Json => export_json(writer, &filtered),
        ExportFormat::Csv => export_csv(writer, &filtered),
        ExportFormat::Txt => export_txt(writer, &filtered),
    }
}

fn export_json<W: Write>(writer: &mut W, items: &[&ScanItem]) -> Result<(), String> {
    writeln!(writer, "[").map_err(|e| format!("Write error: {}", e))?;
    for (i, item) in items.iter().enumerate() {
        let json = serde_json::to_string(item).map_err(|e| format!("Serialize error: {}", e))?;
        if i < items.len() - 1 {
            writeln!(writer, "  {},", json).map_err(|e| format!("Write error: {}", e))?;
        } else {
            writeln!(writer, "  {}", json).map_err(|e| format!("Write error: {}", e))?;
        }
    }
    writeln!(writer, "]").map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

fn export_csv<W: Write>(writer: &mut W, items: &[&ScanItem]) -> Result<(), String> {
    writeln!(
        writer,
        "domain,tld,status,is_available,query_method,response_time_ms,error_message"
    )
    .map_err(|e| format!("Write error: {}", e))?;
    for item in items {
        let status = match &item.status {
            crate::models::scan_item::ScanItemStatus::Pending => "pending",
            crate::models::scan_item::ScanItemStatus::Checking => "checking",
            crate::models::scan_item::ScanItemStatus::Available => "available",
            crate::models::scan_item::ScanItemStatus::Unavailable => "unavailable",
            crate::models::scan_item::ScanItemStatus::Error => "error",
        };
        let is_available = item.is_available.unwrap_or(false).to_string();
        let query_method = item.query_method.as_deref().unwrap_or("");
        let response_time = item
            .response_time_ms
            .map(|t| t.to_string())
            .unwrap_or_default();
        let error = item.error_message.as_deref().unwrap_or("");

        writeln!(
            writer,
            "{},{},{},{},{},{},{}",
            item.domain, item.tld, status, is_available, query_method, response_time, error
        )
        .map_err(|e| format!("Write error: {}", e))?;
    }
    Ok(())
}

fn export_txt<W: Write>(writer: &mut W, items: &[&ScanItem]) -> Result<(), String> {
    for item in items {
        if item.is_available.unwrap_or(false) {
            writeln!(writer, "{}", item.domain).map_err(|e| format!("Write error: {}", e))?;
        }
    }
    Ok(())
}

/// Export only available domains (simplified TXT format)
pub fn export_available_domains<W: Write>(
    writer: &mut W,
    items: &[ScanItem],
) -> Result<(), String> {
    for item in items {
        if item.is_available.unwrap_or(false) {
            writeln!(writer, "{}", item.domain).map_err(|e| format!("Write error: {}", e))?;
        }
    }
    Ok(())
}

/// Get the expected file path for an export
pub fn get_export_path(base_dir: &str, task_name: &str, format: &ExportFormat) -> String {
    let safe_name: String = task_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    Path::new(base_dir)
        .join(format!("{}_export.{}", safe_name, format.file_extension()))
        .to_string_lossy()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::scan_item::ScanItemStatus;

    fn make_test_item(domain: &str, available: bool, error: Option<&str>) -> ScanItem {
        ScanItem {
            id: 0,
            task_id: "task1".to_string(),
            run_id: "run1".to_string(),
            domain: domain.to_string(),
            tld: ".com".to_string(),
            item_index: 0,
            status: if error.is_some() {
                ScanItemStatus::Error
            } else if available {
                ScanItemStatus::Available
            } else {
                ScanItemStatus::Unavailable
            },
            is_available: Some(available),
            query_method: Some("rdap".to_string()),
            response_time_ms: Some(100),
            error_message: error.map(|e| e.to_string()),
            checked_at: Some("2026-01-01T00:00:00".to_string()),
        }
    }

    #[test]
    fn test_export_format_from_str() {
        assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("CSV"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_str("txt"), Some(ExportFormat::Txt));
        assert_eq!(ExportFormat::from_str("xml"), None);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.file_extension(), "json");
        assert_eq!(ExportFormat::Csv.file_extension(), "csv");
        assert_eq!(ExportFormat::Txt.file_extension(), "txt");
    }

    #[test]
    fn test_export_json() {
        let items = vec![
            make_test_item("test.com", true, None),
            make_test_item("demo.com", true, None),
        ];
        let mut buf = Vec::new();
        let options = ExportOptions::default();
        export_items(&mut buf, &items, &options).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.starts_with('['));
        assert!(output.contains("test.com"));
        assert!(output.contains("demo.com"));
    }

    #[test]
    fn test_export_csv() {
        let items = vec![make_test_item("test.com", true, None)];
        let mut buf = Vec::new();
        let options = ExportOptions {
            format: ExportFormat::Csv,
            ..Default::default()
        };
        export_items(&mut buf, &items, &options).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("domain,tld,status"));
        assert!(output.contains("test.com"));
    }

    #[test]
    fn test_export_txt_only_available() {
        let items = vec![
            make_test_item("available.com", true, None),
            make_test_item("taken.com", false, None),
        ];
        let mut buf = Vec::new();
        let options = ExportOptions {
            format: ExportFormat::Txt,
            ..Default::default()
        };
        export_items(&mut buf, &items, &options).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("available.com"));
        assert!(!output.contains("taken.com"));
    }

    #[test]
    fn test_export_with_errors_included() {
        let items = vec![
            make_test_item("ok.com", true, None),
            make_test_item("taken.com", false, None),
            make_test_item("err.com", false, Some("timeout")),
        ];
        let mut buf = Vec::new();
        let options = ExportOptions {
            format: ExportFormat::Txt,
            include_unavailable: true,
            include_errors: true,
        };
        export_items(&mut buf, &items, &options).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("ok.com"));
    }

    #[test]
    fn test_export_available_domains() {
        let items = vec![
            make_test_item("available.com", true, None),
            make_test_item("taken.com", false, None),
        ];
        let mut buf = Vec::new();
        export_available_domains(&mut buf, &items).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("available.com"));
        assert!(!output.contains("taken.com"));
    }

    #[test]
    fn test_get_export_path() {
        let path = get_export_path("/tmp", "my-task", &ExportFormat::Json);
        assert!(path.contains("my-task_export.json"));
    }

    #[test]
    fn test_get_export_path_sanitizes_name() {
        let path = get_export_path("/tmp", "my task\\test", &ExportFormat::Csv);
        // On Windows, path separator is \, on Unix it's /
        // The name should have special chars replaced with _
        assert!(
            path.contains("my_task_test_export.csv") || path.contains("my_task\\test_export.csv")
        );
        assert!(path.ends_with(".csv"));
    }
}
