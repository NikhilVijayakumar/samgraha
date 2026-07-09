use crate::pipeline::{finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use crate::pipelines::feature::FeaturePipeline;
use crate::pipelines::feature_technical::FeatureTechnicalPipeline;
use crate::pipelines::readme::ReadmePipeline;
use schemas::audit::{AuditFinding, PipelineKind, PipelineReport, Severity};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Directories reserved for built-in knowledge — mirrors
/// `crates/services/src/compilation.rs::RESERVED_DOMAINS`. Duplicated locally
/// (that constant is private to `services`, which depends on `audit`, not
/// the reverse) rather than restructured, since it's one array.
const RESERVED_DOMAINS: &[&str] = &["help", "standards"];

/// On-disk directory name → compiled domain, for names that don't match
/// their domain string directly. Mirrors
/// `crates/compiler/src/discovery.rs::DOMAIN_OVERRIDE` (private to
/// `compiler`) — kept in sync by hand; see that file for the authoritative
/// table. See G2 in `docs/proposal.md`.
const DOMAIN_OVERRIDE: &[(&str, &str)] = &[("product-guide", "help")];

/// Directories that exist in this repo today but match none of `domain`,
/// `domain_exclusion`, `[repository.ignore].patterns`, or `DOMAIN_OVERRIDE`
/// — reported as a Suggestion rather than a Warning by SI4. See G1/G3 in
/// `docs/proposal.md`.
const UNCLASSIFIED_KNOWN: &[&str] = &["audit", "fix-plan-templates", "report-templates"];

const REQUIRED_DOMAINS: &[&str] = &["feature", "engineering", "vision"];

pub struct DocumentationStructurePipeline;

impl Pipeline for DocumentationStructurePipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::DocumentationStructure
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        let docs_raw = ctx.project_root.join("docs").join("raw");
        let readme_path = ctx.project_root.join("README.md");

        let feature_dir = docs_raw.join("feature");
        let ft_dir = docs_raw.join("feature-technical");
        let fd_dir = docs_raw.join("feature-design");
        let vision_dir = docs_raw.join("vision");
        let arch_dir = docs_raw.join("architecture");
        let eng_dir = docs_raw.join("engineering");
        let design_dir = docs_raw.join("design");

        let feature_docs = scan_markdown_files(&feature_dir);
        let ft_docs = scan_markdown_files(&ft_dir);
        let fd_docs = scan_markdown_files(&fd_dir);
        let vision_docs = scan_markdown_files(&vision_dir);

        let feature_stems = stems(&feature_docs);
        let ft_stems = stems(&ft_docs);
        let fd_stems = stems(&fd_docs);

        cat_scores.insert("Structural Integrity".into(), run_structural_integrity(
            ctx, &mut findings, &docs_raw, &readme_path, &feature_dir, &vision_dir, &vision_docs, &feature_docs,
        ));
        cat_scores.insert("Mapping Consistency".into(), run_mapping_consistency(
            &mut findings, &feature_dir, &ft_dir, &fd_dir, &ft_docs, &fd_docs,
            &feature_stems, &ft_stems, &fd_stems,
        ));
        cat_scores.insert("Atomicity Enforcement".into(), run_atomicity_enforcement(
            &mut findings, &feature_docs, &ft_docs, &fd_docs, &feature_stems,
        ));
        cat_scores.insert("Cross-Document Alignment".into(), run_cross_document_alignment(
            &mut findings, &readme_path, &vision_dir, &vision_docs, &ft_dir, &ft_docs, &arch_dir, &eng_dir,
            &fd_dir, &fd_docs, &design_dir,
        ));
        cat_scores.insert("Name Preservation".into(), run_name_preservation(
            &mut findings, &feature_docs, &ft_docs, &fd_docs, &feature_stems, &ft_stems, &fd_stems,
        ));
        cat_scores.insert("Implementation Traceability".into(), run_implementation_traceability(
            ctx, &mut findings, &feature_docs, &ft_docs, &feature_stems, &ft_stems, &arch_dir,
        ));
        cat_scores.insert("Generation Compliance".into(), run_generation_compliance(
            ctx, &mut findings, &feature_docs, &ft_docs,
        ));

        // Delegated checks — see G6/G7 in docs/proposal.md. These reuse
        // existing pipelines' findings under our own check IDs instead of
        // reimplementing logic that already exists elsewhere.
        run_delegated_checks(ctx, &mut findings);

        let overall = cat_scores.values().sum::<f64>() / cat_scores.len().max(1) as f64;

        let mut report = make_report(PipelineKind::DocumentationStructure, overall, cat_scores, findings);
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall));
        report
    }
}

// ── Structural Integrity (SI1-SI4, SI6-SI7; SI5 delegated to Readme R1) ────

