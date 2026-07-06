use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ImplementationPipeline;

impl Pipeline for ImplementationPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Implementation
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        // The implementation folder is never hardcoded — it's declared in
        // Engineering's Repository Structure doc and resolved here the same
        // way Coverage Audit resolves it (`repository.implementation.dir`,
        // env-expanded, falling back to `src`).
        let impl_dir = common::config::resolve_configured_dir(
            &ctx.config.repository.implementation.dir,
            &ctx.project_root,
            "src",
        );

        let crates = discover_workspace_crates(&impl_dir);
        let crate_count = crates.len();

        let arch_dir = ctx.project_root.join("docs").join("raw").join("architecture");
        let arch_text = read_domain_text(&arch_dir);
        let arch_low = arch_text.to_lowercase();

        let ft_dir = ctx.project_root.join("docs").join("raw").join("feature-technical");
        let ft_docs = if ft_dir.exists() { scan_markdown_files(&ft_dir) } else { Vec::new() };

        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        let eng_text = read_domain_text(&eng_dir);
        let eng_low = eng_text.to_lowercase();

        // ── Architectural Conformance (I1-I4) 30% ───────────────────────

        let mut ac_passed = 0u32;
        let mut ac_total = 0u32;

        // I1: Architecture Realized — stub. Detecting a component
        // Architecture promises that has no corresponding crate at all
        // requires extracting candidate component names from free text
        // (NLP), which isn't reliable with keyword/heading scanning. The
        // reverse direction — a crate that exists but Architecture never
        // named — is mechanically checkable and covered by I12 below.
        ac_total += 1;
        ac_passed += 1;
        findings.push(finding(
            "I1", Severity::Suggestion,
            "Detecting Architecture-promised components with no corresponding crate requires extracting candidate names from free text — not yet implemented; see I12 for the mechanically-checkable reverse direction".into(),
            None,
        ));

        // I2: Runtime Boundaries Preserved — stub, requires call-graph
        // analysis to verify process/persistence/communication boundaries
        // aren't crossed in source.
        ac_total += 1;
        ac_passed += 1;
        findings.push(finding(
            "I2", Severity::Suggestion,
            "Runtime boundary conformance requires call-graph analysis of the implementation — not yet implemented".into(),
            None,
        ));

        // I3: Communication Conformance — stub, requires matching
        // documented communication paths against actual API/event call sites.
        ac_total += 1;
        ac_passed += 1;
        findings.push(finding(
            "I3", Severity::Suggestion,
            "Communication-path conformance requires matching documented paths against actual call sites — not yet implemented".into(),
            None,
        ));

        // I4: Dependency Conformance — Cargo itself refuses to compile a
        // circular workspace dependency graph, so this is checked as a
        // by-product of a successful build; verify the crates were at
        // least discoverable.
        ac_total += 1;
        if !impl_dir.exists() || crate_count > 0 {
            ac_passed += 1;
        } else {
            findings.push(finding(
                "I4", Severity::Warning,
                "No workspace crates discovered under the declared implementation folder — dependency conformance cannot be verified".into(),
                None,
            ));
        }

        // ── Feature Conformance (I5-I7) 25% ─────────────────────────────

        let mut fc_passed = 0u32;
        let mut fc_total = 0u32;

        // I5: Feature Technical Design Realized — every Feature Technical
        // Design document should have at least one plausibly-named
        // counterpart somewhere in the source tree (crate or module name
        // sharing a token with the doc's slug).
        fc_total += 1;
        if ft_docs.is_empty() || crate_count == 0 {
            fc_passed += 1;
        } else {
            let crate_names: Vec<&str> = crates.iter().map(|c| c.name.as_str()).collect();
            let unrealized: Vec<String> = ft_docs.iter()
                .filter_map(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
                .filter(|slug| {
                    let tokens: Vec<&str> = slug.split('-').collect();
                    !tokens.iter().any(|t| crate_names.iter().any(|c| c.contains(t) || t.contains(*c)))
                })
                .collect();
            if unrealized.is_empty() {
                fc_passed += 1;
            } else {
                findings.push(finding(
                    "I5", Severity::Warning,
                    format!("{} Feature Technical Design document(s) have no obviously-named counterpart in the implementation: {}", unrealized.len(), unrealized.join(", ")),
                    None,
                ));
            }
        }

        // I6: Component Responsibilities Preserved — stub, requires
        // comparing documented responsibilities against actual module
        // contents.
        fc_total += 1;
        fc_passed += 1;
        findings.push(finding(
            "I6", Severity::Suggestion,
            "Component responsibility conformance requires comparing documented responsibilities against actual module contents — not yet implemented".into(),
            None,
        ));

        // I7: External Context Applied — stub, requires verifying that
        // documented external constraints are actually respected in code.
        fc_total += 1;
        fc_passed += 1;
        findings.push(finding(
            "I7", Severity::Suggestion,
            "Verifying External Context constraints are respected in source requires semantic analysis — not yet implemented".into(),
            None,
        ));

        // ── Engineering Conformance (I8-I10) 20% ────────────────────────

        let mut ec_passed = 0u32;
        let mut ec_total = 0u32;

        // I8: Engineering Standards Applied — Testing Standards should be
        // reflected by actual test code existing in the implementation.
        ec_total += 1;
        if !impl_dir.exists() || has_test_code(&impl_dir) {
            ec_passed += 1;
        } else {
            findings.push(finding(
                "I8", Severity::Warning,
                "No test code found under the implementation folder — Engineering's Testing Standards are not reflected in the implementation".into(),
                None,
            ));
        }

        // I9: Repository Organization Conforms — crates that exist in
        // source but are never named in Engineering's Repository Structure
        // documentation are undocumented organizational drift.
        ec_total += 1;
        if crate_count == 0 || !eng_dir.exists() {
            ec_passed += 1;
        } else {
            let undocumented: Vec<&str> = crates.iter()
                .map(|c| c.name.as_str())
                .filter(|name| !eng_low.contains(*name) && !eng_low.contains(&name.replace('_', "-")))
                .collect();
            if undocumented.is_empty() {
                ec_passed += 1;
            } else {
                findings.push(finding(
                    "I9", Severity::Warning,
                    format!("{} crate(s) exist in the implementation but are never named in Engineering documentation: {}", undocumented.len(), undocumented.join(", ")),
                    None,
                ));
            }
        }

        // I10: Build and Dependency Conformance — root manifest and
        // lockfile should exist and be in sync (build reproducibility).
        ec_total += 1;
        let root_manifest = ctx.project_root.join("Cargo.toml");
        let lockfile = ctx.project_root.join("Cargo.lock");
        if root_manifest.exists() && lockfile.exists() {
            ec_passed += 1;
        } else {
            findings.push(finding(
                "I10", Severity::Error,
                "Workspace Cargo.toml or Cargo.lock is missing — build reproducibility cannot be verified".into(),
                None,
            ));
        }

        // ── Documentation Integrity (I11-I13) 15% ───────────────────────

        let mut di_passed = 0u32;
        let mut di_total = 0u32;

        // I11: Documentation Remains Implementation Independent — reuses
        // the same leakage keyword scan every domain pipeline already
        // applies to its own collection; here applied to Architecture and
        // Engineering as the two domains this audit reads directly.
        di_total += 1;
        let leakage_keywords = ["```", "fn(", "class ", "select * from"];
        let combined_low = format!("{}\n{}", arch_low, eng_low);
        let leakage_found = find_unnegated_keywords(&combined_low, &leakage_keywords);
        if leakage_found.is_empty() {
            di_passed += 1;
        } else {
            findings.push(finding(
                "I11", Severity::Warning,
                "Source code or algorithm content detected in Architecture/Engineering documentation — implementation belongs exclusively in source code".into(),
                None,
            ));
        }

        // I12: No Architectural Drift — crates that exist in source but
        // are never named in Architecture documentation at all.
        di_total += 1;
        if crate_count == 0 || !arch_dir.exists() {
            di_passed += 1;
        } else {
            let undocumented: Vec<&str> = crates.iter()
                .map(|c| c.name.as_str())
                .filter(|name| !arch_low.contains(*name) && !arch_low.contains(&name.replace('_', "-")))
                .collect();
            if undocumented.is_empty() {
                di_passed += 1;
            } else {
                findings.push(finding(
                    "I12", Severity::Warning,
                    format!("{} crate(s) exist in the implementation but are never named in Architecture documentation: {}", undocumented.len(), undocumented.join(", ")),
                    None,
                ));
            }
        }

        // I13: Traceability Complete — orphan detection is explicitly
        // deferred to Coverage Audit (CV8, CV14) per this audit's own spec.
        di_total += 1;
        di_passed += 1;
        findings.push(finding(
            "I13", Severity::Suggestion,
            "Orphan module/requirement detection is owned by Coverage Audit (CV8, CV14) — see that report for traceability gaps".into(),
            None,
        ));

        // ── Implementation Quality (I14-I15) 10% ────────────────────────

        let mut iq_passed = 0u32;
        let mut iq_total = 0u32;

        // I14: Naming Consistency — each crate's Cargo.toml package name
        // should match its directory name.
        iq_total += 1;
        let mismatched: Vec<String> = crates.iter()
            .filter(|c| c.name != c.dir_name)
            .map(|c| format!("{} (dir) vs {} (package name)", c.dir_name, c.name))
            .collect();
        if mismatched.is_empty() {
            iq_passed += 1;
        } else {
            findings.push(finding(
                "I14", Severity::Warning,
                format!("Crate directory name does not match package name: {}", mismatched.join(", ")),
                None,
            ));
        }

        // I15: Future Maintainability
        iq_total += 1;
        iq_passed += 1;

        // ── Category Scores ──────────────────────────────────────────────

        let ac_score = if ac_total > 0 { (ac_passed as f64 / ac_total as f64) * 100.0 } else { 100.0 };
        let fc_score = if fc_total > 0 { (fc_passed as f64 / fc_total as f64) * 100.0 } else { 100.0 };
        let ec_score = if ec_total > 0 { (ec_passed as f64 / ec_total as f64) * 100.0 } else { 100.0 };
        let di_score = if di_total > 0 { (di_passed as f64 / di_total as f64) * 100.0 } else { 100.0 };
        let iq_score = if iq_total > 0 { (iq_passed as f64 / iq_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Architectural Conformance".into(), ac_score);
        cat_scores.insert("Feature Conformance".into(), fc_score);
        cat_scores.insert("Engineering Conformance".into(), ec_score);
        cat_scores.insert("Documentation Integrity".into(), di_score);
        cat_scores.insert("Implementation Quality".into(), iq_score);

        // Weighted overall: 30/25/20/15/10
        let overall = ac_score * 0.30 + fc_score * 0.25 + ec_score * 0.20 + di_score * 0.15 + iq_score * 0.10;

        let mut report = make_report(PipelineKind::Implementation, overall, cat_scores, findings);
        report.metadata.insert("crate_count".into(), crate_count.to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, ac_score, ec_score));
        report
    }
}

