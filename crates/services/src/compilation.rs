use anyhow::Result;
use chrono::Utc;
use common::config::SamgrahaConfig;
use compiler::CompilationPipeline;
use providers::traits::EnrichmentProvider;
use providers::RuleBasedProvider;
use registry::RegistryStore;
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::enrichment::{EnrichmentProfile, EnrichmentType};
use schemas::manifest::{AuditSummary, CompilerInfo, KnowledgeLocation, RepoIdentity, RepositoryManifest};
use standards::StandardRegistry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::enrichment::EnrichmentService;
use crate::registry_client::{FileRegistryClient, RegistryClient};

pub struct CompilationService;

/// Read existing manifest audit fields. Returns (status, last_audit) or defaults.
pub fn read_existing_audit(root: &Path) -> (String, Option<String>) {
    let manifest_path = root.join(".samgraha").join("manifest.json");
    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
        if let Ok(manifest) = serde_json::from_str::<RepositoryManifest>(&content) {
            return (manifest.audit.status, manifest.audit.last_audit);
        }
    }
    ("PASS".to_string(), None)
}

impl CompilationService {
    pub fn execute<P: AsRef<Path>>(
        root: P,
        config: &SamgrahaConfig,
        request: &CompilationRequest,
        standard_registry: &StandardRegistry,
        registry: Arc<RegistryStore>,
    ) -> Result<CompilationResult> {
        let root = root.as_ref();
        info!("Compilation started for {:?}", root);

        let standards: Vec<_> = standard_registry.all().into_iter().cloned().collect();

        let scope = match &request.scope {
            schemas::compilation::CompilationScope::Domains(d) => Some(d.clone()),
            _ => None,
        };

        // Load known hashes for incremental unless force=true.
        let known_hashes: HashMap<String, String> = if request.force {
            HashMap::new()
        } else {
            registry
                .get_all_documents()
                .unwrap_or_default()
                .into_iter()
                .map(|d| (d.path.0.to_string_lossy().to_string(), d.hash))
                .collect()
        };

        let output =
            CompilationPipeline::compile(root, &standards, scope.as_deref(), &known_hashes)?;

        // Persist newly compiled documents and their semantic sections to registry.
        for doc in &output.documents {
            registry.insert_document(doc)?;
            let standard_def = standard_registry.get_by_domain(&doc.standard);
            let sections = compiler::parse_sections(doc.body.raw(), &doc.path.as_str(), standard_def);
            registry.insert_document_sections(doc.id, &sections)?;
        }

        // Persist compiled knowledge graph
        registry.clear_graph()?;
        registry.insert_graph(&output.graph)?;

        // Remove registry entries for files that no longer exist on disk.
        let all_docs = registry.get_all_documents()?;
        for stored in &all_docs {
            let abs = root.join(&stored.path.0);
            if !abs.exists() {
                registry.delete_document(stored.id)?;
            }
        }

        // Write Repository Manifest (Phase F2) — only on full success (zero failures).
        let success = output.result.success;
        if success {
            let current_revision = registry.get_revision().unwrap_or(0);
            let changed = !output.documents.is_empty();
            let next_revision = if changed { current_revision + 1 } else { current_revision };

            let uuid = config.repository.uuid.or_else(|| {
                // Reuse existing manifest UUID if one exists, else generate fresh.
                let manifest_path = root.join(".samgraha").join("manifest.json");
                std::fs::read_to_string(&manifest_path).ok().and_then(|c| {
                    serde_json::from_str::<RepositoryManifest>(&c).ok()
                }).map(|m| m.repository.uuid)
            }).unwrap_or_else(|| {
                let new_uuid = Uuid::new_v4();
                tracing::warn!(
                    "Repository UUID not configured in samgraha.toml. \
                     Generated temporary UUID: {}. \
                     Set repository.uuid in samgraha.toml to make it permanent.",
                    new_uuid
                );
                new_uuid
            });

            let mut exports: Vec<String> = registry
                .get_all_documents()
                .unwrap_or_default()
                .iter()
                .map(|d| d.standard.clone())
                .collect();
            exports.sort();
            exports.dedup();

            let mut capabilities: Vec<String> = vec!["compile".to_string(), "mcp".to_string()];
            if config.audit.providers.iter().any(|p| p == "deterministic") {
                capabilities.push("audit".to_string());
            }
            if config.audit.providers.iter().any(|p| p == "semantic") && config.ai.provider.is_some() {
                capabilities.push("semantic-audit".to_string());
            }
            if config.ai.provider.is_some() {
                capabilities.push("enrichment".to_string());
            }

            let dependencies: Vec<String> = config
                .repository
                .dependencies
                .iter()
                .map(|d| d.name.clone())
                .collect();

            let repo_dir_name = || -> String {
                root.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            };

            let (audit_status, audit_last) = read_existing_audit(root);

            let manifest = RepositoryManifest {
                repository: RepoIdentity {
                    id: config.repository.id.clone().unwrap_or_else(repo_dir_name),
                    name: config.repository.name.clone().unwrap_or_else(repo_dir_name),
                    uuid,
                },
                revision: next_revision,
                compiler: CompilerInfo {
                    version: "0.1.0".to_string(),
                },
                audit: AuditSummary {
                    status: audit_status,
                    last_audit: audit_last,
                },
                repository_root: root.to_string_lossy().to_string(),
                knowledge: KnowledgeLocation {
                    location: ".samgraha/knowledge.db".to_string(),
                },
                exports,
                capabilities,
                dependencies,
                generated_at: Utc::now().to_rfc3339(),
            };

            let manifest_dir = root.join(".samgraha");
            std::fs::create_dir_all(&manifest_dir)
                .unwrap_or_else(|e| tracing::warn!("Cannot create .samgraha dir: {}", e));
            let manifest_path = manifest_dir.join("manifest.json");
            match serde_json::to_string_pretty(&manifest) {
                Ok(json) => {
                    std::fs::write(&manifest_path, &json).unwrap_or_else(|e| {
                        tracing::warn!("Cannot write manifest.json: {}", e)
                    });
                    let _ = registry.set_revision(next_revision);
                    info!(
                        "Repository manifest written to {} (rev {})",
                        manifest_path.display(),
                        next_revision,
                    );
                }
                Err(e) => tracing::warn!("Cannot serialize manifest.json: {}", e),
            }

            // Auto-refresh: if enabled, update local registry after successful compile.
            if config.resolver.auto_refresh {
                if let Ok(json) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<RepositoryManifest>(&json) {
                        let client = FileRegistryClient::with_config(root, &config.resolver);
                        if let Err(e) = client.register(&manifest) {
                            tracing::warn!("Auto-refresh registry update failed: {}", e);
                        }
                    }
                }
            }
        }

