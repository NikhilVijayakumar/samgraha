use anyhow::{Context, Result};
use registry::RegistryStore;
use schemas::document::Document;
use schemas::package::{
    KnowledgePackage, PackageArtifact, PackageIntegrity, PackageManifest, PackageProfile,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageFormat {
    /// Single JSON file (legacy)
    Json,
    /// Directory with knowledge.db + docs/ + samgraha-package.json
    Directory,
}

pub struct PackageService;

pub struct PackageRequest {
    pub output_path: std::path::PathBuf,
    pub profile: PackageProfile,
    pub repository_name: String,
    pub format: PackageFormat,
}

impl Default for PackageRequest {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("knowledge-package"),
            profile: PackageProfile::Full,
            repository_name: String::new(),
            format: PackageFormat::Directory,
        }
    }
}

pub struct PackageResult {
    pub package: KnowledgePackage,
    pub output_path: std::path::PathBuf,
    pub documents_packaged: usize,
}

impl PackageService {
    pub fn generate(
        registry: Arc<RegistryStore>,
        registry_path: Option<&Path>,
        request: &PackageRequest,
    ) -> Result<PackageResult> {
        let docs = registry.get_all_documents()?;
        let filtered = Self::filter_by_profile(&docs, &request.profile);

        let (artifacts, artifact_hashes) = if request.format == PackageFormat::Directory {
            Self::build_directory_package(&filtered, &request, &registry, registry_path)?
        } else {
            Self::build_json_artifacts(&filtered, &request, &registry)?
        };

        let domains: Vec<String> = {
            let mut d: Vec<String> = filtered.iter().map(|d| d.standard.clone()).collect();
            d.sort();
            d.dedup();
            d
        };

        let package_hash = Self::hash_artifacts(&artifacts);

        let manifest = PackageManifest {
            name: format!("{}-knowledge", request.repository_name),
            version: "0.1.0".to_string(),
            description: format!(
                "Knowledge package for {} (profile: {})",
                request.repository_name, request.profile
            ),
            repository: request.repository_name.clone(),
            included_repositories: vec![request.repository_name.clone()],
            included_domains: domains,
            dependencies: Vec::new(),
            generated_at: epoch_now(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            profile: request.profile.to_string(),
        };

        let integrity = PackageIntegrity {
            package_hash: package_hash.clone(),
            artifact_hashes,
            signature: None,
        };

        let package = KnowledgePackage {
            manifest,
            artifacts,
            integrity,
        };

        Self::validate(&package)?;

        Self::write_package_manifest(&request.output_path, &package, &request.format)?;

        let count = package.artifacts.len();
        Ok(PackageResult {
            package,
            output_path: request.output_path.clone(),
            documents_packaged: count,
        })
    }

    /// Build artifacts list for JSON-only output (legacy).
    fn build_json_artifacts(
        filtered: &[&Document],
        request: &PackageRequest,
        registry: &RegistryStore,
    ) -> Result<(Vec<PackageArtifact>, HashMap<String, String>)> {
        let mut artifacts = Vec::new();
        let mut hashes = HashMap::new();

        for doc in filtered {
            let path_str = doc.path.0.to_string_lossy().to_string();
            let audit_status = Self::doc_audit_status(registry, doc.id);
            let artifact = PackageArtifact {
                path: path_str.clone(),
                hash: doc.hash.clone(),
                artifact_type: format!("{}:{}", doc.standard, audit_status),
                repository: request.repository_name.clone(),
                source_document: Some(path_str.clone()),
            };
            hashes.insert(path_str, doc.hash.clone());
            artifacts.push(artifact);
        }

        Ok((artifacts, hashes))
    }

    /// Build package as a directory: knowledge.db + docs/ + manifest.
    fn build_directory_package(
        filtered: &[&Document],
        request: &PackageRequest,
        registry: &RegistryStore,
        registry_path: Option<&Path>,
    ) -> Result<(Vec<PackageArtifact>, HashMap<String, String>)> {
        let output_dir = &request.output_path;
        if output_dir.exists() {
            std::fs::remove_dir_all(output_dir)
                .context(format!("Failed to clean output dir {}", output_dir.display()))?;
        }
        std::fs::create_dir_all(output_dir)
            .context(format!("Failed to create {}", output_dir.display()))?;

        // Create docs/ subdirectory with one subdir per standard.
        let docs_dir = output_dir.join("docs");
        std::fs::create_dir_all(&docs_dir)?;

        let mut artifacts = Vec::new();
        let mut hashes = HashMap::new();

        for doc in filtered {
            let standard_dir = docs_dir.join(&doc.standard);
            if !standard_dir.exists() {
                std::fs::create_dir_all(&standard_dir)?;
            }

            let filename = slugify_filename(&doc.title) + ".md";
            let doc_path = standard_dir.join(&filename);
            std::fs::write(&doc_path, doc.body.raw())
                .context(format!("Failed to write {}", doc_path.display()))?;

            let audit_status = Self::doc_audit_status(registry, doc.id);
            let rel_path = format!("docs/{}/{}", doc.standard, filename);
            let artifact = PackageArtifact {
                path: rel_path.clone(),
                hash: doc.hash.clone(),
                artifact_type: format!("{}:{}", doc.standard, audit_status),
                repository: request.repository_name.clone(),
                source_document: Some(doc.path.0.to_string_lossy().to_string()),
            };
            hashes.insert(rel_path, doc.hash.clone());
            artifacts.push(artifact);
        }

        // Copy registry SQLite DB into package directory.
        if let Some(db_path) = registry_path {
            if db_path.exists() && db_path.to_string_lossy() != ":memory:" {
                let target_db = output_dir.join("knowledge.db");
                std::fs::copy(db_path, &target_db).context(format!(
                    "Failed to copy registry DB from {} to {}",
                    db_path.display(),
                    target_db.display()
                ))?;
                let rel_db = "knowledge.db".to_string();
                let db_hash = file_hash(&target_db);
                hashes.insert(rel_db.clone(), db_hash.clone());
                artifacts.push(PackageArtifact {
                    path: rel_db,
                    hash: db_hash,
                    artifact_type: "registry/db".into(),
                    repository: request.repository_name.clone(),
                    source_document: None,
                });
            }
        }

        info!(
            "Wrote {} docs to {}/docs/ in {} standards",
            filtered.len(),
            output_dir.display(),
            artifacts.len()
        );

        Ok((artifacts, hashes))
    }

    fn doc_audit_status(registry: &RegistryStore, doc_id: i64) -> &'static str {
        let findings = registry.get_audit_findings(doc_id).unwrap_or_default();
        if findings.iter().any(|f| matches!(f.severity, schemas::audit::Severity::Error)) {
            "error"
        } else if findings.iter().any(|f| matches!(f.severity, schemas::audit::Severity::Warning)) {
            "warning"
        } else {
            "pass"
        }
    }

