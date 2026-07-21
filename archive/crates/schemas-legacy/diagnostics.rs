use serde::{Deserialize, Serialize};
use crate::document::SourceSpan;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompilationDiagnostic {
    MissingSection {
        semantic_type: String,
        canonical_name: String,
        severity: DiagnosticSeverity,
        message: String,
    },
    ProhibitedContent {
        source_span: Option<SourceSpan>,
        pattern: String,
        severity: DiagnosticSeverity,
        message: String,
    },
    EmptySection {
        semantic_type: String,
        canonical_name: String,
        source_span: Option<SourceSpan>,
        severity: DiagnosticSeverity,
        message: String,
    },
    DuplicateSection {
        semantic_type: String,
        canonical_name: String,
        source_span: Option<SourceSpan>,
        severity: DiagnosticSeverity,
        message: String,
    },
    UnknownSection {
        canonical_name: String,
        source_span: Option<SourceSpan>,
        severity: DiagnosticSeverity,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Info,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warning => write!(f, "warning"),
            Self::Info => write!(f, "info"),
        }
    }
}
