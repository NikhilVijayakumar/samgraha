use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchQuery {
    pub query: String,
    pub domain: Option<String>,
    pub status: Option<String>,
    pub repository: Option<String>,
    pub level: RetrievalLevel,
    pub max_results: usize,
    pub filters: Vec<SearchFilter>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            domain: None,
            status: None,
            repository: None,
            level: RetrievalLevel::Metadata,
            max_results: 20,
            filters: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetrievalLevel {
    Metadata,
    Summary,
    Section,
    Full,
}

impl std::fmt::Display for RetrievalLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata => write!(f, "metadata"),
            Self::Summary => write!(f, "summary"),
            Self::Section => write!(f, "section"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchFilter {
    pub field: String,
    pub value: String,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    In,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub document_id: i64,
    pub path: String,
    pub title: String,
    pub domain: String,
    pub score: f64,
    pub snippet: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub query: String,
    pub duration_ms: u64,
}
