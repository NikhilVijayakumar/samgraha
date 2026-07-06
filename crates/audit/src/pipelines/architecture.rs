use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct ArchitecturePipeline;

impl Pipeline for ArchitecturePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Architecture
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let arch_dir = ctx.project_root.join("docs").join("raw").join("architecture");

        // Scan architecture documents
        let docs = if arch_dir.exists() {
            scan_markdown_files(&arch_dir)
        } else {
            Vec::new()
        };

        let doc_count = docs.len();

        // ── Collection Integrity (A1–A4) 25% ────────────────────────────

        let mut ci_passed = 0u32;
        let mut ci_total = 0u32;

        // A1: Modular Architecture — >1 file or well-structured single doc
        ci_total += 1;
        if doc_count >= 2 {
            ci_passed += 1;
        } else if doc_count == 1 {
            // Single doc: check it has multiple H2 sections indicating modular structure
            let content = fs::read_to_string(&docs[0]).unwrap_or_default();
            let h2_count = content.matches("\n## ").count();
            if h2_count >= 3 {
                ci_passed += 1;
                findings.push(finding(
                    "A1", Severity::Suggestion,
                    format!(
                        "Single architecture document with {} sections — consider decomposing into separate files for each architectural concern",
                        h2_count
                    ),
                    Some(docs[0].to_string_lossy().to_string()),
                ));
            } else {
                findings.push(finding(
                    "A1", Severity::Warning,
                    "Architecture is not modular. Single document with fewer than 3 sections does not demonstrate architectural decomposition".into(),
                    Some(docs[0].to_string_lossy().to_string()),
                ));
            }
        } else {
            findings.push(finding(
                "A1", Severity::Error,
                "No architecture documents found. Architecture must be documented as a collection of focused documents.".into(),
                None,
            ));
        }

        // A2: Architectural Completeness — check for required concerns
        ci_total += 1;
        let required_concerns = [
            "System Overview", "Overview", "Architecture Overview",
            "Component Model", "Components", "Component Architecture",
            "Communication", "Communication Paths",
            "Data Flow", "Data Movement", "Information Flow",
            "Security", "Security Architecture", "Security Considerations",
        ];
        let found_concerns: HashSet<String> = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .flat_map(|content| extract_headings(&content))
            .collect();

        let missing_concerns: Vec<&str> = required_concerns.iter()
            .copied()
            .filter(|c| !found_concerns.iter().any(|f| f.eq_ignore_ascii_case(c)))
            .collect();

        let completeness_score = if missing_concerns.is_empty() {
            ci_passed += 1;
            100.0
        } else if missing_concerns.len() <= 2 {
            ci_passed += 1; // partial pass
            let missing_list = missing_concerns.join(", ");
            findings.push(finding(
                "A2", Severity::Warning,
                format!("Architecture completeness: missing {} — consider adding these concerns", missing_list),
                None,
            ));
            70.0
        } else {
            let missing_list = missing_concerns.join(", ");
            findings.push(finding(
                "A2", Severity::Error,
                format!("Architecture incomplete: missing {} — these are required architectural concerns", missing_list),
                None,
            ));
            30.0
        };

        // A3: Responsibility Separation — each doc should have a clear H1
        ci_total += 1;
        let docs_with_h1: Vec<_> = docs.iter()
            .filter(|p| {
                fs::read_to_string(p).ok()
                    .map(|c| c.starts_with("# "))
                    .unwrap_or(false)
            })
            .collect();
        if docs_with_h1.len() == doc_count && doc_count > 0 {
            ci_passed += 1;
        } else if doc_count > 0 {
            let ratio = docs_with_h1.len() as f64 / doc_count as f64;
            if ratio >= 0.5 {
                ci_passed += 1;
                findings.push(finding(
                    "A3", Severity::Warning,
                    format!("{}/{} architecture documents lack a clear title — each document should describe exactly one architectural concern", doc_count - docs_with_h1.len(), doc_count),
                    None,
                ));
            } else {
                findings.push(finding(
                    "A3", Severity::Error,
                    format!("{}/{} architecture documents lack a clear title — responsibilities are not clearly separated", doc_count - docs_with_h1.len(), doc_count),
                    None,
                ));
            }
        }

        // A4: No Duplication — check for duplicate H1 titles across files
        ci_total += 1;
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
        if !dupes_found && doc_count > 0 {
            ci_passed += 1;
        } else if dupes_found {
            findings.push(finding(
                "A4", Severity::Error,
                "Duplicate architectural concerns detected — multiple documents share the same title. Each concern must be documented once.".into(),
                None,
            ));
        }

        // ── Structural Integrity (A5–A8) 30% ────────────────────────────

        let mut si_passed = 0u32;
        let mut si_total = 0u32;

        // A5: Ownership Explicit — check for ownership, responsibility keywords
        si_total += 1;
        let all_text: String = docs.iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .map(|c| strip_code_fences(&c))
            .collect::<Vec<_>>()
            .join("\n");
        let low = all_text.to_lowercase();
        let has_ownership = low.contains("ownership") || low.contains("owns") || low.contains("responsible") || low.contains("responsibility");
        let has_boundary = low.contains("boundary") || low.contains("scope") || low.contains("boundaries");
        if has_ownership && has_boundary {
            si_passed += 1;
        } else if has_ownership || has_boundary {
            findings.push(finding(
                "A5", Severity::Warning,
                "Ownership is partially defined — each architectural component should clearly define ownership, responsibilities, and boundaries".into(),
                None,
            ));
        } else {
            findings.push(finding(
                "A5", Severity::Error,
                "Ownership not explicitly defined — every architectural component must clearly define ownership, responsibilities, and boundaries".into(),
                None,
            ));
        }

        // A6: Boundaries Explicit — check for boundary keywords
        si_total += 1;
        if low.contains("boundary") || low.contains("boundaries") || low.contains("interface") {
            si_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "A6", Severity::Error,
                "Architectural boundaries not documented — component, runtime, deployment, communication, and persistence boundaries must be explicit".into(),
                None,
            ));
        } else {
            si_passed += 1; // no docs = no boundaries to check
        }

        // A7: Architectural Relationships — check for dependency/interaction keywords
        si_total += 1;
        let has_relations = low.contains("dependencies") || low.contains("depends") || low.contains("interact") || low.contains("relationship") || low.contains("depends on");
        if has_relations {
            si_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "A7", Severity::Warning,
                "Architectural relationships between components are not explicitly documented — consider documenting dependencies, interactions, and delegations".into(),
                None,
            ));
        } else {
            si_passed += 1;
        }

        // A8: Communication & Knowledge Flow
        si_total += 1;
        let has_comm = low.contains("communication") || low.contains("communicates") || low.contains("knowledge flow") || low.contains("message") || low.contains("protocol");
        if has_comm {
            si_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "A8", Severity::Warning,
                "Communication and knowledge flow between components is not documented — communication paths, responsibility flow, and knowledge movement should be understandable".into(),
                None,
            ));
        } else {
            si_passed += 1;
        }

        // ── Consistency (A9–A12) 30% ─────────────────────────────────────

        let mut cy_passed = 0u32;
        let mut cy_total = 0u32;

        // A9: Architectural Consistency — check across files for consistent terminology
        cy_total += 1;
        if doc_count >= 2 {
            // Check that all files share some common terms > 20 chars (likely proper nouns/component names)
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
                    cy_passed += 1;
                } else {
                    findings.push(finding(
                        "A9", Severity::Warning,
                        "Limited shared terminology across architecture documents — consistent naming improves architectural clarity".into(),
                        None,
                    ));
                }
            } else {
                cy_passed += 1; // single file, trivially consistent
            }
        } else {
            cy_passed += 1; // single file or none, trivially consistent
        }

        // A10: Traceability — check for traceability section or cross-references
        cy_total += 1;
        if found_concerns.iter().any(|h| h.to_lowercase().contains("traceability") || h.to_lowercase().contains("traces to") || h.to_lowercase().contains("derived from")) {
            cy_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "A10", Severity::Warning,
                "Architecture documents lack traceability — each document should participate in the traceability chain (derives from higher-level docs, provides context for lower-level)".into(),
                None,
            ));
        } else {
            cy_passed += 1;
        }

        // A11: Technology Independence — check for implementation keywords.
        // "class "/"fn " dropped: too short, false-positive on ordinary prose
        // ("this class of systems"). Negation-aware so "should remain
        // independent of React" doesn't count as a React reference.
        cy_total += 1;
        let tech_keywords = ["react", "axios", "sqlite", "javascript", "typescript", "python", "rust", "function(", "impl ", "fn(", "api endpoint", "http route", "sql query", "algorithm", "implementation"];
        let tech_found = find_unnegated_keywords(&low, &tech_keywords);
        if tech_found.is_empty() {
            cy_passed += 1;
        } else if tech_found.len() <= 2 {
            findings.push(finding(
                "A11", Severity::Suggestion,
                format!("Architecture contains minor implementation references ({}) — architecture should remain technology-independent", tech_found.join(", ")),
                None,
            ));
            cy_passed += 1;
        } else {
            findings.push(finding(
                "A11", Severity::Warning,
                format!("Architecture depends on implementation detail (keywords: {}) — architecture must describe responsibilities, not frameworks or code", tech_found.join(", ")),
                None,
            ));
        }

        // A12: Feature Independence — check for feature-specific language
        cy_total += 1;
        let feature_phrases = ["user story", "acceptance criteria", "feature requirement", "use case", "customer", "user clicks", "button", "ui ", "screen"];
        let feature_found = find_unnegated_keywords(&low, &feature_phrases);
        if feature_found.is_empty() {
            cy_passed += 1;
        } else {
            findings.push(finding(
                "A12", Severity::Suggestion,
                format!("Architecture references feature-specific concepts ({}) — architecture should remain feature-independent and focus on structural foundations", feature_found.join(", ")),
                None,
            ));
            cy_passed += 1; // partial: suggestion, not blocker
        }

        // ── Cross-Repository (A13) 15% ──────────────────────────────────

        let mut cr_passed = 0u32;
        let mut cr_total = 0u32;

        // A13: Cross-Repository References
        cr_total += 1;
        let has_external_ref = low.contains("external") || low.contains("repository") || low.contains("dependency") || low.contains("cross-repo") || low.contains("cross repo");
        if has_external_ref {
            cr_passed += 1;
        } else if doc_count > 0 {
            findings.push(finding(
                "A13", Severity::Suggestion,
                "Cross-repository architecture references not identified — if this repository depends on others, external architectural dependencies should be referenced rather than duplicated".into(),
                None,
            ));
        } else {
            cr_passed += 1;
        }

        // ── Category Scores ──────────────────────────────────────────────

        let ci_score = if ci_total > 0 { (ci_passed as f64 / ci_total as f64) * 100.0 } else { 100.0 };
        let si_score = if si_total > 0 { (si_passed as f64 / si_total as f64) * 100.0 } else { 100.0 };
        let cy_score = if cy_total > 0 { (cy_passed as f64 / cy_total as f64) * 100.0 } else { 100.0 };
        let cr_score = if cr_total > 0 { (cr_passed as f64 / cr_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Collection Integrity".into(), ci_score);
        cat_scores.insert("Structural Integrity".into(), si_score);
        cat_scores.insert("Consistency".into(), cy_score);
        cat_scores.insert("Cross-Repository Architecture".into(), cr_score);

        // Weighted overall: 25/30/30/15
        let overall = ci_score * 0.25 + si_score * 0.30 + cy_score * 0.30 + cr_score * 0.15;

        // Add completeness_score as metadata
        let mut report = make_report(PipelineKind::Architecture, overall, cat_scores, findings);
        report.metadata.insert("completeness_score".into(), format!("{:.1}", completeness_score));
        report.metadata.insert("doc_count".into(), doc_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ci_score, si_score));
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
        .filter(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .chain(
            content.lines()
                .filter(|l| l.starts_with("## "))
                .map(|l| l.trim_start_matches("## ").trim().to_string())
        )
        .collect()
}

fn readiness_label(overall: f64, ci: f64, si: f64) -> String {
    if overall >= 90.0 && ci >= 80.0 && si >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-arch-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_arch_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/architecture");
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
    fn a1_errors_when_no_arch_docs() {
        let proj = TempProject::new();
        let report = ArchitecturePipeline.run(&proj.ctx());
        let a1 = report.findings.iter().find(|f| f.check_id == "A1").unwrap();
        assert_eq!(a1.severity, Severity::Error);
    }

    #[test]
    fn a1_passes_with_multiple_docs() {
        let proj = TempProject::new()
            .with_arch_file("overview.md", "# System Overview\n\nContent")
            .with_arch_file("components.md", "# Component Model\n\nContent");
        let report = ArchitecturePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "A1"));
    }

    #[test]
    fn a4_errors_on_duplicate_titles() {
        let proj = TempProject::new()
            .with_arch_file("a.md", "# Same Title\n\nContent")
            .with_arch_file("b.md", "# Same Title\n\nDifferent content");
        let report = ArchitecturePipeline.run(&proj.ctx());
        let a4 = report.findings.iter().find(|f| f.check_id == "A4").unwrap();
        assert_eq!(a4.severity, Severity::Error);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ArchitecturePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }

    #[test]
    fn a11_detects_tech_leakage() {
        let proj = TempProject::new()
            .with_arch_file("overview.md", "# System Overview\n\nUses React and TypeScript. The API is implemented in Rust.");
        let report = ArchitecturePipeline.run(&proj.ctx());
        let a11 = report.findings.iter().find(|f| f.check_id == "A11").unwrap();
        assert_eq!(a11.severity, Severity::Warning);
    }
}
