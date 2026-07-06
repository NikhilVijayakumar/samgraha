use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct FeatureTechnicalPipeline;

impl Pipeline for FeatureTechnicalPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::FeatureTechnical
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let ft_dir = ctx.project_root.join("docs").join("raw").join("feature-technical");
        let docs = if ft_dir.exists() {
            scan_markdown_files(&ft_dir)
        } else {
            Vec::new()
        };
        let doc_count = docs.len();

        let per_doc_headings: Vec<HashSet<String>> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| extract_headings(&strip_code_fences(&c)))
            .collect();
        let found_headings: HashSet<String> = per_doc_headings.iter().flatten().cloned().collect();
        let has_heading = |names: &[&str]| {
            found_headings.iter().any(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
        };

        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();

        // ── Feature Mapping (FT1-FT5) 20% ───────────────────────────────

        let mut fm_passed = 0u32;
        let mut fm_total = 0u32;

        // FT1: One-to-One Mapping — duplicate title check
        fm_total += 1;
        if doc_count == 0 {
            fm_passed += 1;
        } else {
            let mut titles_seen: HashSet<String> = HashSet::new();
            let mut dupes_found = false;
            for p in &docs {
                if let Some(title) = fs::read_to_string(p).ok()
                    .and_then(|c| c.lines().next().map(|l| l.trim_start_matches("# ").to_lowercase()))
                {
                    if !titles_seen.insert(title) {
                        dupes_found = true;
                    }
                }
            }
            if !dupes_found {
                fm_passed += 1;
            } else {
                findings.push(finding(
                    "FT1", Severity::Error,
                    "Duplicate Feature Technical Design titles detected — each document must map to exactly one Feature Specification".into(),
                    None,
                ));
            }
        }

        // FT2: Feature Coverage Complete — compare filenames against
        // docs/raw/feature/
        fm_total += 1;
        let feature_dir = ctx.project_root.join("docs").join("raw").join("feature");
        if feature_dir.exists() {
            let feature_names: HashSet<String> = scan_markdown_files(&feature_dir).iter()
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .collect();
            let ft_names: HashSet<String> = docs.iter()
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
                .collect();
            let missing: Vec<&String> = feature_names.difference(&ft_names).collect();
            if missing.is_empty() {
                fm_passed += 1;
            } else {
                findings.push(finding(
                    "FT2", Severity::Warning,
                    format!("{} Feature Specification(s) have no corresponding Feature Technical Design document: {}", missing.len(), missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                    None,
                ));
            }
        } else {
            fm_passed += 1;
        }

        // FT3: Architecture Applied
        fm_total += 1;
        if doc_count == 0 || low.contains("architecture") {
            fm_passed += 1;
        } else {
            findings.push(finding(
                "FT3", Severity::Warning,
                "No reference to Architecture found — Feature Technical Design should apply shared Architecture principles".into(),
                None,
            ));
        }

        // FT4: Feature Design Consulted Where Applicable — stub, requires
        // semantic judgment of whether UX decisions influence this feature.
        fm_total += 1;
        fm_passed += 1;
        findings.push(finding(
            "FT4", Severity::Suggestion,
            "Feature Design consultation is judged per-feature — verify manually whether UX decisions influence this feature's architectural realization".into(),
            None,
        ));

        // FT5: Relevant External Context Applied
        fm_total += 1;
        if doc_count == 0 || low.contains("external context") {
            fm_passed += 1;
        } else {
            fm_passed += 1;
            findings.push(finding(
                "FT5", Severity::Suggestion,
                "No reference to External Context found — reference relevant external dependencies if any influence this feature's realization".into(),
                None,
            ));
        }

        // ── Technical Realization (FT6-FT9) 40% ─────────────────────────

        let mut tr_passed = 0u32;
        let mut tr_total = 0u32;

        // FT6: Component Responsibilities Defined
        tr_total += 1;
        if doc_count == 0 || has_heading(&["Participating Components", "Components", "Involved Components", "Component Responsibilities", "Responsibilities", "Component Roles"]) {
            tr_passed += 1;
        } else {
            findings.push(finding(
                "FT6", Severity::Error,
                "No Participating Components or Component Responsibilities section found — required by the Feature Technical Standard".into(),
                None,
            ));
        }

        // FT7: Communication Flow Complete
        tr_total += 1;
        if doc_count == 0 || has_heading(&["Component Interactions", "Interactions", "Communication Flows", "Communication Paths", "Communication", "Message Flows"]) {
            tr_passed += 1;
        } else {
            findings.push(finding(
                "FT7", Severity::Error,
                "No Component Interactions or Communication Paths section found — required by the Feature Technical Standard".into(),
                None,
            ));
        }

        // FT8: Runtime and Architectural Boundaries Respected
        tr_total += 1;
        if doc_count == 0 || has_heading(&["Runtime Constraints", "Operational Constraints", "Architectural Constraints", "Architecture Constraints", "Runtime Behavior", "Behavior", "Execution Model"]) {
            tr_passed += 1;
        } else {
            tr_passed += 1;
            findings.push(finding(
                "FT8", Severity::Suggestion,
                "No Runtime Constraints, Architectural Constraints, or Runtime Behavior section found — document boundaries if the feature has runtime or architectural limits".into(),
                None,
            ));
        }

        // FT9: External Constraints Reflected
        tr_total += 1;
        if doc_count == 0 || has_heading(&["External Dependency Integration", "External Dependencies", "External Systems", "Integration Points", "Integration", "External Integration"]) {
            tr_passed += 1;
        } else {
            tr_passed += 1;
            findings.push(finding(
                "FT9", Severity::Suggestion,
                "No External Dependency Integration or Integration Points section found — document if external systems constrain this feature".into(),
                None,
            ));
        }

        // ── Documentation Quality (FT10-FT13) 20% ───────────────────────

        let mut dq_passed = 0u32;
        let mut dq_total = 0u32;

        // FT10: Technology References Remain Architectural
        dq_total += 1;
        let impl_tech_keywords = ["react hooks", "axios", "sql query", "typescript interface", "rust trait"];
        let impl_tech_found = find_unnegated_keywords(&low, &impl_tech_keywords);
        if impl_tech_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "FT10", Severity::Warning,
                format!("Implementation-specific technology references found ({}) — Feature Technical Design should reference only architecturally significant technology", impl_tech_found.join(", ")),
                None,
            ));
        }

        // FT11: No Implementation Leakage
        dq_total += 1;
        let leakage_keywords = ["```", "fn(", "class ", "select * from"];
        let leakage_found = find_unnegated_keywords(&low, &leakage_keywords);
        if leakage_found.is_empty() {
            dq_passed += 1;
        } else {
            findings.push(finding(
                "FT11", Severity::Warning,
                "Source code or algorithm content detected — Feature Technical Design describes architectural realization, not implementation".into(),
                None,
            ));
        }

        // FT12: References Rather Than Duplication — stub, requires
        // semantic comparison against Architecture/External Context text.
        dq_total += 1;
        dq_passed += 1;
        findings.push(finding(
            "FT12", Severity::Suggestion,
            "Duplication-vs-reference detection requires semantic comparison against Architecture and External Context documents — not yet implemented".into(),
            None,
        ));

        // FT13: Architectural Consistency — shared terminology check, same
        // pattern as other domains' collection-consistency checks.
        dq_total += 1;
        if doc_count >= 2 {
            let all_words: Vec<HashSet<String>> = docs.iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .map(|c| {
                    c.split_whitespace()
                        .filter(|w| w.len() > 3 && w.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-'))
                        .map(|w| w.to_lowercase())
                        .collect()
                })
                .collect();
            if all_words.len() >= 2 {
                let intersection: HashSet<&String> = all_words[0].iter()
                    .filter(|w| all_words[1..].iter().all(|set| set.contains(*w)))
                    .collect();
                if intersection.len() >= 3 {
                    dq_passed += 1;
                } else {
                    findings.push(finding(
                        "FT13", Severity::Warning,
                        "Limited shared terminology across Feature Technical Design documents".into(),
                        None,
                    ));
                }
            } else {
                dq_passed += 1;
            }
        } else {
            dq_passed += 1;
        }

        // ── Implementation Readiness (FT14-FT15) 20% ────────────────────

        let mut ir_passed = 0u32;
        let mut ir_total = 0u32;

        // FT14: Implementation Readiness
        ir_total += 1;
        if doc_count == 0 || (
            has_heading(&["Participating Components", "Components", "Involved Components"])
            && has_heading(&["Component Interactions", "Interactions", "Communication Flows"])
            && has_heading(&["Data Ownership", "Ownership", "Data Responsibilities"])
        ) {
            ir_passed += 1;
        } else {
            findings.push(finding(
                "FT14", Severity::Error,
                "Participating Components, Component Interactions, or Data Ownership missing — Engineering cannot proceed without all three".into(),
                None,
            ));
        }

        // FT15: Future Maintainability
        ir_total += 1;
        if doc_count == 0 || doc_count >= 2 {
            ir_passed += 1;
        } else {
            ir_passed += 1;
            findings.push(finding(
                "FT15", Severity::Suggestion,
                "Only one Feature Technical Design document exists — verify each feature gets its own document as the collection grows".into(),
                None,
            ));
        }

        // ── Category Scores ──────────────────────────────────────────────

        let fm_score = if fm_total > 0 { (fm_passed as f64 / fm_total as f64) * 100.0 } else { 100.0 };
        let tr_score = if tr_total > 0 { (tr_passed as f64 / tr_total as f64) * 100.0 } else { 100.0 };
        let dq_score = if dq_total > 0 { (dq_passed as f64 / dq_total as f64) * 100.0 } else { 100.0 };
        let ir_score = if ir_total > 0 { (ir_passed as f64 / ir_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Feature Mapping".into(), fm_score);
        cat_scores.insert("Technical Realization".into(), tr_score);
        cat_scores.insert("Documentation Quality".into(), dq_score);
        cat_scores.insert("Implementation Readiness".into(), ir_score);

        // Weighted overall: 20/40/20/20
        let overall = fm_score * 0.20 + tr_score * 0.40 + dq_score * 0.20 + ir_score * 0.20;

        let mut report = make_report(PipelineKind::FeatureTechnical, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, tr_score, ir_score));
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

fn readiness_label(overall: f64, tr: f64, ir: f64) -> String {
    if overall >= 90.0 && tr >= 80.0 && ir >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-ft-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_ft_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/feature-technical");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
        }

        fn with_feature_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/feature");
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
    fn ft6_errors_when_components_missing() {
        let proj = TempProject::new()
            .with_ft_file("auth.md", "# Authentication\n\n## Purpose\n\nRealizes login architecturally.");
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        let ft6 = report.findings.iter().find(|f| f.check_id == "FT6").unwrap();
        assert_eq!(ft6.severity, Severity::Error);
    }

    #[test]
    fn ft6_passes_with_participating_components_heading() {
        let proj = TempProject::new()
            .with_ft_file("auth.md", "# Authentication\n\n## Participating Components\n\nAuthService, SessionStore.");
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "FT6"));
    }

    #[test]
    fn ft2_warns_when_feature_has_no_technical_design() {
        let proj = TempProject::new()
            .with_feature_file("authentication.md", "# Authentication\n\n## Purpose\n\nLog users in.")
            .with_ft_file("localization.md", "# Localization\n\n## Purpose\n\nTranslate strings.");
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        let ft2 = report.findings.iter().find(|f| f.check_id == "FT2").unwrap();
        assert_eq!(ft2.severity, Severity::Warning);
    }

    #[test]
    fn ft2_passes_when_every_feature_has_a_technical_design() {
        let proj = TempProject::new()
            .with_feature_file("authentication.md", "# Authentication\n\n## Purpose\n\nLog users in.")
            .with_ft_file("authentication.md", "# Authentication\n\n## Purpose\n\nRealizes login architecturally.");
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "FT2"));
    }

    #[test]
    fn ft14_errors_when_implementation_readiness_incomplete() {
        let proj = TempProject::new()
            .with_ft_file("auth.md", "# Authentication\n\n## Participating Components\n\nAuthService.");
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        let ft14 = report.findings.iter().find(|f| f.check_id == "FT14").unwrap();
        assert_eq!(ft14.severity, Severity::Error);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = FeatureTechnicalPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