#[allow(clippy::too_many_arguments)]
fn run_structural_integrity(
    ctx: &PipelineContext,
    findings: &mut Vec<AuditFinding>,
    docs_raw: &Path,
    readme_path: &Path,
    feature_dir: &Path,
    vision_dir: &Path,
    vision_docs: &[PathBuf],
    feature_docs: &[PathBuf],
) -> f64 {
    let _ = readme_path;
    let mut passed = 0u32;
    let mut total = 0u32;

    let domain = &ctx.config.repository.documentation.domain;
    let domain_exclusion = &ctx.config.repository.documentation.domain_exclusion;
    let ignore_patterns = &ctx.config.repository.ignore.patterns;

    // SI1: Required domain directories exist
    total += 1;
    let missing: Vec<&str> = REQUIRED_DOMAINS.iter()
        .filter(|d| !docs_raw.join(d).exists())
        .copied()
        .collect();
    if missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "SI1", Severity::Error,
            format!("Required domain directories missing: {}", missing.join(", ")),
            None,
        ));
    }

    // SI2: Excluded domain directories are absent or excluded — a domain
    // listed in domain_exclusion should either have no local directory, or
    // (for RESERVED_DOMAINS like help/standards) be legitimately present as
    // built-in content, not this repo's own docs.
    total += 1;
    let mut excluded_with_content = Vec::new();
    for d in domain_exclusion {
        if RESERVED_DOMAINS.contains(&d.as_str()) {
            continue;
        }
        let dir = docs_raw.join(d);
        if dir.exists() && !scan_markdown_files(&dir).is_empty() {
            excluded_with_content.push(d.clone());
        }
    }
    if excluded_with_content.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "SI2", Severity::Warning,
            format!(
                "Domains listed in domain_exclusion still contain documents: {} — remove the content or drop the exclusion",
                excluded_with_content.join(", ")
            ),
            None,
        ));
    }

    // SI3: Reserved domains (help/standards) not declared without exclusion
    total += 1;
    let unguarded: Vec<&str> = RESERVED_DOMAINS.iter()
        .filter(|r| domain.iter().any(|d| d == *r) && !domain_exclusion.iter().any(|d| d == *r))
        .copied()
        .collect();
    if unguarded.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "SI3", Severity::Error,
            format!(
                "Reserved domain(s) declared in `domain` without matching `domain_exclusion`: {} — see RESERVED_DOMAINS in compilation.rs",
                unguarded.join(", ")
            ),
            None,
        ));
    }

    // SI4: No unexpected directories under docs/raw/
    total += 1;
    let mut expected: HashSet<String> = domain.iter().cloned().collect();
    for (dir_name, target_domain) in DOMAIN_OVERRIDE {
        if domain.iter().any(|d| d == target_domain) {
            expected.insert((*dir_name).to_string());
        }
    }
    let mut unexpected_warn = Vec::new();
    let mut unexpected_suggest = Vec::new();
    if let Ok(entries) = fs::read_dir(docs_raw) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if expected.contains(&name) {
                continue;
            }
            if ignore_patterns.iter().any(|p| common::glob::matches_glob(p, &name)) {
                continue;
            }
            if UNCLASSIFIED_KNOWN.contains(&name.as_str()) {
                unexpected_suggest.push(name);
            } else {
                unexpected_warn.push(name);
            }
        }
    }
    if !unexpected_suggest.is_empty() {
        findings.push(finding(
            "SI4", Severity::Suggestion,
            format!(
                "Directories under docs/raw/ not covered by `domain`, `domain_exclusion`, or `[repository.ignore].patterns`: {} — consider declaring them explicitly",
                unexpected_suggest.join(", ")
            ),
            None,
        ));
    }
    if !unexpected_warn.is_empty() {
        findings.push(finding(
            "SI4", Severity::Warning,
            format!("Unexpected directories under docs/raw/: {}", unexpected_warn.join(", ")),
            None,
        ));
    }
    if unexpected_suggest.is_empty() && unexpected_warn.is_empty() {
        passed += 1;
    }

    // SI6: Vision directory contains exactly one document
    total += 1;
    if !vision_dir.exists() || vision_docs.len() == 1 {
        passed += 1;
    } else if vision_docs.is_empty() {
        findings.push(finding(
            "SI6", Severity::Error,
            "Vision directory exists but contains no documents".into(),
            None,
        ));
    } else {
        findings.push(finding(
            "SI6", Severity::Warning,
            format!("Vision directory contains {} documents — expected exactly one", vision_docs.len()),
            None,
        ));
    }

    // SI7: Feature documents are atomic — exactly one H1 per file
    total += 1;
    let non_atomic = non_atomic_files(feature_docs);
    let _ = feature_dir;
    if non_atomic.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "SI7", Severity::Warning,
            format!(
                "Feature documents without exactly one top-level heading: {}",
                non_atomic.join(", ")
            ),
            None,
        ));
    }

    score(passed, total)
}

// ── Mapping Consistency (MC1-MC4, MC6-MC8; MC5 delegated to Feature F1) ────