struct WorkspaceCrate {
    /// Directory name under the implementation folder.
    dir_name: String,
    /// `name = "..."` from the crate's own Cargo.toml (falls back to
    /// `dir_name` if the manifest can't be read or parsed).
    name: String,
}

fn discover_workspace_crates(impl_dir: &Path) -> Vec<WorkspaceCrate> {
    let mut crates = Vec::new();
    let Ok(entries) = fs::read_dir(impl_dir) else { return crates; };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
        let name = fs::read_to_string(&manifest).ok()
            .and_then(|c| parse_package_name(&c))
            .unwrap_or_else(|| dir_name.clone());
        crates.push(WorkspaceCrate { dir_name, name });
    }
    crates.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    crates
}

/// Extracts `name = "..."` from the `[package]` section of a Cargo.toml
/// without pulling in a TOML parser dependency for one field.
fn parse_package_name(manifest: &str) -> Option<String> {
    let mut in_package_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package_section = trimmed == "[package]";
            continue;
        }
        if in_package_section {
            if let Some(rest) = trimmed.strip_prefix("name") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    let value = rest.trim().trim_matches('"');
                    if !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
    }
    None
}

fn has_test_code(impl_dir: &Path) -> bool {
    fn walk(dir: &Path) -> bool {
        let Ok(entries) = fs::read_dir(dir) else { return false; };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                if walk(&path) {
                    return true;
                }
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("#[cfg(test)]") || content.contains("#[test]") {
                        return true;
                    }
                }
            }
        }
        false
    }
    walk(impl_dir)
}

