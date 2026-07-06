use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct PrototypePipeline;

impl Pipeline for PrototypePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Prototype
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let prototype_dir = ctx.project_root.join("docs").join("raw").join("prototype");
        let docs = if prototype_dir.exists() {
            scan_markdown_files(&prototype_dir)
        } else {
            Vec::new()
        };
        let doc_count = docs.len();

        let found_headings: HashSet<String> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .flat_map(|content| extract_headings(&content))
            .collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();

        let has_feature_docs = ctx.project_root.join("docs/raw/feature").exists();
        let has_feature_design_docs = ctx.project_root.join("docs/raw/feature-design").exists();
        let has_feature_technical_docs = ctx.project_root.join("docs/raw/feature-technical").exists();
        let has_external_context_docs = ctx.project_root.join("docs/raw/external-context").exists();

        // ── Product Validation (P1-P4) 30% ──────────────────────────────

        let mut pv_passed = 0u32;
        let mut pv_total = 0u32;

        // P1: Feature Coverage Complete
        pv_total += 1;
        if doc_count == 0 {
            findings.push(finding(
                "P1", Severity::Error,
                "No prototype documents found — feature coverage cannot be verified".into(),
                None,
            ));
        } else if !has_feature_docs {
            pv_passed += 1; // nothing to cover
        } else {
            pv_passed += 1;
            findings.push(finding(
                "P1", Severity::Suggestion,
                "Feature documentation exists — per-feature prototype coverage requires cross-referencing each feature doc, not yet implemented".into(),
                None,
            ));
        }

        // P2: Feature Design Validated
        pv_total += 1;
        if !has_feature_design_docs || doc_count > 0 {
            pv_passed += 1;
        } else {
            findings.push(finding(
                "P2", Severity::Warning,
                "Feature Design documentation exists but no prototype documents found to validate it".into(),
                None,
            ));
        }

        // P3: Feature Technical Design Supported
        pv_total += 1;
        if has_heading(&["Mock APIs", "Mocked APIs", "API Contracts", "Simulated APIs"]) || !has_feature_technical_docs {
            pv_passed += 1;
        } else {
            findings.push(finding(
                "P3", Severity::Warning,
                "No Mock APIs section found — Feature Technical Design's API contracts may not be supported".into(),
                None,
            ));
        }

        // P4: User Workflows Complete
        pv_total += 1;
        if low.contains("workflow") || low.contains("user flow") {
            pv_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P4", Severity::Warning,
                "No user workflows documented — primary, alternative, and failure-scenario workflows should be executable".into(),
                None,
            ));
        }

        // ── Runtime Validation (P5-P8) 30% ──────────────────────────────

        let mut rv_passed = 0u32;
        let mut rv_total = 0u32;

        // P5: Navigation Complete
        rv_total += 1;
        if low.contains("navigation") || low.contains("routing") || low.contains("route") {
            rv_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P5", Severity::Warning,
                "No navigation/routing documented".into(),
                None,
            ));
        }

        // P6: Mock API Contracts
        rv_total += 1;
        if has_heading(&["Mock APIs", "Mocked APIs", "API Contracts", "Simulated APIs"]) {
            rv_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P6", Severity::Error,
                "Mock APIs section missing — required by the Prototype Standard".into(),
                None,
            ));
        }

        // P7: Mock Persistence Consistency
        rv_total += 1;
        if low.contains("persistence") || low.contains("mock data") || low.contains("json document") {
            rv_passed += 1;
        } else {
            rv_passed += 1; // optional capability, absence alone isn't a failure
            findings.push(finding(
                "P7", Severity::Suggestion,
                "No mock persistence documented — document if this prototype simulates data storage".into(),
                None,
            ));
        }

        // P8: External Context Simulated
        rv_total += 1;
        if !has_external_context_docs || low.contains("simulate") || low.contains("mock") {
            rv_passed += 1;
        } else {
            findings.push(finding(
                "P8", Severity::Warning,
                "External Context documentation exists but no simulation of external systems mentioned".into(),
                None,
            ));
        }

        // ── Engineering Validation (P9-P12) 20% ─────────────────────────

        let mut ev_passed = 0u32;
        let mut ev_total = 0u32;

        // P9: Engineering Assumptions Validated — stub, cross-domain semantic
        // comparison against Engineering docs not yet implemented.
        ev_total += 1;
        ev_passed += 1;
        findings.push(finding(
            "P9", Severity::Suggestion,
            "Engineering assumption validation requires cross-referencing Engineering docs — not yet implemented".into(),
            None,
        ));

        // P10: Prototype Isolation
        ev_total += 1;
        let production_keywords = ["production database", "production api", "production credentials", "live production"];
        let production_found = find_unnegated_keywords(&low, &production_keywords);
        if production_found.is_empty() {
            ev_passed += 1;
        } else {
            findings.push(finding(
                "P10", Severity::Error,
                format!("Production references found ({}) — prototype artifacts must remain isolated from production", production_found.join(", ")),
                None,
            ));
        }

        // P11: No Production Dependencies
        ev_total += 1;
        let prod_dep_keywords = ["production infrastructure", "production cloud", "production auth"];
        let prod_dep_found = find_unnegated_keywords(&low, &prod_dep_keywords);
        if prod_dep_found.is_empty() {
            ev_passed += 1;
        } else {
            findings.push(finding(
                "P11", Severity::Error,
                format!("Production dependency references found ({}) — prototype must not depend on production systems", prod_dep_found.join(", ")),
                None,
            ));
        }

        // P12: Disposable Artifacts
        ev_total += 1;
        if low.contains("disposable") || low.contains("regenerat") || low.contains("reproducib") {
            ev_passed += 1;
        } else {
            ev_passed += 1; // absence alone isn't a failure
            findings.push(finding(
                "P12", Severity::Suggestion,
                "No mention of disposability/regeneration — document that prototype artifacts are disposable".into(),
                None,
            ));
        }

        // ── Validation Quality (P13-P15) 20% ────────────────────────────

        let mut vq_passed = 0u32;
        let mut vq_total = 0u32;

        // P13: Documentation Consistency — stub
        vq_total += 1;
        vq_passed += 1;
        findings.push(finding(
            "P13", Severity::Suggestion,
            "Cross-document consistency check requires semantic comparison across Feature/Feature Design/Feature Technical Design — not yet implemented".into(),
            None,
        ));

        // P14: User Experience Fidelity
        vq_total += 1;
        if low.contains("usability") || low.contains("interaction") || low.contains("visual flow") {
            vq_passed += 1;
        } else {
            vq_passed += 1;
            findings.push(finding(
                "P14", Severity::Suggestion,
                "No usability/interaction fidelity notes found".into(),
                None,
            ));
        }

        // P15: Future Maintainability — stub
        vq_total += 1;
        vq_passed += 1;
        findings.push(finding(
            "P15", Severity::Suggestion,
            "Future maintainability assessment requires historical comparison — not yet implemented".into(),
            None,
        ));

        // ── Category Scores ──────────────────────────────────────────────

        let pv_score = if pv_total > 0 { (pv_passed as f64 / pv_total as f64) * 100.0 } else { 100.0 };
        let rv_score = if rv_total > 0 { (rv_passed as f64 / rv_total as f64) * 100.0 } else { 100.0 };
        let ev_score = if ev_total > 0 { (ev_passed as f64 / ev_total as f64) * 100.0 } else { 100.0 };
        let vq_score = if vq_total > 0 { (vq_passed as f64 / vq_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Product Validation".into(), pv_score);
        cat_scores.insert("Runtime Validation".into(), rv_score);
        cat_scores.insert("Engineering Validation".into(), ev_score);
        cat_scores.insert("Validation Quality".into(), vq_score);

        // Weighted overall: 30/30/20/20
        let overall = pv_score * 0.30 + rv_score * 0.30 + ev_score * 0.20 + vq_score * 0.20;

        let mut report = make_report(PipelineKind::Prototype, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, pv_score, rv_score));
        report
    }
}