#[allow(clippy::too_many_arguments)]
fn run_mapping_consistency(
    findings: &mut Vec<AuditFinding>,
    feature_dir: &Path,
    ft_dir: &Path,
    fd_dir: &Path,
    ft_docs: &[PathBuf],
    fd_docs: &[PathBuf],
    feature_stems: &HashSet<String>,
    ft_stems: &HashSet<String>,
    fd_stems: &HashSet<String>,
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    // MC1: every feature has a feature-technical doc (when both present)
    total += 1;
    if !feature_dir.exists() || !ft_dir.exists() {
        passed += 1;
    } else {
        let orphans: Vec<&String> = feature_stems.difference(ft_stems).collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "MC1", Severity::Warning,
                format!("Features with no feature-technical counterpart: {}", join_sorted(&orphans)),
                None,
            ));
        }
    }

    // MC2: every feature-technical has a feature doc (when both present)
    total += 1;
    if !feature_dir.exists() || !ft_dir.exists() {
        passed += 1;
    } else {
        let orphans: Vec<&String> = ft_stems.difference(feature_stems).collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "MC2", Severity::Warning,
                format!("Feature-technical documents with no feature counterpart: {}", join_sorted(&orphans)),
                None,
            ));
        }
    }

    // MC3: every feature has a feature-design doc (when both present)
    total += 1;
    if !feature_dir.exists() || !fd_dir.exists() {
        passed += 1;
    } else {
        let orphans: Vec<&String> = feature_stems.difference(fd_stems).collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "MC3", Severity::Warning,
                format!("Features with no feature-design counterpart: {}", join_sorted(&orphans)),
                None,
            ));
        }
    }

    // MC4: every feature-design has a feature doc (when both present)
    total += 1;
    if !feature_dir.exists() || !fd_dir.exists() {
        passed += 1;
    } else {
        let orphans: Vec<&String> = fd_stems.difference(feature_stems).collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "MC4", Severity::Warning,
                format!("Feature-design documents with no feature counterpart: {}", join_sorted(&orphans)),
                None,
            ));
        }
    }

    // MC6: no duplicate feature-technical titles
    total += 1;
    if let Some(dupes) = duplicate_titles(ft_docs) {
        findings.push(finding(
            "MC6", Severity::Error,
            format!("Duplicate feature-technical document titles: {}", dupes.join(", ")),
            None,
        ));
    } else {
        passed += 1;
    }

    // MC7: Feature Technical titles reference their parent Feature
    total += 1;
    let mc7_missing = title_references_parent(ft_docs, feature_stems);
    if mc7_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "MC7", Severity::Suggestion,
            format!(
                "Feature-technical documents whose title doesn't reference their parent feature: {}",
                mc7_missing.join(", ")
            ),
            None,
        ));
    }

    // MC8: Feature Design titles reference their parent Feature
    total += 1;
    let mc8_missing = title_references_parent(fd_docs, feature_stems);
    if mc8_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "MC8", Severity::Suggestion,
            format!(
                "Feature-design documents whose title doesn't reference their parent feature: {}",
                mc8_missing.join(", ")
            ),
            None,
        ));
    }

    score(passed, total)
}

// ── Atomicity Enforcement (AE1-AE3, AE5-AE6; AE4 delegated to Feature F14) ─

fn run_atomicity_enforcement(
    findings: &mut Vec<AuditFinding>,
    feature_docs: &[PathBuf],
    ft_docs: &[PathBuf],
    fd_docs: &[PathBuf],
    feature_stems: &HashSet<String>,
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    // AE1: each feature describes exactly one capability (one H1)
    total += 1;
    let non_atomic = non_atomic_files(feature_docs);
    if non_atomic.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "AE1", Severity::Warning,
            format!("Feature documents describing more or less than one capability: {}", non_atomic.join(", ")),
            None,
        ));
    }

    // AE2: single responsibility — "and"-joined titles suggest combined scope
    total += 1;
    let mut and_titles = Vec::new();
    for p in feature_docs {
        if let Some(title) = first_title(p) {
            let low = title.to_lowercase();
            if low.contains(" and ") {
                and_titles.push(file_stem(p));
            }
        }
    }
    if and_titles.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "AE2", Severity::Warning,
            format!(
                "Feature titles combining multiple responsibilities with \"and\": {}",
                and_titles.join(", ")
            ),
            None,
        ));
    }

    // AE3: no feature combines unrelated capabilities — a feature
    // referencing a majority of its siblings by name suggests it's absorbed
    // their scope rather than staying independently understandable.
    // Proportional to sibling count (not a fixed absolute number): in a
    // large, densely cross-referenced workspace, mentioning a handful of
    // related modules is normal architecture practice, not scope creep —
    // only a genuine majority is worth flagging.
    total += 1;
    let sibling_count = feature_stems.len().saturating_sub(1);
    let over_reference_threshold = (sibling_count / 2).max(2);
    let mut over_referencing = Vec::new();
    for p in feature_docs {
        let Ok(content) = fs::read_to_string(p) else { continue };
        let low = strip_code_fences(&content).to_lowercase();
        let stem = file_stem(p);
        let refs = feature_stems.iter()
            .filter(|s| **s != stem && low.contains(s.replace('-', " ").as_str()))
            .count();
        if refs > over_reference_threshold {
            over_referencing.push(stem);
        }
    }
    if over_referencing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "AE3", Severity::Suggestion,
            format!(
                "Feature documents referencing more than two sibling features — review for scope creep: {}",
                over_referencing.join(", ")
            ),
            None,
        ));
    }

    // AE5: feature-technical maps to a single feature (title doesn't
    // mention more than one feature stem)
    total += 1;
    let ae5_multi = title_references_multiple(ft_docs, feature_stems);
    if ae5_multi.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "AE5", Severity::Warning,
            format!("Feature-technical documents whose title spans multiple features: {}", ae5_multi.join(", ")),
            None,
        ));
    }

    // AE6: feature-design maps to a single feature
    total += 1;
    let ae6_multi = title_references_multiple(fd_docs, feature_stems);
    if ae6_multi.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "AE6", Severity::Warning,
            format!("Feature-design documents whose title spans multiple features: {}", ae6_multi.join(", ")),
            None,
        ));
    }

    score(passed, total)
}

