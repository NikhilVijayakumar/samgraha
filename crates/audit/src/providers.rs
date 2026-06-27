use rayon::prelude::*;
use schemas::audit::{AuditFinding, Severity};
use schemas::document::Document;
use schemas::standard::AuditRuleDef;

pub struct DeterministicAuditProvider;

impl DeterministicAuditProvider {
    pub fn execute(documents: &[Document], rules: &[AuditRuleDef]) -> Vec<AuditFinding> {
        rules
            .par_iter()
            .flat_map(|rule| Self::check_rule(documents, rule))
            .collect()
    }

    fn check_rule(documents: &[Document], rule: &AuditRuleDef) -> Vec<AuditFinding> {
        match rule.check_type.as_str() {
            "corpus_exists" => {
                if documents.is_empty() {
                    vec![AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: rule.description.clone(),
                        location: None,
                        document_id: None,
                        provider: "deterministic".into(),
                    }]
                } else {
                    vec![]
                }
            }
            "has_title" => documents
                .par_iter()
                .filter(|doc| doc.title.trim().is_empty())
                .map(|doc| AuditFinding {
                    check_id: rule.id.clone(),
                    severity: Severity::from_str(&rule.severity),
                    message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                    location: Some(doc.path.as_str().to_string()),
                    document_id: Some(doc.id),
                    provider: "deterministic".into(),
                })
                .collect(),
            "has_section" => {
                let section_key = rule
                    .scope
                    .to_lowercase()
                    .replace(' ', "_")
                    .replace('-', "_");
                documents
                    .par_iter()
                    .filter(|doc| {
                        doc.quality
                            .per_type
                            .get(&section_key)
                            .copied()
                            .unwrap_or(0)
                            == 0
                    })
                    .map(|doc| AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                        location: Some(doc.path.as_str().to_string()),
                        document_id: Some(doc.id),
                        provider: "deterministic".into(),
                    })
                    .collect()
            }
            "no_implementation" => documents
                .par_iter()
                .filter(|doc| has_implementation_details(doc.body.raw()))
                .map(|doc| AuditFinding {
                    check_id: rule.id.clone(),
                    severity: Severity::from_str(&rule.severity),
                    message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                    location: Some(doc.path.as_str().to_string()),
                    document_id: Some(doc.id),
                    provider: "deterministic".into(),
                })
                .collect(),
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
