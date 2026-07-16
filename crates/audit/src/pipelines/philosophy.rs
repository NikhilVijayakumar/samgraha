use crate::pipeline::{finding, make_report, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;

pub struct PhilosophyPipeline;

impl Pipeline for PhilosophyPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Philosophy
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let phil_dir = ctx.project_root.join("docs").join("raw").join("philosophy");
        let docs: Vec<std::path::PathBuf> = if phil_dir.exists() {
            fs::read_dir(&phil_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|ext| ext == "md").unwrap_or(false))
                .collect()
        } else {
            Vec::new()
        };
        let doc_count = docs.len();

        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();

        // ── Philosophy Content (P1-P5) 50% ──────────────────────────────

        let mut pc_passed = 0u32;
        let mut pc_total = 0u32;

        // P1: Philosophy Document Exists
        pc_total += 1;
        if doc_count == 0 {
            findings.push(finding(
                "P1", Severity::Error,
                "No philosophy documents found — philosophy domain requires at least one document".into(),
                None,
            ));
        } else {
            pc_passed += 1;
        }

        // P2: Guiding Principles Section
        pc_total += 1;
        if low.contains("guiding principles") || low.contains("principles") || low.contains("core principles") {
            pc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P2", Severity::Error,
                "Guiding Principles section not found — core principles should be documented".into(),
                None,
            ));
        }

        // P3: Values Section
        pc_total += 1;
        if low.contains("values") || low.contains("core values") || low.contains("our values") {
            pc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P3", Severity::Error,
                "Values section not found — organizational values should be documented".into(),
                None,
            ));
        }

        // P4: Trade-offs Section
        pc_total += 1;
        if low.contains("trade-offs") || low.contains("tradeoffs") || low.contains("trade offs") {
            pc_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P4", Severity::Error,
                "Trade-offs section not found — documented trade-offs are essential for philosophy".into(),
                None,
            ));
        }

        // P5: No Implementation Technology References
        pc_total += 1;
        let impl_terms = ["react", "vue", "angular", "django", "fastapi", "postgresql", "mongodb", "kubernetes", "docker"];
        let has_impl = impl_terms.iter().any(|t| low.contains(t));
        if !has_impl || doc_count == 0 {
            pc_passed += 1;
        } else {
            findings.push(finding(
                "P5", Severity::Warning,
                "Philosophy document references implementation technology — philosophy should be technology-independent".into(),
                None,
            ));
        }

        let pc_score = if pc_total > 0 { (pc_passed as f64 / pc_total as f64) * 100.0 } else { 100.0 };
        cat_scores.insert("philosophy_content".into(), pc_score);

        // ── Philosophy Structure (P6-P8) 30% ────────────────────────────

        let mut ps_passed = 0u32;
        let mut ps_total = 0u32;

        // P6: Document Has Required Sections
        ps_total += 1;
        let has_principles = low.contains("guiding principles") || low.contains("principles");
        let has_values = low.contains("values");
        let has_tradeoffs = low.contains("trade-offs") || low.contains("tradeoffs");
        if has_principles && has_values && has_tradeoffs {
            ps_passed += 1;
        } else if doc_count > 0 {
            let mut missing = Vec::new();
            if !has_principles { missing.push("Guiding Principles"); }
            if !has_values { missing.push("Values"); }
            if !has_tradeoffs { missing.push("Trade-offs"); }
            findings.push(finding(
                "P6", Severity::Error,
                format!("Missing required sections: {}", missing.join(", ")),
                None,
            ));
        }

        // P7: Sections Have Substantive Content
        ps_total += 1;
        let word_count = all_text.split_whitespace().count();
        if word_count >= 200 || doc_count == 0 {
            ps_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P7", Severity::Warning,
                format!("Philosophy content is thin ({} words) — each section should have substantive content", word_count),
                None,
            ));
        }

        // P8: Cross-references Between Sections
        ps_total += 1;
        let section_count = ["guiding principles", "values", "trade-offs", "tradeoffs"]
            .iter()
            .filter(|s| low.contains(*s))
            .count();
        if section_count >= 2 || doc_count == 0 {
            ps_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "P8", Severity::Suggestion,
                "Philosophy sections should cross-reference each other for coherence".into(),
                None,
            ));
        }

        let ps_score = if ps_total > 0 { (ps_passed as f64 / ps_total as f64) * 100.0 } else { 100.0 };
        cat_scores.insert("philosophy_structure".into(), ps_score);

        // ── Philosophy Completeness (P9-P10) 20% ────────────────────────

        let mut pk_passed = 0u32;
        let mut pk_total = 0u32;

        // P9: Purpose Documented
        pk_total += 1;
        if low.contains("purpose") || low.contains("why") || doc_count == 0 {
            pk_passed += 1;
        } else {
            findings.push(finding(
                "P9", Severity::Suggestion,
                "Purpose section not found — documenting why the philosophy exists adds context".into(),
                None,
            ));
        }

        // P10: Traceability to Other Domains
        pk_total += 1;
        let domain_refs = ["vision", "architecture", "engineering", "feature"]
            .iter()
            .filter(|d| low.contains(*d))
            .count();
        if domain_refs >= 1 || doc_count == 0 {
            pk_passed += 1;
        } else {
            findings.push(finding(
                "P10", Severity::Suggestion,
                "Philosophy should reference at least one other documentation domain for traceability".into(),
                None,
            ));
        }

        let pk_score = if pk_total > 0 { (pk_passed as f64 / pk_total as f64) * 100.0 } else { 100.0 };
        cat_scores.insert("philosophy_completeness".into(), pk_score);

        // ── Overall Score ───────────────────────────────────────────────

        let weights = [
            ("philosophy_content", 0.50),
            ("philosophy_structure", 0.30),
            ("philosophy_completeness", 0.20),
        ];
        let overall: f64 = weights.iter()
            .map(|(cat, w)| cat_scores.get(*cat).unwrap_or(&100.0) * w)
            .sum();

        make_report(PipelineKind::Philosophy, overall, cat_scores, findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn make_ctx(root: std::path::PathBuf) -> PipelineContext {
        PipelineContext::new(root, common::config::SamgrahaConfig::default())
    }

    fn temp_root(label: &str) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("samgraha-philosophy-test-{}-{}", std::process::id(), id));
        std::env::temp_dir().join(format!("samgraha-philosophy-{}-{}", label, id))
    }

    #[test]
    fn p1_fires_when_no_docs() {
        let root = temp_root("empty");
        std::fs::create_dir_all(&root).unwrap();
        let report = PhilosophyPipeline.run(&make_ctx(root));
        assert!(report.findings.iter().any(|f| f.check_id == "P1"));
    }

    #[test]
    fn full_philosophy_passes() {
        let root = temp_root("full");
        let phil_dir = root.join("docs").join("raw").join("philosophy");
        std::fs::create_dir_all(&phil_dir).unwrap();
        std::fs::write(
            phil_dir.join("philosophy.md"),
            "# Philosophy\n\n## Guiding Principles\n\nWe value clarity.\n\n## Values\n\nInnovation, integrity.\n\n## Trade-offs\n\nSpeed vs quality.\n\n## Purpose\n\nWhy we exist.",
        ).unwrap();

        let report = PhilosophyPipeline.run(&make_ctx(root));
        assert!(report.score >= 80.0, "Expected score >= 80, got {}", report.score);
        assert!(!report.findings.iter().any(|f| f.severity == Severity::Error));
    }

    #[test]
    fn score_bounds() {
        let root = temp_root("bounds");
        std::fs::create_dir_all(&root).unwrap();
        let report = PhilosophyPipeline.run(&make_ctx(root));
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
