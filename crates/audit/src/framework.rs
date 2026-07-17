use anyhow::Result;
use rayon::prelude::*;
use schemas::audit::{
    AuditFinding, AuditReport, AuditScore, QualityGate, ReadinessAssessment, SemanticReviewBundle, SemanticReviewTask, Severity,
};
use schemas::document::Document;
use schemas::standard::{AuditRuleDef, StandardDefinition};
use standards::StandardRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub type AuditProviderFn = Arc<
    dyn Fn(&[Document], &[AuditRuleDef], Option<&StandardDefinition>) -> Vec<AuditFinding>
        + Send
        + Sync,
>;

pub struct AuditFramework {
    providers: HashMap<String, AuditProviderFn>,
    standard_registry: Arc<StandardRegistry>,
}

impl AuditFramework {
    pub fn new(standard_registry: Arc<StandardRegistry>) -> Self {
        Self {
            providers: HashMap::new(),
            standard_registry,
        }
    }

    pub fn register_provider(&mut self, name: &str, provider: AuditProviderFn) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    pub fn execute(
        &self,
        domain: Option<&str>,
        documents: &[Document],
        providers: &[String],
    ) -> Result<AuditReport> {
        let standards: Vec<_> = match domain {
            Some(d) => self
                .standard_registry
                .all()
                .into_iter()
                .filter(|s| s.domain == d)
                .cloned()
                .collect(),
            None => self
                .standard_registry
                .all()
                .into_iter()
                .cloned()
                .collect(),
        };

        let rules_by_domain: HashMap<String, Vec<AuditRuleDef>> = standards
            .iter()
            .map(|s| (s.domain.clone(), s.audit_rules.iter().cloned().collect()))
            .collect();
        // Semantic providers get semantic_rules (kind='semantic', evidence_type
        // "llm_judgment"), not audit_rules (kind='deterministic') — was
        // unreachable before semantic_rules existed at all (SemanticAuditProvider
        // ignored its rules param entirely, `_rules`).
        let semantic_rules_by_domain: HashMap<String, Vec<AuditRuleDef>> = standards
            .iter()
            .map(|s| (s.domain.clone(), s.semantic_rules.iter().cloned().collect()))
            .collect();
        let standard_by_domain: HashMap<String, StandardDefinition> = standards
            .iter()
            .map(|s| (s.domain.clone(), s.clone()))
            .collect();

        let domain_docs: Vec<Document> = match domain {
            Some(d) => documents
                .iter()
                .filter(|doc| doc.standard == d)
                .cloned()
                .collect(),
            None => documents.to_vec(),
        };

        let all_findings: Vec<AuditFinding> = match domain {
            Some(d) => {
                let rules = rules_by_domain.get(d).cloned().unwrap_or_default();
                let semantic_rules = semantic_rules_by_domain.get(d).cloned().unwrap_or_default();
                let standard = standard_by_domain.get(d);
                providers
                    .par_iter()
                    .flat_map(|provider_name| {
                        let rules_for_provider = if provider_name == "semantic" { &semantic_rules } else { &rules };
                        self.providers
                            .get(provider_name.as_str())
                            .map(|provider_fn| provider_fn(&domain_docs, rules_for_provider, standard))
                            .unwrap_or_default()
                    })
                    .collect()
            }
            None => {
                let mut doc_groups: HashMap<String, Vec<Document>> = HashMap::new();
                for doc in documents.iter() {
                    doc_groups
                        .entry(doc.standard.clone())
                        .or_default()
                        .push(doc.clone());
                }
                doc_groups
                    .into_par_iter()
                    .flat_map(|(std_name, docs)| {
                        let rules = rules_by_domain.get(&std_name).cloned().unwrap_or_default();
                        let semantic_rules = semantic_rules_by_domain.get(&std_name).cloned().unwrap_or_default();
                        let standard = standard_by_domain.get(&std_name);
                        providers
                            .par_iter()
                            .flat_map(|provider_name| {
                                let rules_for_provider = if provider_name == "semantic" { &semantic_rules } else { &rules };
                                self.providers
                                    .get(provider_name.as_str())
                                    .map(|provider_fn| provider_fn(&docs, rules_for_provider, standard))
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            }
        };

        let total = domain_docs.len();

        // --- Mandatory rule enforcement (Fix 1B) ---
        // Build a set of rule IDs that are mandatory across all loaded standards.
        // Any finding whose check_id matches a mandatory rule is upgraded to Error
        // regardless of the rule's declared severity — a mandatory rule failure is
        // always a blocking error.
        let mandatory_rule_ids: std::collections::HashSet<&str> = standards
            .iter()
            .flat_map(|s| s.audit_rules.iter().chain(s.semantic_rules.iter()))
            .filter(|r| r.mandatory)
            .map(|r| r.id.as_str())
            .collect();

        let all_findings: Vec<AuditFinding> = all_findings
            .into_iter()
            .map(|mut f| {
                if mandatory_rule_ids.contains(f.check_id.as_str())
                    && f.severity != Severity::Error
                {
                    f.severity = Severity::Error;
                }
                f
            })
            .collect();

        let provider_used = providers
            .iter()
            .filter(|name| self.providers.contains_key(name.as_str()))
            .cloned()
            .collect::<Vec<_>>()
            .join(",");
        let error_count = all_findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let passed = total.saturating_sub(error_count);

        // --- Weighted scoring ---
        // Method dispatch, not a hardcoded formula: a standard's own
        // calculation_rules row for the "deterministic_document" bucket
        // names which method applies (see crates/audit/src/calculation.rs).
        // Only "weighted_pass_rate" has a real implementation today — every
        // standard actually loaded so far (python_hackathon, base_dev)
        // declares exactly that for this bucket, but this is checked, not
        // assumed: an unrecognized method logs a warning and falls back
        // rather than silently applying weighted_pass_rate math to a
        // standard that asked for something else.
        let scoring = self.standard_registry.scoring_config();
        let deterministic_method = crate::calculation::find_bucket(&scoring.calculation_rules, "deterministic_document")
            .map(|r| r.calculation_method.as_str());
        if let Some(method) = deterministic_method {
            if method != "weighted_pass_rate" {
                tracing::warn!(
                    "Standard declares calculation_method '{}' for deterministic_document, \
                     but only 'weighted_pass_rate' is implemented — falling back to it anyway.",
                    method
                );
            }
        }

        let mut bucket_scores: HashMap<String, f64> = HashMap::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        for std in &standards {
            let domain_rules: Vec<&AuditRuleDef> = std.audit_rules.iter().collect();
            if domain_rules.is_empty() {
                cat_scores.insert(std.domain.clone(), 100.0);
                continue;
            }
            let domain_docs_list: Vec<&Document> = domain_docs
                .iter()
                .filter(|d| d.standard == std.domain)
                .collect();

            if domain_docs_list.is_empty() {
                // No compiled Document for this domain — not necessarily
                // "nothing to check": a standard whose rules are
                // file_presence/glob_match (python_hackathon's shape, no
                // Document objects involved at all) still has real findings
                // in all_findings, just none tagged with a document_id. The
                // old unconditional "no docs -> score 100" here silently
                // ignored those findings entirely. Score domain-wide instead
                // of per-document-averaged when there's no document to
                // average over.
                let failed_ids: std::collections::HashSet<&str> = all_findings
                    .iter()
                    .filter(|f| domain_rules.iter().any(|r| r.id == f.check_id))
                    .map(|f| f.check_id.as_str())
                    .collect();
                let score = crate::calculation::weighted_pass_rate(&domain_rules, &failed_ids);
                bucket_scores.insert(format!("{}_deterministic_section", std.domain), score);
                cat_scores.insert(std.domain.clone(), score);
                continue;
            }

            let doc_scores: Vec<f64> = domain_docs_list
                .iter()
                .map(|doc| {
                    let failed_ids: std::collections::HashSet<&str> = all_findings
                        .iter()
                        .filter(|f| f.document_id == Some(doc.id))
                        .map(|f| f.check_id.as_str())
                        .collect();
                    crate::calculation::weighted_pass_rate(&domain_rules, &failed_ids)
                })
                .collect();

            // rollup: average over documents present (matches calculation/
            // deterministic/section.yaml's declared rollup method — real
            // per-section rollup within one document is a separate,
            // not-yet-built piece; this averages per-document scores across
            // the domain instead, an existing simplification kept as-is).
            let section_score = if doc_scores.is_empty() {
                100.0
            } else {
                doc_scores.iter().sum::<f64>() / doc_scores.len() as f64
            };

            bucket_scores.insert(
                format!("{}_deterministic_section", std.domain),
                section_score,
            );
            cat_scores.insert(std.domain.clone(), section_score);
        }

        // Document bucket mirrors section for now.
        for (k, v) in bucket_scores.clone() {
            if k.ends_with("_deterministic_section") {
                let doc_key = k.replace("_deterministic_section", "_deterministic_whole");
                bucket_scores.insert(doc_key, v);
            }
        }

        // --- Populate semantic review bundle (MCP-007 fix) ---
        // Collect findings from the semantic provider and convert them into
        // SemanticReviewTask entries so the summary report can roll them up.
        let semantic_findings: Vec<&AuditFinding> = all_findings
            .iter()
            .filter(|f| f.provider == "semantic")
            .collect();

        let semantic_tasks: Vec<SemanticReviewTask> = semantic_findings
            .iter()
            .filter_map(|f| {
                let doc_id = f.document_id?;
                let doc = domain_docs.iter().find(|d| d.id == doc_id)?;
                Some(SemanticReviewTask {
                    document_id: doc_id,
                    section_id: f.section_id.unwrap_or(0),
                    document_title: doc.title.clone(),
                    document_path: doc.path.as_str().to_string(),
                    domain: doc.standard.clone(),
                    semantic_type: f.check_id.clone(),
                    content: f.message.clone(),
                })
            })
            .collect();

        let semantic_review = if semantic_tasks.is_empty() {
            SemanticReviewBundle::default()
        } else {
            SemanticReviewBundle {
                instruction: "Review each section against the domain's writing guidance rubric. \
                    Score completeness, tone, technology independence, and audience appropriateness."
                    .to_string(),
                rubrics: {
                    let mut r = HashMap::new();
                    r.insert("completeness".into(), "Section covers all required aspects per the standard".into());
                    r.insert("tone".into(), "Writing matches the standard's voice guidance".into());
                    r.insert("technology_independence".into(), "Domain docs avoid implementation-specific references".into());
                    r
                },
                tasks: semantic_tasks,
            }
        };

        // Final score: weighted sum of buckets, per calculation_inputs
        // (default 25/25/25/25 when a standard has none loaded).
        if let Some(final_rule) = crate::calculation::find_bucket(&scoring.calculation_rules, "summary_final_score") {
            if final_rule.calculation_method != "weighted_sum" {
                tracing::warn!(
                    "Standard declares calculation_method '{}' for summary_final_score, \
                     but only 'weighted_sum' is implemented — falling back to it anyway.",
                    final_rule.calculation_method
                );
            }
        }
        // calculation_inputs names bare bucket keys (e.g. "deterministic_whole"
        // — see calculation/summary/final_score.yaml, which is written to
        // score one domain, not the whole standard at once), but every entry
        // in bucket_scores above is domain-prefixed ("infrastructure_
        // deterministic_whole"). Looking `calculation_inputs` up directly
        // against `bucket_scores` therefore never matched anything and
        // `overall` silently defaulted to 100 regardless of any actual
        // finding — true for every audit before this fix, not new behavior
        // introduced here, caught by an end-to-end check against real data
        // instead of a synthetic all-pass fixture. Scoped to the audited
        // domain when one was given, by stripping that domain's prefix back
        // off before the lookup; a whole-standard (domain: None) audit has
        // no single prefix to strip and keeps the prior (still-100-default)
        // behavior — computing a real cross-domain final score is a
        // different, not-yet-built piece (see Phase 3a's team/session
        // decision in docs/crates-refactor-proposal.md).
        let scoped_bucket_scores: HashMap<String, f64> = match domain {
            Some(d) => {
                let prefix = format!("{}_", d);
                bucket_scores
                    .iter()
                    .filter_map(|(k, v)| k.strip_prefix(prefix.as_str()).map(|bare| (bare.to_string(), *v)))
                    .collect()
            }
            None => bucket_scores.clone(),
        };
        let overall = crate::calculation::weighted_sum(&scoring.calculation_inputs, &scoped_bucket_scores);

        let rating = crate::calculation::threshold_lookup(overall, &scoring.score_bands)
            .unwrap_or_else(|| "Unknown".to_string());

        let readiness = if overall >= 90.0 && error_count == 0 {
            ReadinessAssessment::Production
        } else if overall >= 80.0 {
            ReadinessAssessment::Implementation
        } else if overall >= 70.0 {
            ReadinessAssessment::Engineering
        } else if overall >= 60.0 {
            ReadinessAssessment::Design
        } else if overall >= 50.0 {
            ReadinessAssessment::Architecture
        } else {
            ReadinessAssessment::Product
        };

        let score = AuditScore {
            overall,
            categories: cat_scores,
            documents_checked: total,
            documents_passed: passed,
            findings_count: all_findings.len(),
            rating,
            bucket_scores,
        };

        let report = AuditReport {
            id: format!("audit-{}", unix_now()),
            domain: domain.map(|d| d.to_string()),
            timestamp: unix_now(),
            provider: provider_used,
            score,
            findings: all_findings,
            readiness,
            metadata: HashMap::new(),
            semantic_review,
        };

        info!(
            "Audit complete: overall={:.1}%, documents={}, findings={}, readiness={}",
            report.score.overall,
            report.score.documents_checked,
            report.score.findings_count,
            report.readiness,
        );

        Ok(report)
    }

    pub fn check_quality_gate(report: &AuditReport, gate: &QualityGate) -> Result<bool> {
        if !gate.enabled {
            return Ok(true);
        }
        if let Some(min) = gate.min_score {
            if report.score.overall < min {
                anyhow::bail!(
                    "Quality gate failed: score {:.1}% < minimum {:.1}%",
                    report.score.overall,
                    min,
                );
            }
        }
        if let Some(ref min_readiness) = gate.min_readiness {
            if !readiness_meets(&report.readiness, min_readiness) {
                anyhow::bail!(
                    "Quality gate failed: readiness {} < minimum {}",
                    report.readiness,
                    min_readiness,
                );
            }
        }
        Ok(true)
    }
}

fn readiness_meets(current: &ReadinessAssessment, required: &ReadinessAssessment) -> bool {
    let order = [
        ReadinessAssessment::None,
        ReadinessAssessment::Product,
        ReadinessAssessment::Architecture,
        ReadinessAssessment::Design,
        ReadinessAssessment::Engineering,
        ReadinessAssessment::Implementation,
        ReadinessAssessment::Production,
    ];
    let current_idx = order.iter().position(|r| r == current).unwrap_or(0);
    let required_idx = order.iter().position(|r| r == required).unwrap_or(0);
    current_idx >= required_idx
}

fn unix_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::standard::{AuditRuleDef, StandardDefinition};

    fn rule(id: &str) -> AuditRuleDef {
        AuditRuleDef {
            id: id.into(),
            name: id.into(),
            description: String::new(),
            severity: "warning".into(),
            evidence_type: "file_presence".into(),
            scope: String::new(),
            weight: 1.0,
            mandatory: false,
            params: HashMap::new(),
        }
    }

    fn finding(check_id: &str) -> AuditFinding {
        AuditFinding {
            check_id: check_id.into(),
            severity: Severity::Warning,
            message: String::new(),
            location: None,
            document_id: None, // file_presence/glob_match findings never reference a Document
            provider: "deterministic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
        }
    }

    /// Regression test: a domain whose rules produce findings with no
    /// document_id (file_presence/glob_match — python_hackathon's shape)
    /// must not silently score 100 regardless of those findings, which is
    /// what happened before this test existed (any domain with zero
    /// Document objects short-circuited straight to a 100.0 default).
    fn framework_with_domain(domain: &str, rules: Vec<AuditRuleDef>, findings: Vec<AuditFinding>) -> AuditFramework {
        let mut registry = StandardRegistry::new();
        registry.register(StandardDefinition {
            id: domain.into(),
            name: domain.into(),
            version: "1.0.0".into(),
            domain: domain.into(),
            description: String::new(),
            required_sections: vec![],
            prohibited_content: vec![],
            relationships: vec![],
            audit_rules: rules,
            semantic_rules: vec![],
            profiles: vec![],
            tier: None,
        });
        let mut framework = AuditFramework::new(Arc::new(registry));
        framework.register_provider("deterministic", Arc::new(move |_docs, _rules, _standard| findings.clone()));
        framework
    }

    #[test]
    fn domain_with_no_documents_scores_from_findings_not_a_hardcoded_100() {
        let framework = framework_with_domain(
            "infrastructure",
            vec![rule("inf-001"), rule("inf-002")],
            vec![finding("inf-001")], // 1 of 2 rules failed
        );
        let report = framework.execute(Some("infrastructure"), &[], &["deterministic".to_string()]).unwrap();
        assert_eq!(report.score.categories.get("infrastructure"), Some(&50.0));
    }

    #[test]
    fn domain_with_no_findings_and_no_documents_scores_100() {
        let framework = framework_with_domain(
            "infrastructure",
            vec![rule("inf-001"), rule("inf-002")],
            vec![],
        );
        let report = framework.execute(Some("infrastructure"), &[], &["deterministic".to_string()]).unwrap();
        assert_eq!(report.score.categories.get("infrastructure"), Some(&100.0));
    }

    #[test]
    fn domain_with_no_rules_still_defaults_to_100() {
        let framework = framework_with_domain("empty-domain", vec![], vec![]);
        let report = framework.execute(Some("empty-domain"), &[], &["deterministic".to_string()]).unwrap();
        assert_eq!(report.score.categories.get("empty-domain"), Some(&100.0));
    }
}
