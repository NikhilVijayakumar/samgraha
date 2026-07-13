use anyhow::Result;
use rayon::prelude::*;
use schemas::audit::{
    AuditFinding, AuditReport, AuditScore, QualityGate, ReadinessAssessment, SemanticReviewBundle, Severity,
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
                let standard = standard_by_domain.get(d);
                providers
                    .par_iter()
                    .flat_map(|provider_name| {
                        self.providers
                            .get(provider_name.as_str())
                            .map(|provider_fn| provider_fn(&domain_docs, &rules, standard))
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
                        let standard = standard_by_domain.get(&std_name);
                        providers
                            .par_iter()
                            .flat_map(|provider_name| {
                                self.providers
                                    .get(provider_name.as_str())
                                    .map(|provider_fn| provider_fn(&docs, &rules, standard))
                                    .unwrap_or_default()
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            }
        };

        let total = domain_docs.len();

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

        // --- Weighted scoring (Phase 4) ---
        let scoring = self.standard_registry.scoring_config();
        let mut bucket_scores: HashMap<String, f64> = HashMap::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        for std in &standards {
            let domain_rules: Vec<&AuditRuleDef> = std.audit_rules.iter().collect();
            let domain_docs_list: Vec<&Document> = domain_docs
                .iter()
                .filter(|d| d.standard == std.domain)
                .collect();

            if domain_rules.is_empty() || domain_docs_list.is_empty() {
                cat_scores.insert(std.domain.clone(), 100.0);
                continue;
            }

            let total_weight: f64 = domain_rules.iter().map(|r| r.weight).sum();
            let mut doc_scores = Vec::new();

            for doc in &domain_docs_list {
                let failed_ids: std::collections::HashSet<&str> = all_findings
                    .iter()
                    .filter(|f| f.document_id == Some(doc.id))
                    .map(|f| f.check_id.as_str())
                    .collect();

                let passed_weight: f64 = domain_rules
                    .iter()
                    .filter(|r| !failed_ids.contains(r.id.as_str()))
                    .map(|r| r.weight)
                    .sum();

                let doc_score = if total_weight > 0.0 {
                    (passed_weight / total_weight) * 100.0
                } else {
                    100.0
                };
                doc_scores.push(doc_score);
            }

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

        // Final score: weighted sum of buckets (default 25/25/25/25).
        let final_inputs: Vec<(&str, f64)> = scoring
            .calculation_inputs
            .iter()
            .map(|ci| (ci.name.as_str(), ci.weight))
            .collect();

        let total_weight: f64 = final_inputs.iter().map(|(_, w)| w).sum();
        let overall = if total_weight > 0.0 {
            final_inputs
                .iter()
                .map(|(name, weight)| {
                    let bucket_score = bucket_scores.get(*name).copied().unwrap_or(100.0);
                    (bucket_score / 100.0) * weight
                })
                .sum::<f64>()
        } else {
            100.0
        };

        let rating = scoring
            .score_bands
            .iter()
            .find(|band| overall >= band.min_score && overall <= band.max_score)
            .map(|band| band.rating.clone())
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
            semantic_review: SemanticReviewBundle::default(),
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
