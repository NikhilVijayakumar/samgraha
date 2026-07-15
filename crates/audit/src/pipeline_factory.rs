use common::config::RepositoryKind;
use schemas::audit::PipelineKind;

/// Selects which pipelines make up "the full audit" for a repository, based
/// on its `RepositoryKind` — the audit-side counterpart to
/// `services::compilation::PipelineFactory`. Pure selection only: dispatching
/// each `PipelineKind` to its actual execution method (`audit()` for `Doc`,
/// `run_pipeline()` for everything else) requires registry/DB access this
/// crate doesn't have, so that stays in `services::runtime::KnowledgeRuntime`.
/// Mirrors the Repository Matrix in `docs/crates-refactor-proposal.md` §5/§6.3.
pub struct PipelineFactory;

impl PipelineFactory {
    pub fn for_kind(kind: &RepositoryKind) -> &'static [PipelineKind] {
        match kind {
            RepositoryKind::Repository => &[
                PipelineKind::Doc,
                PipelineKind::Implementation,
                PipelineKind::Build,
                PipelineKind::Security,
            ],
            RepositoryKind::Knowledge => &[PipelineKind::Doc, PipelineKind::KnowledgeSystem],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_kind_runs_doc_implementation_build_security() {
        let selected = PipelineFactory::for_kind(&RepositoryKind::Repository);
        assert_eq!(
            selected,
            &[
                PipelineKind::Doc,
                PipelineKind::Implementation,
                PipelineKind::Build,
                PipelineKind::Security,
            ]
        );
    }

    #[test]
    fn knowledge_kind_runs_doc_and_knowledge_system() {
        let selected = PipelineFactory::for_kind(&RepositoryKind::Knowledge);
        assert_eq!(selected, &[PipelineKind::Doc, PipelineKind::KnowledgeSystem]);
    }
}
