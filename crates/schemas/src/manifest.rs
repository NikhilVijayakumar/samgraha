use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::SystemTime;
use uuid::Uuid;

/// Repository identity — stable across renames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoIdentity {
    pub id: String,
    pub name: String,
    pub uuid: Uuid,
}

/// Location of compiled knowledge database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeLocation {
    pub location: String,
}

/// Compiler version metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompilerInfo {
    pub version: String,
}

/// Summary of the last audit run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditSummary {
    pub status: String,
    #[serde(default)]
    pub last_audit: Option<String>,
}

/// Repository Manifest — the synchronization artifact exchanged with the Repository Registry.
///
/// Produced by the Knowledge Compiler on every successful compilation.
/// Written to `.samgraha/manifest.json`.
/// Never contains engineering knowledge — only repository metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryManifest {
    pub repository: RepoIdentity,
    pub revision: u64,
    pub compiler: CompilerInfo,
    pub audit: AuditSummary,
    pub repository_root: String,
    pub knowledge: KnowledgeLocation,
    pub exports: Vec<String>,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub generated_at: String,
}

/// Cached repository metadata — stored per-dependency in `.samgraha/dependencies/`.
///
/// Written by the Resolver during sync operations.
/// Read by the Resolver during resolution — never by the Registry at runtime.
/// TTL enforced by comparing `expires` against current time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedRepoMetadata {
    pub repository: RepoIdentity,
    pub revision: u64,
    pub repository_root: String,
    pub knowledge: KnowledgeLocation,
    pub exports: Vec<String>,
    pub audit: String,
    pub last_sync: String,
    pub expires: String,
}

impl CachedRepoMetadata {
    /// Returns true if the metadata cache entry has expired.
    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&self.expires) {
            let now = chrono::Utc::now();
            let expires_utc = expires.with_timezone(&chrono::Utc);
            return now > expires_utc;
        }
        false
    }

    /// Compute repository status from cached metadata and current time.
    ///
    /// Checks: path accessibility, TTL expiry, audit status, and revision staleness
    /// against the dependency's local manifest.json. Returns the most severe state.
    pub fn status(&self, now: SystemTime) -> RepositoryStatus {
        let root = Path::new(&self.repository_root);
        if !root.exists() {
            return RepositoryStatus::Missing;
        }
        if root.metadata().is_err() {
            return RepositoryStatus::Unavailable;
        }
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(&self.expires) {
            let now_dt: chrono::DateTime<chrono::Utc> = now.into();
            let expires_utc = expires.with_timezone(&chrono::Utc);
            if now_dt > expires_utc {
                return RepositoryStatus::StaleMetadata;
            }
        }
        if self.audit == "FAIL" || self.audit == "ERROR" {
            return RepositoryStatus::AuditFailed;
        }
        // Read manifest once for StaleKnowledge + SyncRequired checks.
        let manifest_path = root.join(".samgraha").join("manifest.json");
        if let Ok(content) = std::fs::read_to_string(&manifest_path).and_then(|c| {
            serde_json::from_str::<RepositoryManifest>(&c).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        }) {
            if content.revision > self.revision {
                return RepositoryStatus::StaleKnowledge;
            }
            if let Ok(gen_time) = chrono::DateTime::parse_from_rfc3339(&content.generated_at) {
                if let Ok(sync_time) = chrono::DateTime::parse_from_rfc3339(&self.last_sync) {
                    if gen_time > sync_time {
                        return RepositoryStatus::SyncRequired;
                    }
                }
            }
        }
        RepositoryStatus::Registered
    }
}

/// Repository lifecycle status — computed on demand, never persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryStatus {
    Registered,
    SyncRequired,
    StaleMetadata,
    StaleKnowledge,
    AuditFailed,
    Missing,
    Unavailable,
}

/// Manifest for a virtual (reference-only) Knowledge Package.
///
/// Virtual packages are workspace-local only. They reference source `knowledge.db`
/// files by absolute path instead of copying them. Not portable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VirtualPackageManifest {
    pub layout: String,
    pub generated_at: String,
    pub workspace_root: String,
    pub repositories: Vec<VirtualRepoEntry>,
}

/// A single repository entry in a VirtualPackageManifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VirtualRepoEntry {
    pub id: String,
    pub uuid: Uuid,
    pub knowledge_db: String,
    pub revision: u64,
}
