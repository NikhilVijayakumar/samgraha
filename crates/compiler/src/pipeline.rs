use crate::discovery::{DiscoveredDocument, DiscoveryEngine};
use crate::processing::DocumentProcessor;
use crate::resolution::RelationshipResolver;
use anyhow::Result;
use schemas::compilation::{CompilationError, CompilationErrorType, CompilationResult};
use schemas::document::Document;
use schemas::standard::StandardDefinition;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::info;

pub struct CompilationPipeline;

pub struct CompilationOutput {
    pub result: CompilationResult,
    pub documents: Vec<Document>,
}

impl CompilationPipeline {
    /// `known_hashes`: path → stored hash. Documents whose hash matches are skipped (incremental).
    /// Pass empty map for a full/clean build.
    pub fn compile<P: AsRef<Path>>(
        root: P,
        standards: &[StandardDefinition],
        scope: Option<&[String]>,
        known_hashes: &std::collections::HashMap<String, String>,
    ) -> Result<CompilationOutput> {
        let root = root.as_ref();
        let start = std::time::Instant::now();

        let discovered = DiscoveryEngine::discover(
            root,
            &[],
            &[
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
        )?;

        let filtered: Vec<DiscoveredDocument> = match scope {
            Some(domains) => discovered
                .into_iter()
                .filter(|d| domains.contains(&d.standard))
                .collect(),
            None => discovered,
        };

        info!("Discovered {} documents", filtered.len());

        let standard_map: std::collections::HashMap<&str, &StandardDefinition> = standards
            .iter()
            .map(|s| (s.id.as_str(), s))
            .collect();

        let doc_count = AtomicUsize::new(0);
        let fail_count = AtomicUsize::new(0);
        let skip_count = AtomicUsize::new(0);
        let mut documents: Vec<Document> = Vec::new();
        let mut errors = Vec::new();

        for discovered in &filtered {
            let rel_key = discovered.relative_path.to_string_lossy().to_string();

            // Incremental: read file hash first and compare before full processing.
            if !known_hashes.is_empty() {
                if let Ok(content) = std::fs::read_to_string(&discovered.path) {
                    let hash = crate::processing::compute_content_hash(&content);
                    if known_hashes.get(&rel_key).map(|s| s.as_str()) == Some(&hash) {
                        skip_count.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }
                }
            }

            let standard_def = standard_map.get(discovered.standard.as_str()).copied();
            let id = doc_count.fetch_add(1, Ordering::SeqCst) as i64 + 1;
            match DocumentProcessor::process(
                &discovered.path,
                &discovered.relative_path,
                &discovered.standard,
                id,
                standard_def,
            ) {
                Ok(doc) => {
                    documents.push(doc);
                }
                Err(e) => {
                    fail_count.fetch_add(1, Ordering::SeqCst);
                    errors.push(CompilationError {
                        path: Some(discovered.path.to_string_lossy().to_string()),
                        message: e.to_string(),
                        error_type: CompilationErrorType::Processing,
                    });
                }
            }
        }

        let relationships = RelationshipResolver::resolve_all(&documents, standards);
        info!(
            "Resolved {} relationships across {} documents",
            relationships.len(),
            documents.len()
        );

        let duration = start.elapsed();
        let skipped = skip_count.load(Ordering::SeqCst);
        let failed = fail_count.load(Ordering::SeqCst);

        Ok(CompilationOutput {
            result: CompilationResult {
                success: failed == 0,
                documents_found: filtered.len(),
                documents_processed: documents.len(),
                documents_failed: failed,
                documents_skipped: skipped,
                errors,
                warnings: Vec::new(),
                duration_ms: duration.as_millis() as u64,
                registry_path: None,
            },
            documents,
        })
    }
}
