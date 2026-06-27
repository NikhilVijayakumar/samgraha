use schemas::audit::{AuditFinding, Severity};
use schemas::document::Document;
use schemas::standard::AuditRuleDef;

/// Heuristic-based semantic audit provider.
///
/// Evaluates documentation quality, scope correctness, and technology independence
/// without requiring an AI model. When an AI provider is configured in the future,
/// it can replace or supplement these heuristics while preserving the same interface.
pub struct SemanticAuditProvider;

impl SemanticAuditProvider {
    pub fn execute(docs: &[Document], _rules: &[AuditRuleDef]) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        for doc in docs {
            findings.extend(check_quality(&doc));
            findings.extend(check_technology_independence(&doc));
            findings.extend(check_scope(&doc));
        }
        findings
    }
}

/// FR1/FR5: Documentation quality — body length, section depth, vague language.
fn check_quality(doc: &Document) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let word_count = doc.body.raw().split_whitespace().count();

    if word_count < 50 {
        findings.push(AuditFinding {
            check_id: "sem-001".into(),
            severity: Severity::Warning,
            message: format!(
                "Document is very short ({} words). Consider adding more context.",
                word_count
            ),
            location: Some(doc.path.0.to_string_lossy().to_string()),
            document_id: Some(doc.id),
            provider: "semantic".into(),
        });
    }

    let vague = ["TBD", "TODO", "FIXME", "placeholder", "coming soon", "to be determined"];
    for term in &vague {
        if doc.body.raw().to_lowercase().contains(&term.to_lowercase()) {
            findings.push(AuditFinding {
                check_id: "sem-002".into(),
                severity: Severity::Suggestion,
                message: format!("Document contains placeholder text: \"{}\"", term),
                location: Some(doc.path.0.to_string_lossy().to_string()),
                document_id: Some(doc.id),
                provider: "semantic".into(),
            });
            break;
        }
    }

    findings
}

/// FR2: Technology independence — detect implementation-specific terms in domain docs.
fn check_technology_independence(doc: &Document) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    // Vision and Feature docs should not reference specific implementation technologies.
    let technology_domains = ["vision", "feature", "design"];
    if !technology_domains.contains(&doc.standard.as_str()) {
        return findings;
    }

    let impl_terms = [
        "React", "Vue", "Angular", "Django", "Rails", "FastAPI", "Spring Boot",
        "PostgreSQL", "MySQL", "MongoDB", "Redis", "Kafka", "RabbitMQ",
        "Kubernetes", "Docker", "AWS", "GCP", "Azure",
        "TypeScript", "JavaScript", "Python", "Java", "Rust", "Go",
    ];

    for term in &impl_terms {
        if doc.body.raw().contains(term) {
            findings.push(AuditFinding {
                check_id: "sem-003".into(),
                severity: Severity::Suggestion,
                message: format!(
                    "Document references implementation technology '{}' in a '{}' document. \
                     Consider describing capabilities instead of technologies.",
                    term, doc.standard
                ),
                location: Some(doc.path.0.to_string_lossy().to_string()),
                document_id: Some(doc.id),
                provider: "semantic".into(),
            });
            break; // one finding per doc, don't flood
        }
    }

    findings
}

/// FR3: Scope correctness — verify docs stay within their domain responsibility.
fn check_scope(doc: &Document) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    // Vision docs should not contain implementation requirements.
    if doc.standard == "vision" {
        let impl_keywords = ["SHALL", "MUST", "REQUIRED", "FR1", "FR2", "API", "endpoint"];
        let upper = doc.body.raw().to_uppercase();
        for kw in &impl_keywords {
            if upper.contains(kw) {
                findings.push(AuditFinding {
                    check_id: "sem-004".into(),
                    severity: Severity::Suggestion,
                    message: format!(
                        "Vision document may contain requirement-level language ('{}') \
                         which belongs in Feature or Engineering documents.",
                        kw
                    ),
                    location: Some(doc.path.0.to_string_lossy().to_string()),
                    document_id: Some(doc.id),
                    provider: "semantic".into(),
                });
                break;
            }
        }
    }

    // Engineering docs should have a rationale section.
    if doc.standard == "engineering" || doc.standard == "architecture" {
        let raw = doc.body.raw();
        let has_rationale = raw.to_lowercase().contains("rationale")
            || raw.to_lowercase().contains("reason")
            || raw.to_lowercase().contains("because")
            || raw.to_lowercase().contains("decision");
        if !has_rationale && raw.split_whitespace().count() > 50 {
            findings.push(AuditFinding {
                check_id: "sem-005".into(),
                severity: Severity::Suggestion,
                message: "Document lacks rationale or decision context. \
                          Consider adding a section explaining why this approach was chosen."
                    .into(),
                location: Some(doc.path.0.to_string_lossy().to_string()),
                document_id: Some(doc.id),
                provider: "semantic".into(),
            });
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::document::{DocumentBody, DocumentMetadata, DocumentPath};
    use schemas::quality::ObjectStatistics;
    use std::path::PathBuf;

    fn doc(standard: &str, body: &str) -> Document {
        Document {
            id: 1,
            path: DocumentPath(PathBuf::from("test.md")),
            hash: "abc".into(),
            standard: standard.into(),
            title: "Test".into(),
            body: DocumentBody::Generic {
                raw: body.to_string(),
                sections: vec![],
            },
            metadata: DocumentMetadata::default(),
            provenance: None,
            quality: ObjectStatistics::default(),
            created_at: "0".into(),
            updated_at: "0".into(),
        }
    }

    #[test]
    fn short_doc_warns() {
        let d = doc("feature", "Short.");
        let findings = check_quality(&d);
        assert!(findings.iter().any(|f| f.check_id == "sem-001"));
    }

    #[test]
    fn vision_with_technology_flagged() {
        let d = doc("vision", "We should use React for the frontend and PostgreSQL for storage.");
        let findings = check_technology_independence(&d);
        assert!(!findings.is_empty());
    }

    #[test]
    fn engineering_no_rationale_flagged() {
        let body = "a ".repeat(60);
        let d = doc("engineering", &body);
        let findings = check_scope(&d);
        assert!(findings.iter().any(|f| f.check_id == "sem-005"));
    }
}
