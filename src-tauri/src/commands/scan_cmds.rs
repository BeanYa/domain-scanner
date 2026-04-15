use crate::models::task::ScanMode;
use crate::scanner::list_generator::ListGenerator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ScanPreviewRequest {
    pub scan_mode: ScanMode,
    pub tlds: Vec<String>,
    pub sample_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanPreviewResponse {
    pub tlds: Vec<String>,
    pub total_count: i64,
    pub sample_domains: Vec<String>,
}

#[tauri::command]
pub fn scan_preview(request: ScanPreviewRequest) -> Result<String, String> {
    let tld_count = request.tlds.len() as i64;

    match &request.scan_mode {
        ScanMode::Manual { domains } => {
            let count = domains.len() as i64 * tld_count;
            let samples: Vec<String> = domains
                .iter()
                .take(request.sample_count.unwrap_or(10))
                .flat_map(|d| request.tlds.iter().map(move |t| format!("{}{}", d, t)))
                .collect();
            let response = ScanPreviewResponse {
                tlds: request.tlds.clone(),
                total_count: count,
                sample_domains: samples,
            };
            return serde_json::to_string(&response).map_err(|e| e.to_string());
        }
        ScanMode::Regex { pattern: _pattern } | ScanMode::Wildcard { pattern: _pattern } => {
            // For preview, use first TLD to estimate prefix count
            let mut gen = ListGenerator::new(request.scan_mode.clone(), request.tlds.clone())
                .with_batch_size(request.sample_count.unwrap_or(10));

            let total_count = gen.total_count();
            let batch = gen.next_batch();
            let sample_domains: Vec<String> = batch
                .iter()
                .take(request.sample_count.unwrap_or(10))
                .map(|item| item.domain.clone())
                .collect();

            let response = ScanPreviewResponse {
                tlds: request.tlds.clone(),
                total_count,
                sample_domains,
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
        ScanMode::Llm { .. } => {
            let response = ScanPreviewResponse {
                tlds: request.tlds.clone(),
                total_count: 0, // LLM generates candidates dynamically
                sample_domains: vec![],
            };
            serde_json::to_string(&response).map_err(|e| e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_preview_regex_single_tld() {
        let req = ScanPreviewRequest {
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{2}$".to_string(),
            },
            tlds: vec![".com".to_string()],
            sample_count: Some(5),
        };
        let result = scan_preview(req).unwrap();
        let response: ScanPreviewResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(response.tlds, vec![".com"]);
        assert_eq!(response.total_count, 676); // 26^2 * 1
        assert!(!response.sample_domains.is_empty());
    }

    #[test]
    fn test_scan_preview_regex_multi_tld() {
        let req = ScanPreviewRequest {
            scan_mode: ScanMode::Regex {
                pattern: "^[a-z]{2}$".to_string(),
            },
            tlds: vec![".com".to_string(), ".net".to_string()],
            sample_count: Some(5),
        };
        let result = scan_preview(req).unwrap();
        let response: ScanPreviewResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(response.tlds.len(), 2);
        assert_eq!(response.total_count, 1352); // 676 * 2
    }

    #[test]
    fn test_scan_preview_manual_single_tld() {
        let req = ScanPreviewRequest {
            scan_mode: ScanMode::Manual {
                domains: vec!["test".to_string(), "demo".to_string()],
            },
            tlds: vec![".com".to_string()],
            sample_count: Some(10),
        };
        let result = scan_preview(req).unwrap();
        let response: ScanPreviewResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(response.total_count, 2); // 2 prefixes * 1 TLD
        assert!(response.sample_domains.contains(&"test.com".to_string()));
    }

    #[test]
    fn test_scan_preview_manual_multi_tld() {
        let req = ScanPreviewRequest {
            scan_mode: ScanMode::Manual {
                domains: vec!["alpha".to_string()],
            },
            tlds: vec![".com".to_string(), ".net".to_string(), ".org".to_string()],
            sample_count: Some(10),
        };
        let result = scan_preview(req).unwrap();
        let response: ScanPreviewResponse = serde_json::from_str(&result).unwrap();
        assert_eq!(response.total_count, 3); // 1 prefix * 3 TLDs
        assert_eq!(response.sample_domains.len(), 3);
    }
}
