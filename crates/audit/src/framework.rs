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

        let mut cat_scores: HashMap<String, f64> = HashMap::new();
        for std in &standards {
            let domain_findings: Vec<&AuditFinding> = all_findings
                .iter()
                .filter(|f| {
                    f.document_id
                        .and_then(|id| domain_docs.iter().find(|d| d.id == id))
                        .map_or(false, |d| d.standard == std.domain)
                })
                .collect();
            let domain_total = domain_docs
                .iter()
                .filter(|d| d.standard == std.domain)
                .count();
            let errors = domain_findings
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .count();
            let score = if domain_total == 0 {
                100.0
            } else {
                let passed = domain_total.saturating_sub(errors);
                (passed as f64 / domain_total as f64) * 100.0
            };
            cat_scores.insert(std.domain.clone(), score);
        }

        let overall = if total == 0 {
            100.0
        } else {
            let passed = total.saturating_sub(error_count);
            (passed as f64 / total as f64) * 100.0
        };

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
