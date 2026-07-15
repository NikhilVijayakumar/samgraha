use crate::pipeline::{finding, make_report, Pipeline, PipelineContext};
use common::config::RepositoryKind;
use compiler::{DiscoveredKnowledgeSystem, KnowledgeSystemLoader};
use schemas::audit::{AuditFinding, PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};

pub struct KnowledgeSystemPipeline;

impl Pipeline for KnowledgeSystemPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::KnowledgeSystem
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();

        // KS0: only applicable to Knowledge Repositories — see Repository
        // Matrix, docs/crates-refactor-proposal.md §5. A plain Repository has
        // no `system/` tree to check; treat as N/A (full score) rather than
        // an error, matching how other pipelines pass when their target
        // directory simply doesn't apply (e.g. documentation_structure's
        // vision_dir.exists() checks).
        if ctx.config.repository.kind != RepositoryKind::Knowledge {
            findings.push(finding(
                "KS0", Severity::Suggestion,
                "Repository kind is 'repository', not 'knowledge' — Knowledge System \
                 Integrity checks don't apply here. Set `[repository] kind = \"knowledge\"` \
                 if this repo is meant to produce Knowledge Systems.".to_string(),
                None,
            ));
            let mut cat_scores = HashMap::new();
            cat_scores.insert("Applicability".to_string(), 100.0);
            return make_report(PipelineKind::KnowledgeSystem, 100.0, cat_scores, findings);
        }

        let systems_dir = ctx.project_root.join(&ctx.config.knowledge.root);
        let discovered = match KnowledgeSystemLoader::load_systems(&systems_dir) {
            Ok(d) => d,
            Err(e) => {
                findings.push(finding(
                    "KS1", Severity::Error,
                    format!("Failed to parse Knowledge Systems under '{}': {}", systems_dir.display(), e),
                    Some(systems_dir.display().to_string()),
                ));
                return make_report(PipelineKind::KnowledgeSystem, 0.0, HashMap::new(), findings);
            }
        };

        let mut cat_scores = HashMap::new();
        cat_scores.insert("Discovery".to_string(), run_discovery(&mut findings, &systems_dir, &discovered));
        cat_scores.insert("Structure".to_string(), run_structure(&mut findings, &discovered));
        cat_scores.insert("Identity".to_string(), run_identity(&mut findings, &discovered));

        let overall = cat_scores.values().sum::<f64>() / cat_scores.len().max(1) as f64;
        let mut report = make_report(PipelineKind::KnowledgeSystem, overall, cat_scores, findings);
        report.metadata.insert("systems_discovered".to_string(), discovered.len().to_string());
        report
    }
}

// KS1: at least one Knowledge System discovered under `knowledge.root`.
fn run_discovery(
    findings: &mut Vec<AuditFinding>,
    systems_dir: &std::path::Path,
    discovered: &[DiscoveredKnowledgeSystem],
) -> f64 {
    if discovered.is_empty() {
        findings.push(finding(
            "KS1", Severity::Error,
            format!(
                "No Knowledge Systems found under '{}'. Each system must be a subdirectory \
                 containing a system.toml file.",
                systems_dir.display()
            ),
            Some(systems_dir.display().to_string()),
        ));
        0.0
    } else {
        100.0
    }
}

// KS2: no missing recommended subdirectories — surfaces the loader's own
// per-system warnings (standards/, audit/, templates/) as findings instead
// of re-deriving the same check.
fn run_structure(findings: &mut Vec<AuditFinding>, discovered: &[DiscoveredKnowledgeSystem]) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;
    for system in discovered {
        total += 1;
        if system.warnings.is_empty() {
            passed += 1;
        } else {
            for w in &system.warnings {
                findings.push(finding("KS2", Severity::Warning, w.message.clone(), None));
            }
        }
    }
    score(passed, total)
}

// KS3: no duplicate system ids. KS4: every system has a non-empty name.
fn run_identity(findings: &mut Vec<AuditFinding>, discovered: &[DiscoveredKnowledgeSystem]) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    total += 1;
    let mut seen = HashSet::new();
    let dupes: Vec<&str> = discovered.iter()
        .map(|s| s.identity.id.as_str())
        .filter(|id| !seen.insert(*id))
        .collect();
    if dupes.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "KS3", Severity::Error,
            format!("Duplicate Knowledge System id(s) found: {}", dupes.join(", ")),
            None,
        ));
    }

    total += 1;
    let unnamed: Vec<&str> = discovered.iter()
        .filter(|s| s.identity.name.trim().is_empty())
        .map(|s| s.identity.id.as_str())
        .collect();
    if unnamed.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "KS4", Severity::Warning,
            format!("Knowledge System(s) with an empty `name` in system.toml: {}", unnamed.join(", ")),
            None,
        ));
    }

    score(passed, total)
}

fn score(passed: u32, total: u32) -> f64 {
    if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 100.0 }
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
            let root = std::env::temp_dir().join(format!("samgraha-ks-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_system(self, id: &str, name: &str, with_dirs: bool) -> Self {
            let sys_dir = self.root.join("system").join(id);
            std::fs::create_dir_all(&sys_dir).unwrap();
            std::fs::write(
                sys_dir.join("system.toml"),
                format!("id = \"{}\"\nname = \"{}\"\n", id, name),
            ).unwrap();
            if with_dirs {
                for d in ["standards", "audit", "templates"] {
                    std::fs::create_dir_all(sys_dir.join(d)).unwrap();
                }
            }
            self
        }

        fn ctx(&self) -> PipelineContext {
            let mut config = common::config::SamgrahaConfig::default();
            config.repository.kind = RepositoryKind::Knowledge;
            PipelineContext::new(self.root.clone(), config)
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn ks0_passes_with_full_score_when_repository_kind_is_plain() {
        let proj = TempProject::new();
        let mut config = common::config::SamgrahaConfig::default();
        config.repository.kind = RepositoryKind::Repository;
        let ctx = PipelineContext::new(proj.root.clone(), config);
        let report = KnowledgeSystemPipeline.run(&ctx);
        assert_eq!(report.score, 100.0);
        assert!(report.findings.iter().any(|f| f.check_id == "KS0"));
    }

    #[test]
    fn ks1_errors_when_no_systems_discovered() {
        let proj = TempProject::new();
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        let ks1 = report.findings.iter().find(|f| f.check_id == "KS1").unwrap();
        assert_eq!(ks1.severity, Severity::Error);
    }

    #[test]
    fn ks1_passes_when_a_system_is_discovered() {
        let proj = TempProject::new().with_system("dev", "Software Development", true);
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "KS1"));
        assert_eq!(report.metadata.get("systems_discovered").map(String::as_str), Some("1"));
    }

    #[test]
    fn ks2_warns_on_missing_recommended_directories() {
        let proj = TempProject::new().with_system("dev", "Software Development", false);
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        let ks2 = report.findings.iter().find(|f| f.check_id == "KS2").unwrap();
        assert_eq!(ks2.severity, Severity::Warning);
    }

    #[test]
    fn ks2_passes_when_recommended_directories_present() {
        let proj = TempProject::new().with_system("dev", "Software Development", true);
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "KS2"));
    }

    #[test]
    fn ks4_warns_on_empty_name() {
        let proj = TempProject::new().with_system("dev", "", true);
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        let ks4 = report.findings.iter().find(|f| f.check_id == "KS4").unwrap();
        assert_eq!(ks4.severity, Severity::Warning);
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new().with_system("dev", "Software Development", true);
        let report = KnowledgeSystemPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
