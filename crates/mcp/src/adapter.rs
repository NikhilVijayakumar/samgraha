use crate::protocol::{McpCapabilities, McpError, McpMessage, McpRequest, McpResponse};
use anyhow::Result;
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::search::{RetrievalLevel, SearchQuery};
use services::KnowledgeRuntime;
use std::sync::Arc;

pub struct McpAdapter {
    runtime: Arc<KnowledgeRuntime>,
    capabilities: McpCapabilities,
}

impl McpAdapter {
    pub fn new(runtime: Arc<KnowledgeRuntime>) -> Self {
        Self {
            runtime,
            capabilities: McpCapabilities::default_capabilities(),
        }
    }

    pub fn capabilities(&self) -> &McpCapabilities {
        &self.capabilities
    }

    pub fn handle_message(&self, message: McpMessage) -> McpMessage {
        match message {
            McpMessage::Request(req) => self.handle_request(req),
            McpMessage::Notification(_) => {
                // Notifications are fire-and-forget; no response needed
                McpMessage::Response(McpResponse {
                    id: "ack".to_string(),
                    result: serde_json::json!({}),
                })
            }
            _ => McpMessage::Error(McpError {
                id: None,
                code: -32600,
                message: "Invalid message type".to_string(),
            }),
        }
    }

    fn handle_request(&self, req: McpRequest) -> McpMessage {
        let result: Result<serde_json::Value> = match req.method.as_str() {
            "ping" => Ok(serde_json::json!({"pong": "pong"})),
            "capabilities" => Ok(serde_json::to_value(&self.capabilities).unwrap_or_default()),
            "compile" => self.handle_compile(&req),
            "search" => self.handle_search(&req),
            "audit" => self.handle_audit(&req),
            "info" => self.handle_info(&req),
            "get_document" => self.handle_get_document(&req),
            "list_domains" => self.handle_list_domains(),
            _ => Err(anyhow::anyhow!("Unknown method: {}", req.method)),
        };

        match result {
            Ok(res) => McpMessage::Response(McpResponse {
                id: req.id,
                result: res,
            }),
            Err(e) => McpMessage::Error(McpError {
                id: Some(req.id),
                code: -32000,
                message: e.to_string(),
            }),
        }
    }

    fn handle_compile(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let force = req
            .params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let domains = req
            .params
            .get("domains")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let scope = if domains.is_empty() {
            CompilationScope::Repository
        } else {
            CompilationScope::Domains(domains)
        };

        let request = CompilationRequest {
            scope,
            force,
            watch: false,
        };

        let result = self.runtime.compile(&request)?;
        Ok(serde_json::to_value(&result)?)
    }

    fn handle_search(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let query = req
            .params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

        let level = req
            .params
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("metadata");

        let domain = req.params.get("domain").and_then(|v| v.as_str());

        let max = req.params.get("max").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

        let search_level = match level {
            "summary" => RetrievalLevel::Summary,
            "section" => RetrievalLevel::Section,
            "full" => RetrievalLevel::Full,
            _ => RetrievalLevel::Metadata,
        };

        let search_query = SearchQuery {
            query: query.to_string(),
            domain: domain.map(|d| d.to_string()),
            level: search_level,
            max_results: max,
            ..Default::default()
        };

        let results = self.runtime.search(&search_query)?;
        Ok(serde_json::to_value(&results)?)
    }

    fn handle_audit(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let domain = req.params.get("domain").and_then(|v| v.as_str());
        let providers = req
            .params
            .get("providers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["deterministic".to_string()]);

        let report = self.runtime.audit(domain, &providers, None)?;
        Ok(serde_json::to_value(&report)?)
    }

    fn handle_info(&self, _req: &McpRequest) -> Result<serde_json::Value> {
        let info = self.runtime.info();
        Ok(serde_json::to_value(&info)?)
    }

    fn handle_get_document(&self, req: &McpRequest) -> Result<serde_json::Value> {
        let doc_id = req
            .params
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;

        let doc = self
            .runtime
            .get_document(doc_id)?
            .ok_or_else(|| anyhow::anyhow!("Document not found: {}", doc_id))?;

        Ok(serde_json::to_value(&doc)?)
    }

    fn handle_list_domains(&self) -> Result<serde_json::Value> {
        let domains: Vec<String> = self
            .runtime
            .standard_registry
            .domains()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        Ok(serde_json::json!({
            "domains": domains,
            "count": domains.len(),
        }))
    }
}
