use crate::db::init;
use crate::db::proxy_repo::ProxyRepo;
use crate::models::proxy::{
    ProxyConfig, ProxyEndpointCheck, ProxyStatus, ProxyTestResult, ProxyType,
};
use crate::proxy::manager::ProxyManager;
use crate::scanner::domain_checker::DomainChecker;
use reqwest::{Method, StatusCode};
use serde::Deserialize;
use std::time::{Duration, Instant};

#[tauri::command]
pub fn list_proxies(active_only: Option<bool>) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ProxyRepo::new(&conn);

    let proxies = repo
        .list(active_only.unwrap_or(false))
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&proxies).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProxyRequest {
    pub name: Option<String>,
    pub url: String,
    pub proxy_type: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[tauri::command]
pub fn create_proxy(request: CreateProxyRequest) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ProxyRepo::new(&conn);

    let proxy = proxy_from_request(request)?;

    let id = repo.create(&proxy).map_err(|e| e.to_string())?;
    let created = repo
        .get_by_id(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve created proxy".to_string())?;

    serde_json::to_string(&created).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_proxy(proxy_id: i64) -> Result<(), String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ProxyRepo::new(&conn);
    repo.delete(proxy_id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn test_proxy(proxy_id: i64) -> Result<String, String> {
    let proxy = {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = ProxyRepo::new(&conn);
        let proxy = repo
            .get_by_id(proxy_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Proxy not found: {}", proxy_id))?;
        repo.update_health(proxy_id, &ProxyStatus::Checking, false, None, None)
            .map_err(|e| e.to_string())?;
        proxy
    };

    let result = run_proxy_checks(&proxy).await;
    {
        let conn = init::open_db().map_err(|e| e.to_string())?;
        let repo = ProxyRepo::new(&conn);
        repo.update_health(
            proxy_id,
            &result.status,
            result.success,
            Some(&result.checked_at),
            if result.success {
                None
            } else {
                Some(result.message.as_str())
            },
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(serde_json::to_string(&result).map_err(|e| e.to_string())?)
}

fn proxy_from_request(request: CreateProxyRequest) -> Result<ProxyConfig, String> {
    let proxy_type = match request.proxy_type.to_lowercase().as_str() {
        "http" => ProxyType::Http,
        "https" => ProxyType::Https,
        "socks5" => ProxyType::Socks5,
        _ => return Err(format!("Unsupported proxy type: {}", request.proxy_type)),
    };

    let url = normalize_proxy_url(&request.url, &proxy_type);

    Ok(ProxyConfig {
        id: 0,
        name: request.name,
        url,
        proxy_type,
        username: request.username,
        password: request.password,
        is_active: false,
        status: ProxyStatus::Pending,
        last_checked_at: None,
        last_error: None,
    })
}

fn normalize_proxy_url(url: &str, proxy_type: &ProxyType) -> String {
    let trimmed = url.trim();
    if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("{}://{}", proxy_type.to_url_scheme(), trimmed)
    }
}

async fn run_proxy_checks(proxy: &ProxyConfig) -> ProxyTestResult {
    let checked_at = chrono::Utc::now().to_rfc3339();
    let mut notes = vec![
        "DNS fallback 使用系统解析器，不通过代理，因此本次仅检测 RDAP HTTP(S) 端点。".to_string(),
    ];

    let reqwest_proxy = match ProxyManager::build_reqwest_proxy(proxy) {
        Ok(proxy) => proxy,
        Err(err) => {
            return ProxyTestResult {
                proxy_id: proxy.id,
                success: false,
                status: ProxyStatus::Error,
                message: err,
                checked_at,
                reachable_count: 0,
                total_count: 0,
                endpoints: Vec::new(),
                notes,
            };
        }
    };

    let client = match reqwest::Client::builder()
        .proxy(reqwest_proxy)
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return ProxyTestResult {
                proxy_id: proxy.id,
                success: false,
                status: ProxyStatus::Error,
                message: format!("Failed to build proxy client: {}", err),
                checked_at,
                reachable_count: 0,
                total_count: 0,
                endpoints: Vec::new(),
                notes,
            };
        }
    };

    let mut endpoints = Vec::new();
    for probe in DomainChecker::rdap_probe_endpoints() {
        endpoints.push(run_endpoint_check(&client, &probe).await);
    }

    let reachable_count = endpoints.iter().filter(|item| item.reachable).count();
    let total_count = endpoints.len();
    let failed_labels: Vec<&str> = endpoints
        .iter()
        .filter(|item| !item.reachable)
        .map(|item| item.label.as_str())
        .collect();

    if !failed_labels.is_empty() {
        notes.push(format!("失败端点：{}", failed_labels.join(", ")));
    }

    let (success, status, message) = if total_count > 0 && reachable_count == total_count {
        (
            true,
            ProxyStatus::Available,
            "所有扫描 RDAP 端点均可通过该代理访问".to_string(),
        )
    } else if reachable_count == 0 {
        (
            false,
            ProxyStatus::Error,
            "所有扫描 RDAP 端点均不可达".to_string(),
        )
    } else {
        (
            false,
            ProxyStatus::Unavailable,
            format!(
                "仅 {}/{} 个扫描 RDAP 端点可达，暂不标记为在线",
                reachable_count, total_count
            ),
        )
    };

    ProxyTestResult {
        proxy_id: proxy.id,
        success,
        status,
        message,
        checked_at,
        reachable_count,
        total_count,
        endpoints,
        notes,
    }
}

async fn run_endpoint_check(
    client: &reqwest::Client,
    probe: &crate::scanner::domain_checker::RdapProbeEndpoint,
) -> ProxyEndpointCheck {
    let start = Instant::now();
    match send_probe_request(client, probe.url).await {
        Ok(status) => ProxyEndpointCheck {
            key: probe.key.to_string(),
            label: probe.label.to_string(),
            url: probe.url.to_string(),
            reachable: status != StatusCode::PROXY_AUTHENTICATION_REQUIRED,
            http_status: Some(status.as_u16()),
            response_time_ms: Some(start.elapsed().as_millis() as i64),
            error_message: if status == StatusCode::PROXY_AUTHENTICATION_REQUIRED {
                Some("Proxy authentication required (407)".to_string())
            } else {
                None
            },
        },
        Err(err) => ProxyEndpointCheck {
            key: probe.key.to_string(),
            label: probe.label.to_string(),
            url: probe.url.to_string(),
            reachable: false,
            http_status: None,
            response_time_ms: Some(start.elapsed().as_millis() as i64),
            error_message: Some(err),
        },
    }
}

async fn send_probe_request(client: &reqwest::Client, url: &str) -> Result<StatusCode, String> {
    let head_response = client
        .request(Method::HEAD, url)
        .send()
        .await
        .map_err(|e| format!("HEAD request failed: {}", e))?;

    if head_response.status() == StatusCode::METHOD_NOT_ALLOWED
        || head_response.status() == StatusCode::NOT_IMPLEMENTED
    {
        let get_response = client
            .request(Method::GET, url)
            .send()
            .await
            .map_err(|e| format!("GET fallback failed: {}", e))?;
        Ok(get_response.status())
    } else {
        Ok(head_response.status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_proxies_empty() {
        let result = list_proxies(None).unwrap();
        let proxies: Vec<ProxyConfig> = serde_json::from_str(&result).unwrap();
        assert!(proxies.is_empty());
    }

    #[test]
    fn test_create_proxy() {
        let req = CreateProxyRequest {
            name: Some("Test Proxy".to_string()),
            url: "http://127.0.0.1:8080".to_string(),
            proxy_type: "http".to_string(),
            username: None,
            password: None,
        };
        let result = create_proxy(req).unwrap();
        let proxy: ProxyConfig = serde_json::from_str(&result).unwrap();
        assert_eq!(proxy.url, "http://127.0.0.1:8080");
        assert_eq!(proxy.proxy_type, ProxyType::Http);
        assert_eq!(proxy.status, ProxyStatus::Pending);
        assert!(!proxy.is_active);
    }

    #[test]
    fn test_create_proxy_invalid_type() {
        let req = CreateProxyRequest {
            name: None,
            url: "ftp://127.0.0.1:8080".to_string(),
            proxy_type: "ftp".to_string(),
            username: None,
            password: None,
        };
        let result = create_proxy(req);
        assert!(result.is_err());
    }
}
