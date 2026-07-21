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
                // Collection-scope check: the section must exist *somewhere* in the
                // domain, not in every document.  One finding per missing section, not
                // one per document missing it.  (MCP-001 fix)
                let section_key = rule
                    .scope
                    .to_lowercase()
                    .replace(' ', "_")
                    .replace('-', "_");
                let has_anywhere = documents.iter().any(|doc| {
                    let count = doc
                        .quality
                        .per_type
                        .get(&section_key)
                        .copied()
                        .unwrap_or(0);
                    if count > 0 {
                        return true;
                    }
                    let title_key = doc
                        .title
                        .to_lowercase()
                        .replace(' ', "_")
                        .replace('-', "_");
                    title_key == section_key
                });
                if has_anywhere {
                    vec![]
                } else {
                    vec![AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!(
                            "{}: section '{}' not found in any document in the domain",
                            rule.description, rule.scope
                        ),
                        location: None,
                        document_id: None,
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    }]
                }
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
                            // Uses word-boundary matching to avoid substring false positives
                            // (e.g. "pipelines" should not trigger "pip" check). (MCP-005)
                            param_keywords.iter().any(|kw| contains_word(&body_lower, kw))
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
                                    return keywords.iter().any(|kw| contains_word(&body_lower, kw));
                                }
                                if !pattern.is_empty() {
                                    return contains_word(&body_lower, &pattern);
                                }
                                false
                            }
                            _ => {
                                // Default "must_contain": fail if any required keyword is absent.
                                if !keywords.is_empty() {
                                    return keywords.iter().any(|kw| !contains_word(&body_lower, kw));
                                }
                                if !pattern.is_empty() {
                                    return !contains_word(&body_lower, &pattern);
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
                            !contains_word(&body_lower, &domain_lower)
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
            "file_presence" => {
                let Some(root) = root else { return vec![] };
                let candidates = Self::path_candidates(rule);
                if candidates.is_empty() || candidates.iter().any(|p| root.join(p).exists()) {
                    vec![]
                } else {
                    vec![Self::finding(rule, format!(
                        "{}: none of [{}] found",
                        rule.description,
                        candidates.join(", ")
                    ), None, None)]
                }
            }
            "file_absence" => {
                let Some(root) = root else { return vec![] };
                let candidates = Self::path_candidates(rule);
                let present: Vec<&String> = candidates.iter().filter(|p| root.join(p).exists()).collect();
                if present.is_empty() {
                    vec![]
                } else {
                    vec![Self::finding(rule, format!(
                        "{}: unexpected file(s) present: {}",
                        rule.description,
                        present.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                    ), None, None)]
                }
            }
            "glob_match" => {
                let Some(root) = root else { return vec![] };
                let patterns: Vec<String> = rule.params.get("pattern")
                    .map(|p| p.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                    .unwrap_or_default();
                if patterns.is_empty() {
                    return vec![];
                }
                let matched = walk_repo_relative_paths(root).iter().any(|rel| {
                    patterns.iter().any(|pat| common::glob::matches_glob(pat, rel))
                });
                if matched {
                    vec![]
                } else {
                    vec![Self::finding(rule, format!(
                        "{}: no file matched [{}]",
                        rule.description,
                        patterns.join(", ")
                    ), None, None)]
                }
            }
            // "script_output" is the literal evidence.type string real rule
            // files use (verified: python_hackathon's testing/security/mlops/
            // model-artifact/git domains all use `evidence: {type: script_output,
            // script: ..., check: ...}`) — an alias of "script"/"script_result",
            // not a different protocol.
            "script" | "script_result" | "script_output" => {
                let Some(root) = root else {
                    return vec![];
                };

                // Extract check name from rule_ref (e.g. "script/schema/14-build/build-succeeds.schema.json#..." → "build-succeeds")
                let check_name_from_ref = rule.params.get("rule_ref").and_then(|ref_path| {
                    let file_part = ref_path.split('#').next()?;
                    let stem = std::path::Path::new(file_part).file_stem()?.to_str()?;
                    // strip trailing ".schema" if present
                    Some(stem.strip_suffix(".schema").unwrap_or(stem).to_string())
                });

                // Resolution: check_overrides[check_name] → check_overrides[rule_id] → script_overrides[rule_id] → local → global
                let check_override = check_name_from_ref.as_ref().and_then(|name| {
                    config.and_then(|c| c.repository.documentation.check_overrides.get(name))
                }).or_else(|| {
                    config.and_then(|c| c.repository.documentation.check_overrides.get(&rule.id))
                });
                let override_script = config.and_then(|c| c.repository.documentation.script_overrides.get(&rule.id));

                // Also try local synced copy, using check_name_from_ref as the script filename
                let script_name = check_name_from_ref.as_deref()
                    .or_else(|| rule.params.get("script").map(|s| s.as_str()))
                    .unwrap_or("");

                let resolved = check_override
                    .map(|p| root.join(p))
                    .or_else(|| override_script.map(|p| root.join(p)))
                    .or_else(|| {
                        if script_name.is_empty() {
                            return None;
                        }
                        crate::check_runner::probe_script(&root.join(".samgraha").join("scripts"), script_name)
                            // Namespaced global store: mcp_dir()/systems/<name>/scripts/
                            .or_else(|| {
                                let sys_name = config.and_then(|c| c.repository.documentation.standard_system.as_deref())?;
                                crate::check_runner::probe_script(
                                    &common::env::mcp_dir().join("systems").join(sys_name).join("scripts"),
                                    script_name,
                                )
                            })
                            // Legacy flat global store fallback
                            .or_else(|| crate::check_runner::probe_script(&common::env::mcp_dir().join("scripts"), script_name))
                    });
                let Some(script_path) = resolved else {
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

                // Repo-level check, not per-document: the real scripts take
                // -RepoRoot/-RepoFingerprint/-Out only — no document argument
                // exists in their interface (verified against the actual
                // docs/knowledge-hub/script/{windows,ubuntu} scripts). Running
                // once per document would re-run (e.g.) a full `cargo build`
                // once per doc and multiply one repo-level fact into N
                // identical findings — run it once, attach no document.
                let fingerprint = format!("{}-{}", script_name, root.display());
                match common::env::run_check_script(&script_path, root, &fingerprint, timeout_secs) {
                    Ok(json) => {
                        let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("error");
                        if status == "pass" || status == "not_applicable" {
                            return vec![];
                        }
                        let evidence_msg = json
                            .get("evidence")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            })
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| format!("status: {}", status));
                        vec![AuditFinding {
                            check_id: rule.id.clone(),
                            severity: Severity::from_str(&rule.severity),
                            message: format!("{}: {}", rule.description, evidence_msg),
                            location: None,
                            document_id: None,
                            provider: "deterministic".into(),
                            stage: None,
                            section_id: None,
                            confidence: None,
                            evidence: None,
                            status: None,
                            strategy: None,
                        }]
                    }
                    Err(e) => vec![AuditFinding {
                        check_id: rule.id.clone(),
                        severity: Severity::from_str(&rule.severity),
                        message: format!("Failed to run script: {}", e),
                        location: None,
                        document_id: None,
                        provider: "deterministic".into(),
                        stage: None,
                        section_id: None,
                        confidence: None,
                        evidence: None,
                        status: None,
                        strategy: None,
                    }],
                }
            }
            _ => vec![],
        }
    }

    fn finding(rule: &AuditRuleDef, message: String, location: Option<String>, document_id: Option<i64>) -> AuditFinding {
        AuditFinding {
            check_id: rule.id.clone(),
            severity: Severity::from_str(&rule.severity),
            message,
            location,
            document_id,
            provider: "deterministic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
        }
    }

    /// A file_presence/file_absence rule names its path(s) via `target` (one
    /// path) and/or `paths` (comma-joined alternatives, OR semantics) —
    /// both real shapes, not standard-specific: base_dev-style rules tend to
    /// use `target`, python_hackathon-style rules use both across its own
    /// rule files.
    fn path_candidates(rule: &AuditRuleDef) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if let Some(t) = rule.params.get("target") {
            out.push(t.clone());
        }
        if let Some(p) = rule.params.get("paths") {
            out.extend(p.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        }
        out
    }
}