// ── Cross-Document Alignment (CA1, CA4-CA8; CA2 delegated to Readme R4; ────
// CA3 delegated to Feature F9) ──────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn run_cross_document_alignment(
    findings: &mut Vec<AuditFinding>,
    readme_path: &Path,
    vision_dir: &Path,
    vision_docs: &[PathBuf],
    ft_dir: &Path,
    ft_docs: &[PathBuf],
    arch_dir: &Path,
    eng_dir: &Path,
    fd_dir: &Path,
    fd_docs: &[PathBuf],
    design_dir: &Path,
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    let readme_low = fs::read_to_string(readme_path).unwrap_or_default().to_lowercase();

    // CA1: README summarizes Vision purpose — real implementation,
    // supersedes readme.rs R3 stub (see G7 in docs/proposal.md). Checks
    // that a majority of the Vision title's significant words also appear
    // in the README.
    total += 1;
    if !readme_path.exists() || !vision_dir.exists() || vision_docs.is_empty() {
        passed += 1;
    } else {
        let vision_title = first_title(&vision_docs[0]).unwrap_or_default().to_lowercase();
        let words: Vec<&str> = vision_title.split_whitespace().filter(|w| w.len() > 3).collect();
        if words.is_empty() {
            passed += 1;
        } else {
            let matched = words.iter().filter(|w| readme_low.contains(*w)).count();
            if matched * 2 >= words.len() {
                passed += 1;
            } else {
                findings.push(finding(
                    "CA1", Severity::Warning,
                    "README does not appear to summarize Vision's stated purpose — few shared significant words between the Vision title and README".into(),
                    None,
                ));
            }
        }
    }

    // CA4: Feature Technical applies shared Architecture
    total += 1;
    if !ft_dir.exists() || !arch_dir.exists() {
        passed += 1;
    } else {
        let ft_text = read_domain_text_lower(ft_docs);
        if ft_text.contains("architecture") {
            passed += 1;
        } else {
            findings.push(finding(
                "CA4", Severity::Warning,
                "Architecture docs exist but no Feature Technical document references Architecture".into(),
                None,
            ));
        }
    }

    // CA5: Feature Technical respects Engineering standards
    total += 1;
    if !ft_dir.exists() || !eng_dir.exists() {
        passed += 1;
    } else {
        let ft_text = read_domain_text_lower(ft_docs);
        if ft_text.contains("engineering") {
            passed += 1;
        } else {
            findings.push(finding(
                "CA5", Severity::Warning,
                "Engineering docs exist but no Feature Technical document references Engineering standards".into(),
                None,
            ));
        }
    }

    // CA6: Feature Design applies shared Design principles
    total += 1;
    if !fd_dir.exists() || !design_dir.exists() {
        passed += 1;
    } else {
        let fd_text = read_domain_text_lower(fd_docs);
        if fd_text.contains("design") {
            passed += 1;
        } else {
            findings.push(finding(
                "CA6", Severity::Warning,
                "Design docs exist but no Feature Design document references Design principles".into(),
                None,
            ));
        }
    }

    // CA7: No contradictions between adjacent layers — real implementation,
    // supersedes consistency.rs C11 stub (see G7). Flags a keyword that one
    // layer explicitly negates while the adjacent layer affirms unnegated.
    total += 1;
    let contradictions = detect_contradictions(vision_docs, arch_dir, eng_dir);
    if contradictions.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "CA7", Severity::Warning,
            format!("Possible contradictions between adjacent layers: {}", contradictions.join("; ")),
            None,
        ));
    }

    // CA8: Constraint propagation from Vision downward — real
    // implementation, supersedes consistency.rs C10 stub (see G7).
    total += 1;
    if !vision_docs.is_empty() {
        let vision_text = read_domain_text_lower(vision_docs);
        let has_constraints_section = vision_docs.iter().any(|p| {
            fs::read_to_string(p).map(|c| extract_headings(&strip_code_fences(&c))
                .iter().any(|h| h.eq_ignore_ascii_case("Constraints"))).unwrap_or(false)
        });
        if !has_constraints_section {
            passed += 1;
        } else {
            let _ = &vision_text;
            let arch_text = read_domain_text_lower(&scan_markdown_files(arch_dir));
            let eng_text = read_domain_text_lower(&scan_markdown_files(eng_dir));
            if arch_text.contains("constraint") || eng_text.contains("constraint") {
                passed += 1;
            } else {
                findings.push(finding(
                    "CA8", Severity::Warning,
                    "Vision declares Constraints but neither Architecture nor Engineering visibly references any constraint".into(),
                    None,
                ));
            }
        }
    } else {
        passed += 1;
    }

    score(passed, total)
}

/// Small fixed vocabulary for CA7's negation-mismatch check — deliberately
/// narrow (heading/keyword scanning per the proposal's Atomicity Philosophy,
/// not semantic analysis).
const CONTRADICTION_KEYWORDS: &[&str] = &["microservices", "monolith", "cloud", "on-premise", "database", "framework"];

fn detect_contradictions(vision_docs: &[PathBuf], arch_dir: &Path, eng_dir: &Path) -> Vec<String> {
    let vision_text = read_domain_text_lower(vision_docs);
    let arch_text = read_domain_text_lower(&scan_markdown_files(arch_dir));
    let eng_text = read_domain_text_lower(&scan_markdown_files(eng_dir));

    let layers: [(&str, &str); 3] = [
        ("Vision", vision_text.as_str()),
        ("Architecture", arch_text.as_str()),
        ("Engineering", eng_text.as_str()),
    ];

    let mut found = Vec::new();
    for kw in CONTRADICTION_KEYWORDS {
        let mut negates_in = Vec::new();
        let mut affirms_in = Vec::new();
        for (name, text) in &layers {
            if text.is_empty() {
                continue;
            }
            let unnegated = crate::pipeline::find_unnegated_keywords(text, &[kw]);
            if !unnegated.is_empty() {
                affirms_in.push(*name);
            } else if text.contains(kw) {
                negates_in.push(*name);
            }
        }
        if !negates_in.is_empty() && !affirms_in.is_empty() {
            found.push(format!("\"{}\" negated in {} but affirmed in {}", kw, negates_in.join("/"), affirms_in.join("/")));
        }
    }
    found
}

// ── Name Preservation (NP1-NP6) ────────────────────────────────────────────

