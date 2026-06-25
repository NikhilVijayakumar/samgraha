use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::Result;
use schemas::compilation::{CompilationResult, CompilationError, CompilationErrorType};
use schemas::document::Document;
use schemas::standard::StandardDefinition;
use tracing::info;
use crate::discovery::{DiscoveryEngine, DiscoveredDocument};
use crate::processing::DocumentProcessor;
use crate::resolution::RelationshipResolver;

pub struct CompilationPipeline;

impl CompilationPipeline {
    pub fn compile<P: AsRef<Path>>(
        root: P,
        standards: &[StandardDefinition],
        scope: Option<&[String]>,
    ) -> Result<CompilationResult> {
        let root = root.as_ref();
        let start = std::time::Instant::now();

        let discovered = DiscoveryEngine::discover(root, &[], &[
            "node_modules".to_string(),
            "target".to_string(),
            ".git".to_string(),
        ])?;

        let filtered: Vec<DiscoveredDocument> = match scope {
            Some(domains) => discovered
                .into_iter()
                .filter(|d| domains.contains(&d.standard))
                .collect(),
            None => discovered,
        };

        info!("Discovered {} documents", filtered.len());

        let doc_count = AtomicUsize::new(0);
        let fail_count = AtomicUsize::new(0);
        let mut documents: Vec<Document> = Vec::new();
        let mut errors = Vec::new();

        for discovered in &filtered {
            let id = doc_count.fetch_add(1, Ordering::SeqCst) as i64 + 1;
            match DocumentProcessor::process(
                &discovered.path,
                &discovered.relative_path,
                &discovered.standard,
                id,
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

        Ok(CompilationResult {
            success: fail_count.load(Ordering::SeqCst) == 0,
            documents_found: filtered.len(),
            documents_processed: documents.len(),
            documents_failed: fail_count.load(Ordering::SeqCst),
            documents_skipped: filtered.len() - documents.len() - fail_count.load(Ordering::SeqCst),
            errors,
            warnings: Vec::new(),
            duration_ms: duration.as_millis() as u64,
            registry_path: None,
        })
    }
}
