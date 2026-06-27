use serde::{Deserialize, Serialize};
use crate::diagnostics::CompilationDiagnostic;
use crate::quality::ObjectStatistics;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompilationRequest {
    pub scope: CompilationScope,
    pub force: bool,
    pub watch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompilationScope {
    Repository,
    Workspace,
    Paths(Vec<String>),
    Domains(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilationResult {
    pub success: bool,
    pub documents_found: usize,
    pub documents_processed: usize,
    pub documents_failed: usize,
    pub documents_skipped: usize,
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<String>,
    pub diagnostics: Vec<CompilationDiagnostic>,
    pub quality: Option<ObjectStatistics>,
    pub duration_ms: u64,
    pub registry_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompilationError {
    pub path: Option<String>,
    pub message: String,
    pub error_type: CompilationErrorType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompilationErrorType {
    Discovery,
    Validation,
    Processing,
    Registry,
    Configuration,
    Internal,
}

impl std::fmt::Display for CompilationErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discovery => write!(f, "discovery"),
            Self::Validation => write!(f, "validation"),
            Self::Processing => write!(f, "processing"),
            Self::Registry => write!(f, "registry"),
            Self::Configuration => write!(f, "configuration"),
            Self::Internal => write!(f, "internal"),
        }
    }
}
