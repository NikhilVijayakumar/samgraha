use anyhow::{Context, Result};
use common::config::{SamgrahaConfig, WorkspaceConfig};
use registry::RegistryStore;
use schemas::compilation::{CompilationRequest, CompilationResult, CompilationScope};
use schemas::search::{SearchQuery, SearchResponse, SearchResult};
use standards::StandardRegistry;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

use crate::compilation::CompilationService;
use crate::search::SearchService;

pub struct WorkspaceService;

pub struct WorkspaceBuildResult {
    pub workspace_name: String,
    pub repository_results: Vec<(String, CompilationResult)>,
    pub total_documents: usize,
    pub total_errors: usize,
}

impl WorkspaceService {
    /// FR7: Walk parent dirs looking for `samgraha-workspace.toml`.
    pub fn discover(from: &Path) -> Option<(PathBuf, WorkspaceConfig)> {
        let mut current = from.to_path_buf();
        loop {
            let candidate = current.join("samgraha-workspace.toml");
            if candidate.exists() {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    if let Ok(config) = toml::from_str::<WorkspaceConfig>(&content) {
                        return Some((current, config));
                    }
                }
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// FR4: Compile every member repository independently, preserving isolation.
    pub fn compile(
        workspace_root: &Path,
        workspace_config: &WorkspaceConfig,
        request: &CompilationRequest,
    ) -> Result<WorkspaceBuildResult> {
        let standard_registry = StandardRegistry::with_builtins_and_overrides(workspace_root)?;
        let mut results = Vec::new();
        let mut total_docs = 0;
        let mut total_errors = 0;

        for repo_path_str in &workspace_config.repositories {
            let repo_root = if Path::new(repo_path_str).is_absolute() {
                PathBuf::from(repo_path_str)
            } else {
                workspace_root.join(repo_path_str)
            };

            if !repo_root.exists() {
                info!("Workspace member {:?} not found, skipping", repo_root);
                results.push((
                    repo_path_str.clone(),
                    schemas::compilation::CompilationResult {
                        success: false,
                        documents_found: 0,
                        documents_processed: 0,
                        documents_failed: 0,
                        documents_skipped: 0,
                        errors: vec![schemas::compilation::CompilationError {
                            path: Some(repo_root.display().to_string()),
                            message: "Repository path not found".into(),
                            error_type: schemas::compilation::CompilationErrorType::Configuration,
                        }],
                        warnings: Vec::new(),
                        diagnostics: Vec::new(),
                        quality: None,
                        duration_ms: 0,
                        registry_path: None,
                    },
                ));
                total_errors += 1;
                continue;
            }

            // Each repo gets its own registry.
            let registry_path = repo_root.join("knowledge.db");
            let registry = Arc::new(
                RegistryStore::open(&registry_path)
                    .context(format!("Failed to open registry for {:?}", repo_root))?,
            );

            let repo_config = Self::load_repo_config(&repo_root);
            let repo_request = CompilationRequest {
                scope: CompilationScope::Repository,
                force: request.force,
                watch: false,
            };

            match CompilationService::execute(
                &repo_root,
                &repo_config,
                &repo_request,
                &standard_registry,
                Arc::clone(&registry),
            ) {
                Ok(result) => {
                    total_docs += result.documents_processed;
                    if !result.success {
                        total_errors += result.documents_failed;
                    }
                    info!(
                        "Compiled {:?}: {} docs, {} errors",
                        repo_root, result.documents_processed, result.documents_failed
                    );
                    results.push((repo_path_str.clone(), result));
                }
                Err(e) => {
                    total_errors += 1;
                    results.push((
                        repo_path_str.clone(),
                        schemas::compilation::CompilationResult {
                            success: false,
                            documents_found: 0,
                            documents_processed: 0,
                            documents_failed: 1,
                            documents_skipped: 0,
                            errors: vec![schemas::compilation::CompilationError {
                                path: Some(repo_root.display().to_string()),
                                message: e.to_string(),
                                error_type: schemas::compilation::CompilationErrorType::Internal,
                            }],
                            warnings: Vec::new(),
                            diagnostics: Vec::new(),
                            quality: None,
                            duration_ms: 0,
                            registry_path: None,
                        },
                    ));
                }
            }
        }

        Ok(WorkspaceBuildResult {
            workspace_name: workspace_config.name.clone(),
            repository_results: results,
            total_documents: total_docs,
            total_errors,
        })
    }

    /// FR5: Search across all member repositories and merge results.
    pub fn search(
        workspace_root: &Path,
        workspace_config: &WorkspaceConfig,
        query: &SearchQuery,
    ) -> Result<SearchResponse> {
        let start = std::time::Instant::now();
        let mut all_results: Vec<SearchResult> = Vec::new();

        for repo_path_str in &workspace_config.repositories {
            let repo_root = if Path::new(repo_path_str).is_absolute() {
                PathBuf::from(repo_path_str)
            } else {
                workspace_root.join(repo_path_str)
            };

            let registry_path = repo_root.join("knowledge.db");
            if !registry_path.exists() {
                continue;
            }

            if let Ok(registry) = RegistryStore::open(&registry_path) {
                if let Ok(docs) = registry.get_all_documents() {
                    if let Ok(resp) = SearchService::search(&docs, query) {
                        all_results.extend(resp.results);
                    }
                }
            }
        }

        // Re-sort merged results by score.
        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(query.max_results);

        let total = all_results.len();
        Ok(SearchResponse {
            results: all_results,
            total_count: total,
            query: query.query.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn load_repo_config(repo_root: &Path) -> SamgrahaConfig {
        let config_path = repo_root.join("samgraha.toml");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<SamgrahaConfig>(&content) {
                    return config;
                }
            }
        }
        SamgrahaConfig::default()
    }

    /// FR1: Create a workspace config file at the given root.
    pub fn init(workspace_root: &Path, name: &str, repositories: Vec<String>) -> Result<PathBuf> {
        let config = WorkspaceConfig {
            name: name.to_string(),
            repositories,
            shared: common::config::SharedWorkspaceConfig::default(),
        };
        let content = toml::to_string_pretty(&config)?;
        let path = workspace_root.join("samgraha-workspace.toml");
        std::fs::write(&path, content)?;
        Ok(path)
    }
}