/// Repo-relative paths of every file under `root`, skipping VCS/build/
/// dependency directories a glob_match rule was never meant to search.
/// Bounded at 50k files — ponytail: flat-list walk, not an indexed search;
/// raise the cap or add real ignore-pattern support (`common::config::IgnoreConfig`,
/// already used by compilation) if a glob_match rule ever needs to search a
/// repo bigger than that.
fn walk_repo_relative_paths(root: &std::path::Path) -> Vec<String> {
    const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".samgraha", "__pycache__", ".venv", "venv"];
    const MAX_FILES: usize = 50_000;
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if !SKIP_DIRS.contains(&name.as_ref()) {
                    stack.push(path);
                }
            } else if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
                if out.len() >= MAX_FILES {
                    return out;
                }
            }
        }
    }
    out
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

/// Word-boundary-aware substring match.  Prevents "rust" matching inside
/// "trust", "aspiration" satisfying "aspiration" check when only "aspir"
/// was intended, etc.  Treats hyphens and underscores as word-joining
/// (same as `pipeline::has_word_boundaries`).
fn contains_word(text: &str, keyword: &str) -> bool {
    fn joins_word(c: char) -> bool {
        c.is_alphanumeric() || c == '-' || c == '_'
    }
    let kw_lower = keyword.to_lowercase();
    let mut start = 0;
    while let Some(pos) = text[start..].find(&kw_lower) {
        let abs = start + pos;
        let before_ok = !joins_word(kw_lower.chars().next().unwrap_or('x'))
            || text[..abs].chars().next_back().is_none_or(|c| !joins_word(c));
        let end = abs + kw_lower.len();
        let after_ok = !joins_word(kw_lower.chars().last().unwrap_or('x'))
            || text[end..].chars().next().is_none_or(|c| !joins_word(c));
        if before_ok && after_ok {
            return true;
        }
        start = abs + 1;
    }
    false
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
                section_catalog_id: None,
            }],
            prohibited_content: vec![],
            relationships: vec![],
            audit_rules: vec![],
            semantic_rules: vec![],
            profiles: vec![],
            tier: None,
            content_kind: "documentation".to_string(),
            generation_granularity: "section".to_string(),
            section_dependencies: vec![],
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

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("samgraha-test-providers-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn file_presence_passes_when_target_exists() {
        let dir = scratch_dir("file-presence-pass");
        std::fs::write(dir.join("uv.lock"), "").unwrap();
        let r = rule("inf-001", "file_presence", vec![("target", "uv.lock")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert!(findings.is_empty(), "uv.lock exists, expected no finding");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_presence_fails_when_target_missing() {
        let dir = scratch_dir("file-presence-fail");
        let r = rule("inf-001", "file_presence", vec![("target", "uv.lock")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert_eq!(findings.len(), 1, "uv.lock missing, expected one finding");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_presence_paths_are_or_semantics() {
        // Real shape: multiple alternative paths, any one present satisfies the rule.
        let dir = scratch_dir("file-presence-or");
        std::fs::write(dir.join("poetry.lock"), "").unwrap();
        let r = rule("inf-002", "file_presence", vec![("paths", "uv.lock,poetry.lock")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert!(findings.is_empty(), "poetry.lock satisfies the OR, expected no finding");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn file_absence_fails_when_target_present() {
        let dir = scratch_dir("file-absence-fail");
        std::fs::write(dir.join("secrets.env"), "").unwrap();
        let r = rule("sec-001", "file_absence", vec![("target", "secrets.env")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert_eq!(findings.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn glob_match_finds_file_matching_one_of_several_patterns() {
        let dir = scratch_dir("glob-match");
        std::fs::write(dir.join("docker-compose.yml"), "").unwrap();
        let r = rule("inf-003", "glob_match", vec![("pattern", "docker-compose.yaml,docker-compose.yml")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert!(findings.is_empty(), "docker-compose.yml matches the second pattern, expected no finding");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn glob_match_searches_nested_paths_with_double_star() {
        let dir = scratch_dir("glob-match-recursive");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("main.dockerfile"), "").unwrap();
        let r = rule("inf-004", "glob_match", vec![("pattern", "**/*.dockerfile")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert!(findings.is_empty(), "**/*.dockerfile should match a nested .dockerfile file");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn glob_match_fails_when_nothing_matches() {
        let dir = scratch_dir("glob-match-fail");
        let r = rule("inf-003", "glob_match", vec![("pattern", "docker-compose.yaml,docker-compose.yml")]);
        let findings = DeterministicAuditProvider::execute(&[], &[r], None, None, Some(&dir), &[]);
        assert_eq!(findings.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
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
