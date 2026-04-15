use crate::models::scan_item::ScanItemStatus;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

pub type CheckLogger = Arc<dyn Fn(String, String) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RdapErrorKind {
    Forbidden,
    RateLimited,
    Retryable,
    Permanent,
}

#[derive(Debug, Clone)]
struct RdapError {
    kind: RdapErrorKind,
    message: String,
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
            max_retries: 2,
            retry_delays: vec![Duration::from_millis(800), Duration::from_millis(1500)],
        }
    }
}

/// Domain availability checker using RDAP (primary) and DNS (fallback)
pub struct DomainChecker {
    config: CheckConfig,
    http_client: reqwest::Client,
    proxy_label: Option<String>,
    logger: Option<CheckLogger>,
}

impl DomainChecker {
    pub fn new(config: CheckConfig) -> Self {
        Self::build(config, None, None)
    }

    /// Create a DomainChecker that routes requests through a proxy
    pub fn with_proxy(config: CheckConfig, proxy: reqwest::Proxy, proxy_label: String) -> Self {
        Self::build(config, Some(proxy), Some(proxy_label))
    }

    pub fn with_default_config() -> Self {
        Self::new(CheckConfig::default())
    }

    pub fn with_log_hook(mut self, logger: CheckLogger) -> Self {
        self.logger = Some(logger);
        self
    }

