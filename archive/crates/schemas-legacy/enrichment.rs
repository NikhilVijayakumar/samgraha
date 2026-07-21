use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrichmentArtifact {
    pub document_id: i64,
    pub artifact_type: EnrichmentType,
    pub content: String,
    pub provider: String,
    pub model: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnrichmentType {
    Summary,
    Keywords,
    Embedding,
    Glossary,
}

impl std::fmt::Display for EnrichmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "summary"),
            Self::Keywords => write!(f, "keywords"),
            Self::Embedding => write!(f, "embedding"),
            Self::Glossary => write!(f, "glossary"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrichmentProfile {
    pub name: String,
    pub enabled_types: Vec<EnrichmentType>,
    pub provider: String,
    pub model: Option<String>,
    pub batch_size: usize,
}

impl Default for EnrichmentProfile {
    fn default() -> Self {
        Self {
            name: "minimal".to_string(),
            enabled_types: vec![EnrichmentType::Summary],
            provider: "rule-based".to_string(),
            model: None,
            batch_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnrichmentProviderMetadata {
    pub provider: String,
    pub model: Option<String>,
    pub timestamp: String,
    pub repository_version: String,
    pub document_hash: String,
    pub enrichment_profile: String,
    pub artifact_version: String,
}