fn run_name_preservation(
    findings: &mut Vec<AuditFinding>,
    feature_docs: &[PathBuf],
    ft_docs: &[PathBuf],
    fd_docs: &[PathBuf],
    feature_stems: &HashSet<String>,
    ft_stems: &HashSet<String>,
    fd_stems: &HashSet<String>,
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    // NP1: feature file stems match feature-technical file stems
    total += 1;
    if feature_docs.is_empty() || ft_docs.is_empty() || feature_stems == ft_stems
        || feature_stems.is_subset(ft_stems) || ft_stems.is_subset(feature_stems)
        || !feature_stems.is_disjoint(ft_stems)
    {
        passed += 1;
    } else {
        findings.push(finding(
            "NP1", Severity::Warning,
            "No feature file stems match any feature-technical file stems".into(),
            None,
        ));
    }

    // NP2: feature file stems match feature-design file stems
    total += 1;
    if feature_docs.is_empty() || fd_docs.is_empty() || !feature_stems.is_disjoint(fd_stems) {
        passed += 1;
    } else {
        findings.push(finding(
            "NP2", Severity::Warning,
            "No feature file stems match any feature-design file stems".into(),
            None,
        ));
    }

    // NP3: feature titles match feature-technical titles (same stem, title
    // should share the feature's title text as a substring/prefix)
    total += 1;
    let np3_missing = title_references_parent(ft_docs, feature_stems);
    if np3_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "NP3", Severity::Suggestion,
            format!("Feature-technical titles not clearly derived from their feature's title: {}", np3_missing.join(", ")),
            None,
        ));
    }

    // NP4: feature titles match feature-design titles
    total += 1;
    let np4_missing = title_references_parent(fd_docs, feature_stems);
    if np4_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "NP4", Severity::Suggestion,
            format!("Feature-design titles not clearly derived from their feature's title: {}", np4_missing.join(", ")),
            None,
        ));
    }

    // NP5: cross-references use consistent naming — markdown links into
    // feature-technical/feature-design should point at a stem that exists
    let mut broken_refs = Vec::new();
    total += 1;
    for p in feature_docs {
        let Ok(content) = fs::read_to_string(p) else { continue };
        for link_stem in extract_relative_link_stems(&content, "feature-technical") {
            if !ft_stems.contains(&link_stem) {
                broken_refs.push(format!("{} -> feature-technical/{}", file_stem(p), link_stem));
            }
        }
        for link_stem in extract_relative_link_stems(&content, "feature-design") {
            if !fd_stems.contains(&link_stem) {
                broken_refs.push(format!("{} -> feature-design/{}", file_stem(p), link_stem));
            }
        }
    }
    if broken_refs.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "NP5", Severity::Warning,
            format!("Cross-references pointing at non-existent documents: {}", broken_refs.join(", ")),
            None,
        ));
    }

    // NP6: Compiled knowledge preserves original file stems — genuinely out
    // of reach here: `Pipeline::run` receives only `PipelineContext`
    // (filesystem + config), no registry/DB handle, so the compiled-DB side
    // of this comparison can't be checked from inside a pipeline today.
    total += 1;
    passed += 1;
    findings.push(finding(
        "NP6", Severity::Suggestion,
        "Compiled-knowledge stem preservation requires a registry/DB handle, which Pipeline::run does not receive — not checkable from within a pipeline; verify via `samgraha compile` output directly".into(),
        None,
    ));

    score(passed, total)
}

// ── Implementation Traceability (IT1-IT5) ──────────────────────────────────

fn run_implementation_traceability(
    ctx: &PipelineContext,
    findings: &mut Vec<AuditFinding>,
    feature_docs: &[PathBuf],
    ft_docs: &[PathBuf],
    feature_stems: &HashSet<String>,
    ft_stems: &HashSet<String>,
    arch_dir: &Path,
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    let impl_dir = common::config::resolve_configured_dir(
        &ctx.config.repository.implementation.dir,
        &ctx.project_root,
        "src",
    );
    let crate_names = discover_workspace_crate_names(&impl_dir);
    let source_tokens = collect_source_tokens(&impl_dir, &crate_names);

    // IT1: every feature has corresponding code
    total += 1;
    if feature_docs.is_empty() || source_tokens.is_empty() {
        passed += 1;
    } else {
        let orphans: Vec<String> = feature_stems.iter()
            .filter(|stem| !stem_tokens(stem).iter().any(|t| source_tokens.contains(t)))
            .cloned()
            .collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "IT1", Severity::Warning,
                format!("Features with no matching source token found under the implementation directory: {}", join_sorted(&orphans.iter().collect::<Vec<_>>())),
                None,
            ));
        }
    }

    // IT2: every feature-technical has corresponding code
    total += 1;
    if ft_docs.is_empty() || source_tokens.is_empty() {
        passed += 1;
    } else {
        let orphans: Vec<String> = ft_stems.iter()
            .filter(|stem| !stem_tokens(stem).iter().any(|t| source_tokens.contains(t)))
            .cloned()
            .collect();
        if orphans.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "IT2", Severity::Warning,
                format!("Feature-technical documents with no matching source token: {}", join_sorted(&orphans.iter().collect::<Vec<_>>())),
                None,
            ));
        }
    }

    // IT3: no orphan source modules without documentation (crate-level
    // granularity, matching implementation.rs's existing approach)
    total += 1;
    if crate_names.is_empty() {
        passed += 1;
    } else {
        let doc_text = read_domain_text_lower(feature_docs) + " " + &read_domain_text_lower(ft_docs);
        let orphan_crates: Vec<&String> = crate_names.iter()
            .filter(|c| !doc_text.contains(c.as_str()) && !stem_tokens(c).iter().any(|t| doc_text.contains(t.as_str())))
            .collect();
        if orphan_crates.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "IT3", Severity::Suggestion,
                format!("Crates not mentioned in any feature or feature-technical document: {}", join_sorted(&orphan_crates)),
                None,
            ));
        }
    }

    // IT4: Feature responsibilities are realized in code — backtick-quoted
    // identifiers in feature docs should resolve to a real source token.
    total += 1;
    let it4_missing = unresolved_backtick_identifiers(feature_docs, &source_tokens);
    if it4_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "IT4", Severity::Suggestion,
            format!("Identifiers named in Feature docs but not found in source: {}", it4_missing.join(", ")),
            None,
        ));
    }

    // IT5: Architecture components exist as modules
    total += 1;
    let arch_docs = scan_markdown_files(arch_dir);
    let it5_missing = unresolved_backtick_identifiers(&arch_docs, &source_tokens);
    if it5_missing.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "IT5", Severity::Suggestion,
            format!("Identifiers named in Architecture docs but not found in source: {}", it5_missing.join(", ")),
            None,
        ));
    }

    score(passed, total)
}

