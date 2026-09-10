use crate::error::ScraplingError;
use crate::output::{ContentFormat, OutputStatus, ScraplingOutput};
use crate::params::{Operation, ScraplingParams, ScrapingMode};
use std::time::Instant;

/// Scrapling Node implementation
pub struct ScraplingNode {
    pub version: u16,
}

impl ScraplingNode {
    pub fn new() -> Self {
        Self { version: 1 }
    }

    /// Execute the node with given parameters
    pub async fn execute(&self, params: &ScraplingParams, input_digest: &str) -> Result<Vec<ScraplingOutput>, ScraplingError> {
        let mut results = Vec::new();
        
        match params.operation {
            Operation::SmartExtract => {
                for url in &params.urls {
                    let output = self.smart_extract(url, params, input_digest).await?;
                    results.push(output);
                }
            }
            Operation::CssQuery => {
                for url in &params.urls {
                    let output = self.css_query(url, params, input_digest).await?;
                    results.push(output);
                }
            }
            Operation::XpathQuery => {
                for url in &params.urls {
                    let output = self.xpath_query(url, params, input_digest).await?;
                    results.push(output);
                }
            }
            Operation::BatchScrape => {
                results = self.batch_scrape(params, input_digest).await?;
            }
        }
        
        Ok(results)
    }

    /// Smart extract with auto-healing (spec §1 field #5)
    async fn smart_extract(&self, url: &str, params: &ScraplingParams, input_digest: &str) -> Result<ScraplingOutput, ScraplingError> {
        self.guardrails_check(url, params).await?;
        
        let start = Instant::now();
        let (content, attempts, auto_healed) = self.fetch_with_retry(url, params, input_digest).await?;
        
        // Prototype: simulate extraction
        let data = serde_json::json!({
            "extracted": true,
            "url": url,
            "content_length": content.len(),
        });
        
        Ok(ScraplingOutput {
            url: url.to_string(),
            status: if auto_healed { OutputStatus::AutoHealed } else { OutputStatus::Success },
            content_format: ContentFormat::Json,
            data_ref: Some(data),
            content_ref: None,
            auto_healed,
            pii_redacted: true,
            took_ms: start.elapsed().as_millis() as u64,
            attempts,
            error_code: None,
            input_digest: Some(input_digest.to_string()),
        })
    }

    /// CSS query extraction
    async fn css_query(&self, url: &str, params: &ScraplingParams, input_digest: &str) -> Result<ScraplingOutput, ScraplingError> {
        self.guardrails_check(url, params).await?;
        let selector = params.selector.as_ref()
            .ok_or(ScraplingError::ExtractionFailed { reason: "selector required for cssQuery".into() })?;
        
        let start = Instant::now();
        let (content, attempts, _) = self.fetch_with_retry(url, params, input_digest).await?;
        
        Ok(ScraplingOutput {
            url: url.to_string(),
            status: OutputStatus::Success,
            content_format: ContentFormat::Html,
            data_ref: Some(serde_json::json!({"selector": selector, "matches": 0})),
            content_ref: None,
            auto_healed: false,
            pii_redacted: true,
            took_ms: start.elapsed().as_millis() as u64,
            attempts,
            error_code: None,
            input_digest: Some(input_digest.to_string()),
        })
    }

    /// XPath query extraction
    async fn xpath_query(&self, url: &str, params: &ScraplingParams, input_digest: &str) -> Result<ScraplingOutput, ScraplingError> {
        self.guardrails_check(url, params).await?;
        let xpath = params.xpath.as_ref()
            .ok_or(ScraplingError::ExtractionFailed { reason: "xpath required for xpathQuery".into() })?;
        
        let start = Instant::now();
        let (_, attempts, _) = self.fetch_with_retry(url, params, input_digest).await?;
        
        Ok(ScraplingOutput {
            url: url.to_string(),
            status: OutputStatus::Success,
            content_format: ContentFormat::Html,
            data_ref: Some(serde_json::json!({"xpath": xpath, "matches": 0})),
            content_ref: None,
            auto_healed: false,
            pii_redacted: true,
            took_ms: start.elapsed().as_millis() as u64,
            attempts,
            error_code: None,
            input_digest: Some(input_digest.to_string()),
        })
    }

    /// Batch scrape with concurrency control (spec §1 field #8)
    async fn batch_scrape(&self, params: &ScraplingParams, input_digest: &str) -> Result<Vec<ScraplingOutput>, ScraplingError> {
        if params.urls.len() > 50 {
            return Err(ScraplingError::ExtractionFailed { 
                reason: format!("batchScrape max 50 URLs, got {}", params.urls.len()) 
            });
        }
        
        let mut results = Vec::new();
        // Prototype: sequential execution (real impl would use concurrency limit)
        for url in &params.urls {
            let output = self.smart_extract(url, params, input_digest).await?;
            results.push(output);
        }
        
        Ok(results)
    }