fn scan_markdown_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "md").unwrap_or(false) {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn extract_headings(content: &str) -> HashSet<String> {
    content.lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

fn readiness_label(overall: f64, pv: f64, rv: f64) -> String {
    if overall >= 90.0 && pv >= 80.0 && rv >= 80.0 {
        "READY".to_string()
    } else if overall >= 70.0 {
        "NEEDS_WORK".to_string()
    } else {
        "NOT_READY".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempProject {
        root: std::path::PathBuf,
    }

    impl TempProject {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!("samgraha-prototype-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_prototype_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/prototype");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
        }

        fn ctx(&self) -> PipelineContext {
            PipelineContext::new(self.root.clone(), common::config::SamgrahaConfig::default())
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn p1_errors_when_no_prototype_docs() {
        let proj = TempProject::new();
        let report = PrototypePipeline.run(&proj.ctx());
        let p1 = report.findings.iter().find(|f| f.check_id == "P1").unwrap();
        assert_eq!(p1.severity, Severity::Error);
    }

    #[test]
    fn p6_errors_when_mock_apis_missing() {
        let proj = TempProject::new()
            .with_prototype_file("runtime.md", "# Runtime\n\nSimulates workflow and navigation.");
        let report = PrototypePipeline.run(&proj.ctx());
        let p6 = report.findings.iter().find(|f| f.check_id == "P6").unwrap();
        assert_eq!(p6.severity, Severity::Error);
    }

    #[test]
    fn p6_passes_with_mock_apis_heading() {
        let proj = TempProject::new()
            .with_prototype_file("api.md", "# Mock APIs\n\nGET /items returns a list.");
        let report = PrototypePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "P6"));
    }

    #[test]
    fn p10_errors_on_production_reference() {
        let proj = TempProject::new()
            .with_prototype_file("api.md", "# Mock APIs\n\nConnects to the production database for realism.");
        let report = PrototypePipeline.run(&proj.ctx());
        let p10 = report.findings.iter().find(|f| f.check_id == "P10").unwrap();
        assert_eq!(p10.severity, Severity::Error);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = PrototypePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
