use schemas::audit::{AuditFinding, Severity};
use schemas::document::Document;
use schemas::standard::AuditRuleDef;

pub struct DeterministicAuditProvider;

impl DeterministicAuditProvider {
    pub fn execute(documents: &[Document], rules: &[AuditRuleDef]) -> Vec<AuditFinding> {
        let mut findings = Vec::new();

        for rule in rules {
            match rule.id.as_str() {
                id if id.ends_with("-001") => {
                    let has_required = !documents.is_empty();
                    if !has_required {
                        findings.push(AuditFinding {
                            check_id: rule.id.clone(),
                            severity: Severity::from_str(&rule.severity),
                            message: rule.description.clone(),
                            location: None,
                            document_id: None,
                            provider: "deterministic".into(),
                        });
                    }
                }
                id if id.ends_with("-002") || id.ends_with("-003") => {
                    for doc in documents {
                        let has_title = !doc.title.is_empty();
                        if !has_title && id.ends_with("-002") {
                            findings.push(AuditFinding {
                                check_id: rule.id.clone(),
                                severity: Severity::from_str(&rule.severity),
                                message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                                location: Some(doc.path.as_str().to_string()),
                                document_id: Some(doc.id),
                                provider: "deterministic".into(),
                            });
                        }
                    }
                }
                _ => {
                    for doc in documents {
                        let pass = match rule.id.as_str() {
                            "arch-003" | "feat-004" | "fd-004" | "ft-004" => {
                                !has_implementation_details(&doc.body)
                            }
                            _ => {
                                has_section(&doc.body, &rule.name)
                            }
                        };
                        if !pass {
                            findings.push(AuditFinding {
                                check_id: rule.id.clone(),
                                severity: Severity::from_str(&rule.severity),
                                message: format!("{}: '{}'", rule.description, doc.path.as_str()),
                                location: Some(doc.path.as_str().to_string()),
                                document_id: Some(doc.id),
                                provider: "deterministic".into(),
                            });
                        }
                    }
                }
            }
        }

        findings
    }
}

fn has_section(body: &str, name: &str) -> bool {
    let lower = body.to_lowercase();
    let keywords = name.to_lowercase();
    lower.contains(&format!("## {}", keywords))
        || lower.contains(&format!("# {}", keywords))
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
