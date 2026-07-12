use crate::pipeline::{finding, make_report, strip_code_fences, Pipeline, PipelineContext};
use schemas::audit::{AuditFinding, PipelineKind, PipelineReport, Severity};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct HelpPipeline;

impl Pipeline for HelpPipeline {
    fn name(&self) -> PipelineKind {
        PipelineKind::Help
    }

    fn run(&self, ctx: &PipelineContext) -> PipelineReport {
        // Canonical name is `product-guide/`; fall back to the pre-rename
        // `help/` for repos that haven't renamed their checkout yet (see
        // `DOMAIN_OVERRIDE` in crates/compiler/src/discovery.rs, which keeps
        // the same back-compat for the compiled `standard`/domain).
        let product_guide_dir = ctx.project_root.join("docs/raw/product-guide");
        let guide_dir = if product_guide_dir.is_dir() {
            product_guide_dir
        } else {
            ctx.project_root.join("docs/raw/help")
        };
        let mut files: Vec<GuideFile> = Vec::new();
        walk_md_files(&guide_dir, &guide_dir, &mut files);

        let mut all_content = String::new();
        let mut all_low = String::new();
        for f in &files {
            all_content.push_str(&f.content);
            all_content.push('\n');
            all_low.push_str(&f.content_low);
            all_low.push('\n');
        }

        let mut findings = Vec::new();
        let mut cat_scores: HashMap<String, f64> = HashMap::new();

        // ── Coverage (PC1-PC7) 30% ───────────────────────────────────────

        let mut pc = Tally::default();

        let cli_commands = schemas::code_inventory::CLI_COMMANDS;
        let mcp_methods = schemas::code_inventory::MCP_METHODS;
        let config_fields = schemas::code_inventory::CONFIG_FIELDS;

        let missing_commands = missing_by_stem(&files, "commands", cli_commands);
        pc.check(missing_commands.is_empty(), || finding(
            "PC1", Severity::Error,
            format!("CLI commands missing a doc in commands/: {:?}", missing_commands),
            Some("docs/raw/product-guide/commands/".into()),
        ), &mut findings);

        let tools_md = files.iter().find(|f| f.rel_path == Path::new("mcp-guide/tools.md"));
        let missing_mcp: Vec<&str> = match tools_md {
            Some(f) => {
                mcp_methods.iter().filter(|m| !f.content_low.contains(&m.to_lowercase())).copied().collect()
            }
            None => mcp_methods.to_vec(),
        };
        pc.check(missing_mcp.is_empty(), || finding(
            "PC2", Severity::Error,
            format!("MCP methods missing from mcp-guide/tools.md: {:?}", missing_mcp),
            Some("docs/raw/product-guide/mcp-guide/tools.md".into()),
        ), &mut findings);

        let missing_config = missing_by_stem(&files, "configuration", config_fields);
        pc.check(missing_config.is_empty(), || finding(
            "PC3", Severity::Error,
            format!("Config fields missing a doc in configuration/: {:?}", missing_config),
            Some("docs/raw/product-guide/configuration/".into()),
        ), &mut findings);

        // Phase 2: built-in definitions removed; domain list will come from
        // the knowledge-hub DB once pipelines are DB-aware (Phase 3).
        let standard_domains: Vec<String> = Vec::new();
        let missing_domains = missing_by_stem(&files, "documentation-guide", &standard_domains.iter().map(String::as_str).collect::<Vec<_>>());
        pc.check(missing_domains.is_empty(), || finding(
            "PC4", Severity::Suggestion,
            format!("Domain standards missing a documentation-guide entry: {:?}", missing_domains),
            Some("docs/raw/product-guide/documentation-guide/".into()),
        ), &mut findings);

        let feature_titles = titled_docs(&ctx.project_root.join("docs/raw/feature"));
        let missing_features = missing_by_mention(&feature_titles, &all_low);
        pc.check(missing_features.is_empty(), || finding(
            "PC5", Severity::Warning,
            format!("Features not explained anywhere in the guide: {:?}", missing_features),
            Some("docs/raw/feature/".into()),
        ), &mut findings);

        let architecture_titles = titled_docs(&ctx.project_root.join("docs/raw/architecture"));
        let missing_architecture = missing_by_mention(&architecture_titles, &all_low);
        pc.check(missing_architecture.is_empty(), || finding(
            "PC6", Severity::Suggestion,
            format!("Architecture decisions not reflected in the guide: {:?}", missing_architecture),
            Some("docs/raw/architecture/".into()),
        ), &mut findings);

        let pc7_ok = ctx.config.repository.name.as_deref()
            .map(|name| all_low.contains(&name.to_lowercase()))
            .unwrap_or(true);
        pc.check(pc7_ok, || finding(
            "PC7", Severity::Suggestion,
            format!(
                "Repository name '{}' (from samgraha.toml) is not mentioned anywhere in the product guide",
                ctx.config.repository.name.as_deref().unwrap_or("")
            ),
            Some("docs/raw/product-guide/index.md".into()),
        ), &mut findings);

        // ── Navigation (PN1-PN4) 20% ─────────────────────────────────────

        let mut pn = Tally::default();

        let subdirs = top_level_subdirs(&guide_dir);
        let index_low = files.iter()
            .find(|f| f.rel_path == Path::new("index.md"))
            .map(|f| f.content_low.clone())
            .unwrap_or_default();
        // Prose naturally writes "Audit Guide", not the literal directory
        // slug "audit-guide" — accept either so a written-out subsection
        // name in the TOC isn't flagged as missing.
        let missing_subdirs: Vec<&String> = subdirs.iter()
            .filter(|d| {
                let slug = d.to_lowercase();
                let spaced = slug.replace('-', " ");
                !index_low.contains(&slug) && !index_low.contains(&spaced)
            })
            .collect();
        pn.check(missing_subdirs.is_empty(), || finding(
            "PN1", Severity::Suggestion,
            format!("index.md doesn't mention these existing subsections: {:?}", missing_subdirs),
            Some("docs/raw/product-guide/index.md".into()),
        ), &mut findings);

        let broken_related = broken_related_links(&files, &guide_dir);
        pn.check(broken_related.is_empty(), || finding(
            "PN2", Severity::Warning,
            format!("Related/See Also links that don't resolve: {:?}", broken_related),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let broken_links = broken_link_patterns(&files, &guide_dir);
        pn.check(broken_links.is_empty(), || finding(
            "PN3", Severity::Warning,
            format!("{} broken internal link(s) found: {:?}", broken_links.len(), broken_links),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let heading_gaps = heading_hierarchy_gaps(&files);
        pn.check(heading_gaps.is_empty(), || finding(
            "PN4", Severity::Warning,
            format!("Heading levels skip a level (e.g. H1 to H3) in: {:?}", heading_gaps),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        // ── Quality (PQ1-PQ5) 25% ────────────────────────────────────────

        let mut pq = Tally::default();

        let placeholder_signals = &["todo", "tbd", "tbc", "coming soon", "under construction", "wip"];
        let placeholders: Vec<&str> = placeholder_signals.iter().filter(|s| all_low.contains(**s)).copied().collect();
        pq.check(placeholders.is_empty(), || finding(
            "PQ1", Severity::Warning,
            format!("Placeholder text found: {:?}", placeholders),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let empty_sections = empty_section_bodies(&files);
        pq.check(empty_sections.is_empty(), || finding(
            "PQ2", Severity::Error,
            format!("Headings with no body content: {:?}", empty_sections),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let duplicate_signals = duplicate_content_patterns(&files);
        pq.check(duplicate_signals.is_empty(), || finding(
            "PQ3", Severity::Suggestion,
            format!("Duplicated content found across pages: {:?}", duplicate_signals),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let short_pages = short_pages(&files);
        pq.check(short_pages.is_empty(), || finding(
            "PQ4", Severity::Warning,
            format!("Pages under 50 words: {:?}", short_pages),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let stale_versions = stale_version_refs(&all_content, env!("CARGO_PKG_VERSION"));
        pq.check(stale_versions.is_empty(), || finding(
            "PQ5", Severity::Suggestion,
            format!("Version references that don't match the current version ({}): {:?}", env!("CARGO_PKG_VERSION"), stale_versions),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        // ── Accuracy (PA1-PA7) 25% ───────────────────────────────────────

        let mut pa = Tally::default();

        let total_inventory = cli_commands.len() + mcp_methods.len() + config_fields.len();
        let total_missing = missing_commands.len() + missing_mcp.len() + missing_config.len();
        let aggregate_covered = total_inventory.saturating_sub(total_missing) as f64 / total_inventory.max(1) as f64;
        pa.check(aggregate_covered >= 0.9, || finding(
            "PA1", Severity::Error,
            format!("Aggregate code-inventory coverage {:.0}% is below 90%", aggregate_covered * 100.0),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let vision_content = fs::read_to_string(ctx.project_root.join("docs/raw/vision/vision.md")).unwrap_or_default();
        let vision_purpose = section_body(&vision_content, "purpose").unwrap_or(vision_content);
        let pa2_ok = keyword_overlap(&vision_purpose.to_lowercase(), &all_low) >= 0.3;
        pa.check(pa2_ok, || finding(
            "PA2", Severity::Suggestion,
            "Guide's overview doesn't clearly reflect the Vision doc's stated purpose".to_string(),
            Some("docs/raw/vision/vision.md".into()),
        ), &mut findings);

        let all_headings_low: String = files.iter()
            .flat_map(|f| f.content.lines())
            .filter(|l| l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .to_lowercase();
        let missing_feature_headings = missing_by_mention(&feature_titles, &all_headings_low);
        pa.check(missing_feature_headings.is_empty(), || finding(
            "PA3", Severity::Suggestion,
            format!("Features not explained under a dedicated heading: {:?}", missing_feature_headings),
            Some("docs/raw/feature/".into()),
        ), &mut findings);

        let engineering_titles = titled_docs(&ctx.project_root.join("docs/raw/engineering"));
        let missing_engineering = missing_by_mention(&engineering_titles, &all_low);
        pa.check(missing_engineering.is_empty(), || finding(
            "PA4", Severity::Suggestion,
            format!("Engineering topics not reflected in the guide: {:?}", missing_engineering),
            Some("docs/raw/engineering/".into()),
        ), &mut findings);

        let internal_signals = &["impl ", "fn ", "struct ", "trait ", "crate::"];
        let internal_found = find_unnegated(&all_low, internal_signals);
        pa.check(internal_found.len() <= 1, || finding(
            "PA5", Severity::Warning,
            format!("Internal implementation details leaked into the public guide: {:?}", internal_found),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        let declared_deps = &ctx.config.knowledge.dependencies;
        let missing_deps: Vec<&String> = declared_deps.iter().filter(|d| !all_low.contains(&d.to_lowercase())).collect();
        pa.check(missing_deps.is_empty(), || finding(
            "PA6", Severity::Suggestion,
            format!("Declared dependencies not documented in the guide: {:?}", missing_deps),
            Some("docs/raw/product-guide/".into()),
        ), &mut findings);

        // `Pipeline::run` has no registry/DB access, so `ctx.repository_metadata`
        // is whatever the caller (`run_pipeline`/`run_pipeline_with_id`) chose
        // to fetch and attach — see `PipelineContext::with_repository_metadata`.
        if ctx.repository_metadata.is_empty() {
            // Nothing compiled yet — nothing to compare against, and that's
            // not the guide author's fault, so don't penalize the score.
            pa.total += 1;
            pa.passed += 1;
            findings.push(finding(
                "PA7", Severity::Suggestion,
                "Repository metadata has not been populated yet — run compile to enable this check.".to_string(),
                None,
            ));
        } else {
            let mismatches = repository_metadata_mismatches(&ctx.repository_metadata, &ctx.config, &ctx.project_root);
            pa.check(mismatches.is_empty(), || finding(
                "PA7", Severity::Warning,
                format!("Repository metadata is stale relative to samgraha.toml: {:?}", mismatches),
                None,
            ), &mut findings);
        }

        let pc_score = pc.score();
        let pn_score = pn.score();
        let pq_score = pq.score();
        let pa_score = pa.score();

        cat_scores.insert("Coverage".into(), pc_score);
        cat_scores.insert("Navigation".into(), pn_score);
        cat_scores.insert("Quality".into(), pq_score);
        cat_scores.insert("Accuracy".into(), pa_score);

        // Weighted overall: 30/20/25/25
        let overall = pc_score * 0.30 + pn_score * 0.20 + pq_score * 0.25 + pa_score * 0.25;

        let mut report = make_report(PipelineKind::Help, overall, cat_scores, findings);
        report.metadata.insert("doc_count".into(), files.len().to_string());
        report.metadata.insert("engineering_readiness".into(), readiness_label(overall, pc_score, pn_score));
        report
    }
}

/// Pass/fail accumulator shared by all four categories — keeps the
/// increment-then-check pattern in one place instead of repeating
/// `total += 1; if ... { passed += 1 } else { push finding }` by hand.
#[derive(Default)]
struct Tally {
    passed: u32,
    total: u32,
}

impl Tally {
    fn check(&mut self, ok: bool, make_finding: impl FnOnce() -> AuditFinding, findings: &mut Vec<AuditFinding>) {
        self.total += 1;
        if ok {
            self.passed += 1;
        } else {
            findings.push(make_finding());
        }
    }

    fn score(&self) -> f64 {
        if self.total > 0 { (self.passed as f64 / self.total as f64) * 100.0 } else { 100.0 }
    }
}

struct GuideFile {
    /// Path relative to the guide root, e.g. `commands/audit.md`.
    rel_path: PathBuf,
    /// Code fences replaced with a `[code example]` placeholder line — real
    /// content (word count, section bodies, headings) is preserved, so a
    /// section whose only content is a code sample isn't mistaken for empty,
    /// while shell comments (`# ...`) inside the fence can't be misread as
    /// markdown headings. Used for every structural check.
    content: String,
    /// Code fences fully deleted and lowercased — used for keyword/mention
    /// scans, where a code sample's identifiers must not produce a false
    /// match (e.g. `impl Foo` inside an example triggering the PA5 leakage
    /// scan, or a diagram's ASCII art triggering PQ1's placeholder scan).
    content_low: String,
}

fn walk_md_files(dir: &Path, base: &Path, out: &mut Vec<GuideFile>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            walk_md_files(&path, base, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            if let Ok(raw) = fs::read_to_string(&path) {
                let rel_path = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
                out.push(GuideFile {
                    rel_path,
                    content: mask_code_fences(&raw),
                    content_low: strip_code_fences(&raw).to_lowercase(),
                });
            }
        }
    }
}

/// Like `strip_code_fences`, but replaces fenced content with a placeholder
/// line instead of deleting it — see `GuideFile::content` doc comment.
fn mask_code_fences(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            if !in_fence {
                out.push_str("[code example]\n");
            }
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn top_level_subdirs(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .map(|entries| {
            entries.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Which of `items` (case-insensitive) have no file whose stem matches under
/// `guide_dir/subdir/` — used by PC1 (commands/), PC3 (configuration/), and
/// PC4 (documentation-guide/), which all follow the same one-file-per-item
/// convention.
fn missing_by_stem(files: &[GuideFile], subdir: &str, items: &[&str]) -> Vec<String> {
    let stems: Vec<String> = files.iter()
        .filter(|f| f.rel_path.starts_with(subdir))
        .filter_map(|f| f.rel_path.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .collect();
    items.iter()
        .filter(|item| !stems.iter().any(|s| s == &item.to_lowercase()))
        .map(|s| s.to_string())
        .collect()
}

/// Read every `.md` file directly under `dir` (non-recursive — these are
/// flat doc corpora) and extract a title: the first `# ` heading, or the
/// filename with hyphens turned to spaces if there's no heading.
fn titled_docs(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
    entries.filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let content = fs::read_to_string(e.path()).ok()?;
            let title = content.lines()
                .find_map(|l| l.trim().strip_prefix("# ").map(str::trim))
                .map(str::to_string);
            Some(title.unwrap_or_else(|| {
                e.path().file_stem().unwrap_or_default().to_string_lossy().replace('-', " ")
            }))
        })
        .collect()
}

/// Which `titles` don't appear (as a lowercase substring) anywhere in
/// `haystack_low` — used by PC5/PC6/PA3/PA4's cross-reference checks.
fn missing_by_mention(titles: &[String], haystack_low: &str) -> Vec<String> {
    titles.iter()
        .filter(|t| !haystack_low.contains(&t.to_lowercase()))
        .cloned()
        .collect()
}

/// Extract the body text of a `## <name>` (or any level) section, matched
/// case-insensitively, up to the next heading of the same or higher level.
fn section_body(content: &str, name: &str) -> Option<String> {
    let name_low = name.to_lowercase();
    let mut in_section = false;
    let mut section_level = 0usize;
    let mut body = String::new();
    for line in content.lines() {
        if let Some(level) = heading_level(line) {
            let title = line.trim_start_matches('#').trim().to_lowercase();
            if in_section && level <= section_level {
                break;
            }
            if !in_section && title == name_low {
                in_section = true;
                section_level = level;
                continue;
            }
        } else if in_section {
            body.push_str(line);
            body.push('\n');
        }
    }
    if body.trim().is_empty() { None } else { Some(body) }
}

/// Fraction of `needle_low`'s significant (>3 char) words that also appear in
/// `haystack_low` — a cheap keyword-overlap heuristic for PA2's "Vision
/// purpose reflected in guide overview" cross-reference.
fn keyword_overlap(needle_low: &str, haystack_low: &str) -> f64 {
    let words: Vec<&str> = needle_low.split_whitespace().filter(|w| w.len() > 3).collect();
    if words.is_empty() { return 1.0; }
    let matched = words.iter().filter(|w| haystack_low.contains(**w)).count();
    matched as f64 / words.len() as f64
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') { return None; }
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    let rest = &trimmed[level..];
    if level == 0 || level > 6 || !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    Some(level)
}

struct Section {
    level: usize,
    body: String,
}

fn parse_sections(content: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current: Option<Section> = None;
    for line in content.lines() {
        if let Some(level) = heading_level(line) {
            if let Some(sec) = current.take() {
                sections.push(sec);
            }
            current = Some(Section { level, body: String::new() });
        } else if let Some(sec) = current.as_mut() {
            sec.body.push_str(line);
            sec.body.push('\n');
        }
    }
    if let Some(sec) = current.take() {
        sections.push(sec);
    }
    sections
}

/// PQ2: headings with no body content before the next heading, unless that
/// next heading is a child (deeper level) — a `# Title` immediately
/// followed by `## Section` deliberately has no direct body of its own, its
/// content is delegated to the child section, and isn't "empty".
fn empty_section_bodies(files: &[GuideFile]) -> Vec<String> {
    let mut problems = Vec::new();
    for f in files {
        let sections = parse_sections(&f.content);
        for (i, sec) in sections.iter().enumerate() {
            let next_is_child = sections.get(i + 1).is_some_and(|n| n.level > sec.level);
            if sec.body.trim().is_empty() && !next_is_child {
                problems.push(f.rel_path.to_string_lossy().to_string());
                break;
            }
        }
    }
    problems
}

/// PN4: a heading level that skips over its parent (e.g. `###` appearing
/// with no preceding `##` under the current `#`) — the doc's heading
/// hierarchy, and therefore any TOC generated from it, would be inaccurate.
fn heading_hierarchy_gaps(files: &[GuideFile]) -> Vec<String> {
    let mut problems = Vec::new();
    for f in files {
        let mut prev_level = 0usize;
        for line in f.content.lines() {
            if let Some(level) = heading_level(line) {
                if prev_level > 0 && level > prev_level + 1 {
                    problems.push(f.rel_path.to_string_lossy().to_string());
                    break;
                }
                prev_level = level;
            }
        }
    }
    problems
}

/// PQ4: pages under 50 words, listed by relative path.
fn short_pages(files: &[GuideFile]) -> Vec<String> {
    files.iter()
        .filter(|f| f.content.split_whitespace().count() < 50)
        .map(|f| f.rel_path.to_string_lossy().to_string())
        .collect()
}

fn find_unnegated(haystack_low: &str, keywords: &[&str]) -> Vec<String> {
    crate::pipeline::find_unnegated_keywords(haystack_low, keywords)
}

/// PN2: links inside a "Related"/"See Also"/"References" section that don't
/// resolve to an existing file.
fn broken_related_links(files: &[GuideFile], guide_dir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for f in files {
        let related = ["related", "see also", "references"].iter()
            .find_map(|name| section_body(&f.content, name));
        let Some(body) = related else { continue };
        let file_dir = guide_dir.join(f.rel_path.parent().unwrap_or(Path::new("")));
        for target in extract_link_targets(&body) {
            if !link_resolves(&target, &file_dir, guide_dir) {
                problems.push(format!("{}: {}", f.rel_path.display(), target));
            }
        }
    }
    problems
}

/// PN3: broken links anywhere in the guide (general scan, not scoped to any
/// particular section).
fn broken_link_patterns(files: &[GuideFile], guide_dir: &Path) -> Vec<String> {
    let mut problems = Vec::new();
    for f in files {
        let file_dir = guide_dir.join(f.rel_path.parent().unwrap_or(Path::new("")));
        for line in f.content.lines() {
            for token in line.split_whitespace() {
                if (token.starts_with("http://") || token.starts_with("https://"))
                    && (token.contains("localhost") || token.contains("127.0.0.1"))
                {
                    problems.push(format!("{}: local URL {}", f.rel_path.display(), token));
                }
            }
        }
        for target in extract_link_targets(&f.content) {
            if !link_resolves(&target, &file_dir, guide_dir) {
                problems.push(format!("{}: broken link {}", f.rel_path.display(), target));
            }
        }
    }
    problems
}

/// Extract every markdown link target `[text](target)` from `content`.
/// Whitespace-token splitting can't find these (no space before `(`), so
/// this scans each line directly for the `](...)` shape instead.
fn extract_link_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in content.lines() {
        let mut rest = line;
        while let Some(open) = rest.find("](") {
            let after = &rest[open + 2..];
            let Some(close) = after.find(')') else { break };
            targets.push(after[..close].to_string());
            rest = &after[close + 1..];
        }
    }
    targets
}

fn link_resolves(target: &str, file_dir: &Path, guide_dir: &Path) -> bool {
    if target.starts_with("http") || target.starts_with('#') || target.is_empty() {
        return true;
    }
    let path_part = target.split('#').next().unwrap_or(target);
    if path_part.is_empty() {
        return true;
    }
    let candidate = if let Some(stripped) = path_part.strip_prefix('/') {
        guide_dir.join(stripped)
    } else {
        file_dir.join(path_part)
    };
    candidate.exists() || guide_dir.join(path_part).exists()
}

/// Heuristic duplicate-content detector: flags paragraphs (blank-line
/// separated blocks, normalized) repeated verbatim across 2+ distinct pages.
/// Embedding-based near-duplicate detection is deferred (see Future Scope).
fn duplicate_content_patterns(files: &[GuideFile]) -> Vec<String> {
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut problems = Vec::new();
    for f in files {
        let filename = f.rel_path.to_string_lossy().to_string();
        for para in f.content.split("\n\n") {
            let normalized: String = para.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
            if normalized.len() < 40 {
                continue;
            }
            if let Some(first_file) = seen.get(&normalized) {
                if first_file != &filename {
                    problems.push(format!("{} duplicates content from {}", filename, first_file));
                }
            } else {
                seen.insert(normalized, filename.clone());
            }
        }
    }
    problems
}

/// PQ5: `v<major>.<minor>[.<patch>]` tokens whose major.minor doesn't match
/// the current crate version — a manual scan rather than a `regex` dependency
/// (ladder: stdlib covers this, no need for the crate).
fn stale_version_refs(content: &str, current_version: &str) -> Vec<String> {
    let current_mm: Vec<&str> = current_version.split('.').take(2).collect();
    let mut refs = Vec::new();
    for tok in content.split_whitespace() {
        let t = tok.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
        let Some(rest) = t.strip_prefix('v').or_else(|| t.strip_prefix('V')) else { continue };
        let parts: Vec<&str> = rest.split('.').collect();
        let looks_like_version = parts.len() >= 2
            && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()));
        if looks_like_version && parts[..2] != current_mm[..] {
            refs.push(t.to_string());
        }
    }
    refs
}

/// PA7: field-by-field comparison of the compiled `repository_metadata`
/// snapshot against the live `samgraha.toml`/project root — catches the
/// table going stale relative to a config edit made without recompiling.
fn repository_metadata_mismatches(
    meta: &HashMap<String, String>,
    config: &common::config::SamgrahaConfig,
    project_root: &Path,
) -> Vec<String> {
    let mut mismatches = Vec::new();

    if let (Some(stored), Some(live)) = (meta.get("repo_name"), config.repository.name.as_deref()) {
        if stored != live {
            mismatches.push(format!("repo_name: table has '{}', samgraha.toml has '{}'", stored, live));
        }
    }
    if let Some(stored) = meta.get("source_dir") {
        if stored != &config.repository.implementation.dir {
            mismatches.push(format!(
                "source_dir: table has '{}', samgraha.toml has '{}'",
                stored, config.repository.implementation.dir
            ));
        }
    }
    if let Some(stored) = meta.get("repo_root") {
        let live_root = project_root.to_string_lossy().to_string();
        if stored != &live_root {
            mismatches.push(format!("repo_root: table has '{}', live root is '{}'", stored, live_root));
        }
    }

    mismatches
}

fn readiness_label(overall: f64, pc: f64, pn: f64) -> String {
    if overall >= 90.0 && pc >= 80.0 && pn >= 80.0 {
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
            let root = std::env::temp_dir()
                .join(format!("samgraha-help-test-{}-{}", std::process::id(), id));
            std::fs::create_dir_all(root.join("docs/raw/help")).unwrap();
            Self { root }
        }

        /// Write a file at `rel_path` under `docs/raw/product-guide/`, creating parent
        /// subdirectories as needed (e.g. `commands/audit.md`).
        fn with_file(self, rel_path: &str, content: &str) -> Self {
            let path = self.root.join("docs/raw/help").join(rel_path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
            self
        }

        /// Write a file at `rel_path` under the project root directly (e.g.
        /// `docs/raw/feature/x.md`) — for cross-doc checks (PC5/PC6/PA2-4)
        /// that read other domains, not the guide itself.
        fn with_project_file(self, rel_path: &str, content: &str) -> Self {
            let path = self.root.join(rel_path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
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
    fn score_is_within_bounds() {
        let proj = TempProject::new();
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.score >= 0.0 && report.score <= 100.0);
    }

    #[test]
    fn category_scores_are_populated() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\nCommands, MCP, config.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.categories.contains_key("Coverage"));
        assert!(report.categories.contains_key("Navigation"));
        assert!(report.categories.contains_key("Quality"));
        assert!(report.categories.contains_key("Accuracy"));
    }

    #[test]
    fn pc1_flags_missing_command_docs() {
        let proj = TempProject::new();
        let report = HelpPipeline.run(&proj.ctx());
        let pc1 = report.findings.iter().find(|f| f.check_id == "PC1").unwrap();
        assert_eq!(pc1.severity, Severity::Error);
    }

    #[test]
    fn pc1_passes_when_every_command_has_a_doc() {
        let mut proj = TempProject::new();
        for cmd in schemas::code_inventory::CLI_COMMANDS {
            proj = proj.with_file(&format!("commands/{}.md", cmd), "# Command\n\nDocs here.");
        }
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PC1"));
    }

    #[test]
    fn pc2_flags_missing_tools_md() {
        let proj = TempProject::new();
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PC2"));
    }

    #[test]
    fn pc2_passes_when_tools_md_lists_every_method() {
        let body = schemas::code_inventory::MCP_METHODS.join("\n- ");
        let proj = TempProject::new().with_file("mcp-guide/tools.md", &format!("# Tools\n\n- {}", body));
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PC2"));
    }

    #[test]
    fn pq2_flags_heading_with_no_body() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\n## Empty Section\n## Next Section\n\nSome real content.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PQ2"));
    }

    #[test]
    fn pq2_passes_when_every_heading_has_body() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\n## Section\n\nReal content here.\n");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PQ2"));
    }

    #[test]
    fn pq2_does_not_flag_a_section_whose_only_content_is_a_code_example() {
        // regression: found against the real repo corpus — deleting fenced
        // code (the shared `strip_code_fences`) made a section whose entire
        // body is a command example look empty. A section's content being
        // "just a code sample" isn't the same as having no content.
        let proj = TempProject::new().with_file(
            "index.md",
            "# Guide\n\n## Synopsis\n\n```bash\nsamgraha audit --pipeline help\n```\n\n## Next\n\nMore text.\n",
        );
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PQ2"));
    }

    #[test]
    fn pq2_does_not_flag_a_deep_heading_with_body() {
        // regression: PQ2 must walk the whole file, not just the first
        // section — a nested heading's body shouldn't be mistaken for empty
        // just because it isn't the top-level section.
        let proj = TempProject::new().with_file(
            "index.md",
            "# Guide\n\nIntro text.\n\n## Section A\n\n### Subsection\n\nReal content.\n",
        );
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PQ2"));
    }

    #[test]
    fn resolves_product_guide_directory_when_present() {
        // The rename's whole point: `docs/raw/product-guide/` must be read
        // preferentially when it exists, not just as a fallback.
        let proj = TempProject::new()
            .with_project_file("docs/raw/product-guide/index.md", "# Guide\n\nContent lives here now.");
        let report = HelpPipeline.run(&proj.ctx());
        assert_eq!(report.metadata.get("doc_count").map(String::as_str), Some("1"));
    }

    #[test]
    fn falls_back_to_help_directory_when_product_guide_absent() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\nStill the old path.");
        let report = HelpPipeline.run(&proj.ctx());
        assert_eq!(report.metadata.get("doc_count").map(String::as_str), Some("1"));
    }

    #[test]
    fn pq4_flags_short_page() {
        let proj = TempProject::new().with_file("short.md", "# Short");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PQ4"));
    }

    #[test]
    fn pq2_flags_link_to_nonexistent_file() {
        let proj = TempProject::new()
            .with_file("index.md", "# Guide\n\nSee [other page](does-not-exist.md) for more.");
        let report = HelpPipeline.run(&proj.ctx());
        let pn3 = report.findings.iter().find(|f| f.check_id == "PN3");
        assert!(pn3.is_some(), "expected PN3 to flag the broken link: {:?}", report.findings);
    }

    #[test]
    fn pq2_passes_when_link_target_exists() {
        let proj = TempProject::new()
            .with_file("index.md", "# Guide\n\nSee [other page](other.md) for more.")
            .with_file("other.md", "# Other\n\nContent here.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PN3"));
    }

    #[test]
    fn pq3_flags_duplicated_paragraph_across_pages() {
        let shared = "This exact paragraph appears verbatim on two different pages, which is long enough to clear the trivial-length filter.";
        let proj = TempProject::new()
            .with_file("a.md", &format!("# Page A\n\n{}", shared))
            .with_file("b.md", &format!("# Page B\n\n{}", shared));
        let report = HelpPipeline.run(&proj.ctx());
        let pq3 = report.findings.iter().find(|f| f.check_id == "PQ3");
        assert!(pq3.is_some(), "expected PQ3 to flag duplicated content: {:?}", report.findings);
    }

    #[test]
    fn pn1_flags_subdir_not_mentioned_in_index() {
        let proj = TempProject::new()
            .with_file("index.md", "# Guide\n\nNo mention of subsections here.")
            .with_file("commands/audit.md", "# Audit\n\nDocs.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PN1"));
    }

    #[test]
    fn pn4_flags_skipped_heading_level() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\n### Skipped H2\n\nContent.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PN4"));
    }

    #[test]
    fn pa7_passes_informationally_when_metadata_not_yet_populated() {
        let proj = TempProject::new();
        let report = HelpPipeline.run(&proj.ctx());
        let pa7 = report.findings.iter().find(|f| f.check_id == "PA7").unwrap();
        assert_eq!(pa7.severity, Severity::Suggestion);
    }

    #[test]
    fn pa7_flags_stale_metadata() {
        let proj = TempProject::new();
        let mut meta = HashMap::new();
        meta.insert("repo_name".to_string(), "old-name".to_string());
        let mut config = common::config::SamgrahaConfig::default();
        config.repository.name = Some("new-name".to_string());
        let ctx = PipelineContext::new(proj.root.clone(), config).with_repository_metadata(meta);
        let report = HelpPipeline.run(&ctx);
        let pa7 = report.findings.iter().find(|f| f.check_id == "PA7").unwrap();
        assert_eq!(pa7.severity, Severity::Warning);
    }

    #[test]
    fn pa7_passes_when_metadata_matches() {
        let proj = TempProject::new();
        let mut meta = HashMap::new();
        meta.insert("repo_name".to_string(), "same-name".to_string());
        let mut config = common::config::SamgrahaConfig::default();
        config.repository.name = Some("same-name".to_string());
        let ctx = PipelineContext::new(proj.root.clone(), config).with_repository_metadata(meta);
        let report = HelpPipeline.run(&ctx);
        assert!(!report.findings.iter().any(|f| f.check_id == "PA7"));
    }

    #[test]
    fn pa5_flags_internal_implementation_details() {
        let proj = TempProject::new().with_file(
            "index.md",
            "# Guide\n\nSee `impl Pipeline for HelpPipeline` and `struct Foo` in crate::pipelines for details.",
        );
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PA5"));
    }

    #[test]
    fn pa5_passes_on_public_facing_content() {
        let proj = TempProject::new().with_file("index.md", "# Guide\n\nRun `samgraha audit` to check your documentation.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PA5"));
    }

    #[test]
    fn pc5_flags_feature_not_mentioned_anywhere_in_guide() {
        let proj = TempProject::new()
            .with_project_file("docs/raw/feature/widget-builder.md", "# Widget Builder\n\nBuilds widgets.")
            .with_file("index.md", "# Guide\n\nUnrelated content only.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(report.findings.iter().any(|f| f.check_id == "PC5"));
    }

    #[test]
    fn pc5_passes_when_feature_title_is_mentioned() {
        let proj = TempProject::new()
            .with_project_file("docs/raw/feature/widget-builder.md", "# Widget Builder\n\nBuilds widgets.")
            .with_file("index.md", "# Guide\n\nLearn about the Widget Builder and how it works.");
        let report = HelpPipeline.run(&proj.ctx());
        assert!(!report.findings.iter().any(|f| f.check_id == "PC5"));
    }

    #[test]
    fn recursive_walk_finds_files_in_subdirectories() {
        // regression: the original implementation only read the guide
        // directory's immediate children, so the ~100 files actually living
        // in commands/, concepts/, mcp-guide/, etc. were invisible to every
        // check.
        let proj = TempProject::new().with_file("commands/audit.md", "# Audit\n\nRun audits, contains at least fifty words of real filler text so this page clears the short-page threshold and does not trip PQ4 while we are only testing whether the recursive walk itself finds nested files at all in this temp fixture directory tree structure.");
        let report = HelpPipeline.run(&proj.ctx());
        assert_eq!(report.metadata.get("doc_count").map(String::as_str), Some("1"));
    }
}
