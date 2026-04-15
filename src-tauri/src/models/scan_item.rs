use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScanItemStatus {
    Pending,
    Checking,
    Available,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: i64,
    pub task_id: String,
    pub run_id: String,
    pub domain: String,
    pub tld: String,
    pub item_index: i64,
    pub status: ScanItemStatus,
    pub is_available: Option<bool>,
    pub query_method: Option<String>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub checked_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_item_status_serialization() {
        let status = ScanItemStatus::Available;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"available\"");

        let deserialized: ScanItemStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_scan_item_serialization() {
        let item = ScanItem {
            id: 1,
            task_id: "task-1".to_string(),
            run_id: "run-1".to_string(),
            domain: "test.com".to_string(),
            tld: ".com".to_string(),
            item_index: 0,
            status: ScanItemStatus::Available,
            is_available: Some(true),
            query_method: Some("rdap".to_string()),
            response_time_ms: Some(150),
            error_message: None,
            checked_at: Some("2026-01-01T00:00:00".to_string()),
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: ScanItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.domain, deserialized.domain);
        assert_eq!(item.is_available, deserialized.is_available);
    }
}
