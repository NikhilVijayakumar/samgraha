use schemas::diagnostics::{CompilationDiagnostic, DiagnosticSeverity};
use schemas::document::DocumentSection;
use schemas::standard::StandardDefinition;

pub struct SectionValidator;

impl SectionValidator {
    pub fn validate(
        sections: &[DocumentSection],
        standard: Option<&StandardDefinition>,
        prohibited_patterns: &[String],
        file_path: &str,
    ) -> Vec<CompilationDiagnostic> {
        let mut diagnostics = Vec::new();

        let Some(std) = standard else {
            return diagnostics;
        };

        let present_types: Vec<&str> = sections.iter().map(|s| s.semantic_type.as_str()).collect();
        for req in &std.required_sections {
            if req.required && !present_types.contains(&req.semantic_type.as_str()) {
                diagnostics.push(CompilationDiagnostic::MissingSection {
                    semantic_type: req.semantic_type.clone(),
                    canonical_name: req.canonical_name.clone(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!(
                        "Missing required section '{}' (type: {})",
                        req.canonical_name, req.semantic_type
                    ),
                });
            }
        }

        let mut seen_types: Vec<String> = Vec::new();
        for section in sections {
            Self::validate_section(
                section,
                std,
                prohibited_patterns,
                file_path,
                &mut seen_types,
                &mut diagnostics,
            );
        }

        diagnostics
    }

    fn validate_section(
        section: &DocumentSection,
        standard: &StandardDefinition,
        prohibited_patterns: &[String],
        file_path: &str,
        seen_types: &mut Vec<String>,
        diagnostics: &mut Vec<CompilationDiagnostic>,
    ) {
        if section.body.trim().is_empty() && section.semantic_type != "generic" {
            diagnostics.push(CompilationDiagnostic::EmptySection {
                semantic_type: section.semantic_type.clone(),
                canonical_name: section.heading.clone(),
                source_span: section.source_span.clone(),
                severity: DiagnosticSeverity::Info,
                message: format!("Section '{}' is empty", section.heading),
            });
        }

        if seen_types.contains(&section.semantic_type)
            && section.semantic_type != "generic"
        {
            diagnostics.push(CompilationDiagnostic::DuplicateSection {
                semantic_type: section.semantic_type.clone(),
                canonical_name: section.heading.clone(),
                source_span: section.source_span.clone(),
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "Duplicate section of type '{}' ('{}')",
                    section.semantic_type, section.heading
                ),
            });
        } else if section.semantic_type != "generic" {
            seen_types.push(section.semantic_type.clone());
        }

        for pattern in prohibited_patterns {
            if section.body.to_lowercase().contains(&pattern.to_lowercase()) {
                diagnostics.push(CompilationDiagnostic::ProhibitedContent {
                    source_span: section.source_span.clone(),
                    pattern: pattern.clone(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!(
                        "Section '{}' contains prohibited content matching '{}'",
                        section.heading, pattern
                    ),
                });
            }
        }

        for sub in &section.subsections {
            Self::validate_section(sub, standard, prohibited_patterns, file_path, seen_types, diagnostics);
        }
    }
}