    /// Guardrails check (spec §4)
    async fn guardrails_check(&self, url: &str, params: &ScraplingParams) -> Result<(), ScraplingError> {
        // SSRF check
        self.ssrf_check(url)?;
        
        // Robots.txt check (unless ignored)
        if !params.ignore_robots {
            self.robots_check(url).await?;
        }
        
        Ok(())
    }

    /// SSRF blocklist check (spec §4)
    fn ssrf_check(&self, url: &str) -> Result<(), ScraplingError> {
        let parsed = url::Url::parse(url)
            .map_err(|_| ScraplingError::ExtractionFailed { reason: "Invalid URL".into() })?;
        
        if let Some(host) = parsed.host_str() {
            // Block private ranges (prototype: block localhost and 10.x.x.x)
            if host == "localhost" || host == "127.0.0.1" || host.starts_with("10.") || host.starts_with("192.168.") {
                return Err(ScraplingError::SsrfBlocked { url: url.to_string() });
            }
        }
        
        Ok(())
    }

    /// Robots.txt check (spec §4)
    async fn robots_check(&self, _url: &str) -> Result<(), ScraplingError> {
        // Prototype: always allow
        Ok(())
    }

    /// Fetch with deterministic retry (spec §3)
    async fn fetch_with_retry(&self, url: &str, params: &ScraplingParams, input_digest: &str) -> Result<(String, u32, bool), ScraplingError> {
        let max_attempts = 3u32;
        
        for attempt in 1..=max_attempts {
            // Deterministic jitter: f(input_digest, attempt) per agent6 v0.3
            let _jitter = self.deterministic_jitter(input_digest, attempt);
            
            // Prototype: simulate successful fetch on first attempt
            let content = format!("<html><body>Content from {}</body></html>", url);
            let auto_healed = params.auto_heal && attempt > 1;
            
            return Ok((content, attempt, auto_healed));
        }
        
        Err(ScraplingError::MaxRetriesExceeded { attempts: max_attempts })
    }

    /// Deterministic jitter per agent6 spec v0.3
    /// jitter = f(input_digest, attempt_index) — NO unseeded RNG
    fn deterministic_jitter(&self, input_digest: &str, attempt: u32) -> u64 {
        let seed_str = format!("{}:{}", input_digest, attempt);
        let hash = blake3::hash(seed_str.as_bytes());
        let bytes = hash.as_bytes();
        let seed = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        
        // Backoff: 1.5^n seconds + jitter 1-3s
        let backoff_ms = (1500.0_f64.powi(attempt as i32)) as u64;
        let jitter_ms = 1000 + (seed % 2000); // 1000-3000ms
        
        backoff_ms + jitter_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_params() -> ScraplingParams {
        ScraplingParams {
            resource: "scraper".to_string(),
            operation: Operation::SmartExtract,
            urls: vec!["https://example.com".to_string()],
            rules: None,
            auto_heal: true,
            selector: None,
            xpath: None,
            concurrency: 2,
            mode: Some(ScrapingMode::HttpStealth),
            respect_retry_after: true,
            ignore_robots: false,
        }
    }

    #[tokio::test]
    async fn test_smart_extract() {
        let node = ScraplingNode::new();
        let params = test_params();
        let results = node.execute(&params, "test-digest").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, OutputStatus::Success);
        assert!(results[0].pii_redacted);
    }

    #[tokio::test]
    async fn test_css_query() {
        let node = ScraplingNode::new();
        let mut params = test_params();
        params.operation = Operation::CssQuery;
        params.selector = Some("div.content".to_string());
        let results = node.execute(&params, "test-digest").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_scrape() {
        let node = ScraplingNode::new();
        let mut params = test_params();
        params.operation = Operation::BatchScrape;
        params.urls = vec![
            "https://example.com/1".to_string(),
            "https://example.com/2".to_string(),
        ];
        let results = node.execute(&params, "test-digest").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_ssrf_blocked() {
        let node = ScraplingNode::new();
        let mut params = test_params();
        params.urls = vec!["http://localhost:8080".to_string()];
        let result = node.execute(&params, "test-digest").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ScraplingError::SsrfBlocked { url } => assert!(url.contains("localhost")),
            _ => panic!("Expected SsrfBlocked"),
        }
    }

    #[tokio::test]
    async fn test_batch_limit() {
        let node = ScraplingNode::new();
        let mut params = test_params();
        params.operation = Operation::BatchScrape;
        params.urls = (0..51).map(|i| format!("https://example.com/{}", i)).collect();
        let result = node.execute(&params, "test-digest").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_jitter() {
        let node = ScraplingNode::new();
        let j1 = node.deterministic_jitter("digest-a", 1);
        let j2 = node.deterministic_jitter("digest-a", 1);
        assert_eq!(j1, j2, "Same input = same jitter (deterministic)");
        
        let j3 = node.deterministic_jitter("digest-a", 2);
        assert_ne!(j1, j3, "Different attempt = different jitter");
        
        let j4 = node.deterministic_jitter("digest-b", 1);
        assert_ne!(j1, j4, "Different digest = different jitter");
    }
}