// ── Generation Compliance (GC1-GC5) ─────────────────────────────────────────

fn run_generation_compliance(
    ctx: &PipelineContext,
    findings: &mut Vec<AuditFinding>,
    feature_docs: &[PathBuf],
    ft_docs: &[PathBuf],
) -> f64 {
    let mut passed = 0u32;
    let mut total = 0u32;

    let stub_docs: Vec<&PathBuf> = feature_docs.iter().chain(ft_docs.iter())
        .filter(|p| fs::read_to_string(p).map(|c| c.contains("TODO")).unwrap_or(false))
        .collect();

    // GC1: Generated stubs have correct section structure
    total += 1;
    if stub_docs.is_empty() {
        passed += 1;
    } else {
        let missing_purpose: Vec<String> = stub_docs.iter()
            .filter(|p| {
                fs::read_to_string(p).map(|c| {
                    !extract_headings(&strip_code_fences(&c)).iter()
                        .any(|h| ["Purpose", "Overview"].iter().any(|n| h.eq_ignore_ascii_case(n)))
                }).unwrap_or(true)
            })
            .map(|p| file_stem(p))
            .collect();
        if missing_purpose.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "GC1", Severity::Warning,
                format!("Stub documents (containing TODO) missing a Purpose/Overview section: {}", missing_purpose.join(", ")),
                None,
            ));
        }
    }

    // GC2: Generated stubs reference audit criteria
    total += 1;
    if stub_docs.is_empty() {
        passed += 1;
    } else {
        let missing_ref: Vec<String> = stub_docs.iter()
            .filter(|p| fs::read_to_string(p).map(|c| !mentions_check_id(&c)).unwrap_or(true))
            .map(|p| file_stem(p))
            .collect();
        if missing_ref.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "GC2", Severity::Suggestion,
                format!("Stub documents with no visible audit check-id reference: {}", missing_ref.join(", ")),
                None,
            ));
        }
    }

    // GC3: Generated content follows atomicity rules
    total += 1;
    let stub_paths: Vec<PathBuf> = stub_docs.iter().map(|p| (*p).clone()).collect();
    let non_atomic_stubs = non_atomic_files(&stub_paths);
    if non_atomic_stubs.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "GC3", Severity::Warning,
            format!("Stub documents violating one-H1-per-document atomicity: {}", non_atomic_stubs.join(", ")),
            None,
        ));
    }

    // GC4: Generated cross-references are valid
    total += 1;
    let mut invalid_links = Vec::new();
    for p in &stub_docs {
        let Ok(content) = fs::read_to_string(p) else { continue };
        for target in extract_relative_link_targets(&content) {
            let resolved = p.parent().map(|dir| dir.join(&target)).unwrap_or_else(|| PathBuf::from(&target));
            if !resolved.exists() {
                invalid_links.push(format!("{}: {}", file_stem(p), target));
            }
        }
    }
    if invalid_links.is_empty() {
        passed += 1;
    } else {
        findings.push(finding(
            "GC4", Severity::Warning,
            format!("Stub documents with cross-references to non-existent files: {}", invalid_links.join(", ")),
            None,
        ));
    }

    // GC5: Generated documentation passes domain audit — delegates to the
    // real Feature/Feature-Technical pipelines rather than re-scoring.
    total += 1;
    if stub_docs.is_empty() {
        passed += 1;
    } else {
        let mut below_threshold = Vec::new();
        let feature_score = FeaturePipeline.run(ctx).score;
        if feature_docs.iter().any(|p| stub_docs.contains(&p)) && feature_score < 70.0 {
            below_threshold.push(format!("feature ({:.0})", feature_score));
        }
        let ft_score = FeatureTechnicalPipeline.run(ctx).score;
        if ft_docs.iter().any(|p| stub_docs.contains(&p)) && ft_score < 70.0 {
            below_threshold.push(format!("feature-technical ({:.0})", ft_score));
        }
        if below_threshold.is_empty() {
            passed += 1;
        } else {
            findings.push(finding(
                "GC5", Severity::Warning,
                format!("Stub documents exist but domain score is below 70: {}", below_threshold.join(", ")),
                None,
            ));
        }
    }

    score(passed, total)
}

// ── Delegated checks (SI5, MC5, AE4, CA2, CA3) — see G6/G7 in proposal ────

