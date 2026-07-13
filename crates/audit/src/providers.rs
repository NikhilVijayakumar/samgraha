use compiler::validation::SectionValidator;
use rayon::prelude::*;
use schemas::audit::{AuditFinding, Severity};
use schemas::diagnostics::{CompilationDiagnostic, DiagnosticSeverity};
use schemas::document::Document;
use schemas::standard::{AuditRuleDef, ScriptCheck, StandardDefinition};

pub struct DeterministicAuditProvider;

impl DeterministicAuditProvider {
    pub fn execute(
        documents: &[Document],
        rules: &[AuditRuleDef],
        standard: Option<&StandardDefinition>,
        config: Option<&common::config::SamgrahaConfig>,
        root: Option<&std::path::Path>,
        script_checks: &[ScriptCheck],
    ) -> Vec<AuditFinding> {
        let mut findings: Vec<AuditFinding> = rules
            .par_iter()
            .flat_map(|rule| Self::check_rule(documents, rule, config, root, script_checks))
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

    fn check_rule(
        documents: &[Document], 
        rule: &AuditRuleDef,
        config: Option<&common::config::SamgrahaConfig>,
        root: Option<&std::path::Path>,
        script_checks: &[ScriptCheck],
    ) -> Vec<AuditFinding> {
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
            "keyword_absence" => {
                // Use keywords from evidence params if provided by the DB rule;
                // fall back to the built-in implementation-details heuristic
                // when no params are declared (backward compat with Phase 1
                // rules that predate rule_evidence_params population).
                let param_keywords: Vec<String> = rule
                    .params
                    .get("keywords")
                    .map(|k| k.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect())
                    .unwrap_or_default();
                documents
                    .par_iter()
                    .filter(|doc| {
                        let body_lower = doc.body.raw().to_lowercase();
                        if param_keywords.is_empty() {
                            // Legacy fallback: check for hardcoded implementation indicators.
                            has_implementation_details(doc.body.raw())
                        } else {
                            // DB-driven: fail if any of the declared keywords appear in the body.
                            param_keywords.iter().any(|kw| body_lower.contains(kw.as_str()))
                        }
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
            "content_check" => {
                // content_check verifies that certain content is *present* in a document.
                // The "mode" param can override to "must_not_contain" for exclusion checks.
                let mode = rule.params.get("mode").map(|s| s.as_str()).unwrap_or("must_contain");
                let keywords: Vec<String> = rule.params.get("keywords")
                    .map(|k| k.split(',').map(|s| s.trim().to_lowercase()).collect())
                    .unwrap_or_default();
                let pattern = rule.params.get("pattern").cloned().unwrap_or_default();
                documents
                    .par_iter()
                    .filter(|doc| {
                        let body_lower = doc.body.raw().to_lowercase();
                        match mode {
                            "must_not_contain" => {
                                // Fail if any keyword or pattern is present.
                                if !keywords.is_empty() {
                                    return keywords.iter().any(|kw| body_lower.contains(kw.as_str()));
                                }
                                if !pattern.is_empty() {
                                    return body_lower.contains(pattern.as_str());
                                }
                                false
                            }
                            _ => {
                                // Default "must_contain": fail if any required keyword is absent.
                                if !keywords.is_empty() {
                                    return keywords.iter().any(|kw| !body_lower.contains(kw.as_str()));
                                }
                                if !pattern.is_empty() {
                                    return !body_lower.contains(pattern.as_str());
                                }
                                false
                            }
                        }
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
            "script" | "script_result" => {
                let Some(root) = root else {
                    return vec![];
                };
                let override_script = config.and_then(|c| c.repository.documentation.script_overrides.get(&rule.id));
                let default_script = rule.params.get("script");

                // Also check for a local synced copy first (from standards sync).
                let local_script = root.join(".samgraha").join("scripts")
                    .join(rule.params.get("script").map(|s| s.as_str()).unwrap_or(""));
                
                let script_path = if let Some(override_path) = override_script {
                    root.join(override_path)
                } else if local_script.exists() && default_script.is_some() {
                    // Prefer local synced copy over global ~/.samgraha/scripts/
                    local_script
                } else if let Some(default_path) = default_script {
                    common::env::home_dir().join(".samgraha").join("scripts").join(default_path)
                } else {
                    return vec![];
                };

                // Look up timeout from ScriptCheck registry (Fix 2C).
                let timeout_secs: Option<u64> = script_checks
                    .iter()
                    .find(|sc| sc.check_name == rule.id || sc.check_name == rule.params.get("script").map(|s| s.as_str()).unwrap_or(""))
                    .filter(|sc| sc.timeout_seconds > 0)
                    .map(|sc| sc.timeout_seconds as u64);

                if !script_path.exists() {
                    return vec![AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("Script not found: {}", script_path.display()),
                        location: None,
                        document_id: None,
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    }];
                }

                documents
                    .par_iter()
                    .filter_map(|doc| {
                        let mut cmd = std::process::Command::new(&script_path);
                        cmd.arg(doc.path.as_str());
                        cmd.current_dir(root);

                        // Apply timeout from ScriptCheck registry if available.
                        let output_result = if let Some(secs) = timeout_secs {
                            use std::time::Duration;
                            cmd.spawn()
                                .and_then(|mut child| {
                                    let deadline = std::time::Instant::now() + Duration::from_secs(secs);
                                    loop {
                                        match child.try_wait() {
                                            Ok(Some(status)) => {
                                                // Collect output after process exits.
                                                return child.wait_with_output().map(|o| {
                                                    // Reconstruct with real status.
                                                    let _ = status;
                                                    o
                                                });
                                            }
                                            Ok(None) => {
                                                if std::time::Instant::now() >= deadline {
                                                    let _ = child.kill();
                                                    return Err(std::io::Error::new(
                                                        std::io::ErrorKind::TimedOut,
                                                        format!("Script timed out after {}s", secs),
                                                    ));
                                                }
                                                std::thread::sleep(Duration::from_millis(100));
                                            }
                                            Err(e) => return Err(e),
                                        }
                                    }
                                })
                        } else {
                            cmd.output()
                        };
                        
                        match output_result {
                            Ok(output) if !output.status.success() => {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                let msg = if !stderr.is_empty() { stderr } else { stdout };
                                Some(AuditFinding {
                                    check_id: rule.id.clone(),
                                    severity: Severity::from_str(&rule.severity),
                                    message: format!("{}: {}", rule.description, msg.trim()),
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
                            }
                            Err(e) => {
                                Some(AuditFinding {
                                    check_id: rule.id.clone(),
                                    severity: Severity::from_str(&rule.severity),
                                    message: format!("Failed to run script: {}", e),
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
                            }
                            _ => None,
                        }
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
        let doc = doc_with_no_sections();
        let standard = standard_requiring("Communication");

        let findings = DeterministicAuditProvider::execute(&[doc], &[], Some(&standard), None, None, &[]);

        assert!(findings.iter().any(|f| f.check_id == "compile-missing-section"));
    }

    #[test]
    fn execute_without_standard_only_runs_declared_rules() {
        let doc = doc_with_no_sections();
        let findings = DeterministicAuditProvider::execute(&[doc], &[], None, None, None, &[]);
        assert!(findings.is_empty());
    }

    fn doc_with_body(body: &str) -> Document {
        Document {
            id: 2,
            path: DocumentPath(PathBuf::from("docs/vision/overview.md")),
            hash: "def".into(),
            standard: "vision".into(),
            title: "Vision".into(),
            body: DocumentBody::Generic { raw: body.to_string(), sections: vec![] },
            metadata: DocumentMetadata::default(),
            provenance: None,
            quality: ObjectStatistics::default(),
            created_at: "0".into(),
            updated_at: "0".into(),
        }
    }

    fn rule(id: &str, evidence_type: &str, params: Vec<(&str, &str)>) -> AuditRuleDef {
        AuditRuleDef {
            id: id.into(),
            name: id.into(),
            description: format!("Rule {}", id),
            severity: "warning".into(),
            evidence_type: evidence_type.into(),
            scope: String::new(),
            weight: 1.0,
            mandatory: false,
            params: params.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        }
    }

    #[test]
    fn keyword_absence_uses_params_keywords_not_hardcoded_heuristic() {
        // DB-driven: rule declares ["prototype"] as forbidden keyword.
        let doc = doc_with_body("This is a prototype document.");
        let r = rule("ka-001", "keyword_absence", vec![("keywords", "prototype")]); 
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty(), "Expected finding for 'prototype' keyword");

        // A doc without the keyword should pass.
        let doc2 = doc_with_body("This is a clean vision document about goals.");
        let r2 = rule("ka-001", "keyword_absence", vec![("keywords", "prototype")]);
        let findings2 = DeterministicAuditProvider::execute(&[doc2], &[r2], None, None, None, &[]);
        assert!(findings2.is_empty(), "No finding expected when keyword absent");
    }

    #[test]
    fn keyword_absence_fallback_to_heuristic_when_no_params() {
        // Legacy fallback: no params → uses hardcoded impl-detail heuristic.
        let doc = doc_with_body("```rust\nfn main() {}\n```");
        let r = rule("ka-legacy", "keyword_absence", vec![]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty(), "Legacy heuristic should fire on code blocks");
    }

    #[test]
    fn content_check_must_contain_fails_when_keyword_absent() {
        let doc = doc_with_body("A document without overview section.");
        let r = rule("cc-001", "content_check", vec![("keywords", "overview,introduction")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty());
    }

    #[test]
    fn content_check_must_contain_passes_when_keyword_present() {
        let doc = doc_with_body("This document has an overview section.");
        let r = rule("cc-002", "content_check", vec![("keywords", "overview")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(findings.is_empty());
    }

    #[test]
    fn content_check_must_not_contain_fires_when_present() {
        let doc = doc_with_body("Document mentions implementation details.");
        let r = rule("cc-003", "content_check", vec![
            ("mode", "must_not_contain"),
            ("keywords", "implementation details"),
        ]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty());
    }

    #[test]
    fn word_count_fails_above_limit() {
        let body = "word ".repeat(6000);
        let doc = doc_with_body(&body);
        let r = rule("wc-001", "word_count", vec![("max_words", "5000")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty());
    }

    #[test]
    fn word_count_passes_below_limit() {
        let body = "word ".repeat(100);
        let doc = doc_with_body(&body);
        let r = rule("wc-002", "word_count", vec![("max_words", "5000")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(findings.is_empty());
    }

    #[test]
    fn cross_reference_fires_when_domain_missing() {
        let doc = doc_with_body("This doc only mentions vision.");
        let r = rule("cr-001", "cross_reference", vec![("expected_domains", "architecture,engineering")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(!findings.is_empty());
    }

    #[test]
    fn cross_reference_passes_when_all_domains_mentioned() {
        let doc = doc_with_body("This doc mentions architecture and engineering domains.");
        let r = rule("cr-002", "cross_reference", vec![("expected_domains", "architecture,engineering")]);
        let findings = DeterministicAuditProvider::execute(&[doc], &[r], None, None, None, &[]);
        assert!(findings.is_empty());
    }
}
