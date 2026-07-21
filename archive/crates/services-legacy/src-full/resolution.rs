use anyhow::{Context, Result};
use common::config::{parse_ttl_duration, DependencyConfig, SamgrahaConfig};
use registry::registry_db::RegistryDb;
use schemas::manifest::CachedRepoMetadata;
use schemas::package::{KnowledgePackage, PackageLayout, PackageProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::debug;

use crate::metadata_cache::MetadataCache;
use crate::package::{DependencyRepo, PackageFormat, PackageRequest, PackageService};

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
    pub domains: Vec<String>,
    pub revision: u64,
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
        registry: Arc<registry::RegistryStore>,
        registry_path: &Path,
        profile: PackageProfile,
        output_path: PathBuf,
        format: PackageFormat,
        layout: PackageLayout,
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

        // Phase 6: Resolve dependency graph via SQLite-backed RegistryDb.
        let db = RegistryDb::open(root).ok();
        let ttl_seconds: i64 = parse_ttl_duration(&config.resolver.metadata_ttl).unwrap_or(86400);
        let (resolved_deps, unresolved) = Self::resolve_dependency_graph(
            &config.repository.dependencies,
            root,
            db.as_ref(),
            ttl_seconds,
        );

        let dep_repos: Vec<DependencyRepo> = resolved_deps
            .iter()
            .filter(|d| d.available)
            .map(|d| DependencyRepo {
                id: d.name.clone(),
                name: d.name.clone(),
                root: d
                    .path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                knowledge_db: d
                    .path
                    .as_ref()
                    .map(|p| {
                        p.join(".samgraha")
                            .join("knowledge.db")
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_default(),
                revision: d.revision,
            })
            .collect();

        // FR4: Context reduction — fail on missing required deps.
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

        let pkg_request = PackageRequest {
            output_path: output_path.clone(),
            profile,
            repository_name: repo_name,
            format,
            layout,
            primary_root: Some(root.to_string_lossy().to_string()),
        };
        let pkg_result = PackageService::generate(
            Arc::clone(&registry),
            Some(registry_path),
            &pkg_request,
            &dep_repos,
        )?;

        Ok(ResolutionResult {
            context,
            package: pkg_result.package,
            output_path: pkg_result.output_path,
        })
    }

    /// Resolve the full dependency graph with DFS + cycle detection.
    ///
    /// Each dependency is checked against the SQLite-backed registry cache first.
    /// On cache miss, falls back to reading the dependency's manifest.json directly.
    /// Detects cycles via recursion stack — aborts with full cycle path on detection.
    ///
    /// Known limitation: transitive deps extracted from manifests have no explicit path.
    /// If a transitive dep has no matching entry in the top-level `deps` slice, it is
    /// marked unavailable and DFS does not recurse into it. Cycles that run entirely
    /// through such transitive-only deps are not detected. Declare all cycle-forming
    /// deps in the top-level config to ensure detection.
    pub fn resolve_dependency_graph(
        deps: &[DependencyConfig],
        root: &Path,
        db: Option<&RegistryDb>,
        ttl_seconds: i64,
    ) -> (Vec<ResolvedDependency>, Vec<String>) {
        let mut resolved: Vec<ResolvedDependency> = Vec::new();
        let mut unresolved: Vec<String> = Vec::new();
        let mut globally_visited: HashSet<String> = HashSet::new();

        for dep in deps {
            let mut stack: Vec<(String, PathBuf)> = Vec::new();
            if let Err(e) = Self::dfs_resolve(
                dep, deps, root, db, &mut resolved, &mut unresolved,
                &mut globally_visited, &mut stack,
                ttl_seconds,
            ) {
                let msg = format!("{:#}", e);
                unresolved.push(msg);
            }
        }

        (resolved, unresolved)
    }

    fn dfs_resolve(
        dep: &DependencyConfig,
        config_deps: &[DependencyConfig],
        current_root: &Path,
        db: Option<&RegistryDb>,
        resolved: &mut Vec<ResolvedDependency>,
        unresolved: &mut Vec<String>,
        globally_visited: &mut HashSet<String>,
        stack: &mut Vec<(String, PathBuf)>,
        ttl_seconds: i64,
    ) -> Result<()> {
        if stack.iter().any(|(name, _)| name == &dep.name) {
            let cycle_path: Vec<String> = stack.iter().map(|(n, _)| n.clone()).collect();
            anyhow::bail!(
                "Dependency cycle detected: {} → {}",
                cycle_path.join(" → "),
                dep.name,
            );
        }
        if !globally_visited.insert(dep.name.clone()) {
            return Ok(());
        }

        let effective_path = dep.path.as_ref().or_else(|| {
            config_deps.iter().find(|d| d.name == dep.name)
                .and_then(|d| d.path.as_ref())
        });
        let dep_root = effective_path.map(|p| {
            let p = Path::new(p);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                current_root.join(p)
            }
        });

        let available = dep_root.as_ref().map(|p| p.exists()).unwrap_or(false);

        if !available {
            // For required deps: path missing means fail — cache can't substitute
            // for a deleted dependency. Only non-required deps get cache fallback
            // for offline mode.
            if !dep.required {
                if let Some(ref db) = db {
                    if let Ok(Some(cached)) = db.cache_read(&dep.name) {
                        if !cached.is_expired() {
                            debug!("Metadata Cache → Resolver (offline, non-expired cache for '{}')", dep.name);
                            let transitive_deps = cached_to_dep_configs(&cached);
                            resolved.push(ResolvedDependency {
                                name: dep.name.clone(),
                                path: dep_root,
                                available: true,
                                required: dep.required,
                                domains: cached.exports.clone(),
                                revision: cached.revision,
                            });
                            let dep_root = cached.repository_root.clone();
                            for trans_dep in transitive_deps {
                                Self::dfs_resolve(
                                    &trans_dep, config_deps, Path::new(&dep_root), Some(db),
                                    resolved, unresolved, globally_visited, stack,
                                    ttl_seconds,
                                ).with_context(|| format!("While resolving dependency '{}'", dep.name))?;
                            }
                            return Ok(());
                        } else {
                            debug!(
                                "Warning: stale cache used for '{}' (path not found, cache expired)",
                                dep.name
                            );
                            let transitive_deps = cached_to_dep_configs(&cached);
                            resolved.push(ResolvedDependency {
                                name: dep.name.clone(),
                                path: dep_root,
                                available: true,
                                required: dep.required,
                                domains: cached.exports.clone(),
                                revision: cached.revision,
                            });
                            let dep_root = cached.repository_root.clone();
                            for trans_dep in transitive_deps {
                                Self::dfs_resolve(
                                    &trans_dep, config_deps, Path::new(&dep_root), Some(db),
                                    resolved, unresolved, globally_visited, stack,
                                    ttl_seconds,
                                ).with_context(|| format!("While resolving dependency '{}'", dep.name))?;
                            }
                            return Ok(());
                        }
                    }
                }
            }

            if dep.required {
                unresolved.push(dep.name.clone());
            }
            resolved.push(ResolvedDependency {
                name: dep.name.clone(),
                path: dep_root,
                available: false,
                required: dep.required,
                domains: Vec::new(),
                revision: 0,
            });
            return Ok(());
        }

        let dep_root = dep_root.unwrap();
        let (domains, revision, transitive_deps) =
            Self::resolve_dependency_metadata(&dep.name, &dep_root, db, ttl_seconds);

        resolved.push(ResolvedDependency {
            name: dep.name.clone(),
            path: Some(dep_root.clone()),
            available,
            required: dep.required,
            domains,
            revision,
        });

        stack.push((dep.name.clone(), dep_root.clone()));

        for trans_dep in transitive_deps {
            Self::dfs_resolve(
                &trans_dep, config_deps, &dep_root, db, resolved, unresolved,
                globally_visited, stack, ttl_seconds,
            ).with_context(|| format!("While resolving dependency '{}'", dep.name))?;
        }

        stack.pop();
        Ok(())
    }

    /// Resolve a single dependency — use SQLite cache FIRST, fall back to manifest on disk.
    ///
    /// On cache hit (non-expired), returns cached data without disk I/O.
    /// On cache miss or expired, reads manifest from disk and writes to cache.
    fn resolve_dependency_metadata(
        name: &str,
        dep_root: &Path,
        db: Option<&RegistryDb>,
        ttl_seconds: i64,
    ) -> (Vec<String>, u64, Vec<DependencyConfig>) {
        // Try SQLite-backed registry cache FIRST — avoids disk I/O on cache hit.
        if let Some(ref db) = db {
            if let Ok(Some(cached)) = db.cache_read(name) {
                if !cached.is_expired() {
                    debug!("Metadata Cache → Resolver (cache hit for '{}')", name);
                    let transitive_deps = cached_to_dep_configs(&cached);
                    return (cached.exports.clone(), cached.revision, transitive_deps);
                } else {
                    debug!("Registry → Metadata Cache → Resolver (cache expired for '{}')", name);
                }
            }
        }

        // Cache miss or expired — read manifest from disk.
        let manifest = MetadataCache::read_dependency_manifest(dep_root).ok().flatten();
        let transitive_deps: Vec<DependencyConfig> = manifest
            .as_ref()
            .map(|m| {
                m.dependencies
                    .iter()
                    .map(|d| DependencyConfig {
                        name: d.clone(),
                        path: None,
                        required: false,
                    })
                    .collect()
            })
            .unwrap_or_default();

        if let Some(ref m) = manifest {
            // Write to cache for future resolves.
            if let Some(ref db) = db {
                let now = chrono::Utc::now();
                let expires = (now + chrono::Duration::seconds(ttl_seconds)).to_rfc3339();
                let mut meta = MetadataCache::from_manifest(m);
                meta.expires = expires;
                let _ = db.cache_write(&meta);
                debug!("Cached metadata for '{}' (TTL: {}s)", name, ttl_seconds);
            }
            return (m.exports.clone(), m.revision, transitive_deps);
        }

        (Vec::new(), 0, Vec::new())
    }
}

/// Convert cached dependency names into DependencyConfig list (paths unknown, optional).
fn cached_to_dep_configs(cached: &CachedRepoMetadata) -> Vec<DependencyConfig> {
    cached.dependencies
        .iter()
        .map(|d| DependencyConfig {
            name: d.clone(),
            path: None,
            required: false,
        })
        .collect()
}
