use crate::discovery::{DiscoveredDocument, DiscoveryEngine};
use crate::graph::RelationshipBuilder;
use crate::processing::DocumentProcessor;
use crate::resolution::RelationshipResolver;
use schemas::graph::EdgeType;
use crate::validation::SectionValidator;
use anyhow::Result;
use schemas::compilation::{CompilationError, CompilationErrorType, CompilationResult};
use schemas::document::Document;
use schemas::standard::StandardDefinition;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::info;

pub struct CompilationPipeline;

pub struct CompilationOutput {
    pub result: CompilationResult,
    pub documents: Vec<Document>,
    pub graph: schemas::graph::KnowledgeGraph,
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

        let standard_map: HashMap<&str, &StandardDefinition> = standards
            .iter()
            .map(|s| (s.id.as_str(), s))
            .collect();

        let doc_count = AtomicUsize::new(0);
        let fail_count = AtomicUsize::new(0);
        let skip_count = AtomicUsize::new(0);
        let mut documents: Vec<Document> = Vec::new();
        let mut errors = Vec::new();
        let mut all_diagnostics = Vec::new();

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
                    // Run validation
                    let file_path = doc.path.as_str().to_string();
                    let prohibited = standard_def
                        .map(|s| s.prohibited_content.clone())
                        .unwrap_or_default();
                    let doc_sections: Vec<schemas::document::DocumentSection> =
                        doc.body.sections().into_iter().cloned().collect();
                    let diagnostics =
                        SectionValidator::validate(&doc_sections, standard_def, &prohibited, &file_path);
                    all_diagnostics.extend(diagnostics);
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

        let graph = RelationshipBuilder::build_graph(&documents);
        info!(
            "Built knowledge graph with {} nodes and {} edges",
            graph.nodes.len(),
            graph.edges.len()
        );

        // Update per-document relationship counts from graph edges.
        // Sub-item nodes have document_id 0, so count by matching document node URNs.
        {
            let doc_urn_to_id: std::collections::HashMap<String, i64> = graph
                .nodes
                .iter()
                .filter(|n| n.node_type == "document")
                .filter_map(|n| n.document_id.map(|id| (n.urn.as_str().to_string(), id)))
                .collect();
            let mut doc_rel_counts: std::collections::HashMap<i64, usize> =
                std::collections::HashMap::new();
            for edge in &graph.edges {
                if edge.edge_type != EdgeType::ParentOf && edge.edge_type != EdgeType::ChildOf {
                    if let Some(&id) = doc_urn_to_id.get(edge.source_urn.as_str()) {
                        *doc_rel_counts.entry(id).or_default() += 1;
                    }
                }
            }
            for doc in &mut documents {
                doc.quality.total_relationship_count =
                    doc_rel_counts.get(&doc.id).copied().unwrap_or(0);
            }
        }

        // Aggregate quality by summing per-document stats.
        // body.sections() only works for Generic variant, so we use pre-computed per-doc quality.
        let quality_stats = if documents.is_empty() {
            None
        } else {
            let mut agg = schemas::quality::ObjectStatistics::default();
            for doc in &documents {
                agg.total_section_count += doc.quality.total_section_count;
                agg.required_section_count += doc.quality.required_section_count;
                agg.missing_section_count += doc.quality.missing_section_count;
                agg.empty_section_count += doc.quality.empty_section_count;
                agg.total_knowledge_object_count += doc.quality.total_knowledge_object_count;
                agg.total_relationship_count += doc.quality.total_relationship_count;
                for (k, v) in &doc.quality.per_type {
                    *agg.per_type.entry(k.clone()).or_default() += v;
                }
            }
            agg.coverage = agg.coverage_ratio();
            Some(agg)
        };

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
                diagnostics: all_diagnostics,
                quality: quality_stats,
                duration_ms: duration.as_millis() as u64,
                registry_path: None,
            },
            documents,
            graph,
        })
    }
}
