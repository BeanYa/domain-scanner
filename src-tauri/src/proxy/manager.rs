use crate::models::proxy::ProxyConfig;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Proxy rotation manager with Round-Robin and health checking
pub struct ProxyManager {
    proxies: Vec<ProxyConfig>,
    current_index: AtomicUsize,
}

impl ProxyManager {
    pub fn new(proxies: Vec<ProxyConfig>) -> Self {
        Self {
            proxies: proxies.into_iter().filter(|p| p.is_active).collect(),
            current_index: AtomicUsize::new(0),
        }
    }

    pub fn with_no_proxy() -> Self {
        Self {
            proxies: Vec::new(),
            current_index: AtomicUsize::new(0),
        }
    }

    /// Get the next proxy using Round-Robin
    pub fn next_proxy(&self) -> Option<&ProxyConfig> {
        if self.proxies.is_empty() {
            return None;
        }
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.proxies.len();
        Some(&self.proxies[index])
    }

    /// Build a reqwest proxy from a ProxyConfig, including authentication if present
    pub fn build_reqwest_proxy(proxy: &ProxyConfig) -> Result<reqwest::Proxy, String> {
        let p = reqwest::Proxy::all(&proxy.url)
            .map_err(|e| format!("Failed to create {:?} proxy: {}", proxy.proxy_type, e))?;

        let p = match (&proxy.username, &proxy.password) {
            (Some(user), Some(pass)) => p.basic_auth(user, pass),
            (Some(user), None) => p.basic_auth(user, ""),
            _ => p,
        };

        Ok(p)
    }

    /// Get the number of active proxies
    pub fn active_count(&self) -> usize {
        self.proxies.len()
    }

    /// Check if there are any proxies available
    pub fn has_proxies(&self) -> bool {
        !self.proxies.is_empty()
    }

    /// Mark a proxy as failed (remove from rotation)
    pub fn mark_failed(&mut self, proxy_url: &str) {
        self.proxies.retain(|p| p.url != proxy_url);
    }

    /// Test a proxy by making a request through it
    pub async fn test_proxy(proxy: &ProxyConfig, test_url: &str) -> Result<(), String> {
        let reqwest_proxy = Self::build_reqwest_proxy(proxy)?;
        let client = reqwest::Client::builder()
            .proxy(reqwest_proxy)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to build client: {}", e))?;

        let response = client
            .get(test_url)
            .send()
            .await
            .map_err(|e| format!("Proxy test failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Proxy returned status: {}", response.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::proxy::{ProxyStatus, ProxyType};

    fn make_proxy(id: i64, url: &str, active: bool) -> ProxyConfig {
        ProxyConfig {
            id,
            name: Some(format!("Proxy {}", id)),
            url: url.to_string(),
            proxy_type: ProxyType::Socks5,
            username: None,
            password: None,
            is_active: active,
            status: if active {
                ProxyStatus::Available
            } else {
                ProxyStatus::Pending
            },
            last_checked_at: None,
            last_error: None,
        }
    }

    #[test]
    fn test_round_robin_rotation() {
        let mgr = ProxyManager::new(vec![
            make_proxy(1, "socks5://p1:1080", true),
            make_proxy(2, "socks5://p2:1080", true),
            make_proxy(3, "socks5://p3:1080", true),
        ]);
        let p1 = mgr.next_proxy().unwrap();
        let p2 = mgr.next_proxy().unwrap();
        let p3 = mgr.next_proxy().unwrap();
        let p4 = mgr.next_proxy().unwrap(); // wraps around
        assert_eq!(p1.url, "socks5://p1:1080");
        assert_eq!(p2.url, "socks5://p2:1080");
        assert_eq!(p3.url, "socks5://p3:1080");
        assert_eq!(p4.url, "socks5://p1:1080"); // back to first
    }

    #[test]
    fn test_inactive_proxies_filtered() {
        let mgr = ProxyManager::new(vec![
            make_proxy(1, "socks5://p1:1080", true),
            make_proxy(2, "socks5://p2:1080", false),
            make_proxy(3, "socks5://p3:1080", true),
        ]);
        assert_eq!(mgr.active_count(), 2);
    }

    #[test]
    fn test_no_proxy_available() {
        let mgr = ProxyManager::with_no_proxy();
        assert!(mgr.next_proxy().is_none());
        assert!(!mgr.has_proxies());
    }

    #[test]
    fn test_mark_failed() {
        let mut mgr = ProxyManager::new(vec![
            make_proxy(1, "socks5://p1:1080", true),
            make_proxy(2, "socks5://p2:1080", true),
        ]);
        mgr.mark_failed("socks5://p1:1080");
        assert_eq!(mgr.active_count(), 1);
        assert_eq!(mgr.next_proxy().unwrap().url, "socks5://p2:1080");
    }

    #[test]
    fn test_build_reqwest_proxy() {
        let proxy = make_proxy(1, "socks5://127.0.0.1:1080", true);
        let result = ProxyManager::build_reqwest_proxy(&proxy);
        assert!(result.is_ok());

        let http_proxy = ProxyConfig {
            id: 2,
            name: None,
            url: "http://127.0.0.1:8080".to_string(),
            proxy_type: ProxyType::Http,
            username: None,
            password: None,
            is_active: true,
            status: ProxyStatus::Available,
            last_checked_at: None,
            last_error: None,
        };
        let result = ProxyManager::build_reqwest_proxy(&http_proxy);
        assert!(result.is_ok());
    }
}
