use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type DocumentId = i64;
pub type ContentHash = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub id: DocumentId,
    pub path: DocumentPath,
    pub hash: ContentHash,
    pub standard: String,
    pub title: String,
    pub body: String,
    pub metadata: DocumentMetadata,
    pub sections: Vec<DocumentSection>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentPath(pub PathBuf);

impl DocumentPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub title: String,
    pub document_type: Option<String>,
    pub status: Option<String>,
    pub ownership: Option<String>,
    pub tags: Vec<String>,
    pub extra: std::collections::HashMap<String, String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            document_type: None,
            status: None,
            ownership: None,
            tags: Vec::new(),
            extra: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentSection {
    pub heading: String,
    pub semantic_type: String,
    pub level: u32,
    pub body: String,
    pub required: bool,
    pub subsections: Vec<DocumentSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentStatus {
    Draft,
    Review,
    Approved,
    Deprecated,
    Superseded,
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Review => write!(f, "review"),
            Self::Approved => write!(f, "approved"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}