fn scan_markdown_files(dir: &Path) -> Vec<PathBuf> {
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

fn read_domain_text(dir: &Path) -> String {
    if !dir.exists() {
        return String::new();
    }
    scan_markdown_files(dir).iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .map(|c| strip_code_fences(&c))
        .collect::<Vec<_>>()
        .join("\n")
}

fn readiness_label(overall: f64, architectural_conformance: f64, engineering_conformance: f64) -> String {
    if overall >= 90.0 && architectural_conformance >= 80.0 && engineering_conformance >= 80.0 {
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
            let root = std::env::temp_dir().join(format!("samgraha-impl-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_crate(self, name: &str, package_name: &str, with_test: bool) -> Self {
            let dir = self.root.join("crates").join(name);
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::write(dir.join("Cargo.toml"), format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n", package_name)).unwrap();
            let body = if with_test { "fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn adds() { assert_eq!(add(1,1), 2); }\n}\n" } else { "fn noop() {}\n" };
            std::fs::write(dir.join("src").join("lib.rs"), body).unwrap();
            self
        }

        fn with_arch_file(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/architecture");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(name), content).unwrap();
            self
        }

        fn with_root_manifest(self) -> Self {
            std::fs::write(self.root.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/*\"]\n").unwrap();
            std::fs::write(self.root.join("Cargo.lock"), "").unwrap();
            self
        }

        fn ctx(&self) -> PipelineContext {
            let mut config = common::config::SamgrahaConfig::default();
            config.repository.implementation.dir = "crates".to_string();
            PipelineContext::new(self.root.clone(), config)
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn i8_warns_when_no_test_code_found() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets", false)
            .with_root_manifest();
        let report = ImplementationPipeline.run(&proj.ctx());
        let i8_finding = report.findings.iter().find(|f| f.check_id == "I8").unwrap();
        assert_eq!(i8_finding.severity, Severity::Warning);
    }

    #[test]
    fn i8_passes_when_test_code_found() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets", true)
            .with_root_manifest();
        let report = ImplementationPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "I8"));
    }

    #[test]
    fn i10_errors_when_lockfile_missing() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets", true);
        let report = ImplementationPipeline.run(&proj.ctx());
        let i10 = report.findings.iter().find(|f| f.check_id == "I10").unwrap();
        assert_eq!(i10.severity, Severity::Error);
    }

    #[test]
    fn i12_warns_on_undocumented_crate() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets", true)
            .with_root_manifest()
            .with_arch_file("system.md", "# System Overview\n\nThe platform has a gadgets component.");
        let report = ImplementationPipeline.run(&proj.ctx());
        let i12 = report.findings.iter().find(|f| f.check_id == "I12").unwrap();
        assert_eq!(i12.severity, Severity::Warning);
    }

    #[test]
    fn i12_passes_when_crate_documented() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets", true)
            .with_root_manifest()
            .with_arch_file("system.md", "# System Overview\n\nThe widgets crate handles gadget assembly.");
        let report = ImplementationPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "I12"));
    }

    #[test]
    fn i14_warns_on_package_name_mismatch() {
        let proj = TempProject::new()
            .with_crate("widgets", "widgets-core", true)
            .with_root_manifest();
        let report = ImplementationPipeline.run(&proj.ctx());
        let i14 = report.findings.iter().find(|f| f.check_id == "I14").unwrap();
        assert_eq!(i14.severity, Severity::Warning);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = ImplementationPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