        // Enrich newly compiled documents (not skipped ones — their enrichment is still valid).
        if !output.documents.is_empty() {
            let provider = RuleBasedProvider::new();
            let profile = EnrichmentProfile {
                name: "compile".to_string(),
                enabled_types: vec![EnrichmentType::Summary, EnrichmentType::Keywords],
                provider: "rule-based".to_string(),
                model: None,
                batch_size: 50,
            };

            match EnrichmentService::enrich_batch(&provider, &output.documents, &profile) {
                Ok(artifacts) => {
                    registry.insert_enrichments(&artifacts)?;
                    info!("Enriched {} artifacts for {} documents", artifacts.len(), output.documents.len());
                }
                Err(e) => {
                    // Enrichment failure is non-fatal — compilation still succeeds.
                    info!("Enrichment skipped: {}", e);
                }
            }

            // Glossary is a batch operation across all docs.
            let all_compiled = registry.get_all_documents()?;
            match provider.glossary(&all_compiled) {
                Ok(entries) => {
                    let glossary_entries: Vec<schemas::registry::GlossaryEntry> = entries
                        .into_iter()
                        .map(|g| schemas::registry::GlossaryEntry {
                            id: 0,
                            term: g.term,
                        definition: g.definition,
                        source_document_id: None,
                    })
                    .collect();
                    let _ = registry.insert_glossary_entries(&glossary_entries);
                }
                Err(_) => {}
            }
        }

        let registry_path = registry.path_str().map(|s| s.to_string());

        let mut result = output.result;
        result.registry_path = registry_path;

        info!(
            "Compilation complete: {} processed, {} skipped, {} failed in {}ms",
            result.documents_processed,
            result.documents_skipped,
            result.documents_failed,
            result.duration_ms,
        );

        Ok(result)
    }

    pub fn validate_config(config: &SamgrahaConfig, registry: &StandardRegistry) -> Result<()> {
        let decls = &config.repository.documentation.standards;
        for decl in decls {
            if !registry.has_standard(decl) {
                anyhow::bail!("Standard '{}' not found in registry", decl);
            }
        }
        Ok(())
    }
}
