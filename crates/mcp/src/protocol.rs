use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// MCP protocol message types following the Model Context Protocol spec

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpMessage {
    #[serde(rename = "request")]
    Request(McpRequest),
    #[serde(rename = "response")]
    Response(McpResponse),
    #[serde(rename = "notification")]
    Notification(McpNotification),
    #[serde(rename = "error")]
    Error(McpError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub id: String,
    pub method: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub repo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub id: String,
    #[serde(default)]
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotification {
    pub method: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub id: Option<String>,
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub version: String,
    pub methods: Vec<String>,
    pub protocol_version: String,
}

impl McpCapabilities {
    pub fn default_capabilities() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            methods: vec![
                "compile".to_string(),
                "search".to_string(),
                "get_sections".to_string(),
                "audit".to_string(),
                "audit_runs".to_string(),
                "info".to_string(),
                "get_document".to_string(),
                "get_document_section".to_string(),
                "list_domains".to_string(),
            ],
            protocol_version: "2025-03-26".to_string(),
        }
    }
}
