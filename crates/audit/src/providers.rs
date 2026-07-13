use compiler::validation::SectionValidator;
use rayon::prelude::*;
use schemas::audit::{AuditFinding, Severity};
use schemas::diagnostics::{CompilationDiagnostic, DiagnosticSeverity};
use schemas::document::Document;
use schemas::standard::{AuditRuleDef, StandardDefinition};

pub struct DeterministicAuditProvider;

impl DeterministicAuditProvider {
    pub fn execute(
        documents: &[Document],
        rules: &[AuditRuleDef],
        standard: Option<&StandardDefinition>,
    ) -> Vec<AuditFinding> {
        let mut findings: Vec<AuditFinding> = rules
            .par_iter()
            .flat_map(|rule| Self::check_rule(documents, rule))
            .collect();

        // Reuse the compiler's own section validator so missing required
        // sections / prohibited content that compile already detected as
        // warnings show up as audit findings instead of being silently
        // dropped between the two pipelines.
        if let Some(std) = standard {
            let diagnostic_findings: Vec<AuditFinding> = documents
                .par_iter()
                .flat_map(|doc| Self::check_compile_diagnostics(doc, std))
                .collect();
            findings.extend(diagnostic_findings);
        }

        findings
    }

    fn check_compile_diagnostics(doc: &Document, standard: &StandardDefinition) -> Vec<AuditFinding> {
        let sections: Vec<_> = doc.body.sections().into_iter().cloned().collect();
        SectionValidator::validate(&sections, Some(standard), &standard.prohibited_content, doc.path.as_str())
            .into_iter()
            .map(|diagnostic| Self::diagnostic_to_finding(diagnostic, doc))
            .collect()
    }

    fn diagnostic_to_finding(diagnostic: CompilationDiagnostic, doc: &Document) -> AuditFinding {
        let (check_id, message, severity) = match diagnostic {
            CompilationDiagnostic::MissingSection { severity, message, .. } => {
                ("compile-missing-section", message, severity)
            }
            CompilationDiagnostic::ProhibitedContent { severity, message, .. } => {
                ("compile-prohibited-content", message, severity)
            }
            CompilationDiagnostic::EmptySection { severity, message, .. } => {
                ("compile-empty-section", message, severity)
            }
            CompilationDiagnostic::DuplicateSection { severity, message, .. } => {
                ("compile-duplicate-section", message, severity)
            }
            CompilationDiagnostic::UnknownSection { severity, message, .. } => {
                ("compile-unknown-section", message, severity)
            }
        };
        AuditFinding {
            check_id: check_id.into(),
            severity: match severity {
                DiagnosticSeverity::Warning => Severity::Warning,
                DiagnosticSeverity::Info => Severity::Suggestion,
            },
            message,
            location: Some(doc.path.as_str().to_string()),
            document_id: Some(doc.id),
            provider: "deterministic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
        }
    }