    /// Write the manifest JSON to the package location.
    fn write_package_manifest(
        output_path: &Path,
        package: &KnowledgePackage,
        format: &PackageFormat,
    ) -> Result<()> {
        let manifest_path = match format {
            PackageFormat::Directory => output_path.join("samgraha-package.json"),
            PackageFormat::Json => output_path.to_path_buf(),
        };

        if let Some(parent) = manifest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(package)?;
        std::fs::write(&manifest_path, &json)
            .context(format!("Failed to write manifest to {}", manifest_path.display()))?;

        Ok(())
    }

    fn filter_by_profile<'a>(docs: &'a [Document], profile: &PackageProfile) -> Vec<&'a Document> {
        match profile {
            PackageProfile::Minimal => docs
                .iter()
                .filter(|d| matches!(d.standard.as_str(), "vision" | "readme"))
                .collect(),
            PackageProfile::Documentation => docs
                .iter()
                .filter(|d| matches!(d.standard.as_str(), "vision" | "readme" | "design" | "feature"))
                .collect(),
            PackageProfile::Engineering => docs
                .iter()
                .filter(|d| {
                    matches!(
                        d.standard.as_str(),
                        "vision" | "architecture" | "engineering" | "feature" | "feature-technical"
                    )
                })
                .collect(),
            PackageProfile::AiAssistant => docs
                .iter()
                .filter(|d| {
                    !matches!(d.standard.as_str(), "prototype")
                })
                .collect(),
            PackageProfile::Development | PackageProfile::Full => docs.iter().collect(),
        }
    }

    fn validate(pkg: &KnowledgePackage) -> Result<()> {
        if pkg.manifest.name.is_empty() {
            anyhow::bail!("Package manifest missing name");
        }
        if pkg.manifest.repository.is_empty() {
            anyhow::bail!("Package manifest missing repository");
        }
        if pkg.integrity.package_hash.is_empty() {
            anyhow::bail!("Package integrity hash missing");
        }
        Ok(())
    }

    fn hash_artifacts(artifacts: &[PackageArtifact]) -> String {
        let mut hasher = Sha256::new();
        for a in artifacts {
            hasher.update(a.path.as_bytes());
            hasher.update(a.hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

fn slugify_filename(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn file_hash(path: &Path) -> String {
    let data = std::fs::read(path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    format!("{:x}", hasher.finalize())
}

fn epoch_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
