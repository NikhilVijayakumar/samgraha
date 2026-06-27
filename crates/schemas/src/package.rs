use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgePackage {
    pub manifest: PackageManifest,
    pub artifacts: Vec<PackageArtifact>,
    pub integrity: PackageIntegrity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: String,
    pub included_repositories: Vec<String>,
    pub included_domains: Vec<String>,
    pub dependencies: Vec<PackageDependency>,
    pub generated_at: String,
    pub compiler_version: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageArtifact {
    pub path: String,
    pub hash: String,
    pub artifact_type: String,
    pub repository: String,
    pub source_document: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageIntegrity {
    pub package_hash: String,
    pub artifact_hashes: HashMap<String, String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageProfile {
    Minimal,
    Development,
    Documentation,
    Engineering,
    AiAssistant,
    Full,
}

/// Package layout strategy.
///
/// `Physical` copies knowledge databases and documentation into the output directory.
/// `Virtual` creates a reference-only manifest with absolute paths — workspace-local only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageLayout {
    #[serde(rename = "physical")]
    Physical,
    #[serde(rename = "virtual")]
    Virtual,
}

impl std::fmt::Display for PackageLayout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Physical => write!(f, "physical"),
            Self::Virtual => write!(f, "virtual"),
        }
    }
}

impl std::fmt::Display for PackageProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Development => write!(f, "development"),
            Self::Documentation => write!(f, "documentation"),
            Self::Engineering => write!(f, "engineering"),
            Self::AiAssistant => write!(f, "ai-assistant"),
            Self::Full => write!(f, "full"),
        }
    }
}
