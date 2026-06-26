use anyhow::Result;
use schemas::audit::{
    AuditFinding, AuditReport, AuditScore, QualityGate, ReadinessAssessment, Severity,
};
use schemas::document::Document;
use schemas::standard::{AuditRuleDef, StandardDefinition};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub type AuditProviderFn =
    Arc<dyn Fn(&[Document], &[AuditRuleDef]) -> Vec<AuditFinding> + Send + Sync>;

pub struct AuditFramework {
    providers: HashMap<String, AuditProviderFn>,
    _rules_cache: HashMap<String, Vec<AuditRuleDef>>,
}

impl AuditFramework {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            _rules_cache: HashMap::new(),
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
        standards: &[StandardDefinition],
        providers: &[String],
    ) -> Result<AuditReport> {
        let domain_docs: Vec<Document> = match domain {
            Some(d) => documents
                .iter()
                .filter(|doc| doc.standard == d)
                .cloned()
                .collect(),
            None => documents.to_vec(),
        };

        let standards: Vec<&StandardDefinition> = match domain {
            Some(d) => standards.iter().filter(|s| s.domain == d).collect(),
            None => standards.iter().collect(),
        };

        let mut all_findings = Vec::new();
        let mut provider_used = String::new();

        for provider_name in providers {
            if let Some(provider_fn) = self.providers.get(provider_name) {
                let rules: Vec<AuditRuleDef> = standards
                    .iter()
                    .flat_map(|s| s.audit_rules.clone())
                    .collect();
                let findings = provider_fn(&domain_docs, &rules);
                all_findings.extend(findings);
                provider_used = provider_name.clone();
            }
        }

        let total = domain_docs.len();
        let passed = total.saturating_sub(
            all_findings
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .count(),
        );

        let mut cat_scores = HashMap::new();
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
                ((domain_total - errors) as f64 / domain_total as f64) * 100.0
            };
            cat_scores.insert(std.domain.clone(), score);
        }

        let total_errors = all_findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count();
        let overall = if total == 0 {
            100.0
        } else {
            ((total - total_errors) as f64 / total as f64) * 100.0
        };

        let readiness = if overall >= 90.0 && total_errors == 0 {
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
            id: format!("audit-{}", chrono_now()),
            domain: domain.map(|d| d.to_string()),
            timestamp: chrono_now(),
            provider: provider_used,
            score,
            findings: all_findings,
            readiness,
            metadata: HashMap::new(),
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
            let meets = readiness_meets(&report.readiness, min_readiness);
            if !meets {
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

impl Default for AuditFramework {
    fn default() -> Self {
        Self::new()
    }
}

fn readiness_meets(current: &ReadinessAssessment, required: &ReadinessAssessment) -> bool {
    let order = vec![
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

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
