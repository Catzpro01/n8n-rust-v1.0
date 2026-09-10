use serde::{Deserialize, Serialize};

/// Operation types (spec §1 field #2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    SmartExtract,
    CssQuery,
    XpathQuery,
    BatchScrape,
}

/// Mode (spec §1 field #9)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScrapingMode {
    HttpStealth,
    BrowserFallback,
}

/// Node parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScraplingParams {
    pub resource: String,                    // field #1: always "scraper"
    pub operation: Operation,                // field #2
    pub urls: Vec<String>,                   // field #3: single URL or batch
    #[serde(default)]
    pub rules: Option<serde_json::Value>,    // field #4: smartExtract rules (opaque JSON)
    #[serde(default = "default_true")]
    pub auto_heal: bool,                     // field #5
    #[serde(default)]
    pub selector: Option<String>,            // field #6: cssQuery
    #[serde(default)]
    pub xpath: Option<String>,              // field #7: xpathQuery
    #[serde(default = "default_concurrency")]
    pub concurrency: u8,                     // field #8: batchScrape (1-5)
    #[serde(default)]
    pub mode: Option<ScrapingMode>,          // field #9
    #[serde(default = "default_true")]
    pub respect_retry_after: bool,           // field #10
    #[serde(default)]
    pub ignore_robots: bool,                 // field #11
}

fn default_true() -> bool { true }
fn default_concurrency() -> u8 { 2 }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_params_deserialize() {
        let json = r#"{
            "resource": "scraper",
            "operation": "smartExtract",
            "urls": ["https://example.com"],
            "autoHeal": true,
            "concurrency": 3
        }"#;
        let params: ScraplingParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.operation, Operation::SmartExtract);
        assert_eq!(params.concurrency, 3);
        assert!(params.auto_heal);
    }
    
    #[test]
    fn test_operations() {
        for op in ["smartExtract", "cssQuery", "xpathQuery", "batchScrape"] {
            let json = format!(r#"{{"resource":"scraper","operation":"{}","urls":["https://x.com"]}}"#, op);
            let params: ScraplingParams = serde_json::from_str(&json).unwrap();
            assert!(!params.urls.is_empty());
        }
    }
}
