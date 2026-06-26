use anyhow::{Context, Result};
use registry::RegistryStore;
use schemas::document::Document;
use schemas::package::{
    KnowledgePackage, PackageArtifact, PackageIntegrity, PackageManifest, PackageProfile,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PackageService;

pub struct PackageRequest {
    pub output_path: std::path::PathBuf,
    pub profile: PackageProfile,
    pub repository_name: String,
}

pub struct PackageResult {
    pub package: KnowledgePackage,
    pub output_path: std::path::PathBuf,
    pub documents_packaged: usize,
}

impl PackageService {
    pub fn generate(registry: Arc<RegistryStore>, request: &PackageRequest) -> Result<PackageResult> {
        let docs = registry.get_all_documents()?;
        let filtered = Self::filter_by_profile(&docs, &request.profile);

        // Build artifacts list.
        let mut artifacts: Vec<PackageArtifact> = Vec::new();
        let mut artifact_hashes: HashMap<String, String> = HashMap::new();

        for doc in &filtered {
            let path_str = doc.path.0.to_string_lossy().to_string();
            // Load audit findings for this document.
            let findings = registry.get_audit_findings(doc.id).unwrap_or_default();
            let audit_status = if findings.iter().any(|f| {
                matches!(f.severity, schemas::audit::Severity::Error)
            }) {
                "error"
            } else if findings.iter().any(|f| {
                matches!(f.severity, schemas::audit::Severity::Warning)
            }) {
                "warning"
            } else {
                "pass"
            };

            let artifact = PackageArtifact {
                path: path_str.clone(),
                hash: doc.hash.clone(),
                artifact_type: format!("{}:{}", doc.standard, audit_status),
                repository: request.repository_name.clone(),
                source_document: Some(path_str.clone()),
            };
            artifact_hashes.insert(path_str, doc.hash.clone());
            artifacts.push(artifact);
        }

        let domains: Vec<String> = {
            let mut d: Vec<String> = filtered.iter().map(|d| d.standard.clone()).collect();
            d.sort();
            d.dedup();
            d
        };

        let package_hash = Self::hash_package(&artifacts);

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

        // Validate before writing.
        Self::validate(&package)?;

        // Write to output path.
        let json = serde_json::to_string_pretty(&package)?;
        if let Some(parent) = request.output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&request.output_path, &json)
            .context(format!("Failed to write package to {}", request.output_path.display()))?;

        let count = package.artifacts.len();
        Ok(PackageResult {
            package,
            output_path: request.output_path.clone(),
            documents_packaged: count,
        })
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

    fn hash_package(artifacts: &[PackageArtifact]) -> String {
        let mut hasher = Sha256::new();
        for a in artifacts {
            hasher.update(a.path.as_bytes());
            hasher.update(a.hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

fn epoch_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
