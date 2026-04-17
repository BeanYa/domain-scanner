use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyStatus {
    Pending,
    Checking,
    Available,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub id: i64,
    pub name: Option<String>,
    pub url: String,
    pub proxy_type: ProxyType,
    pub username: Option<String>,
    pub password: Option<String>,
    pub is_active: bool,
    pub status: ProxyStatus,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEndpointCheck {
    pub key: String,
    pub label: String,
    pub url: String,
    pub reachable: bool,
    pub http_status: Option<u16>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTestResult {
    pub proxy_id: i64,
    pub success: bool,
    pub status: ProxyStatus,
    pub message: String,
    pub checked_at: String,
    pub reachable_count: usize,
    pub total_count: usize,
    pub endpoints: Vec<ProxyEndpointCheck>,
    pub notes: Vec<String>,
}

impl ProxyType {
    pub fn to_url_scheme(&self) -> &str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ProxyType::Socks5).unwrap(),
            "\"socks5\""
        );
    }

    #[test]
    fn test_proxy_config_roundtrip() {
        let proxy = ProxyConfig {
            id: 1,
            name: Some("Test Proxy".to_string()),
            url: "socks5://127.0.0.1:1080".to_string(),
            proxy_type: ProxyType::Socks5,
            username: None,
            password: None,
            is_active: true,
            status: ProxyStatus::Available,
            last_checked_at: Some("2026-04-16T00:00:00Z".to_string()),
            last_error: None,
        };
        let json = serde_json::to_string(&proxy).unwrap();
        let deserialized: ProxyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(proxy.url, deserialized.url);
        assert_eq!(proxy.status, deserialized.status);
    }
}
