use std::time::{Duration, Instant};
use crate::models::scan_item::ScanItemStatus;

/// Result of a domain availability check
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub domain: String,
    pub status: ScanItemStatus,
    pub is_available: Option<bool>,
    pub query_method: Option<String>,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
}

/// Configuration for domain checking
#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub rdap_timeout: Duration,
    pub dns_timeout: Duration,
    pub max_retries: u32,
    pub retry_delays: Vec<Duration>,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            rdap_timeout: Duration::from_secs(10),
            dns_timeout: Duration::from_secs(5),
            max_retries: 3,
            retry_delays: vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4),
            ],
        }
    }
}

/// Domain availability checker using RDAP (primary) and DNS (fallback)
pub struct DomainChecker {
    config: CheckConfig,
    http_client: reqwest::Client,
}

impl DomainChecker {
    pub fn new(config: CheckConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.rdap_timeout)
            .build()
            .unwrap_or_default();
        Self { config, http_client }
    }

    pub fn with_default_config() -> Self {
        Self::new(CheckConfig::default())
    }

    /// Check domain availability via RDAP + DNS fallback
    pub async fn check_domain(&self, domain: &str) -> CheckResult {
        let start = Instant::now();

        // Try RDAP first with retry
        match self.check_rdap_with_retry(domain).await {
            Ok(result) => {
                let elapsed = start.elapsed().as_millis() as i64;
                return CheckResult {
                    domain: domain.to_string(),
                    status: if result { ScanItemStatus::Available } else { ScanItemStatus::Unavailable },
                    is_available: Some(result),
                    query_method: Some("rdap".to_string()),
                    response_time_ms: Some(elapsed),
                    error_message: None,
                };
            }
            Err(e) => {
                // RDAP failed, try DNS fallback
                match self.check_dns_fallback(domain).await {
                    Ok(result) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        CheckResult {
                            domain: domain.to_string(),
                            status: if result { ScanItemStatus::Available } else { ScanItemStatus::Unavailable },
                            is_available: Some(result),
                            query_method: Some("dns".to_string()),
                            response_time_ms: Some(elapsed),
                            error_message: Some(format!("RDAP failed: {}, used DNS fallback", e)),
                        }
                    }
                    Err(dns_err) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        CheckResult {
                            domain: domain.to_string(),
                            status: ScanItemStatus::Error,
                            is_available: None,
                            query_method: None,
                            response_time_ms: Some(elapsed),
                            error_message: Some(format!("RDAP: {}, DNS: {}", e, dns_err)),
                        }
                    }
                }
            }
        }
    }

    /// Check via RDAP with exponential backoff retry
    async fn check_rdap_with_retry(&self, domain: &str) -> Result<bool, String> {
        let tld = extract_tld(domain);
        let rdap_url = self.get_rdap_url(domain, &tld);

        let mut last_error = String::new();
        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let default_delay = Duration::from_secs(8);
                let delay = self.config.retry_delays
                    .get((attempt - 1) as usize)
                    .unwrap_or(&default_delay);
                tokio::time::sleep(*delay).await;
            }

            match self.query_rdap(&rdap_url).await {
                Ok(available) => return Ok(available),
                Err(e) => {
                    last_error = e;
                    // If rate limited, always retry
                    // If not found (404), domain is likely available
                    continue;
                }
            }
        }
        Err(last_error)
    }

    /// Query RDAP server
    async fn query_rdap(&self, url: &str) -> Result<bool, String> {
        let response = self.http_client.get(url).send().await;
        match response {
            Ok(resp) => {
                match resp.status() {
                    s if s == reqwest::StatusCode::OK => {
                        // Domain exists in RDAP registry -> registered
                        Ok(false)
                    }
                    s if s == reqwest::StatusCode::NOT_FOUND => {
                        // Domain not found in RDAP -> likely available
                        Ok(true)
                    }
                    s if s == reqwest::StatusCode::TOO_MANY_REQUESTS => {
                        Err("Rate limited".to_string())
                    }
                    s => {
                        Err(format!("RDAP returned status: {}", s))
                    }
                }
            }
            Err(e) => Err(format!("RDAP request failed: {}", e)),
        }
    }

    /// DNS fallback: if no DNS records, domain might be available
    async fn check_dns_fallback(&self, domain: &str) -> Result<bool, String> {
        use hickory_resolver::TokioAsyncResolver;
        use std::net::SocketAddr;

        let config = hickory_resolver::config::ResolverConfig::default();
        let opts = hickory_resolver::config::ResolverOpts::default();
        let resolver = TokioAsyncResolver::tokio(config, opts);

        match resolver.lookup_ip(domain).await {
            Ok(_) => Ok(false), // DNS record exists -> likely registered
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("No records found") || err_str.contains("NXDOMAIN") {
                    Ok(true) // No DNS record -> likely available
                } else {
                    Err(format!("DNS lookup error: {}", e))
                }
            }
        }
    }

    /// Get RDAP URL for a domain
    fn get_rdap_url(&self, domain: &str, tld: &str) -> String {
        // Common RDAP server URLs
        let rdap_base = match tld {
            ".com" => "https://rdap.verisign.com/com/v1",
            ".net" => "https://rdap.verisign.com/net/v1",
            ".org" => "https://rdap.publicinterestregistry.org/rdap",
            ".io" => "https://rdap.nic.io",
            ".co" => "https://rdap.nic.co",
            ".dev" => "https://rdap.nic.dev",
            ".app" => "https://rdap.nic.app",
            ".ai" => "https://rdap.nic.ai",
            _ => "https://rdap.org", // Generic bootstrap
        };
        format!("{}/domain/{}", rdap_base, domain)
    }

    /// Check multiple domains concurrently
    pub async fn check_domains(&self, domains: &[String], concurrency: usize) -> Vec<CheckResult> {
        use futures::stream::{self, StreamExt};
        stream::iter(domains)
            .map(|d| self.check_domain(d))
            .buffer_unordered(concurrency)
            .collect()
            .await
    }
}

/// Extract TLD from a domain string
fn extract_tld(domain: &str) -> String {
    let parts: Vec<&str> = domain.rsplitn(2, '.').collect();
    if parts.len() >= 2 {
        format!(".{}", parts[0])
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_config_default() {
        let config = CheckConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delays.len(), 3);
        assert_eq!(config.rdap_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_extract_tld() {
        assert_eq!(extract_tld("example.com"), ".com");
        assert_eq!(extract_tld("test.co.uk"), ".uk");
        assert_eq!(extract_tld("simple.org"), ".org");
    }

    #[test]
    fn test_get_rdap_url() {
        let checker = DomainChecker::with_default_config();
        assert_eq!(checker.get_rdap_url("example.com", ".com"), "https://rdap.verisign.com/com/v1/domain/example.com");
        assert_eq!(checker.get_rdap_url("example.net", ".net"), "https://rdap.verisign.com/net/v1/domain/example.net");
        assert_eq!(checker.get_rdap_url("example.org", ".org"), "https://rdap.publicinterestregistry.org/rdap/domain/example.org");
    }

    #[tokio::test]
    async fn test_check_result_structure() {
        let result = CheckResult {
            domain: "test.com".to_string(),
            status: ScanItemStatus::Available,
            is_available: Some(true),
            query_method: Some("rdap".to_string()),
            response_time_ms: Some(150),
            error_message: None,
        };
        assert_eq!(result.domain, "test.com");
        assert!(result.is_available.unwrap());
    }
}
