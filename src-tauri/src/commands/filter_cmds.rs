use crate::db::filter_repo::FilterRepo;
use crate::db::init;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FilterRequest {
    pub task_id: String,
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterResult {
    pub items: Vec<crate::db::filter_repo::FilteredResult>,
    pub total: i64,
}

#[tauri::command]
pub fn filter_exact(request: FilterRequest) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = FilterRepo::new(&conn);

    // Exact match: query scan_items table directly
    let mut stmt = conn.prepare(
        "SELECT id FROM scan_items WHERE task_id = ?1 AND domain = ?2 LIMIT 1000"
    ).map_err(|e| e.to_string())?;

    let items: Vec<crate::db::filter_repo::FilteredResult> = stmt
        .query_map(rusqlite::params![request.task_id, request.query], |_row| {
            // Placeholder - would construct FilteredResult from matched items
            Ok(())
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .map(|_| crate::db::filter_repo::FilteredResult {
            id: 0,
            task_id: request.task_id.clone(),
            domain: request.query.clone(),
            filter_type: "exact".to_string(),
            filter_pattern: Some(request.query.clone()),
            is_matched: true,
            score: None,
            embedding_id: None,
        })
        .collect();

    let result = FilterResult {
        total: items.len() as i64,
        items,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn filter_fuzzy(request: FilterRequest) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = FilterRepo::new(&conn);

    // Fuzzy match: LIKE query
    let pattern = format!("%{}%", request.query.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = conn.prepare(
        "SELECT id, task_id, domain FROM scan_items WHERE task_id = ?1 AND domain LIKE ?2 ESCAPE '\\' LIMIT 1000"
    ).map_err(|e| e.to_string())?;

    let items: Vec<crate::db::filter_repo::FilteredResult> = stmt
        .query_map(rusqlite::params![request.task_id, pattern], |row| {
            Ok(crate::db::filter_repo::FilteredResult {
                id: row.get(0)?,
                task_id: row.get(1)?,
                domain: row.get(2)?,
                filter_type: "fuzzy".to_string(),
                filter_pattern: Some(request.query.clone()),
                is_matched: true,
                score: None,
                embedding_id: None,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Save filter results
    if !items.is_empty() {
        repo.batch_insert(&items).map_err(|e| e.to_string())?;
    }

    let result = FilterResult {
        total: items.len() as i64,
        items,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn filter_regex(request: FilterRequest) -> Result<String, String> {
    let conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;
    let repo = FilterRepo::new(&conn);

    // Regex match: use SQLite REGEXP (requires extension)
    // For now, do client-side regex matching
    let re = regex_lite::Regex::new(&request.query)
        .map_err(|e| format!("Invalid regex: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, task_id, domain FROM scan_items WHERE task_id = ?1 LIMIT 10000"
    ).map_err(|e| e.to_string())?;

    let items: Vec<crate::db::filter_repo::FilteredResult> = stmt
        .query_map(rusqlite::params![request.task_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|(_, _, domain)| re.is_match(domain))
        .map(|(id, task_id, domain)| crate::db::filter_repo::FilteredResult {
            id,
            task_id,
            domain,
            filter_type: "regex".to_string(),
            filter_pattern: Some(request.query.clone()),
            is_matched: true,
            score: None,
            embedding_id: None,
        })
        .collect();

    if !items.is_empty() {
        repo.batch_insert(&items).map_err(|e| e.to_string())?;
    }

    let result = FilterResult {
        total: items.len() as i64,
        items,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct SemanticFilterRequest {
    pub task_id: String,
    pub description: String,
    pub similarity_threshold: Option<f32>,
    pub limit: Option<i64>,
}

#[tauri::command]
pub fn filter_semantic(request: SemanticFilterRequest) -> Result<String, String> {
    let _conn = init::open_and_init(":memory:").map_err(|e| e.to_string())?;

    // Semantic filtering requires embedding generation and vector search
    // Full implementation would:
    // 1. Generate embedding for the description
    // 2. Search sqlite-vec for similar vectors
    // 3. Return matching scan items

    let result = FilterResult {
        items: vec![],
        total: 0,
    };

    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_exact_returns_empty() {
        let req = FilterRequest {
            task_id: "nonexistent".to_string(),
            query: "test".to_string(),
        };
        let result = filter_exact(req).unwrap();
        let parsed: FilterResult = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.total, 0);
    }
}
