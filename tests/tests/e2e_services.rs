mod fixtures;

use providers::traits::EnrichmentProvider;
use schemas::audit::{AuditScore, QualityGate, ReadinessAssessment};
use std::collections::HashMap;

#[test]
fn test_provider_integration() {
    let provider = providers::RuleBasedProvider::new();
    let doc = fixtures::sample_document(1, "architecture", "Test", "# Test\n\nContent here");
    let summary = provider.summarize(&doc).unwrap();
    assert!(!summary.summary.is_empty());
}

#[test]
fn test_quality_gate_passes() {
    let score = AuditScore {
        overall: 95.0,
        categories: HashMap::new(),
        documents_checked: 10,
        documents_passed: 10,
        findings_count: 0,
    };
    let report = schemas::audit::AuditReport {
        id: "test".into(),
        domain: None,
        timestamp: "now".into(),
        provider: "test".into(),
        score,
        findings: vec![],
        readiness: ReadinessAssessment::Production,
        metadata: HashMap::new(),
    };
    let gate = QualityGate {
        enabled: true,
        min_score: Some(90.0),
        min_readiness: Some(ReadinessAssessment::Implementation),
        required_domains: vec![],
    };
    assert!(audit::AuditFramework::check_quality_gate(&report, &gate).unwrap());
}