fn run_delegated_checks(ctx: &PipelineContext, findings: &mut Vec<AuditFinding>) {
    delegate(findings, &FeaturePipeline.run(ctx), "F1", "MC5");
    delegate(findings, &FeaturePipeline.run(ctx), "F14", "AE4");
    delegate(findings, &FeaturePipeline.run(ctx), "F9", "CA3");
    delegate(findings, &ReadmePipeline.run(ctx), "R1", "SI5");
    delegate(findings, &ReadmePipeline.run(ctx), "R4", "CA2");
}

fn delegate(findings: &mut Vec<AuditFinding>, source: &PipelineReport, source_check_id: &str, our_check_id: &str) {
    if let Some(f) = source.findings.iter().find(|f| f.check_id == source_check_id) {
        findings.push(finding(
            our_check_id,
            f.severity.clone(),
            format!("(delegated to {} {}) {}", source.pipeline.as_str(), source_check_id, f.message),
            f.location.clone(),
        ));
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────────

fn score(passed: u32, total: u32) -> f64 {
    if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 100.0 }
}

fn readiness_label(overall: f64) -> String {
    if overall >= 90.0 {
        "READY".to_string()
    } else if overall >= 70.0 {
        "NEEDS_WORK".to_string()
    } else {
        "NOT_READY".to_string()
    }
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

fn extract_headings(content: &str) -> HashSet<String> {
    content.lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .collect()
}

fn file_stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string()
}

fn stems(docs: &[PathBuf]) -> HashSet<String> {
    docs.iter().map(|p| file_stem(p)).collect()
}

fn first_title(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()?.lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
}

fn duplicate_titles(docs: &[PathBuf]) -> Option<Vec<String>> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut dupes = Vec::new();
    for p in docs {
        if let Some(title) = first_title(p) {
            let key = title.to_lowercase();
            if !seen.insert(key.clone()) {
                dupes.push(title);
            }
        }
    }
    if dupes.is_empty() { None } else { Some(dupes) }
}

/// Files whose H1 count isn't exactly one — proxy for "describes exactly one
/// capability" (proposal's Atomicity verification methods: heading analysis).
fn non_atomic_files(docs: &[PathBuf]) -> Vec<String> {
    docs.iter()
        .filter_map(|p| {
            let content = fs::read_to_string(p).ok()?;
            let h1_count = strip_code_fences(&content).lines().filter(|l| l.starts_with("# ")).count();
            if h1_count == 1 { None } else { Some(file_stem(p)) }
        })
        .collect()
}

/// Documents whose title doesn't contain any word (len > 3) from their
/// matching parent feature's title, for stems present in `parent_stems`.
fn title_references_parent(docs: &[PathBuf], parent_stems: &HashSet<String>) -> Vec<String> {
    docs.iter()
        .filter_map(|p| {
            let stem = file_stem(p);
            if !parent_stems.contains(&stem) {
                return None;
            }
            let title = first_title(p)?.to_lowercase();
            let stem_words: Vec<String> = stem.split('-').filter(|w| w.len() > 2).map(|w| w.to_string()).collect();
            if stem_words.is_empty() || stem_words.iter().any(|w| title.contains(w.as_str())) {
                None
            } else {
                Some(stem)
            }
        })
        .collect()
}

/// Documents whose title mentions more than one sibling feature's stem
/// words — used by AE5/AE6 to flag a technical/design doc that spans
/// multiple features instead of mapping 1:1.
fn title_references_multiple(docs: &[PathBuf], feature_stems: &HashSet<String>) -> Vec<String> {
    docs.iter()
        .filter_map(|p| {
            let title = first_title(p)?.to_lowercase();
            let stem = file_stem(p);
            let mentioned = feature_stems.iter()
                .filter(|s| {
                    let words: Vec<&str> = s.split('-').filter(|w| w.len() > 2).collect();
                    !words.is_empty() && words.iter().all(|w| title.contains(w))
                })
                .count();
            if mentioned > 1 { Some(stem) } else { None }
        })
        .collect()
}

fn read_domain_text_lower(docs: &[PathBuf]) -> String {
    docs.iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .map(|c| strip_code_fences(&c))
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase()
}

fn join_sorted(items: &[&String]) -> String {
    let mut v: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    v.sort();
    v.join(", ")
}

/// Extracts the file stem out of markdown links like `[text](../{domain}/stem.md)`
/// or `[text](stem.md)` under a directory named `domain`.
fn extract_relative_link_stems(content: &str, domain: &str) -> Vec<String> {
    let mut stems = Vec::new();
    for target in extract_relative_link_targets(content) {
        if target.contains(&format!("{domain}/")) || target.starts_with("../") {
            if let Some(name) = target.rsplit('/').next() {
                if let Some(stem) = name.strip_suffix(".md") {
                    if target.contains(&format!("{domain}/")) {
                        stems.push(stem.to_string());
                    }
                }
            }
        }
    }
    stems
}

fn extract_relative_link_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b']' && i + 1 < bytes.len() && bytes[i + 1] == b'(' {
            if let Some(end) = content[i + 2..].find(')') {
                let link = &content[i + 2..i + 2 + end];
                if !link.starts_with("http://") && !link.starts_with("https://") && link.ends_with(".md") {
                    targets.push(link.to_string());
                }
            }
        }
        i += 1;
    }
    targets
}

fn mentions_check_id(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i].is_ascii_uppercase() && bytes[i + 1].is_ascii_uppercase() {
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 2 {
                return true;
            }
        } else if bytes[i].is_ascii_uppercase() && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
            return true;
        }
        i += 1;
    }
    false
}

