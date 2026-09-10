use serde::{Deserialize, Serialize};

/// Canonical output per agent9 spec §5
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScraplingOutput {
    pub url: String,
    pub status: OutputStatus,
    pub content_format: ContentFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_ref: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_ref: Option<String>,
    pub auto_healed: bool,
    pub pii_redacted: bool,
    pub took_ms: u64,
    pub attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum OutputStatus {
    Success,
    Failed,
    AutoHealed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
    Json,
    Markdown,
    Html,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_serialize() {
        let output = ScraplingOutput {
            url: "https://example.com".to_string(),
            status: OutputStatus::Success,
            content_format: ContentFormat::Json,
            data_ref: Some(serde_json::json!({"title": "Test"})),
            content_ref: None,
            auto_healed: false,
            pii_redacted: true,
            took_ms: 250,
            attempts: 1,
            error_code: None,
            input_digest: Some("abc123".to_string()),
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"autoHealed\":false"));
        assert!(json.contains("\"piiRedacted\":true"));
    }
}
