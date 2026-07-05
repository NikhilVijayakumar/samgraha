use anyhow::Result;
use common::config::{parse_ttl_duration, SamgrahaConfig};
use registry::RegistryStore;
use schemas::document::Document;
use schemas::search::{SearchQuery, SearchResponse};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::planner::{KnowledgePlan, KnowledgePlanEntry, Planner};
use crate::search::SearchService;

/// In-memory multi-repo knowledge package assembled from a KnowledgePlan.
pub struct RuntimePackage {
    /// Available entries with their open stores, in plan order (primary first).
    pub repos: Vec<(KnowledgePlanEntry, Arc<RegistryStore>)>,
}

impl RuntimePackage {
    /// Open RegistryStore for every available entry in the plan.
    pub fn from_plan(plan: &KnowledgePlan) -> Result<Self> {
        let mut repos = Vec::new();
        for entry in plan.available() {
            match RegistryStore::open(&entry.root.join(".samgraha").join("knowledge.db")) {
                Ok(store) => repos.push((entry.clone(), Arc::new(store))),
                Err(e) => tracing::warn!("Cannot open store for '{}': {}", entry.name, e),
            }
        }
        Ok(Self { repos })
    }

    pub fn all_documents(&self) -> Result<Vec<Document>> {
        let mut docs = Vec::new();
        for (_, store) in &self.repos {
            docs.extend(store.get_all_documents()?);
        }
        Ok(docs)
    }

    pub fn store_count(&self) -> usize {
        self.repos.len()
    }

    /// Loaded repo names in priority order.
    pub fn repo_names(&self) -> Vec<&str> {
        self.repos.iter().map(|(e, _)| e.name.as_str()).collect()
    }
}

fn revision_on_disk(root: &Path) -> Option<u64> {
    let path = root.join(".samgraha").join("manifest.json");
    std::fs::read_to_string(path).ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v["revision"].as_u64())
}

/// Multi-repo knowledge context. Survives MCP disconnects; disposed on TTL expiry or explicit drop.
/// Lifecycle: create → serve → (reuse on reconnect | dispose on TTL).
pub struct KnowledgeContext {
    pub package: RuntimePackage,
    pub plan: KnowledgePlan,
    assembly_time: Instant,
    ttl_secs: u64,
}

impl KnowledgeContext {
    pub fn create(root: &Path, config: &SamgrahaConfig) -> Result<Self> {
        let plan = Planner::plan(root, config);
        let package = RuntimePackage::from_plan(&plan)?;
        let ttl_secs = parse_ttl_duration(&config.resolver.knowledge_ttl)
            .map(|s| s.max(0) as u64)
            .unwrap_or(720 * 3600);
        tracing::debug!(
            "Session planned: {} entries ({} available), TTL {}s",
            plan.entries.len(),
            package.store_count(),
            ttl_secs,
        );
        Ok(Self { package, plan, assembly_time: Instant::now(), ttl_secs })
    }

    /// False when TTL exceeded or any available repo's on-disk revision changed since assembly.
    pub fn is_valid(&self) -> bool {
        if self.assembly_time.elapsed().as_secs() >= self.ttl_secs {
            return false;
        }
        self.plan.entries.iter().filter(|e| e.available).all(|e| {
            revision_on_disk(&e.root).map_or(true, |r| r == e.revision)
        })
    }

    /// Search across all loaded stores (primary + deps + interests).
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResponse> {
        let docs = self.package.all_documents()?;
        SearchService::search(&docs, query)
    }

    pub fn store_count(&self) -> usize {
        self.package.store_count()
    }

    pub fn dispose(self) {
        drop(self);
    }
}

