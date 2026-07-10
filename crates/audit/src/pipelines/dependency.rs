use crate::pipeline::{find_unnegated_keywords, finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Names dependency-standards.md's "Prohibited Dependency Patterns" section
/// bans outright (async runtimes) — checked against every crate's own
/// `[dependencies]` table, not `[dev-dependencies]`/`[build-dependencies]`.
const PROHIBITED_ASYNC_RUNTIMES: &[&str] = &["tokio", "async-std", "smol"];

pub struct DependencyPipeline;

impl Pipeline for DependencyPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Dependency
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let workspace_deps = parse_workspace_dependencies(ctx);
        let external: Vec<&WorkspaceDep> = workspace_deps.iter().filter(|d| !d.is_path).collect();

        let eng_dir = ctx.project_root.join("docs").join("raw").join("engineering");
        let eng_text_lower = if eng_dir.exists() {
            scan_markdown_files(&eng_dir)
                .iter()
                .filter_map(|p| fs::read_to_string(p).ok())
                .map(|c| strip_code_fences(&c))
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase()
        } else {
            String::new()
        };

        // ── Dependency Justification (D1-D3) 40% ────────────────────────

        let mut just_passed = 0u32;
        let mut just_total = 0u32;

        // D1: Every Dependency Justified — every external workspace
        // dependency must be mentioned somewhere in Engineering docs.
        just_total += 1;
        if external.is_empty() {
            just_passed += 1;
        } else {
            let names: Vec<&str> = external.iter().map(|d| d.name.as_str()).collect();
            let mentioned = find_unnegated_keywords(&eng_text_lower, &names);
            let missing: Vec<&str> = names
                .iter()
                .filter(|n| !mentioned.iter().any(|m| m == *n))
                .copied()
                .collect();
            if missing.is_empty() {
                just_passed += 1;
            } else {
                findings.push(finding(
                    "D1",
                    Severity::Warning,
                    format!(
                        "Dependencies with no rationale found in docs/raw/engineering/: {}",
                        missing.join(", ")
                    ),
                    None,
                ));
            }
        }

        // D2: Every Dependency Documented (owner/purpose/version policy) —
        // stub. Engineering docs document dependencies by category
        // ("Allowed Dependency Categories"), not with per-dependency
        // owner/purpose/version-policy fields — that structure doesn't
        // exist yet to check against.
        just_total += 1;
        just_passed += 1;
        findings.push(finding(
            "D2",
            Severity::Suggestion,
            "docs/raw/engineering/dependency-standards.md documents dependencies by category, not with per-dependency owner/purpose/version-policy fields — structured per-dependency metadata is needed before this can be automated".into(),
            None,
        ));

        // D3: Dependency Ownership Explicit — stub, same reason as D2; no
        // per-dependency owner is tracked anywhere in this project's docs.
        just_total += 1;
        just_passed += 1;
        findings.push(finding(
            "D3",
            Severity::Suggestion,
            "No per-dependency owner is tracked in Engineering docs today — not yet automated".into(),
            None,
        ));

        // ── Version Policy (D4-D5) 25% ───────────────────────────────────

        let mut ver_passed = 0u32;
        let mut ver_total = 0u32;

        // D4: Version Policy Respected — an unconstrained "*" version is
        // flagged as risk directly per this check's own spec example.
        ver_total += 1;
        let wildcard: Vec<&str> = external
            .iter()
            .filter(|d| d.version.as_deref() == Some("*"))
            .map(|d| d.name.as_str())
            .collect();
        if wildcard.is_empty() {
            ver_passed += 1;
        } else {
            findings.push(finding(
                "D4",
                Severity::Warning,
                format!("Unconstrained version (\"*\") — no version policy is possible: {}", wildcard.join(", ")),
                None,
            ));
        }

        // D5: Supply-Chain Policy Applied — any dependency sourced outside
        // the primary registry (git) must have its rationale declared in
        // Engineering docs.
        ver_total += 1;
        let non_registry: Vec<&str> = external.iter().filter(|d| d.is_git).map(|d| d.name.as_str()).collect();
        if non_registry.is_empty() {
            ver_passed += 1;
        } else {
            let declared = find_unnegated_keywords(&eng_text_lower, &non_registry);
            if declared.len() == non_registry.len() {
                ver_passed += 1;
            } else {
                findings.push(finding(
                    "D5",
                    Severity::Warning,
                    format!(
                        "Git-sourced dependencies with no declared rationale in Engineering docs: {}",
                        non_registry.join(", ")
                    ),
                    None,
                ));
            }
        }

        // ── Dependency Health (D6-D7) 25% ────────────────────────────────

        let mut health_passed = 0u32;
        let mut health_total = 0u32;

        // D6: Dependency Health Check — stub. Deprecated/unmaintained/
        // yanked status requires a live crates.io query, which would make
        // this deterministic pipeline's default (network-free) run violate
        // this project's own Offline-First engineering principle — deferred
        // behind an explicit opt-in, same shape the security pipeline
        // already uses for its `runtime: true` live checks.
        health_total += 1;
        health_passed += 1;
        findings.push(finding(
            "D6",
            Severity::Suggestion,
            "Deprecated/unmaintained/yanked status requires live crates.io queries — deferred rather than violating this project's Offline-First engineering principle for the default deterministic run; would need an explicit runtime-mode opt-in".into(),
            None,
        ));

        // D7: Dependency Scope Correct — prohibited async runtimes in any
        // crate's runtime [dependencies], and notify's actual declaration
        // checked against dependency-standards.md's specific claim that
        // it's "a compile-time optional feature."
        health_total += 1;
        let mut scope_issues: Vec<String> = Vec::new();
        for crate_dir in workspace_member_dirs(ctx) {
            let manifest_path = crate_dir.join("Cargo.toml");
            let Ok(text) = fs::read_to_string(&manifest_path) else { continue };
            let Ok(parsed) = text.parse::<toml::Value>() else { continue };
            let crate_name = crate_dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();

            let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) else { continue };

            for name in deps.keys() {
                if PROHIBITED_ASYNC_RUNTIMES.contains(&name.as_str()) {
                    scope_issues.push(format!("{crate_name}: prohibited async runtime '{name}' in [dependencies]"));
                }
            }

            if let Some(notify_value) = deps.get("notify") {
                let declared_optional = notify_value
                    .as_table()
                    .and_then(|t| t.get("optional"))
                    .and_then(|o| o.as_bool())
                    .unwrap_or(false);
                let has_feature_gate = parsed
                    .get("features")
                    .and_then(|f| f.as_table())
                    .map(|t| !t.is_empty())
                    .unwrap_or(false);
                if !declared_optional || !has_feature_gate {
                    scope_issues.push(format!(
                        "{crate_name}: notify is an unconditional [dependencies] entry (optional={declared_optional}, has [features] gate={has_feature_gate}), \
                         but dependency-standards.md claims 'the watcher is a compile-time optional feature'"
                    ));
                }
            }
        }
        if scope_issues.is_empty() {
            health_passed += 1;
        } else {
            findings.push(finding("D7", Severity::Warning, scope_issues.join("; "), None));
        }

        // ── Cross-References (D8) 10% ────────────────────────────────────

        // D8: always-pass informational note — orphan dependency detection
        // is Coverage Audit's (CV12) job, not this audit's, per this
        // check's own spec text.
        let cross_total = 1u32;
        let cross_passed = 1u32;
        findings.push(finding(
            "D8",
            Severity::Suggestion,
            "Orphan dependency detection (documented dependencies missing from the manifest, or vice versa) is Coverage Audit's responsibility (CV12), not Dependency Governance's".into(),
            None,
        ));

        // ── Category Scores ──────────────────────────────────────────────

        let just_score = if just_total > 0 { (just_passed as f64 / just_total as f64) * 100.0 } else { 100.0 };
        let ver_score = if ver_total > 0 { (ver_passed as f64 / ver_total as f64) * 100.0 } else { 100.0 };
        let health_score = if health_total > 0 { (health_passed as f64 / health_total as f64) * 100.0 } else { 100.0 };
        let cross_score = if cross_total > 0 { (cross_passed as f64 / cross_total as f64) * 100.0 } else { 100.0 };

        cat_scores.insert("Dependency Justification".into(), just_score);
        cat_scores.insert("Version Policy".into(), ver_score);
        cat_scores.insert("Dependency Health".into(), health_score);
        cat_scores.insert("Cross-References".into(), cross_score);

        // Weighted overall per docs/raw/audit/dependency-audit.md's Category Weights: 40/25/25/10.
        let overall = just_score * 0.40 + ver_score * 0.25 + health_score * 0.25 + cross_score * 0.10;

        let mut report = make_report(PipelineKind::Dependency, overall, cat_scores, findings);
        report.metadata.insert("external_dependency_count".into(), external.len().to_string());
        report.metadata.insert(
            "engineering_readiness".into(),
            if overall >= 80.0 { "YES".into() } else { "NO".into() },
        );
        report
    }
}