    fn check_rule(documents: &[Document], rule: &AuditRuleDef) -> Vec<AuditFinding> {
        match rule.evidence_type.as_str() {
            "section_presence" => {
                let section_key = rule
                    .scope
                    .to_lowercase()
                    .replace(' ', "_")
                    .replace('-', "_");
                documents
                    .par_iter()
                    .filter(|doc| {
                        let count = doc
                            .quality
                            .per_type
                            .get(&section_key)
                            .copied()
                            .unwrap_or(0);
                        if count > 0 {
                            return false;
                        }
                        let title_key = doc
                            .title
                            .to_lowercase()
                            .replace(' ', "_")
                            .replace('-', "_");
                        title_key != section_key
                    })
                    .map(|doc| AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                        location: Some(doc.path.as_str().to_string()),
                        document_id: Some(doc.id),
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    })
                    .collect()
            }
            "keyword_absence" => documents
                .par_iter()
                .filter(|doc| has_implementation_details(doc.body.raw()))
                .map(|doc| AuditFinding {
                    check_id: rule.id.clone(),
                    severity: Severity::from_str(&rule.severity),
                    message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                    location: Some(doc.path.as_str().to_string()),
                    document_id: Some(doc.id),
                    provider: "deterministic".into(),
                    stage: None,
                    section_id: None,
                    confidence: None,
                    evidence: None,
                    status: None,
                    strategy: None,
                })
                .collect(),
            "content_check" => {
                let keywords: Vec<String> = rule.params.get("keywords")
                    .map(|k| k.split(',').map(|s| s.trim().to_lowercase()).collect())
                    .unwrap_or_default();
                let pattern = rule.params.get("pattern").cloned().unwrap_or_default();
                documents
                    .par_iter()
                    .filter(|doc| {
                        let body_lower = doc.body.raw().to_lowercase();
                        // Check if any keyword is missing from the document.
                        if !keywords.is_empty() {
                            return keywords.iter().any(|kw| !body_lower.contains(kw.as_str()));
                        }
                        // Check if pattern is absent.
                        if !pattern.is_empty() {
                            return !body_lower.contains(pattern.as_str());
                        }
                        false
                    })
                    .map(|doc| AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                        location: Some(doc.path.as_str().to_string()),
                        document_id: Some(doc.id),
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    })
                    .collect()
            }
            "word_count" => {
                let max_words: usize = rule.params.get("max_words")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5000);
                documents
                    .par_iter()
                    .filter(|doc| doc.body.raw().split_whitespace().count() > max_words)
                    .map(|doc| AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                        location: Some(doc.path.as_str().to_string()),
                        document_id: Some(doc.id),
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    })
                    .collect()
            }
            "cross_reference" => {
                let expected_domains: Vec<String> = rule.params.get("expected_domains")
                    .map(|d| d.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();
                documents
                    .par_iter()
                    .filter(|doc| {
                        let body_lower = doc.body.raw().to_lowercase();
                        expected_domains.iter().any(|domain| {
                            let domain_lower = domain.to_lowercase();
                            !body_lower.contains(domain_lower.as_str())
                        })
                    })
                    .map(|doc| AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                        location: Some(doc.path.as_str().to_string()),
                        document_id: Some(doc.id),
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    })
                    .collect()
            }
            _ => vec![],
        }
    }
}

fn has_implementation_details(body: &str) -> bool {
    let indicators = [
        "```rust",
        "```python",
        "```javascript",
        "```typescript",
        "```cargo",
        "fn ",
        "impl ",
        "struct ",
        "pub ",
        "let ",
        "npm install",
        "cargo install",
        "pip install",
    ];
    indicators.iter().any(|i| body.contains(i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::document::{DocumentBody, DocumentMetadata, DocumentPath};
    use schemas::quality::ObjectStatistics;
    use schemas::standard::SectionDefinition;
    use std::path::PathBuf;

    fn doc_with_no_sections() -> Document {
        Document {
            id: 1,
            path: DocumentPath(PathBuf::from("docs/architecture/overview.md")),
            hash: "abc".into(),
            standard: "architecture".into(),
            title: "Overview".into(),
            body: DocumentBody::Generic { raw: String::new(), sections: vec![] },
            metadata: DocumentMetadata::default(),
            provenance: None,
            quality: ObjectStatistics::default(),
            created_at: "0".into(),
            updated_at: "0".into(),
        }
    }

    fn standard_requiring(section_name: &str) -> StandardDefinition {
        StandardDefinition {
            id: "architecture".into(),
            name: "Architecture".into(),
            version: "1.0.0".into(),
            domain: "architecture".into(),
            description: String::new(),
            required_sections: vec![SectionDefinition {
                canonical_name: section_name.into(),
                semantic_type: section_name.to_lowercase().replace(' ', "_"),
                aliases: vec![],
                required: true,
                description: String::new(),
            }],
            prohibited_content: vec![],
            relationships: vec![],
            audit_rules: vec![],
            profiles: vec![],
        }
    }

    #[test]
    fn execute_promotes_missing_section_diagnostic_to_finding() {
        // Regression: compile's SectionValidator detects a missing required
        // section as a warning, but audit's deterministic checks never saw
        // it — this reproduces (and closes) the original "audit says
        // 100/0 despite compile warnings" bug.
        let doc = doc_with_no_sections();
        let standard = standard_requiring("Communication");

        let findings = DeterministicAuditProvider::execute(&[doc], &[], Some(&standard));

        assert!(findings.iter().any(|f| f.check_id == "compile-missing-section"));
    }

    #[test]
    fn execute_without_standard_only_runs_declared_rules() {
        let doc = doc_with_no_sections();
        let findings = DeterministicAuditProvider::execute(&[doc], &[], None);
        assert!(findings.is_empty());
    }
}
