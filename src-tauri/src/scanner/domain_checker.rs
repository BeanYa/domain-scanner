use crate::models::scan_item::ScanItemStatus;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::Method;
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
    pub proxy_error: bool,
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
    proxy_related: bool,
}

#[derive(Debug, Clone)]
struct RdapEndpoint {
    url: String,
    uses_aggregator: bool,
}

#[derive(Debug, Clone)]
pub struct RdapProbeEndpoint {
    pub key: &'static str,
    pub label: &'static str,
    pub url: &'static str,
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
            max_retries: 1,
            retry_delays: vec![Duration::from_millis(800)],
        }
    }
}

/// Domain availability checker using RDAP (primary) and DNS (fallback)
#[derive(Clone)]
pub struct DomainChecker {
    config: CheckConfig,
    http_client: reqwest::Client,
    proxy_label: Option<String>,
    logger: Option<CheckLogger>,
}

impl DomainChecker {
    pub fn rdap_probe_endpoints() -> Vec<RdapProbeEndpoint> {
        vec![
            RdapProbeEndpoint {
                key: "rdap-com",
                label: ".com / Verisign",
                url: "https://rdap.verisign.com/com/v1/domain/example.com",
            },
            RdapProbeEndpoint {
                key: "rdap-net",
                label: ".net / Verisign",
                url: "https://rdap.verisign.com/net/v1/domain/example.net",
            },
            RdapProbeEndpoint {
                key: "rdap-org",
                label: ".org / PIR",
                url: "https://rdap.publicinterestregistry.org/rdap/domain/example.org",
            },
            RdapProbeEndpoint {
                key: "rdap-io",
                label: ".io / NIC.IO",
                url: "https://rdap.nic.io/domain/example.io",
            },
            RdapProbeEndpoint {
                key: "rdap-co",
                label: ".co / NIC.CO",
                url: "https://rdap.nic.co/domain/example.co",
            },
            RdapProbeEndpoint {
                key: "rdap-dev",
                label: ".dev / Google Registry",
                url: "https://rdap.nic.dev/domain/example.dev",
            },
            RdapProbeEndpoint {
                key: "rdap-app",
                label: ".app / Google Registry",
                url: "https://rdap.nic.app/domain/example.app",
            },
            RdapProbeEndpoint {
                key: "rdap-ai",
                label: ".ai / NIC.AI",
                url: "https://rdap.nic.ai/domain/example.ai",
            },
            RdapProbeEndpoint {
                key: "rdap-de",
                label: ".de / DENIC",
                url: "https://rdap.denic.de/domain/example.de",
            },
            RdapProbeEndpoint {
                key: "rdap-aggregator",
                label: "Fallback / rdap.org",
                url: "https://rdap.org/domain/example.xyz",
            },
        ]
    }

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
                    proxy_error: false,
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
                        let hint = if result {
                            "DNS fallback suggests likely available"
                        } else {
                            "DNS fallback suggests likely registered"
                        };
                        self.log(
                            "warn",
                            format!("DNS fallback completed for {}: {}", domain, hint),
                        );
                        CheckResult {
                            domain: domain.to_string(),
                            status: ScanItemStatus::Error,
                            is_available: None,
                            query_method: Some("dns".to_string()),
                            response_time_ms: Some(elapsed),
                            error_message: Some(format!(
                                "RDAP failed: {}; DNS fallback completed but result is not authoritative, {}",
                                e.message, hint
                            )),
                            proxy_error: e.proxy_related,
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
                            proxy_error: e.proxy_related,
                        }
                    }
                }
            }
        }
    }

    /// Check via RDAP with exponential backoff retry
    async fn check_rdap_with_retry(&self, domain: &str) -> Result<bool, RdapError> {
        let tld = extract_tld(domain);
        let endpoint = self.get_rdap_endpoint(domain, &tld);
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
                    endpoint.url,
                    self.transport_label()
                ),
            );

            match self.query_rdap(&endpoint).await {
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
            proxy_related: false,
        }))
    }

    /// Query RDAP server
    async fn query_rdap(&self, endpoint: &RdapEndpoint) -> Result<bool, RdapError> {
        match self.query_rdap_with_method(endpoint, Method::HEAD).await? {
            Some(available) => Ok(available),
            None => {
                self.log(
                    "info",
                    format!(
                        "RDAP HEAD not supported for {}. Retrying with GET",
                        endpoint.url
                    ),
                );
                self.query_rdap_with_method(endpoint, Method::GET)
                    .await?
                    .ok_or_else(|| RdapError {
                        kind: RdapErrorKind::Permanent,
                        message: "RDAP GET request returned no result".to_string(),
                        proxy_related: false,
                    })
            }
        }
    }

    async fn query_rdap_with_method(
        &self,
        endpoint: &RdapEndpoint,
        method: Method,
    ) -> Result<Option<bool>, RdapError> {
        let response = self
            .http_client
            .request(method.clone(), &endpoint.url)
            .send()
            .await;
        match response {
            Ok(resp) => match resp.status() {
                s if s == reqwest::StatusCode::OK => Ok(Some(false)),
                s if s == reqwest::StatusCode::NOT_FOUND => Ok(Some(true)),
                s if method == Method::HEAD
                    && (s == reqwest::StatusCode::METHOD_NOT_ALLOWED
                        || s == reqwest::StatusCode::NOT_IMPLEMENTED) =>
                {
                    Ok(None)
                }
                s if s == reqwest::StatusCode::TOO_MANY_REQUESTS => Err(RdapError {
                    kind: RdapErrorKind::RateLimited,
                    message: format!("RDAP {} returned status: 429 Too Many Requests", method),
                    proxy_related: false,
                }),
                s if s == reqwest::StatusCode::PROXY_AUTHENTICATION_REQUIRED => Err(RdapError {
                    kind: RdapErrorKind::Forbidden,
                    message: format!(
                        "RDAP {} returned proxy authentication status: {}",
                        method, s
                    ),
                    proxy_related: true,
                }),
                s if s == reqwest::StatusCode::FORBIDDEN
                    || s == reqwest::StatusCode::UNAUTHORIZED =>
                {
                    Err(RdapError {
                        kind: RdapErrorKind::Forbidden,
                        message: format!("RDAP {} returned status: {}", method, s),
                        proxy_related: false,
                    })
                }
                s if s.is_server_error() || s == reqwest::StatusCode::REQUEST_TIMEOUT => {
                    Err(RdapError {
                        kind: RdapErrorKind::Retryable,
                        message: format!("RDAP {} returned status: {}", method, s),
                        proxy_related: false,
                    })
                }
                s => Err(RdapError {
                    kind: RdapErrorKind::Permanent,
                    message: format!("RDAP {} returned status: {}", method, s),
                    proxy_related: false,
                }),
            },
            Err(e) => {
                let request_transport_error = e.is_timeout() || e.is_connect() || e.is_request();
                Err(RdapError {
                    kind: if request_transport_error {
                        if endpoint.uses_aggregator {
                            RdapErrorKind::Permanent
                        } else {
                            RdapErrorKind::Retryable
                        }
                    } else {
                        RdapErrorKind::Permanent
                    },
                    message: format!("RDAP {} request failed: {}", method, e),
                    proxy_related: self.proxy_label.is_some() && request_transport_error,
                })
            }
        }
    }

    /// Get RDAP URL for a domain
    fn get_rdap_endpoint(&self, domain: &str, tld: &str) -> RdapEndpoint {
        let (rdap_base, uses_aggregator) = match tld {
            ".com" => ("https://rdap.verisign.com/com/v1", false),
            ".net" => ("https://rdap.verisign.com/net/v1", false),
            ".org" => ("https://rdap.publicinterestregistry.org/rdap", false),
            ".io" => ("https://rdap.nic.io", false),
            ".co" => ("https://rdap.nic.co", false),
            ".dev" => ("https://rdap.nic.dev", false),
            ".app" => ("https://rdap.nic.app", false),
            ".ai" => ("https://rdap.nic.ai", false),
            ".de" => ("https://rdap.denic.de", false),
            _ => ("https://rdap.org", true),
        };

        RdapEndpoint {
            url: format!("{}/domain/{}", rdap_base, domain),
            uses_aggregator,
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
    use mockito::Server;

    #[test]
    fn test_check_config_default() {
        let config = CheckConfig::default();
        assert_eq!(config.max_retries, 1);
        assert_eq!(config.retry_delays.len(), 1);
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
            checker.get_rdap_endpoint("example.com", ".com").url,
            "https://rdap.verisign.com/com/v1/domain/example.com"
        );
        assert_eq!(
            checker.get_rdap_endpoint("example.net", ".net").url,
            "https://rdap.verisign.com/net/v1/domain/example.net"
        );
        assert_eq!(
            checker.get_rdap_endpoint("example.org", ".org").url,
            "https://rdap.publicinterestregistry.org/rdap/domain/example.org"
        );
        assert_eq!(
            checker.get_rdap_endpoint("example.de", ".de").url,
            "https://rdap.denic.de/domain/example.de"
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
            proxy_error: false,
        };
        assert_eq!(result.domain, "test.com");
        assert!(result.is_available.unwrap());
    }

    #[tokio::test]
    async fn test_query_rdap_falls_back_to_get_when_head_is_not_supported() {
        let mut server = Server::new_async().await;
        let path = "/domain/example.test";
        let _head_mock = server
            .mock("HEAD", path)
            .with_status(405)
            .create_async()
            .await;
        let _get_mock = server
            .mock("GET", path)
            .with_status(404)
            .create_async()
            .await;

        let checker = DomainChecker {
            config: CheckConfig::default(),
            http_client: reqwest::Client::builder().build().unwrap(),
            proxy_label: None,
            logger: None,
        };
        let endpoint = RdapEndpoint {
            url: format!("{}{}", server.url(), path),
            uses_aggregator: false,
        };

        let result = checker.query_rdap(&endpoint).await.unwrap();
        assert!(result);
    }
}
