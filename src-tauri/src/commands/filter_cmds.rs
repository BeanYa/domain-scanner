use crate::db::filter_repo::FilterRepo;
use crate::db::init;
use crate::db::llm_repo::LlmRepo;
use crate::db::vector_repo::{VectorRepo, EMBEDDING_DIM};
use crate::llm::client::LlmClient;
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
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let _repo = FilterRepo::new(&conn);

    // Exact match: query scan_items table directly
    let mut stmt = conn
        .prepare("SELECT id FROM scan_items WHERE task_id = ?1 AND domain = ?2 LIMIT 1000")
        .map_err(|e| e.to_string())?;

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
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = FilterRepo::new(&conn);

    // Fuzzy match: LIKE query
    let pattern = format!(
        "%{}%",
        request.query.replace('%', "\\%").replace('_', "\\_")
    );
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
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = FilterRepo::new(&conn);

    // Regex match: use SQLite REGEXP (requires extension)
    // For now, do client-side regex matching
    let re = regex_lite::Regex::new(&request.query).map_err(|e| format!("Invalid regex: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, task_id, domain FROM scan_items WHERE task_id = ?1 LIMIT 10000")
        .map_err(|e| e.to_string())?;

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
        .map(
            |(id, task_id, domain)| crate::db::filter_repo::FilteredResult {
                id,
                task_id,
                domain,
                filter_type: "regex".to_string(),
                filter_pattern: Some(request.query.clone()),
                is_matched: true,
                score: None,
                embedding_id: None,
            },
        )
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
pub async fn filter_semantic(request: SemanticFilterRequest) -> Result<String, String> {
    let config = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = LlmRepo::new(&conn);
        repo.get_default()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| {
                "未配置默认 Embedding API。请先在设置中保存 OpenAI 兼容 embedding 配置并设为默认。"
                    .to_string()
            })?
    };
    if config.embedding_dim as usize != EMBEDDING_DIM {
        return Err(format!(
            "当前向量表固定为 {} 维，但默认配置为 {} 维。请使用支持 dimensions={} 的 embedding 模型，或将配置维度设为 {}。",
            EMBEDDING_DIM, config.embedding_dim, EMBEDDING_DIM, EMBEDDING_DIM
        ));
    }
    if config.embedding_model.is_none() {
        return Err("默认 Embedding 配置缺少 embedding_model，无法执行语义筛选。".to_string());
    }

    let client = LlmClient::new(config);
    let embeddings = client.embed(vec![request.description.clone()]).await?;
    let query_embedding = embeddings
        .first()
        .ok_or_else(|| "Embedding API 没有返回向量。".to_string())?;
    if query_embedding.len() != EMBEDDING_DIM {
        return Err(format!(
            "Embedding API returned {} dimensions, expected {}",
            query_embedding.len(),
            EMBEDDING_DIM
        ));
    }

    let limit = request.limit.unwrap_or(100).clamp(1, 1000);
    let threshold = request.similarity_threshold.unwrap_or(0.0);
    let matches = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let vector_repo = VectorRepo::new(&conn);
        vector_repo
            .search_similar_by_task(&request.task_id, query_embedding, limit)
            .map_err(|e| e.to_string())?
    };

    let items: Vec<crate::db::filter_repo::FilteredResult> = matches
        .into_iter()
        .filter_map(|(scan_item_id, domain, distance)| {
            let score = (1.0_f32 - distance).clamp(0.0, 1.0) as f64;
            if score < threshold as f64 {
                return None;
            }
            Some(crate::db::filter_repo::FilteredResult {
                id: scan_item_id,
                task_id: request.task_id.clone(),
                domain,
                filter_type: "semantic".to_string(),
                filter_pattern: Some(request.description.clone()),
                is_matched: true,
                score: Some(score),
                embedding_id: Some(scan_item_id),
            })
        })
        .collect();

    if !items.is_empty() {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = FilterRepo::new(&conn);
        repo.batch_insert(&items).map_err(|e| e.to_string())?;
    }

    let result = FilterResult {
        total: items.len() as i64,
        items,
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
