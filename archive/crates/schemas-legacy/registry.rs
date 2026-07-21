use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryMetadata {
    pub version: String,
    pub repository: String,
    pub document_count: usize,
    pub build_timestamp: String,
    pub compiler_version: String,
    pub integrity_hash: Option<String>,
    pub status: RegistryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegistryStatus {
    Valid,
    Degraded,
    Corrupt,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Relationship {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    pub rel_type: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlossaryEntry {
    pub id: i64,
    pub term: String,
    pub definition: String,
    pub source_document_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildMetadata {
    pub document_hashes: HashMap<String, String>,
    pub artifact_hashes: HashMap<String, String>,
    pub compiler_version: String,
    pub build_version: String,
    pub build_timestamp: String,
    pub enrichment_version: Option<String>,
    pub audit_version: Option<String>,
}