struct WorkspaceDep {
    name: String,
    is_path: bool,
    is_git: bool,
    version: Option<String>,
}

/// Parses the root `Cargo.toml`'s `[workspace.dependencies]` table. Entries
/// with a `path` key are this repo's own member crates, not real
/// dependencies to govern — callers filter those out.
fn parse_workspace_dependencies(ctx: &PipelineContext) -> Vec<WorkspaceDep> {
    let manifest_path = ctx.project_root.join("Cargo.toml");
    let Ok(text) = fs::read_to_string(&manifest_path) else { return Vec::new() };
    let Ok(parsed) = text.parse::<toml::Value>() else { return Vec::new() };
    let Some(deps) = parsed
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table())
    else {
        return Vec::new();
    };

    deps.iter()
        .map(|(name, value)| match value {
            toml::Value::String(v) => WorkspaceDep {
                name: name.clone(),
                is_path: false,
                is_git: false,
                version: Some(v.clone()),
            },
            toml::Value::Table(t) => WorkspaceDep {
                name: name.clone(),
                is_path: t.contains_key("path"),
                is_git: t.contains_key("git"),
                version: t.get("version").and_then(|v| v.as_str()).map(String::from),
            },
            _ => WorkspaceDep { name: name.clone(), is_path: false, is_git: false, version: None },
        })
        .collect()
}

