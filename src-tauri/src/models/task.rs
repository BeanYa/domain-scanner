use serde::{Deserialize, Serialize};

/// Task status state machine:
/// pending -> running -> paused -> running (resume)
///                  \-> completed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Paused,
    Completed,
}

impl TaskStatus {
    pub fn can_transition_to(&self, target: &TaskStatus) -> bool {
        match (self, target) {
            (TaskStatus::Pending, TaskStatus::Running) => true,
            (TaskStatus::Running, TaskStatus::Paused) => true,
            (TaskStatus::Running, TaskStatus::Completed) => true,
            (TaskStatus::Paused, TaskStatus::Running) => true,
            _ => false,
        }
    }
}

/// Scan mode determines how domain candidates are generated
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScanMode {
    Regex { pattern: String },
    Wildcard { pattern: String },
    Llm { config_id: String, prompt: String },
    Manual { domains: Vec<String> },
}

/// Core task model: 1 task = n prefixes x 1 TLD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub batch_id: Option<String>,
    pub name: String,
    pub signature: String,
    pub status: TaskStatus,
    pub scan_mode: ScanMode,
    pub config_json: String,
    pub tld: String,
    pub prefix_pattern: Option<String>,
    pub total_count: i64,
    pub completed_count: i64,
    pub completed_index: i64,
    pub available_count: i64,
    pub error_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Batch model: groups tasks created together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBatch {
    pub id: String,
    pub name: String,
    pub task_count: i64,
    pub created_at: String,
}

/// Batch creation result with dedup info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateResult {
    pub created: u32,
    pub skipped: u32,
    pub task_ids: Vec<String>,
    pub skipped_tlds: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_transitions() {
        assert!(TaskStatus::Pending.can_transition_to(&TaskStatus::Running));
        assert!(TaskStatus::Running.can_transition_to(&TaskStatus::Paused));
        assert!(TaskStatus::Running.can_transition_to(&TaskStatus::Completed));
        assert!(TaskStatus::Paused.can_transition_to(&TaskStatus::Running));

        assert!(!TaskStatus::Pending.can_transition_to(&TaskStatus::Completed));
        assert!(!TaskStatus::Pending.can_transition_to(&TaskStatus::Paused));
        assert!(!TaskStatus::Completed.can_transition_to(&TaskStatus::Running));
        assert!(!TaskStatus::Paused.can_transition_to(&TaskStatus::Completed));
    }

    #[test]
    fn test_scan_mode_serialization() {
        let regex_mode = ScanMode::Regex {
            pattern: "^[a-z]{4}$".to_string(),
        };
        let json = serde_json::to_string(&regex_mode).unwrap();
        let deserialized: ScanMode = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, ScanMode::Regex { .. }));

        let llm_mode = ScanMode::Llm {
            config_id: "cfg1".to_string(),
            prompt: "AI domains".to_string(),
        };
        let json = serde_json::to_string(&llm_mode).unwrap();
        assert!(json.contains("llm"));
    }

    #[test]
    fn test_task_serialization_roundtrip() {
        let task = Task {
            id: "test-id".to_string(),
            batch_id: Some("batch-1".to_string()),
            name: "Test Task".to_string(),
            signature: "abc123".to_string(),
            status: TaskStatus::Pending,
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{3}$".to_string(),
            },
            config_json: "{}".to_string(),
            tld: ".com".to_string(),
            prefix_pattern: Some("3-letter".to_string()),
            total_count: 17576,
            completed_count: 0,
            completed_index: 0,
            available_count: 0,
            error_count: 0,
            created_at: "2026-01-01T00:00:00".to_string(),
            updated_at: "2026-01-01T00:00:00".to_string(),
        };

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.tld, deserialized.tld);
        assert_eq!(task.status, deserialized.status);
    }
}
