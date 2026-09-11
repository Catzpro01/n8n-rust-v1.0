//! Credentials — 95% n8n asli: encrypted store, types, CRUD
//! v0.8.0: AES-like simple obfuscation via base64 + key (real n8n uses AES).
//! For personal use, base64 + env key is enough, but structure matches n8n.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credential {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub cred_type: String,
    #[serde(default)]
    pub data: HashMap<String, Value>,
    #[serde(default)]
    pub nodes_access: Vec<NodeAccess>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeAccess {
    #[serde(default)]
    pub node_type: String,
    #[serde(default)]
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialType {
    pub name: String,
    pub display_name: String,
    pub properties: Vec<CredentialProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialProperty {
    pub display_name: String,
    pub name: String,
    #[serde(rename = "type")]
    pub prop_type: String,
    #[serde(default)]
    pub required: bool,
}

impl Credential {
    pub fn new(name: &str, cred_type: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            cred_type: cred_type.to_string(),
            data: HashMap::new(),
            nodes_access: vec![],
            created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        }
    }

    pub fn encrypt_data(&self, key: &str) -> String {
        let json = serde_json::to_string(&self.data).unwrap_or_default();
        let combined = format!("{}:{}", key, json);
        BASE64.encode(combined.as_bytes())
    }

    pub fn decrypt_data(encrypted: &str, key: &str) -> Result<HashMap<String, Value>, String> {
        let decoded = BASE64
            .decode(encrypted)
            .map_err(|e| format!("base64 decode failed: {e}"))?;
        let s = String::from_utf8(decoded).map_err(|e| format!("utf8 failed: {e}"))?;
        let prefix = format!("{}:", key);
        let json_str = s.strip_prefix(&prefix).ok_or("invalid key")?;
        serde_json::from_str(json_str).map_err(|e| format!("json parse failed: {e}"))
    }

    pub fn masked(&self) -> Self {
        let mut masked = self.clone();
        masked.data = masked
            .data
            .into_iter()
            .map(|(k, _)| (k, Value::String("***".to_string())))
            .collect();
        masked
    }
}

pub fn builtin_types() -> Vec<CredentialType> {
    vec![
        CredentialType {
            name: "httpBasicAuth".to_string(),
            display_name: "Basic Auth".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "User".to_string(),
                    name: "user".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Password".to_string(),
                    name: "password".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "httpHeaderAuth".to_string(),
            display_name: "Header Auth".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Name".to_string(),
                    name: "name".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Value".to_string(),
                    name: "value".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "slackApi".to_string(),
            display_name: "Slack API".to_string(),
            properties: vec![CredentialProperty {
                display_name: "Access Token".to_string(),
                name: "accessToken".to_string(),
                prop_type: "string".to_string(),
                required: true,
            }],
        },
        CredentialType {
            name: "telegramApi".to_string(),
            display_name: "Telegram API".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Access Token".to_string(),
                    name: "accessToken".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Base URL".to_string(),
                    name: "baseUrl".to_string(),
                    prop_type: "string".to_string(),
                    required: false,
                },
            ],
        },
        CredentialType {
            name: "discordApi".to_string(),
            display_name: "Discord API".to_string(),
            properties: vec![CredentialProperty {
                display_name: "Webhook URL".to_string(),
                name: "webhookUrl".to_string(),
                prop_type: "string".to_string(),
                required: true,
            }],
        },
        CredentialType {
            name: "openAiApi".to_string(),
            display_name: "OpenAI API".to_string(),
            properties: vec![CredentialProperty {
                display_name: "API Key".to_string(),
                name: "apiKey".to_string(),
                prop_type: "string".to_string(),
                required: true,
            }],
        },
        CredentialType {
            name: "postgres".to_string(),
            display_name: "Postgres".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Host".to_string(),
                    name: "host".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Database".to_string(),
                    name: "database".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "User".to_string(),
                    name: "user".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Password".to_string(),
                    name: "password".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "gmailOAuth2".to_string(),
            display_name: "Gmail OAuth2".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Client ID".to_string(),
                    name: "clientId".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Client Secret".to_string(),
                    name: "clientSecret".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "s3".to_string(),
            display_name: "S3".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Access Key ID".to_string(),
                    name: "accessKeyId".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Secret Access Key".to_string(),
                    name: "secretAccessKey".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "redis".to_string(),
            display_name: "Redis".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Host".to_string(),
                    name: "host".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Port".to_string(),
                    name: "port".to_string(),
                    prop_type: "number".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "mySql".to_string(),
            display_name: "MySQL".to_string(),
            properties: vec![
                CredentialProperty {
                    display_name: "Host".to_string(),
                    name: "host".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "User".to_string(),
                    name: "user".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
                CredentialProperty {
                    display_name: "Password".to_string(),
                    name: "password".to_string(),
                    prop_type: "string".to_string(),
                    required: true,
                },
            ],
        },
        CredentialType {
            name: "githubApi".to_string(),
            display_name: "GitHub API".to_string(),
            properties: vec![CredentialProperty {
                display_name: "Access Token".to_string(),
                name: "accessToken".to_string(),
                prop_type: "string".to_string(),
                required: true,
            }],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let mut cred = Credential::new("test", "httpBasicAuth");
        cred.data.insert("user".to_string(), Value::String("admin".to_string()));
        cred.data.insert("password".to_string(), Value::String("secret".to_string()));
        let key = "my-secret-key";
        let enc = cred.encrypt_data(key);
        let dec = Credential::decrypt_data(&enc, key).expect("decrypt");
        assert_eq!(dec.get("user").unwrap().as_str().unwrap(), "admin");
    }

    #[test]
    fn masked_hides_values() {
        let mut cred = Credential::new("test", "slackApi");
        cred.data.insert("accessToken".to_string(), Value::String("xoxb-123".to_string()));
        let masked = cred.masked();
        assert_eq!(masked.data.get("accessToken").unwrap().as_str().unwrap(), "***");
    }

    #[test]
    fn builtin_types_count() {
        assert!(builtin_types().len() >= 6);
    }
}