/// Directories of every workspace member crate, from the root `Cargo.toml`'s
/// `[workspace] members` array — read from the manifest rather than
/// hardcoded, so this stays correct as crates are added or removed.
fn workspace_member_dirs(ctx: &PipelineContext) -> Vec<PathBuf> {
    let manifest_path = ctx.project_root.join("Cargo.toml");
    let Ok(text) = fs::read_to_string(&manifest_path) else { return Vec::new() };
    let Ok(parsed) = text.parse::<toml::Value>() else { return Vec::new() };
    let Some(members) = parsed.get("workspace").and_then(|w| w.get("members")).and_then(|m| m.as_array()) else {
        return Vec::new();
    };

    members
        .iter()
        .filter_map(|m| m.as_str())
        .map(|rel| ctx.project_root.join(rel))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new() -> Self {
            let id = COUNTER.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir().join(format!("samgraha-dep-test-{}-{}", std::process::id(), id));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_root_manifest(self, content: &str) -> Self {
            fs::write(self.root.join("Cargo.toml"), content).unwrap();
            self
        }

        fn with_member(self, rel: &str, content: &str) -> Self {
            let dir = self.root.join(rel);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("Cargo.toml"), content).unwrap();
            self
        }

        fn with_engineering_doc(self, name: &str, content: &str) -> Self {
            let dir = self.root.join("docs/raw/engineering");
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(name), content).unwrap();
            self
        }

        fn ctx(&self) -> PipelineContext {
            PipelineContext::new(self.root.clone(), common::config::SamgrahaConfig::default())
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn d1_warns_on_undocumented_dependency() {
        let proj = TempProject::new()
            .with_root_manifest(
                "[workspace]\nmembers = []\n\n[workspace.dependencies]\nserde = \"1\"\nleftpad = \"1\"\n",
            )
            .with_engineering_doc("deps.md", "We use serde for serialization.");
        let report = DependencyPipeline.run(&proj.ctx());
        let d1 = report.findings.iter().find(|f| f.check_id == "D1").unwrap();
        assert_eq!(d1.severity, Severity::Warning);
        assert!(d1.message.contains("leftpad"));
        assert!(!d1.message.contains("serde"));
    }

    #[test]
    fn d1_passes_when_all_dependencies_are_documented() {
        let proj = TempProject::new()
            .with_root_manifest("[workspace]\nmembers = []\n\n[workspace.dependencies]\nserde = \"1\"\n")
            .with_engineering_doc("deps.md", "We use serde for serialization.");
        let report = DependencyPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "D1"));
    }

    #[test]
    fn d1_ignores_path_dependencies() {
        let proj = TempProject::new().with_root_manifest(
            "[workspace]\nmembers = []\n\n[workspace.dependencies]\nschemas = { path = \"crates/schemas\" }\n",
        );
        let report = DependencyPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "D1"));
    }

    #[test]
    fn d4_warns_on_wildcard_version() {
        let proj = TempProject::new().with_root_manifest(
            "[workspace]\nmembers = []\n\n[workspace.dependencies]\nfoo = \"*\"\n",
        ).with_engineering_doc("deps.md", "We use foo.");
        let report = DependencyPipeline.run(&proj.ctx());
        let d4 = report.findings.iter().find(|f| f.check_id == "D4").unwrap();
        assert_eq!(d4.severity, Severity::Warning);
        assert!(d4.message.contains("foo"));
    }

    #[test]
    fn d5_warns_on_undeclared_git_source() {
        let proj = TempProject::new()
            .with_root_manifest(
                "[workspace]\nmembers = []\n\n[workspace.dependencies]\nforked-thing = { git = \"https://example.com/fork\" }\n",
            )
            .with_engineering_doc("deps.md", "Nothing about forks here.");
        let report = DependencyPipeline.run(&proj.ctx());
        let d5 = report.findings.iter().find(|f| f.check_id == "D5").unwrap();
        assert_eq!(d5.severity, Severity::Warning);
        assert!(d5.message.contains("forked-thing"));
    }

    #[test]
    fn d7_flags_prohibited_async_runtime() {
        let proj = TempProject::new()
            .with_root_manifest("[workspace]\nmembers = [\"crates/x\"]\n\n[workspace.dependencies]\n")
            .with_member("crates/x", "[package]\nname = \"x\"\n\n[dependencies]\ntokio = \"1\"\n");
        let report = DependencyPipeline.run(&proj.ctx());
        let d7 = report.findings.iter().find(|f| f.check_id == "D7").unwrap();
        assert!(d7.message.contains("tokio"));
    }

    #[test]
    fn d7_flags_notify_without_feature_gate() {
        let proj = TempProject::new()
            .with_root_manifest("[workspace]\nmembers = [\"crates/x\"]\n\n[workspace.dependencies]\n")
            .with_member("crates/x", "[package]\nname = \"x\"\n\n[dependencies]\nnotify = \"6\"\n");
        let report = DependencyPipeline.run(&proj.ctx());
        let d7 = report.findings.iter().find(|f| f.check_id == "D7").unwrap();
        assert!(d7.message.contains("notify"));
    }

    #[test]
    fn d7_passes_when_notify_is_optional_and_feature_gated() {
        let proj = TempProject::new()
            .with_root_manifest("[workspace]\nmembers = [\"crates/x\"]\n\n[workspace.dependencies]\n")
            .with_member(
                "crates/x",
                "[package]\nname = \"x\"\n\n[dependencies]\nnotify = { version = \"6\", optional = true }\n\n[features]\nwatch = [\"notify\"]\n",
            );
        let report = DependencyPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "D7"));
    }

    #[test]
    fn always_present_stub_findings_are_suggestions_not_errors() {
        let proj = TempProject::new().with_root_manifest("[workspace]\nmembers = []\n\n[workspace.dependencies]\n");
        let report = DependencyPipeline.run(&proj.ctx());
        for check_id in ["D2", "D3", "D6", "D8"] {
            let f = report.findings.iter().find(|f| f.check_id == check_id).unwrap();
            assert_eq!(f.severity, Severity::Suggestion, "{check_id} should be a Suggestion stub");
        }
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new().with_root_manifest("[workspace]\nmembers = []\n\n[workspace.dependencies]\n");
        let report = DependencyPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