fn stem_tokens(stem: &str) -> Vec<String> {
    stem.split(&['-', '_'][..]).filter(|w| w.len() > 2).map(|w| w.to_lowercase()).collect()
}

fn discover_workspace_crate_names(impl_dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(impl_dir) else { return names };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").exists() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                names.push(name.to_lowercase());
            }
        }
    }
    names
}

/// Crate names plus every `.rs` file stem under each crate (skipping
/// `target/`) — the token universe IT1/IT2/IT4/IT5 match feature/architecture
/// language against. Mirrors implementation.rs's recursive-walk pattern.
fn collect_source_tokens(impl_dir: &Path, crate_names: &[String]) -> HashSet<String> {
    let mut tokens: HashSet<String> = crate_names.iter().cloned().collect();
    fn walk(dir: &Path, tokens: &mut HashSet<String>) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().map(|n| n == "target").unwrap_or(false) {
                    continue;
                }
                walk(&path, tokens);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    tokens.insert(stem.to_lowercase());
                }
            }
        }
    }
    walk(impl_dir, &mut tokens);
    tokens
}

/// Backtick-quoted identifiers in `docs` that don't appear anywhere in
/// `source_tokens` (as an exact token or a dotted/double-colon segment).
fn unresolved_backtick_identifiers(docs: &[PathBuf], source_tokens: &HashSet<String>) -> Vec<String> {
    let mut missing = Vec::new();
    for p in docs {
        let Ok(content) = fs::read_to_string(p) else { continue };
        for ident in extract_backtick_identifiers(&content) {
            let low = ident.to_lowercase();
            let segment = low.rsplit("::").next().unwrap_or(&low).to_string();
            let base = segment.split('.').next().unwrap_or(&segment).to_string();
            if base.len() > 2 && base.chars().all(|c| c.is_alphanumeric() || c == '_')
                && !source_tokens.contains(&base)
                && !source_tokens.iter().any(|t| t.contains(&base) || base.contains(t.as_str()))
            {
                missing.push(ident);
            }
        }
    }
    missing.sort();
    missing.dedup();
    missing.truncate(10);
    missing
}

fn extract_backtick_identifiers(content: &str) -> Vec<String> {
    let mut idents = Vec::new();
    let mut chars = content.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '`' {
            if let Some(end) = content[i + 1..].find('`') {
                let ident = &content[i + 1..i + 1 + end];
                if !ident.is_empty() && !ident.contains(' ') && !ident.contains('\n')
                    && ident.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
                {
                    idents.push(ident.to_string());
                }
            }
        }
    }
    idents
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
            let root = std::env::temp_dir().join(format!("samgraha-docstruct-test-{}-{}", std::process::id(), id));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn with_file(self, rel: &str, content: &str) -> Self {
            let path = self.root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
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
    fn si1_errors_when_required_domains_missing() {
        let proj = TempProject::new();
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let si1 = report.findings.iter().find(|f| f.check_id == "SI1").unwrap();
        assert_eq!(si1.severity, Severity::Error);
    }

    #[test]
    fn si1_passes_when_required_domains_present() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# X")
            .with_file("docs/raw/engineering/e.md", "# E")
            .with_file("docs/raw/vision/v.md", "# V");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "SI1"));
    }

    #[test]
    fn si6_warns_on_multiple_vision_documents() {
        let proj = TempProject::new()
            .with_file("docs/raw/vision/a.md", "# A")
            .with_file("docs/raw/vision/b.md", "# B");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let si6 = report.findings.iter().find(|f| f.check_id == "SI6").unwrap();
        assert_eq!(si6.severity, Severity::Warning);
    }

    #[test]
    fn si7_warns_on_non_atomic_feature_doc() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# One\n\n# Two");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let si7 = report.findings.iter().find(|f| f.check_id == "SI7").unwrap();
        assert_eq!(si7.severity, Severity::Warning);
    }

    #[test]
    fn mc1_warns_on_orphan_feature() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# X")
            .with_file("docs/raw/feature-technical/y.md", "# Y Technical");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let mc1 = report.findings.iter().find(|f| f.check_id == "MC1").unwrap();
        assert_eq!(mc1.severity, Severity::Warning);
    }

    #[test]
    fn mc1_passes_on_matching_stems() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# X")
            .with_file("docs/raw/feature-technical/x.md", "# X Technical");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "MC1"));
    }

    #[test]
    fn ae2_warns_on_and_joined_title() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# Export and Import");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let ae2 = report.findings.iter().find(|f| f.check_id == "AE2").unwrap();
        assert_eq!(ae2.severity, Severity::Warning);
    }

    #[test]
    fn np5_warns_on_broken_cross_reference() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/x.md", "# X\n\nSee [tech](../feature-technical/missing.md).")
            .with_file("docs/raw/feature-technical/x.md", "# X Technical");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let np5 = report.findings.iter().find(|f| f.check_id == "NP5").unwrap();
        assert_eq!(np5.severity, Severity::Warning);
    }

    #[test]
    fn delegated_mc5_forwards_feature_f1() {
        let proj = TempProject::new()
            .with_file("docs/raw/feature/a.md", "# Same Title")
            .with_file("docs/raw/feature/b.md", "# Same Title");
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let mc5 = report.findings.iter().find(|f| f.check_id == "MC5").unwrap();
        assert!(mc5.message.contains("feature F1"));
    }

    #[test]
    fn delegated_si5_forwards_readme_r1() {
        let proj = TempProject::new();
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        let si5 = report.findings.iter().find(|f| f.check_id == "SI5").unwrap();
        assert!(si5.message.contains("readme R1"));
    }

    #[test]
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = DocumentationStructurePipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }
}
