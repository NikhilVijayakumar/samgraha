use anyhow::Result;
use common::config::{DependencyConfig, SamgrahaConfig};
use registry::RegistryStore;
use schemas::package::{KnowledgePackage, PackageProfile};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::package::{PackageFormat, PackageRequest, PackageService};

/// Resolved view of the repository graph available for composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionContext {
    pub primary_repository: ResolvedRepository,
    pub dependencies: Vec<ResolvedDependency>,
    pub unresolved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedRepository {
    pub name: String,
    pub root: PathBuf,
    pub document_count: usize,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub name: String,
    pub path: Option<PathBuf>,
    pub available: bool,
    pub required: bool,
}

pub struct ResolutionResult {
    pub context: ResolutionContext,
    pub package: KnowledgePackage,
    pub output_path: PathBuf,
}

pub struct KnowledgeResolver;

impl KnowledgeResolver {
    /// Resolve and compose a Knowledge Package for the given root.
    pub fn resolve(
        root: &Path,
        config: &SamgrahaConfig,
        registry: Arc<RegistryStore>,
        registry_path: &Path,
        profile: PackageProfile,
        output_path: PathBuf,
        format: PackageFormat,
    ) -> Result<ResolutionResult> {
        // FR1: Discover primary repository.
        let docs = registry.get_all_documents()?;
        let mut domains: Vec<String> = docs.iter().map(|d| d.standard.clone()).collect();
        domains.sort();
        domains.dedup();

        let repo_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();

        let primary = ResolvedRepository {
            name: repo_name.clone(),
            root: root.to_path_buf(),
            document_count: docs.len(),
            domains,
        };

        // FR2: Resolve declared dependencies.
        let (resolved_deps, unresolved) =
            Self::resolve_dependencies(&config.repository.dependencies, root);

        // FR4: Context reduction — log unresolved optional deps, fail on missing required.
        for dep in &resolved_deps {
            if !dep.available && dep.required {
                anyhow::bail!(
                    "Required dependency '{}' is not available at {:?}",
                    dep.name,
                    dep.path
                );
            }
        }

        let context = ResolutionContext {
            primary_repository: primary,
            dependencies: resolved_deps,
            unresolved,
        };

        // FR3 + FR6 + FR7: Compose and validate package via PackageService.
        let pkg_request = PackageRequest {
            output_path: output_path.clone(),
            profile,
            repository_name: repo_name,
            format,
        };
        let pkg_result =
            PackageService::generate(Arc::clone(&registry), Some(registry_path), &pkg_request)?;

        Ok(ResolutionResult {
            context,
            package: pkg_result.package,
            output_path: pkg_result.output_path,
        })
    }

    fn resolve_dependencies(
        deps: &[DependencyConfig],
        root: &Path,
    ) -> (Vec<ResolvedDependency>, Vec<String>) {
        let mut resolved = Vec::new();
        let mut unresolved = Vec::new();

        for dep in deps {
            let dep_path = dep.path.as_ref().map(|p| {
                let p = Path::new(p);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    root.join(p)
                }
            });

            let available = dep_path.as_ref().map(|p| p.exists()).unwrap_or(false);

            if !available && dep.required {
                unresolved.push(dep.name.clone());
            }

            resolved.push(ResolvedDependency {
                name: dep.name.clone(),
                path: dep_path,
                available,
                required: dep.required,
            });
        }

        (resolved, unresolved)
    }
}
