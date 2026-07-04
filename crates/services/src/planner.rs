use common::config::{parse_ttl_duration, SamgrahaConfig};
use schemas::manifest::{CachedRepoMetadata, RepositoryManifest};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Load priority for a repository within a session — determined by config position, never query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Primary,
    Dependency,
    Interest,
}

/// Computed status of a planned repository entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepStatus {
    /// The primary (local) repository — always loaded.
    Primary,
    /// Available, .meta fresh, revision matches.
    Loaded,
    /// Available, but .meta TTL expired — re-sync recommended.
    Stale,
    /// Available and .meta fresh, but cached revision != actual manifest revision.
    Outdated,
    /// Path resolved, but knowledge.db absent — compile needed.
    Missing,
    /// No path in config or .meta — dep was never synced.
    Unresolved,
    /// Required dependency that is unavailable (missing or unresolved) — error state.
    RequiredMissing,
}

/// One resolved candidate in a Knowledge Plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePlanEntry {
    pub name: String,
    pub root: PathBuf,
    pub priority: Priority,
    pub status: DepStatus,
    pub revision: u64,
    pub exports: Vec<String>,
    /// True when knowledge.db is present and loadable.
    pub available: bool,
}

/// Deterministic candidate list produced by the Planner.
/// Same samgraha.toml + same manifest/.meta files → identical plan every time.
#[derive(Debug)]
pub struct KnowledgePlan {
    pub entries: Vec<KnowledgePlanEntry>,
    pub planned_at: SystemTime,
}

impl KnowledgePlan {
    pub fn available(&self) -> impl Iterator<Item = &KnowledgePlanEntry> {
        self.entries.iter().filter(|e| e.available)
    }
}

pub struct Planner;

impl Planner {
    /// Build the plan from config + manifest.json / .meta files. No database. No query context.
    /// Infallible — unavailable/unresolved entries stay in the plan with appropriate status.
    pub fn plan(root: &Path, config: &SamgrahaConfig) -> KnowledgePlan {
        let mut entries = Vec::new();
        let meta_ttl = parse_ttl_duration(&config.resolver.metadata_ttl).unwrap_or(86400);

        // Primary repository — always first, always loaded.
        let primary_manifest = read_manifest(root);
        entries.push(KnowledgePlanEntry {
            name: dir_name(root),
            root: root.to_path_buf(),
            priority: Priority::Primary,
            status: DepStatus::Primary,
            revision: primary_manifest.as_ref().map(|m| m.revision).unwrap_or(0),
            exports: primary_manifest.map(|m| m.exports).unwrap_or_default(),
            available: true,
        });

        // Dependencies (required, high priority) then interests (optional, lower priority).
        let candidates: Vec<(&str, Priority, bool)> = config.knowledge.dependencies.iter()
            .map(|n| (n.as_str(), Priority::Dependency, true))
            .chain(config.knowledge.interests.iter().map(|n| (n.as_str(), Priority::Interest, false)))
            .collect();

        for (name, priority, required) in candidates {
            let entry = Self::plan_dep(name, priority, required, root, config, meta_ttl);
            entries.push(entry);
        }

        KnowledgePlan { entries, planned_at: SystemTime::now() }
    }

    fn plan_dep(
        name: &str,
        priority: Priority,
        required: bool,
        root: &Path,
        config: &SamgrahaConfig,
        _meta_ttl: i64,
    ) -> KnowledgePlanEntry {
        // 1. Explicit path from [repository].dependencies (config is authoritative).
        let config_path = resolve_dep_path(name, config, root);

        // 2. .meta file — written by sync, stores path + cached metadata.
        let meta = read_meta_file(root, name);
        let meta_fresh = meta.as_ref().map(|m| !m.is_expired()).unwrap_or(false);
        let meta_path = meta.as_ref()
            .filter(|_| meta_fresh)
            .map(|m| PathBuf::from(&m.repository_root))
            .filter(|p| !p.as_os_str().is_empty());

        // Config path wins; .meta path is the fallback for interests not in [repository].dependencies.
        let dep_root = config_path.or(meta_path);

        let db_exists = dep_root.as_ref()
            .map(|r| r.join(".samgraha").join("knowledge.db").exists())
            .unwrap_or(false);

        // Revision and exports: prefer .meta (faster, offline) with actual manifest as check.
        let cached_revision = meta.as_ref().map(|m| m.revision);
        let actual_manifest = dep_root.as_ref().and_then(|r| read_manifest(r));
        let actual_revision = actual_manifest.as_ref().map(|m| m.revision);

        let revision = cached_revision.or(actual_revision).unwrap_or(0);
        let exports = meta.as_ref().map(|m| m.exports.clone())
            .or_else(|| actual_manifest.map(|m| m.exports))
            .unwrap_or_default();

        let status = compute_status(
            dep_root.is_some(),
            db_exists,
            meta_fresh,
            cached_revision,
            actual_revision,
            required,
        );

        KnowledgePlanEntry {
            name: name.to_string(),
            root: dep_root.unwrap_or_default(),
            priority,
            status,
            revision,
            exports,
            available: db_exists,
        }
    }
}

fn compute_status(
    path_resolved: bool,
    db_exists: bool,
    meta_fresh: bool,
    cached_rev: Option<u64>,
    actual_rev: Option<u64>,
    required: bool,
) -> DepStatus {
    if !path_resolved {
        return if required { DepStatus::RequiredMissing } else { DepStatus::Unresolved };
    }
    if !db_exists {
        return if required { DepStatus::RequiredMissing } else { DepStatus::Missing };
    }
    if !meta_fresh {
        return DepStatus::Stale;
    }
    // Both revisions present and differ → cache is behind the actual manifest.
    if let (Some(c), Some(a)) = (cached_rev, actual_rev) {
        if c != a {
            return DepStatus::Outdated;
        }
    }
    DepStatus::Loaded
}

/// Read `.samgraha/dependencies/<name>.meta` from the local (primary) repo root.
pub fn read_meta_file(root: &Path, name: &str) -> Option<CachedRepoMetadata> {
    let path = root.join(".samgraha").join("dependencies").join(format!("{}.meta", name));
    std::fs::read_to_string(path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Write `.samgraha/dependencies/<name>.meta` to the local repo root.
/// Overwrites any existing file. TTL is baked into `expires` field.
pub fn write_meta_file(root: &Path, meta: &CachedRepoMetadata) -> anyhow::Result<()> {
    let dir = root.join(".samgraha").join("dependencies");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.meta", meta.repository.id));
    let json = serde_json::to_string_pretty(meta)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn resolve_dep_path(name: &str, config: &SamgrahaConfig, root: &Path) -> Option<PathBuf> {
    config.repository.dependencies.iter()
        .find(|d| d.name == name)
        .and_then(|d| d.path.as_ref())
        .map(|p| {
            let p = Path::new(p);
            if p.is_absolute() { p.to_path_buf() } else { root.join(p) }
        })
}

fn read_manifest(root: &Path) -> Option<RepositoryManifest> {
    let path = root.join(".samgraha").join("manifest.json");
    std::fs::read_to_string(path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn dir_name(root: &Path) -> String {
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("primary")
        .to_string()
}
