use anyhow::Result;
use common::config::SamgrahaConfig;
use compiler::CompilationPipeline;
use providers::traits::EnrichmentProvider;
use providers::RuleBasedProvider;
use registry::RegistryStore;
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::enrichment::{EnrichmentProfile, EnrichmentType};
use standards::StandardRegistry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

use crate::enrichment::EnrichmentService;

pub struct CompilationService;

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
            let doc_sections: Vec<schemas::document::DocumentSection> = doc.body.sections().into_iter().cloned().collect();
            registry.insert_document_sections(doc.id, &doc_sections)?;
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

        // Enrich newly compiled documents (not skipped ones — their enrichment is still valid).
        if !output.documents.is_empty() {
            let enrichment_enabled = config.ai.provider.is_some()
                || config.ai.lms.is_some()
                || config.ai.ollama.is_some()
                || config.ai.openai.is_some()
                || true; // always run rule-based enrichment

            if enrichment_enabled {
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
