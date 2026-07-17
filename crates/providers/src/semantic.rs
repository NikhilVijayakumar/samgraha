use schemas::audit::{AuditFinding, Severity};
use schemas::document::Document;
use schemas::standard::{AuditRuleDef, StandardDefinition};

/// Heuristic-based semantic audit provider.
///
/// Evaluates documentation quality, scope correctness, and technology independence
/// without requiring an AI model. When an AI provider is configured in the future,
/// it can replace or supplement these heuristics while preserving the same interface.
pub struct SemanticAuditProvider;

impl SemanticAuditProvider {
    pub fn execute(
        docs: &[Document],
        rules: &[AuditRuleDef],
        _standard: Option<&StandardDefinition>,
    ) -> Vec<AuditFinding> {
        let mut findings = Vec::new();
        for doc in docs {
            findings.extend(check_quality(&doc));
            findings.extend(check_technology_independence(&doc));
            findings.extend(check_scope(&doc));
        }
        findings.extend(review_tasks_for_rules(docs, rules));
        findings
    }
}

/// Turn a standard's own `kind = 'semantic'` rules into one review-task
/// finding per (rule, doc) — samgraha has no way to *score* these itself
/// (that needs an LLM); `AuditFramework::execute`'s existing
/// `semantic_review` bundling already converts any `provider: "semantic"`
/// finding into a `SemanticReviewTask` for the calling agent to judge
/// (`content: f.message`), so a rule's rubric/prompt text goes straight into
/// `message` and the rest of the pipeline needs no changes to pick it up.
fn review_tasks_for_rules(docs: &[Document], rules: &[AuditRuleDef]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for rule in rules {
        if rule.evidence_type != "llm_judgment" {
            continue; // a semantic rule type with no known evidence shape yet
        }
        let rubric = rule.params.get("prompt_template").cloned().unwrap_or_else(|| rule.description.clone());
        if docs.is_empty() {
            // Document-scope rubric with no compiled Document to attach it
            // to (python_hackathon's shape — LLM review of the repo as a
            // whole, not one markdown file) — still a real task, just not
            // tied to a document_id/location.
            findings.push(AuditFinding {
                check_id: rule.id.clone(),
                severity: Severity::Suggestion,
                message: rubric.clone(),
                location: None,
                document_id: None,
                provider: "semantic".into(),
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
            });
            continue;
        }
        for doc in docs {
            findings.push(AuditFinding {
                check_id: rule.id.clone(),
                severity: Severity::Suggestion,
                message: rubric.clone(),
                location: Some(doc.path.0.to_string_lossy().to_string()),
                document_id: Some(doc.id),
                provider: "semantic".into(),
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
            });
        }
    }
    findings
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
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
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
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
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
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
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
            if upper.contains(kw) && !is_safe_requirement_mention(&upper, kw) {
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
                    stage: None,
                    section_id: None,
                    confidence: None,
                    evidence: None,
                    status: None,
                    strategy: None,
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
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
            });
        }
    }

    findings
}

/// Whether a requirement-level keyword (MUST, REQUIRED, etc.) appears in a
/// safe context within a vision document — e.g. describing the problem
/// space ("AI must either..."), listing interface types ("REST APIs"), or
/// describing section-definition metadata ("required flags") — rather than
/// as an actual implementation requirement.
fn is_safe_requirement_mention(upper: &str, keyword: &str) -> bool {
    match keyword {
        "MUST" => {
            // "AI engineering assistants MUST either:" — problem description
            upper.contains("ASSISTANTS MUST EITHER")
        }
        "REQUIRED" => {
            // "required flags" — section-definition metadata description
            upper.contains("REQUIRED FLAGS")
        }
        "API" => {
            // "REST APIs" — listing interface types the platform may expose
            upper.contains("REST APIS")
        }
        _ => false,
    }
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

    fn semantic_rule(id: &str, prompt: Option<&str>) -> AuditRuleDef {
        let mut params = std::collections::HashMap::new();
        if let Some(p) = prompt {
            params.insert("prompt_template".to_string(), p.to_string());
        }
        AuditRuleDef {
            id: id.into(),
            name: id.into(),
            description: format!("Semantic review of {}", id),
            severity: "warning".into(),
            evidence_type: "llm_judgment".into(),
            scope: "document".into(),
            weight: 1.0,
            mandatory: false,
            params,
        }
    }

    #[test]
    fn review_tasks_for_rules_one_per_doc_when_docs_present() {
        let docs = vec![doc("infrastructure", "some content"), doc("infrastructure", "more content")];
        let rules = vec![semantic_rule("llm-review", Some("Analyze the repo..."))];
        let findings = review_tasks_for_rules(&docs, &rules);
        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|f| f.provider == "semantic"));
        assert!(findings.iter().all(|f| f.message == "Analyze the repo..."));
        assert!(findings.iter().all(|f| f.document_id.is_some()));
    }

    #[test]
    fn review_tasks_for_rules_one_task_when_no_docs() {
        // python_hackathon's real shape: document-scope LLM review of a repo
        // that has no compiled Document objects at all — still a real task.
        let rules = vec![semantic_rule("llm-review", Some("Analyze the repo..."))];
        let findings = review_tasks_for_rules(&[], &rules);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].document_id, None);
        assert_eq!(findings[0].message, "Analyze the repo...");
    }

    #[test]
    fn review_tasks_for_rules_falls_back_to_description_without_prompt_template() {
        let rules = vec![semantic_rule("llm-review", None)];
        let findings = review_tasks_for_rules(&[], &rules);
        assert_eq!(findings[0].message, "Semantic review of llm-review");
    }

    #[test]
    fn review_tasks_for_rules_skips_non_llm_judgment_rules() {
        let mut rule = semantic_rule("weird", Some("prompt"));
        rule.evidence_type = "script_output".into();
        let findings = review_tasks_for_rules(&[], &[rule]);
        assert!(findings.is_empty());
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
