use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
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
        };
        let json = serde_json::to_string(&proxy).unwrap();
        let deserialized: ProxyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(proxy.url, deserialized.url);
    }
}