    /// Check domain availability via RDAP + DNS fallback
    pub async fn check_domain(&self, domain: &str) -> CheckResult {
        let start = Instant::now();

        // Try RDAP first with retry
        match self.check_rdap_with_retry(domain).await {
            Ok(result) => {
                let elapsed = start.elapsed().as_millis() as i64;
                CheckResult {
                    domain: domain.to_string(),
                    status: if result {
                        ScanItemStatus::Available
                    } else {
                        ScanItemStatus::Unavailable
                    },
                    is_available: Some(result),
                    query_method: Some("rdap".to_string()),
                    response_time_ms: Some(elapsed),
                    error_message: None,
                }
            }
            Err(e) => {
                self.log(
                    "warn",
                    format!(
                        "RDAP failed for {}: {}. Falling back to DNS lookup",
                        domain, e.message
                    ),
                );
                match self.check_dns_fallback(domain).await {
                    Ok(result) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        self.log(
                            "warn",
                            format!(
                                "DNS fallback completed for {}: {}",
                                domain,
                                if result {
                                    "likely available"
                                } else {
                                    "registered or resolves"
                                }
                            ),
                        );
                        CheckResult {
                            domain: domain.to_string(),
                            status: if result {
                                ScanItemStatus::Available
                            } else {
                                ScanItemStatus::Unavailable
                            },
                            is_available: Some(result),
                            query_method: Some("dns".to_string()),
                            response_time_ms: Some(elapsed),
                            error_message: Some(format!(
                                "RDAP failed: {}, used DNS fallback",
                                e.message
                            )),
                        }
                    }
                    Err(dns_err) => {
                        let elapsed = start.elapsed().as_millis() as i64;
                        self.log(
                            "error",
                            format!("DNS fallback failed for {}: {}", domain, dns_err),
                        );
                        CheckResult {
                            domain: domain.to_string(),
                            status: ScanItemStatus::Error,
                            is_available: None,
                            query_method: None,
                            response_time_ms: Some(elapsed),
                            error_message: Some(format!("RDAP: {}, DNS: {}", e.message, dns_err)),
                        }
                    }
                }
            }
        }
    }

    /// Check via RDAP with exponential backoff retry
    async fn check_rdap_with_retry(&self, domain: &str) -> Result<bool, RdapError> {
        let tld = extract_tld(domain);
        let rdap_url = self.get_rdap_url(domain, &tld);
        let total_attempts = self.config.max_retries + 1;
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let attempt_number = attempt + 1;
            self.log(
                "info",
                format!(
                    "Sending RDAP request [{}/{}] for {} -> {} via {}",
                    attempt_number,
                    total_attempts,
                    domain,
                    rdap_url,
                    self.transport_label()
                ),
            );

            match self.query_rdap(&rdap_url).await {
                Ok(available) => {
                    self.log(
                        "info",
                        format!(
                            "RDAP request succeeded for {} on attempt {}/{}: {}",
                            domain,
                            attempt_number,
                            total_attempts,
                            if available {
                                "likely available"
                            } else {
                                "registered"
                            }
                        ),
                    );
                    return Ok(available);
                }
                Err(err) => {
                    let should_retry = attempt < self.config.max_retries
                        && matches!(
                            err.kind,
                            RdapErrorKind::RateLimited | RdapErrorKind::Retryable
                        );

                    if should_retry {
                        let default_delay = Duration::from_secs(3);
                        let delay = *self
                            .config
                            .retry_delays
                            .get(attempt as usize)
                            .unwrap_or(&default_delay);
                        self.log(
                            "warn",
                            format!(
                                "RDAP attempt [{}/{}] failed for {}: {}. Retrying in {} ms",
                                attempt_number,
                                total_attempts,
                                domain,
                                err.message,
                                delay.as_millis()
                            ),
                        );
                        last_error = Some(err);
                        tokio::time::sleep(delay).await;
                        continue;
                    }

                    if matches!(err.kind, RdapErrorKind::Forbidden) {
                        self.log(
                            "warn",
                            format!(
                                "RDAP access forbidden for {} via {}. Skipping further retries",
                                domain,
                                self.transport_label()
                            ),
                        );
                    } else {
                        self.log(
                            "warn",
                            format!(
                                "RDAP failed for {} after {}/{} attempts: {}",
                                domain, attempt_number, total_attempts, err.message
                            ),
                        );
                    }
                    return Err(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| RdapError {
            kind: RdapErrorKind::Permanent,
            message: "RDAP request failed".to_string(),
        }))
    }

    /// Query RDAP server
    async fn query_rdap(&self, url: &str) -> Result<bool, RdapError> {
        let response = self.http_client.get(url).send().await;
        match response {
            Ok(resp) => match resp.status() {
                s if s == reqwest::StatusCode::OK => Ok(false),
                s if s == reqwest::StatusCode::NOT_FOUND => Ok(true),
                s if s == reqwest::StatusCode::TOO_MANY_REQUESTS => Err(RdapError {
                    kind: RdapErrorKind::RateLimited,
                    message: "RDAP returned status: 429 Too Many Requests".to_string(),
                }),
                s if s == reqwest::StatusCode::FORBIDDEN
                    || s == reqwest::StatusCode::UNAUTHORIZED =>
                {
                    Err(RdapError {
                        kind: RdapErrorKind::Forbidden,
                        message: format!("RDAP returned status: {}", s),
                    })
                }
                s if s.is_server_error() || s == reqwest::StatusCode::REQUEST_TIMEOUT => {
                    Err(RdapError {
                        kind: RdapErrorKind::Retryable,
                        message: format!("RDAP returned status: {}", s),
                    })
                }
                s => Err(RdapError {
                    kind: RdapErrorKind::Permanent,
                    message: format!("RDAP returned status: {}", s),
                }),
            },
            Err(e) => Err(RdapError {
                kind: if e.is_timeout() || e.is_connect() || e.is_request() {
                    RdapErrorKind::Retryable
                } else {
                    RdapErrorKind::Permanent
                },
                message: format!("RDAP request failed: {}", e),
            }),
        }
    }

    /// DNS fallback: if no DNS records, domain might be available
    async fn check_dns_fallback(&self, domain: &str) -> Result<bool, String> {
        use hickory_resolver::TokioAsyncResolver;

        let config = hickory_resolver::config::ResolverConfig::default();
        let opts = hickory_resolver::config::ResolverOpts::default();
        let resolver = TokioAsyncResolver::tokio(config, opts);
        self.log(
            "info",
            format!(
                "Sending DNS lookup for {} via system resolver (proxy not used)",
                domain
            ),
        );

        match resolver.lookup_ip(domain).await {
            Ok(_) => Ok(false),
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("No records found") || err_str.contains("NXDOMAIN") {
                    Ok(true)
                } else {
                    Err(format!("DNS lookup error: {}", e))
                }
            }
        }
    }

    /// Get RDAP URL for a domain
    fn get_rdap_url(&self, domain: &str, tld: &str) -> String {
        let rdap_base = match tld {
            ".com" => "https://rdap.verisign.com/com/v1",
            ".net" => "https://rdap.verisign.com/net/v1",
            ".org" => "https://rdap.publicinterestregistry.org/rdap",
            ".io" => "https://rdap.nic.io",
            ".co" => "https://rdap.nic.co",
            ".dev" => "https://rdap.nic.dev",
            ".app" => "https://rdap.nic.app",
            ".ai" => "https://rdap.nic.ai",
            _ => "https://rdap.org",
        };
        format!("{}/domain/{}", rdap_base, domain)
    }

    /// Check multiple domains concurrently
    pub async fn check_domains(&self, domains: &[String], concurrency: usize) -> Vec<CheckResult> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let sem = Arc::new(tokio::sync::Semaphore::new(concurrency));
        let mut futs = FuturesUnordered::new();

        for domain in domains {
            let d = domain.clone();
            let permit = sem.clone().acquire_owned().await.unwrap();
            futs.push(async move {
                let result = self.check_domain(&d).await;
                drop(permit);
                result
            });
        }

        let mut results = Vec::with_capacity(domains.len());
        while let Some(result) = futs.next().await {
            results.push(result);
        }
        results
    }

    fn build(
        config: CheckConfig,
        proxy: Option<reqwest::Proxy>,
        proxy_label: Option<String>,
    ) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/rdap+json, application/json;q=0.9, */*;q=0.8"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("domain-scanner-app/0.1"),
        );

        let mut builder = reqwest::Client::builder()
            .timeout(config.rdap_timeout)
            .default_headers(headers);

        if let Some(proxy) = proxy {
            builder = builder.proxy(proxy);
        }

        let http_client = builder.build().unwrap_or_default();
        Self {
            config,
            http_client,
            proxy_label,
            logger: None,
        }
    }

    fn transport_label(&self) -> &str {
        self.proxy_label.as_deref().unwrap_or("direct")
    }

    fn log(&self, level: &str, message: String) {
        if let Some(logger) = &self.logger {
            logger(level.to_string(), message);
        }
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
        assert_eq!(config.max_retries, 2);
        assert_eq!(config.retry_delays.len(), 2);
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
        assert_eq!(
            checker.get_rdap_url("example.com", ".com"),
            "https://rdap.verisign.com/com/v1/domain/example.com"
        );
        assert_eq!(
            checker.get_rdap_url("example.net", ".net"),
            "https://rdap.verisign.com/net/v1/domain/example.net"
        );
        assert_eq!(
            checker.get_rdap_url("example.org", ".org"),
            "https://rdap.publicinterestregistry.org/rdap/domain/example.org"
        );
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
