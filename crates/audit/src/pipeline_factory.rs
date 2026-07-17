use common::config::RepositoryKind;
use schemas::audit::PipelineKind;
use std::path::Path;

use crate::yaml_runner::{self, YamlPipeline};

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

    /// Load YAML-defined pipelines from a standard's audit directory.
    /// Looks for `audit/pipelines/*.yaml` files in the given path.
    pub fn load_yaml_pipelines_from_dir(dir: &Path) -> Vec<YamlPipeline> {
        let mut pipelines = Vec::new();
        let pipelines_dir = dir.join("audit").join("pipelines");

        if !pipelines_dir.exists() {
            return pipelines;
        }

        if let Ok(entries) = std::fs::read_dir(&pipelines_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("yaml")
                    || path.extension().and_then(|e| e.to_str()) == Some("yml")
                {
                    match yaml_runner::load_pipeline_from_file(&path) {
                        Ok(def) => {
                            pipelines.push(YamlPipeline::new(def));
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load YAML pipeline from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        pipelines
    }

    /// Load a single YAML pipeline from a file path.
    pub fn load_yaml_pipeline(path: &Path) -> Option<YamlPipeline> {
        match yaml_runner::load_pipeline_from_file(path) {
            Ok(def) => Some(YamlPipeline::new(def)),
            Err(e) => {
                tracing::warn!("Failed to load YAML pipeline from {}: {}", path.display(), e);
                None
            }
        }
    }

    /// Load the YAML pipelines defined by one specific registered standard
    /// (`.samgraha/standards/{standard_name}/audit/pipelines/*.yaml`).
    /// A standard directory holds one pipeline per domain, so the caller
    /// still needs `domain` to pick among them when there's more than one.
    pub fn load_yaml_pipelines_for_standard(project_root: &Path, standard_name: &str) -> Vec<YamlPipeline> {
        let dir = project_root.join(".samgraha").join("standards").join(standard_name);
        Self::load_yaml_pipelines_from_dir(&dir)
    }

    /// Discover all YAML pipelines from the project's standard directories.
    /// Checks `.samgraha/standards/` for any registered standards with audit pipelines.
    pub fn discover_yaml_pipelines(project_root: &Path) -> Vec<YamlPipeline> {
        let mut pipelines = Vec::new();

        // Check the project's own audit directory
        let project_audit_dir = project_root.join("audit").join("pipelines");
        if project_audit_dir.exists() {
            pipelines.extend(Self::load_yaml_pipelines_from_dir(project_root));
        }

        // Check .samgraha/standards/ for registered standards
        let standards_dir = project_root.join(".samgraha").join("standards");
        if standards_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&standards_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        pipelines.extend(Self::load_yaml_pipelines_from_dir(&path));
                    }
                }
            }
        }

        pipelines
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

    #[test]
    fn load_yaml_pipeline_from_valid_file() {
        let dir = std::env::temp_dir().join("samgraha-test-pipeline-factory");
        let pipelines_dir = dir.join("audit").join("pipelines");
        std::fs::create_dir_all(&pipelines_dir).unwrap();

        let yaml = r#"
pipeline:
  name: test-pipeline
  version: "1.0.0"
  description: "Test pipeline"
workflow:
  stages:
    - id: det
      type: deterministic
stages:
  det:
    rules:
      - id: T-1
        evidence:
          type: file_presence
          paths: ["test.txt"]
calculation:
  method: weighted_average
  inputs:
    - name: det
      weight: 100
      source: det
"#;
        std::fs::write(pipelines_dir.join("test.yaml"), yaml).unwrap();

        let pipelines = PipelineFactory::load_yaml_pipelines_from_dir(&dir);
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].def.pipeline.name, "test-pipeline");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_yaml_pipelines_for_standard_scopes_to_that_standards_directory() {
        // Two standards each define an "infrastructure" pipeline — the
        // per-standard loader must not conflate them (the bug fixed here:
        // resolving a standard used to flatten every registered standard's
        // pipelines into one list and match by pipeline name alone).
        let root = std::env::temp_dir().join("samgraha-test-pipeline-factory-standards");
        for standard in ["python_hackathon", "other_standard"] {
            let pipelines_dir = root.join(".samgraha").join("standards").join(standard).join("audit").join("pipelines");
            std::fs::create_dir_all(&pipelines_dir).unwrap();
            let yaml = format!(
                r#"
pipeline:
  name: infrastructure
  version: "1.0.0"
  description: "{standard}'s infrastructure pipeline"
workflow:
  stages:
    - id: det
      type: deterministic
stages:
  det:
    rules:
      - id: T-1
        evidence:
          type: file_presence
          paths: ["test.txt"]
calculation:
  method: weighted_average
  inputs:
    - name: det
      weight: 100
      source: det
"#
            );
            std::fs::write(pipelines_dir.join("infrastructure.yaml"), yaml).unwrap();
        }

        let pipelines = PipelineFactory::load_yaml_pipelines_for_standard(&root, "python_hackathon");
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].def.pipeline.description, "python_hackathon's infrastructure pipeline");

        let other = PipelineFactory::load_yaml_pipelines_for_standard(&root, "other_standard");
        assert_eq!(other.len(), 1);
        assert_eq!(other[0].def.pipeline.description, "other_standard's infrastructure pipeline");

        assert!(PipelineFactory::load_yaml_pipelines_for_standard(&root, "nonexistent").is_empty());

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn load_yaml_pipeline_returns_none_for_invalid() {
        let dir = std::env::temp_dir().join("samgraha-test-pipeline-factory-invalid");
        let pipelines_dir = dir.join("audit").join("pipelines");
        std::fs::create_dir_all(&pipelines_dir).unwrap();
        std::fs::write(pipelines_dir.join("bad.yaml"), "not: valid: yaml: [").unwrap();

        let pipelines = PipelineFactory::load_yaml_pipelines_from_dir(&dir);
        assert!(pipelines.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }
}
