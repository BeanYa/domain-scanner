use crate::db::init;
use crate::db::proxy_repo::ProxyRepo;
use crate::models::proxy::{ProxyConfig, ProxyType};
use serde::Deserialize;

#[tauri::command]
pub fn list_proxies(active_only: Option<bool>) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ProxyRepo::new(&conn);

    let proxies = repo
        .list(active_only.unwrap_or(false))
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&proxies).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
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

    let proxy_type = match request.proxy_type.to_lowercase().as_str() {
        "http" => ProxyType::Http,
        "https" => ProxyType::Https,
        "socks5" => ProxyType::Socks5,
        _ => return Err(format!("Unsupported proxy type: {}", request.proxy_type)),
    };

    let proxy = ProxyConfig {
        id: 0, // Auto-generated
        name: request.name,
        url: request.url,
        proxy_type,
        username: request.username,
        password: request.password,
        is_active: true,
    };

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
pub fn test_proxy(proxy_id: i64) -> Result<String, String> {
    let conn = init::open_db().map_err(|e| e.to_string())?;
    let repo = ProxyRepo::new(&conn);

    let proxy = repo
        .get_by_id(proxy_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Proxy not found: {}", proxy_id))?;

    // Test proxy connectivity
    let proxy_url = match &proxy.username {
        Some(user) => {
            let pass = proxy.password.as_deref().unwrap_or("");
            format!(
                "{}://{}:{}@{}",
                proxy.proxy_type.to_url_scheme(),
                user,
                pass,
                proxy
                    .url
                    .trim_start_matches(&format!("{}://", proxy.proxy_type.to_url_scheme()))
            )
        }
        None => proxy.url.clone(),
    };

    // Attempt a test connection
    let result = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let client = reqwest::Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).map_err(|e| e.to_string())?)
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .map_err(|e| format!("Client build error: {}", e))?;

            client
                .get("https://httpbin.org/ip")
                .send()
                .await
                .map_err(|e| format!("Proxy test failed: {}", e))?;

            Ok::<(), String>(())
        })
    })
    .join()
    .unwrap();

    match result {
        Ok(()) => {
            repo.set_active(proxy_id, true).map_err(|e| e.to_string())?;
            Ok(
                serde_json::json!({"success": true, "message": "Proxy connection successful"})
                    .to_string(),
            )
        }
        Err(e) => {
            let _ = repo.set_active(proxy_id, false);
            Ok(serde_json::json!({"success": false, "message": e}).to_string())
        }
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
