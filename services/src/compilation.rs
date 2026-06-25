use std::path::Path;
use anyhow::Result;
use schemas::compilation::{CompilationRequest, CompilationResult};
use schemas::config::SamgrahaConfig;
use compiler::CompilationPipeline;
use standards::StandardRegistry;
use tracing::info;

pub struct CompilationService;

impl CompilationService {
    pub fn execute<P: AsRef<Path>>(
        root: P,
        _config: &SamgrahaConfig,
        request: &CompilationRequest,
        standard_registry: &StandardRegistry,
    ) -> Result<CompilationResult> {
        let root = root.as_ref();
        info!("Compilation started for {:?}", root);

        let standards: Vec<_> = standard_registry.all().into_iter().cloned().collect();

        let scope = if request.force {
            None
        } else {
            match &request.scope {
                schemas::compilation::CompilationScope::Domains(d) => Some(d.clone()),
                _ => None,
            }
        };

        let result = CompilationPipeline::compile(
            root,
            &standards,
            scope.as_deref(),
        )?;

        info!(
            "Compilation complete: {} processed, {} failed in {}ms",
            result.documents_processed,
            result.documents_failed,
            result.duration_ms,
        );

        Ok(result)
    }

    pub fn validate_config(
        config: &SamgrahaConfig,
        registry: &StandardRegistry,
    ) -> Result<()> {
        let decls = &config.repository.documentation.standards;
        for decl in decls {
            if !registry.has_standard(decl) {
                anyhow::bail!("Standard '{}' not found in registry", decl);
            }
        }
        Ok(())
    }
}
