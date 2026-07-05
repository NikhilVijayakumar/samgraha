use anyhow::Result;
use common::config::{parse_ttl_duration, SamgrahaConfig};
use registry::RegistryStore;
use schemas::document::Document;
use schemas::search::{SearchQuery, SearchResponse, SectionQuery, SectionQueryResponse};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::planner::{KnowledgePlan, KnowledgePlanEntry, Planner, Priority};
use std::sync::Mutex;
use crate::search::SearchService;

/// In-memory multi-repo knowledge package assembled from a KnowledgePlan.
pub struct RuntimePackage {
    /// Primary + dependency stores — opened eagerly (always needed).
    eager: Vec<(KnowledgePlanEntry, Arc<RegistryStore>)>,
    /// Interest entries — store opened on first all_stores() call.
    // ponytail: lazy open; interests are optional and may never be queried
    interests: Mutex<Vec<(KnowledgePlanEntry, Option<Arc<RegistryStore>>)>>,
    /// Read-only built-in knowledge stores (help, standards) shipped next to the binary.
    builtin: Vec<(String, Arc<RegistryStore>)>,
}

impl RuntimePackage {
    /// Open primary + dependency stores eagerly; defer interest stores.
    pub fn from_plan(plan: &KnowledgePlan) -> Result<Self> {
        let mut eager = Vec::new();
        let mut pending = Vec::new();
        for entry in plan.available() {
            if entry.priority == Priority::Interest {
                pending.push((entry.clone(), None));
            } else {
                match RegistryStore::open(&entry.root.join(".samgraha").join("knowledge.db")) {
                    Ok(store) => eager.push((entry.clone(), Arc::new(store))),
                    Err(e) => tracing::warn!("Cannot open store for '{}': {}", entry.name, e),
                }
            }
        }
        let builtin = crate::builtin::load_builtin_stores();
        Ok(Self { eager, interests: Mutex::new(pending), builtin })
    }

    /// All stores in priority order (primary, deps, interests, built-in); opens interest stores on first call.
    fn all_stores(&self) -> Vec<Arc<RegistryStore>> {
        let mut stores: Vec<Arc<RegistryStore>> = self.eager.iter().map(|(_, s)| Arc::clone(s)).collect();
        let mut interests = self.interests.lock().unwrap();
        for (entry, slot) in interests.iter_mut() {
            if slot.is_none() {
                match RegistryStore::open(&entry.root.join(".samgraha").join("knowledge.db")) {
                    Ok(s) => *slot = Some(Arc::new(s)),
                    Err(e) => tracing::warn!("Cannot open interest store for '{}': {}", entry.name, e),
                }
            }
            if let Some(s) = slot {
                stores.push(Arc::clone(s));
            }
        }
        stores.extend(self.builtin.iter().map(|(_, s)| Arc::clone(s)));
        stores
    }

    pub fn all_documents(&self) -> Result<Vec<Document>> {
        let mut docs = Vec::new();
        for store in self.all_stores() {
            docs.extend(store.get_all_documents()?);
        }
        Ok(docs)
    }

    /// Count of eagerly-open stores (primary + deps loaded at create time).
    pub fn store_count(&self) -> usize {
        self.eager.len()
    }

    /// Eagerly-loaded repo names in priority order.
    pub fn repo_names(&self) -> Vec<&str> {
        self.eager.iter().map(|(e, _)| e.name.as_str()).collect()
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

    /// Sections across all loaded stores, merged by semantic_type.
    pub fn get_sections(&self, query: &SectionQuery) -> Result<SectionQueryResponse> {
        let mut sections = Vec::new();
        let mut duration_ms = 0u64;
        for store in self.package.all_stores() {
            let resp = store.get_sections_by_type(query)?;
            sections.extend(resp.sections);
            duration_ms = duration_ms.max(resp.duration_ms);
        }
        let total_count = sections.len();
        Ok(SectionQueryResponse { sections, total_count, semantic_type: query.semantic_type.clone(), duration_ms })
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

