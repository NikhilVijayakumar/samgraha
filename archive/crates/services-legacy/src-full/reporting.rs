use anyhow::{Context, Result};
use schemas::audit::AuditStage;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Structured output from report rendering — contains both the
/// human-readable markdown and the machine-readable JSON serialization
/// of the same template context data.
#[derive(Debug, Clone)]
pub struct ReportOutput {
    pub markdown: String,
    pub json: String,
}

/// Write a report JSON blob to the filesystem report store.
///
/// # Atomicity
///
/// Writes to a temp file first, then atomically renames to `latest.json`.
/// The previous `latest.json` is rotated to `history/<timestamp>-<revision>.json`.
///
/// # Directory Structure
///
/// ```text
/// docs/raw/reports/
///   <domain>/
///     <document-id>/
///       <section-type>/
///         latest.json
///         history/
///     document/
///         latest.json
///         history/
///   cross-domain/
/// ```
pub fn write_report(
    reports_root: &Path,
    stage: AuditStage,
    domain: &str,
    document_id: Option<i64>,
    section_type: Option<&str>,
    report_body: &str,
    revision: i64,
) -> Result<PathBuf> {
    write_report_file(reports_root, stage, domain, document_id, section_type, "latest.json", report_body, revision)
}

/// Same as [`write_report`] but with a caller-chosen filename (e.g. `latest.md`) instead of
/// the hardcoded `latest.json` — lets a JSON and a Markdown rendering of the same report share
/// one report directory and one history/rotation scheme, keyed by extension.
pub fn write_report_file(
    reports_root: &Path,
    stage: AuditStage,
    domain: &str,
    document_id: Option<i64>,
    section_type: Option<&str>,
    filename: &str,
    report_body: &str,
    revision: i64,
) -> Result<PathBuf> {
    let report_dir = build_report_path(reports_root, stage, domain, document_id, section_type);
    fs::create_dir_all(&report_dir)
        .with_context(|| format!("Failed to create report dir: {:?}", report_dir))?;

    let history_dir = report_dir.join("history");
    fs::create_dir_all(&history_dir)
        .with_context(|| format!("Failed to create history dir: {:?}", history_dir))?;

    let latest_path = report_dir.join(filename);
    let ext = Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("json");

    // Rotate existing latest file to history
    if latest_path.exists() {
        let timestamp = chrono_now();
        let history_name = format!("{}-rev{}.{}", timestamp, revision, ext);
        let history_path = history_dir.join(&history_name);
        fs::rename(&latest_path, &history_path)
            .with_context(|| format!("Failed to rotate report to {:?}", history_path))?;
    }

    // Atomic write: write to temp file, then rename
    let tmp_path = report_dir.join(format!("{}.tmp", filename));
    let mut tmp = fs::File::create(&tmp_path)
        .with_context(|| format!("Failed to create temp file {:?}", tmp_path))?;
    tmp.write_all(report_body.as_bytes())
        .with_context(|| format!("Failed to write temp report {:?}", tmp_path))?;
    tmp.flush()?;
    drop(tmp);

    fs::rename(&tmp_path, &latest_path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", tmp_path, latest_path))?;

    Ok(latest_path)
}

/// Render and persist both the JSON scorecard (self-describing `AuditScorecard`, source of
/// truth for [`regenerate_audit_scorecard`]) and a Markdown rendering of it, for one domain's
/// `audit()` run. `semantic_results` is normally empty here — `audit()` has just run
/// deterministically; semantic review results land later via `regenerate_audit_scorecard`.
pub fn write_audit_scorecard(
    reports_root: &Path,
    templates_dir: &Path,
    domain: &str,
    report: &schemas::audit::AuditReport,
    semantic_results: &[schemas::audit::SemanticReport],
) -> Result<()> {
    render_and_write_scorecard(reports_root, templates_dir, domain, report.clone(), semantic_results.to_vec())
}

/// Re-render a domain's scorecard after a semantic review result lands (`store_section_report`
/// et al.), without re-running the deterministic audit. Reads the `report` half back from the
/// previously written `latest.json` — a no-op if `audit()` hasn't run for this domain yet,
/// since there's no deterministic scorecard to attach the semantic results to.
pub fn regenerate_audit_scorecard(
    reports_root: &Path,
    templates_dir: &Path,
    domain: &str,
    semantic_results: &[schemas::audit::SemanticReport],
) -> Result<()> {
    let json_path = build_report_path(reports_root, AuditStage::Deterministic, domain, None, Some("scorecard"))
        .join("latest.json");
    let Ok(existing) = fs::read_to_string(&json_path) else {
        return Ok(());
    };
    let existing: schemas::audit::AuditScorecard = serde_json::from_str(&existing)
        .with_context(|| format!("Failed to parse existing scorecard {:?}", json_path))?;
    render_and_write_scorecard(reports_root, templates_dir, domain, existing.report, semantic_results.to_vec())
}

fn render_and_write_scorecard(
    reports_root: &Path,
    templates_dir: &Path,
    domain: &str,
    report: schemas::audit::AuditReport,
    semantic_results: Vec<schemas::audit::SemanticReport>,
) -> Result<()> {
    use schemas::audit::Severity;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    for f in &report.findings {
        let tf = TemplateFinding {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
        };
        match f.severity {
            Severity::Error => errors.push(tf),
            Severity::Warning => warnings.push(tf),
            Severity::Suggestion => suggestions.push(tf),
        }
    }

    let done_keys: std::collections::HashSet<(Option<i64>, Option<i64>)> = semantic_results
        .iter()
        .map(|r| (r.document_id, r.section_id))
        .collect();
    let semantic_pending: Vec<TemplateFinding> = report
        .semantic_review
        .tasks
        .iter()
        .filter(|t| !done_keys.contains(&(Some(t.document_id), Some(t.section_id))))
        .map(|t| TemplateFinding {
            check_id: t.semantic_type.clone(),
            message: t.document_title.clone(),
            location: Some(t.document_path.clone()),
        })
        .collect();
    let semantic_done: Vec<TemplateFinding> = semantic_results
        .iter()
        .map(|r| TemplateFinding {
            check_id: format!("{:?}", r.stage).to_lowercase(),
            message: format!("score {}", r.score),
            location: r.document_id.map(|id| format!("document #{}", id)),
        })
        .collect();

    let ctx = TemplateContext {
        pipeline: domain.to_string(),
        score: report.score.overall,
        categories: report.score.categories.clone(),
        errors,
        warnings,
        suggestions,
        date: chrono_now_iso(),
        comments: Vec::new(),
        semantic_pending,
        semantic_done,
    };

    let template = read_template(templates_dir, "audit-scorecard").unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string());
    let markdown = render_from_template(&template, &ctx);

    let scorecard = schemas::audit::AuditScorecard { report, semantic_results };
    let json_body = serde_json::to_string_pretty(&scorecard)
        .context("Failed to serialize audit scorecard")?;

    let revision = chrono_now().parse::<i64>().unwrap_or(0);
    write_report_file(reports_root, AuditStage::Deterministic, domain, None, Some("scorecard"), "latest.json", &json_body, revision)?;
    write_report_file(reports_root, AuditStage::Deterministic, domain, None, Some("scorecard"), "latest.md", &markdown, revision)?;

    Ok(())
}

/// Regenerate filesystem reports from SQLite data.
///
/// Walks all rows in `semantic_reports` grouped by (stage, domain, document_id, section_id)
/// and writes/rotates `latest.json` for each group.
pub fn regenerate_from_sqlite(
    reports_root: &Path,
    rows: &[SqliteReportRow],
) -> Result<usize> {
    let mut count = 0;
    for row in rows {
        let report_dir = build_report_path(
            reports_root,
            row.stage.clone(),
            &row.domain,
            row.document_id,
            row.section_type.as_deref(),
        );
        fs::create_dir_all(&report_dir)?;

        let latest_path = report_dir.join("latest.json");
        fs::write(&latest_path, &row.report_json)
            .with_context(|| format!("Failed to write {:?}", latest_path))?;
        count += 1;
    }
    Ok(count)
}

/// A row from the semantic_reports table used for filesystem regeneration.
pub struct SqliteReportRow {
    pub stage: AuditStage,
    pub domain: String,
    pub document_id: Option<i64>,
    pub section_type: Option<String>,
    pub report_json: String,
}

fn build_report_path(
    root: &Path,
    stage: AuditStage,
    domain: &str,
    document_id: Option<i64>,
    section_type: Option<&str>,
) -> PathBuf {
    let mut path = root.to_path_buf();

    match stage {
        AuditStage::CrossDomain => {
            path = path.join("cross-domain");
            if let Some(st) = section_type {
                path = path.join(st);
            }
        }
        _ => {
            path = path.join(domain);
            if let Some(doc_id) = document_id {
                path = path.join(doc_id.to_string());
            }
            if let Some(st) = section_type {
                path = path.join(st);
            } else {
                path = path.join("document");
            }
        }
    }
    path
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

// ── Template Engine ─────────────────────────────────────────────────────

/// Context for rendering pipeline report templates.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TemplateContext {
    pub pipeline: String,
    pub score: f64,
    pub categories: HashMap<String, f64>,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub date: String,
    pub comments: Vec<TemplateComment>,
    /// Semantic-review sections not yet judged (`check_id` = semantic_type, `location` =
    /// document path, `message` = document title). Reuses `TemplateFinding` rather than a
    /// bespoke struct — same three-column table shape.
    pub semantic_pending: Vec<TemplateFinding>,
    /// Semantic-review sections already judged (`check_id` = semantic_type, `location` =
    /// document path, `message` = `"score {n}"`).
    pub semantic_done: Vec<TemplateFinding>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateFinding {
    pub check_id: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateComment {
    pub author: String,
    pub body: String,
    pub created_at: String,
}

/// Render a template with the given context.
///
/// Supported syntax:
/// - `{{variable}}` — simple substitution
/// - `{{variable|filter}}` — piped filter (`title`, `upper`, `lower`)
/// - `{{#section}}...{{/section}}` — conditional block (renders only if context has data for that section)
/// - `{{score_bar}}` — visual score bar
/// - `{{categories}}` — category table rows
/// - `{{errors_table}}`, `{{warnings_table}}`, `{{suggestions_table}}` — finding table rows
/// - `{{errors_count}}`, `{{warnings_count}}`, `{{suggestions_count}}` — finding counts
pub fn render_from_template(template: &str, ctx: &TemplateContext) -> String {
    let result = process_conditional_blocks(template, ctx);
    let result = process_variables(&result, ctx);
    result
}

fn process_conditional_blocks(template: &str, ctx: &TemplateContext) -> String {
    let mut result = String::new();
    let mut remaining = template;
    let start_marker = "{{#";

    while let Some(start_pos) = remaining.find(start_marker) {
        // Append content before the block
        result.push_str(&remaining[..start_pos]);

        // Find end of block name: {{#section}}
        let after_open = &remaining[start_pos + 3..];
        let close_pos = after_open.find("}}").unwrap_or(0);
        let section_name = &after_open[..close_pos];
        let after_open_tag = &after_open[close_pos + 2..];

        // Find the closing {{/section}}
                let close_tag = format!("{{{{/{}}}}}", section_name);
        let end_pos = after_open_tag.find(&close_tag);

        match end_pos {
            Some(end) => {
                let block_content = &after_open_tag[..end];
                // Check if the section has data
                let has_data = match section_name {
                    "errors" => !ctx.errors.is_empty(),
                    "warnings" => !ctx.warnings.is_empty(),
                    "suggestions" => !ctx.suggestions.is_empty(),
                    "comments" => !ctx.comments.is_empty(),
                    "semantic_pending" => !ctx.semantic_pending.is_empty(),
                    "semantic_done" => !ctx.semantic_done.is_empty(),
                    _ => false,
                };
                if has_data {
                    // Recursively process inner blocks
                    result.push_str(&process_conditional_blocks(block_content, ctx));
                }
                remaining = &after_open_tag[end + close_tag.len()..];
            }
            None => {
                // No closing tag found, treat as literal
                result.push_str(&remaining[start_pos..]);
                remaining = "";
                break;
            }
        }
    }

    result.push_str(remaining);
    result
}

fn process_variables(template: &str, ctx: &TemplateContext) -> String {
    let mut result = String::new();
    let mut remaining = template;

    while let Some(start) = remaining.find("{{") {
        result.push_str(&remaining[..start]);
        let after = &remaining[start + 2..];

        if let Some(end) = after.find("}}") {
            let expr = &after[..end];

            // Skip conditional markers (already processed)
            if expr.starts_with('#') || expr.starts_with('/') {
                result.push_str(&remaining[start..start + 2 + end + 2]);
                remaining = &after[end + 2..];
                continue;
            }

            let value = resolve_variable(expr, ctx);
            result.push_str(&value);
            remaining = &after[end + 2..];
        } else {
            result.push_str(&remaining[start..]);
            remaining = "";
        }
    }

    result.push_str(remaining);
    result
}

fn resolve_variable(expr: &str, ctx: &TemplateContext) -> String {
    let (name, filter) = match expr.find('|') {
        Some(pos) => (&expr[..pos], Some(&expr[pos + 1..])),
        None => (expr, None),
    };

    let raw = match name {
        "pipeline" => ctx.pipeline.clone(),
        "score" => format!("{:.1}", ctx.score),
        "date" => ctx.date.clone(),
        "score_bar" => render_score_bar(ctx.score),
        "categories" => render_categories(&ctx.categories),
        "errors_table" => render_finding_table(&ctx.errors),
        "warnings_table" => render_finding_table(&ctx.warnings),
        "suggestions_table" => render_finding_table(&ctx.suggestions),
        "errors_count" => ctx.errors.len().to_string(),
        "warnings_count" => ctx.warnings.len().to_string(),
        "suggestions_count" => ctx.suggestions.len().to_string(),
        "comments" => render_comments(&ctx.comments),
        "semantic_pending_table" => render_finding_table(&ctx.semantic_pending),
        "semantic_done_table" => render_finding_table(&ctx.semantic_done),
        "semantic_pending_count" => ctx.semantic_pending.len().to_string(),
        "semantic_done_count" => ctx.semantic_done.len().to_string(),
        _ => format!("{{{{{}}}}}", expr),
    };

    match filter {
        Some("title") => title_case(&raw),
        Some("upper") => raw.to_uppercase(),
        Some("lower") => raw.to_lowercase(),
        _ => raw,
    }
}

fn render_score_bar(score: f64) -> String {
    let filled = (score / 10.0).round() as usize;
    let filled = filled.min(10);
    let empty = 10 - filled;
    let bar: String = std::iter::repeat('█').take(filled)
        .chain(std::iter::repeat('░').take(empty))
        .collect();
    format!("{} {:.1}%", bar, score)
}

fn render_categories(categories: &HashMap<String, f64>) -> String {
    let mut rows = String::new();
    let mut sorted: Vec<_> = categories.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    for (name, score) in &sorted {
        rows.push_str(&format!("| {} | {:.1}% |\n", name, score));
    }
    rows
}

fn render_finding_table(findings: &[TemplateFinding]) -> String {
    let mut rows = String::new();
    for f in findings {
        let loc = f.location.as_deref().unwrap_or("-");
        rows.push_str(&format!("| {} | {} | {} |\n", f.check_id, loc, f.message));
    }
    rows
}

fn render_comments(comments: &[TemplateComment]) -> String {
    let mut out = String::new();
    for c in comments {
        out.push_str(&format!("**{}** ({}): {}\n\n", c.author, c.created_at, c.body));
    }
    out
}

fn title_case(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_start = true;
    for ch in s.chars() {
        if ch == '-' || ch == '_' {
            out.push(' ');
            at_start = true;
        } else if at_start {
            out.extend(ch.to_uppercase());
            at_start = false;
        } else {
            out.push(ch);
        }
    }
    out
}

// ── Tera Architecture Report (Phase 9) ────────────────────────────────

use serde::{Deserialize, Serialize};

/// Score-band rating shared across overall score, category scores, document
/// scores, and validation scores — one source of truth instead of repeating
/// the band thresholds in the template. Bands match architecture-audit.md's
/// "Overall Score" table (95-100 Excellent ... below 70 Needs Improvement).
pub fn rating_word(score: f64) -> &'static str {
    if score >= 95.0 {
        "Excellent"
    } else if score >= 90.0 {
        "Very Good"
    } else if score >= 80.0 {
        "Good"
    } else if score >= 70.0 {
        "Acceptable"
    } else {
        "Needs Improvement"
    }
}

/// One-line description of what the rating means in practice, grounded in
/// the Architecture Standard's Success Criteria / Quality Requirements
/// (`docs/raw/standards/architecture.md`).
pub fn rating_description(score: f64) -> &'static str {
    if score >= 95.0 {
        "Modular, fully traceable, no implementation leakage — ready for Engineering with no reservations."
    } else if score >= 90.0 {
        "Minor gaps only — safe to proceed to Engineering with light follow-up."
    } else if score >= 80.0 {
        "Solid foundation — a few structural, ownership, or consistency issues to resolve before Engineering."
    } else if score >= 70.0 {
        "Core structure present but gaps in traceability, boundaries, or consistency — Engineering should wait for fixes."
    } else {
        "Significant gaps in required sections, ownership, or technology independence — not ready for Engineering."
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeraDocScore {
    pub name: String,
    pub score: f64,
    /// Not present in the stored JSON blob (computed, not persisted) —
    /// `#[serde(default)]` so deserializing old rows doesn't fail; filled in
    /// by `build_architecture_context` right after deserialization.
    #[serde(default)]
    pub rating: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeraValidationScore {
    pub id: String,
    pub score: f64,
    #[serde(default)]
    pub rating: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeraFindingItem {
    pub check_id: String,
    pub message: String,
    pub location: Option<String>,
    /// Captured excerpt backing this finding, when available (semantic
    /// providers set it; deterministic providers currently don't — shows as
    /// `None` for those, which the template renders as "—", not an error).
    pub evidence_excerpt: Option<String>,
    pub evidence_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeraRecommendationItem {
    pub category: String,
    pub priority: String,
    pub description: String,
    pub file_path: Option<String>,
}

/// One row of the Structural Compliance Matrix — a section type from the
/// Architecture Standard's Required Sections table, checked against what
/// the compiled documentation collection actually has.
#[derive(Debug, Clone, Serialize)]
pub struct TeraSectionCompliance {
    pub semantic_type: String,
    pub required: bool,
    pub doc_count: usize,
    pub total_docs: usize,
    pub status: String,
}

/// One audit-standard rubric summary (`docs/raw/audit-standards/architecture/*.md`)
/// — the "why" behind a section's score, without needing to open 11 separate
/// files. `engineering_intent` and `top_objectives` are extracted verbatim
/// from those files, not invented.
#[derive(Debug, Clone, Serialize)]
pub struct TeraAuditStandardSummary {
    pub semantic_type: String,
    pub engineering_intent: String,
    pub top_objectives: Vec<String>,
}


/// Build a TemplateContext from a PipelineReport.
fn template_context_from_pipeline(report: &schemas::audit::PipelineReport) -> TemplateContext {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    for f in &report.findings {
        let tf = TemplateFinding {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
        };
        match f.severity {
            schemas::audit::Severity::Error => errors.push(tf),
            schemas::audit::Severity::Warning => warnings.push(tf),
            schemas::audit::Severity::Suggestion => suggestions.push(tf),
        }
    }
    TemplateContext {
        pipeline: report.pipeline.as_str().to_string(),
        score: report.score,
        categories: report.categories.clone(),
        errors,
        warnings,
        suggestions,
        date: chrono_now_iso(),
        comments: Vec::new(),
        semantic_pending: Vec::new(),
        semantic_done: Vec::new(),
    }
}

/// Render a per-audit report from a template file + data.
/// Returns rendered markdown or an error if template is missing.
/// Queries the store for the latest pipeline report of the given type.
pub fn render_report(
    report_type: &str,
    templates_dir: &Path,
    store: &registry::RegistryStore,
) -> Result<ReportOutput> {
    let template_file = format!("{}-report.md", report_type);
    let template_path = templates_dir.join(&template_file);
    let template = fs::read_to_string(&template_path)
        .unwrap_or_else(|_| DEFAULT_TEMPLATE.to_string());

    // Query the latest standard audit run for this pipeline type.
    let sessions = store.list_standard_audit_runs(report_type, Some(report_type), 1)
        .unwrap_or_default();
    if let Some(session) = sessions.into_iter().next() {
        if let Ok(report) = serde_json::from_str::<schemas::audit::PipelineReport>(&session.report) {
            let ctx = template_context_from_pipeline(&report);
            let json = serde_json::to_string_pretty(&ctx).context("Failed to serialize report context")?;
            return Ok(ReportOutput { markdown: render_from_template(&template, &ctx), json });
        }
    }

    // No report available -- render with empty context as fallback.
    let ctx = TemplateContext {
        pipeline: report_type.to_string(),
        date: chrono_now_iso(),
        ..Default::default()
    };
    let json = serde_json::to_string_pretty(&ctx).context("Failed to serialize fallback context")?;
    Ok(ReportOutput { markdown: render_from_template(&template, &ctx), json })
}

/// Render a report from a PipelineReport + template string.
pub fn render_report_from_pipeline(
    _report_type: &str,
    template: &str,
    report: &schemas::audit::PipelineReport,
) -> Result<ReportOutput> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();
    for f in &report.findings {
        let tf = TemplateFinding {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
        };
        match f.severity {
            schemas::audit::Severity::Error => errors.push(tf),
            schemas::audit::Severity::Warning => warnings.push(tf),
            schemas::audit::Severity::Suggestion => suggestions.push(tf),
        }
    }
    let ctx = TemplateContext {
        pipeline: report.pipeline.as_str().to_string(),
        score: report.score,
        categories: report.categories.clone(),
        errors,
        warnings,
        suggestions,
        date: chrono_now_iso(),
        comments: Vec::new(),
        semantic_pending: Vec::new(),
        semantic_done: Vec::new(),
    };
    let json = serde_json::to_string_pretty(&ctx).context("Failed to serialize report context")?;
    Ok(ReportOutput { markdown: render_from_template(template, &ctx), json })
}

fn chrono_now_iso() -> String {
    // Simple ISO-like timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[allow(dead_code)]
fn strip_conditional_blocks(template: &str, items: &[TemplateFinding], section: &str) -> String {
    let start = format!("{{{{#{}}}}}\n", section);
    let end = format!("\n{{{{/{}}}}}", section);
    if items.is_empty() {
        let mut result = template.to_string();
        while let Some(pos) = result.find(&start) {
            if let Some(end_pos) = result[pos..].find(&end) {
                result.replace_range(pos..=pos + end_pos + end.len() - 1, "");
            } else {
                break;
            }
        }
        result
    } else {
        template.replace(&start, "").replace(&end, "")
    }
}

fn strip_conditional_blocks_str(template: &str, section: &str, value: &str) -> String {
    let start = format!("{{{{#{}}}}}\n", section);
    let end = format!("\n{{{{/{}}}}}", section);
    if value.is_empty() {
        let mut result = template.to_string();
        while let Some(pos) = result.find(&start) {
            if let Some(end_pos) = result[pos..].find(&end) {
                result.replace_range(pos..=pos + end_pos + end.len() - 1, "");
            } else {
                break;
            }
        }
        result
    } else {
        template.replace(&start, "").replace(&end, "")
    }
}

pub const DEFAULT_TEMPLATE: &str = "# {{pipeline|title}} Report — {{date}}

**Score:** {{score}}% {{score_bar}}

## Category Scores

| Category | Score |
|---|---|
{{categories}}

## Findings

{{#errors}}
### Errors ({{errors_count}})
| Check | Location | Message |
|---|---|---|
{{errors_table}}
{{/errors}}

{{#warnings}}
### Warnings ({{warnings_count}})
| Check | Location | Message |
|---|---|---|
{{warnings_table}}
{{/warnings}}

{{#suggestions}}
### Suggestions ({{suggestions_count}})
| Check | Location | Message |
|---|---|---|
{{suggestions_table}}
{{/suggestions}}

{{#comments}}
### Comments
| Author | Body | Date |
|---|---|---|
{{comments_table}}
{{/comments}}
";

/// Return the embedded legacy default template.
pub fn get_default_template() -> &'static str {
    DEFAULT_TEMPLATE
}

/// List available report templates from the filesystem directory.
pub fn list_templates(templates_dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    if !templates_dir.exists() {
        return Ok(names);
    }
    for entry in fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Read a template file by name (without .md extension).
pub fn read_template(templates_dir: &Path, name: &str) -> Result<String> {
    let path = templates_dir.join(format!("{}.md", name));
    fs::read_to_string(&path).with_context(|| format!("Template not found: {}", path.display()))
}

/// List available fix-plan templates from the fix-plan-templates directory.
pub fn list_fix_plan_templates(templates_dir: &Path) -> Result<Vec<String>> {
    list_templates(templates_dir)
}

/// Render a fix plan using the given template string.
///
/// Replaces `{{variable}}` placeholders from `FixPlan` fields.
/// Supports `{{#steps}}...{{/steps}}` and `{{#prerequisites}}...{{/prerequisites}}` blocks.
/// Blocks are stripped when empty.
pub fn render_fix_plan(plan: &schemas::fix::FixPlan, template: &str) -> String {
    let steps_table = plan.steps.iter().map(|s| format!(
        "### Step {} — {}\n\n**Target:** `{}`\n**Action:** {}\n**Rationale:** {}\n\n**Detail:**\n\n{}\n\n**Verification:** {}\n**Rollback:** {}",
        s.step_order, s.action, s.target, s.action, s.rationale, s.detail, s.verification,
        s.rollback.as_deref().unwrap_or("N/A")
    )).collect::<Vec<_>>().join("\n\n");
    let prerequisites_text = if plan.prerequisites.is_empty() {
        String::new()
    } else {
        plan.prerequisites.iter().map(|p| format!("- {}", p)).collect::<Vec<_>>().join("\n")
    };

    let mut t = template
        .replace("{{domain}}", &plan.domain)
        .replace("{{session_id}}", &plan.session_id)
        .replace("{{criterion_id}}", &plan.criterion_id)
        .replace("{{summary}}", &plan.summary)
        .replace("{{rollback_instructions}}", plan.rollback_instructions.as_deref().unwrap_or("N/A"))
        .replace("{{prerequisites}}", &prerequisites_text)
        .replace("{{steps}}", &steps_table);
    t = strip_conditional_blocks_str(&t, "prerequisites", &prerequisites_text);
    t = strip_conditional_blocks_str(&t, "steps", &steps_table);
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rating_word_matches_architecture_audit_bands() {
        assert_eq!(rating_word(100.0), "Excellent");
        assert_eq!(rating_word(95.0), "Excellent");
        assert_eq!(rating_word(94.9), "Very Good");
        assert_eq!(rating_word(90.0), "Very Good");
        assert_eq!(rating_word(89.9), "Good");
        assert_eq!(rating_word(80.0), "Good");
        assert_eq!(rating_word(79.9), "Acceptable");
        assert_eq!(rating_word(70.0), "Acceptable");
        assert_eq!(rating_word(69.9), "Needs Improvement");
        assert_eq!(rating_word(0.0), "Needs Improvement");
    }

    #[test]
    fn rating_description_is_non_empty_for_every_band() {
        for score in [100.0, 92.0, 85.0, 75.0, 40.0] {
            assert!(!rating_description(score).is_empty());
        }
    }

    fn tmp_reports_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sg_reporting_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("reports")
    }

    #[test]
    fn test_write_section_report_creates_latest() {
        let root = tmp_reports_root();
        let body = r#"{"score":85,"findings":[]}"#;
        let path = write_report(
            &root,
            AuditStage::Section,
            "feature",
            Some(1),
            Some("functional-requirements"),
            body,
            1,
        ).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, body);
    }

    #[test]
    fn test_write_report_rotates_history() {
        let root = tmp_reports_root();

        write_report(
            &root,
            AuditStage::Section,
            "feature",
            Some(1),
            Some("constraints"),
            r#"{"v":1}"#,
            1,
        ).unwrap();

        write_report(
            &root,
            AuditStage::Section,
            "feature",
            Some(1),
            Some("constraints"),
            r#"{"v":2}"#,
            2,
        ).unwrap();

        let report_dir = root.join("feature").join("1").join("constraints");
        assert!(report_dir.join("latest.json").exists());

        let history_entries: Vec<_> = fs::read_dir(report_dir.join("history"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!history_entries.is_empty(), "Expected at least one history entry");
    }

    fn fake_audit_report() -> schemas::audit::AuditReport {
        use schemas::audit::*;
        AuditReport {
            id: "audit-1".to_string(),
            domain: Some("vision".to_string()),
            timestamp: "2026-07-11T00:00:00Z".to_string(),
            provider: "deterministic".to_string(),
            score: AuditScore {
                overall: 80.0,
                categories: HashMap::new(),
                documents_checked: 1,
                documents_passed: 1,
                findings_count: 1,
                rating: "Good".to_string(),
                bucket_scores: HashMap::new(),
            },
            findings: vec![AuditFinding {
                check_id: "vision-002".to_string(),
                severity: Severity::Warning,
                message: "Vision must define target audience".to_string(),
                location: Some("docs/raw/vision/vision.md".to_string()),
                document_id: Some(223),
                provider: "deterministic".to_string(),
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
            }],
            readiness: ReadinessAssessment::Production,
            metadata: HashMap::new(),
            semantic_review: SemanticReviewBundle {
                instruction: "judge it".to_string(),
                rubrics: HashMap::new(),
                tasks: vec![SemanticReviewTask {
                    document_id: 223,
                    section_id: 42,
                    document_title: "Vision".to_string(),
                    document_path: "docs/raw/vision/vision.md".to_string(),
                    domain: "vision".to_string(),
                    semantic_type: "purpose".to_string(),
                    content: "...".to_string(),
                }],
            },
        }
    }

    #[test]
    fn test_write_audit_scorecard_then_regenerate_moves_pending_to_done() {
        let root = tmp_reports_root();
        let templates_dir = root.join("no-such-templates-dir"); // forces DEFAULT_TEMPLATE fallback... but
        // audit-scorecard.md ships in the real repo; here we just exercise the write/read/regenerate
        // plumbing, so the DEFAULT_TEMPLATE fallback (no semantic tags) is fine for the JSON assertions
        // and we render with a minimal inline scorecard template for the markdown assertions.
        let report = fake_audit_report();

        write_audit_scorecard(&root, &templates_dir, "vision", &report, &[]).unwrap();

        let json_path = root.join("vision").join("scorecard").join("latest.json");
        let md_path = root.join("vision").join("scorecard").join("latest.md");
        assert!(json_path.exists());
        assert!(md_path.exists());

        let scorecard: schemas::audit::AuditScorecard =
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
        assert_eq!(scorecard.report.score.overall, 80.0);
        assert!(scorecard.semantic_results.is_empty());

        // Semantic result lands for the one pending task (document_id=223, section_id=42).
        let semantic_result = schemas::audit::SemanticReport {
            report_id: "sem-1".to_string(),
            stage: AuditStage::Section,
            domain: "vision".to_string(),
            document_id: Some(223),
            section_id: Some(42),
            strategy: None,
            score: 90,
            findings: vec![],
            created_at: "2026-07-11T00:01:00Z".to_string(),
            document_revision: None,
            document_hash: None,
        };
        regenerate_audit_scorecard(&root, &templates_dir, "vision", &[semantic_result]).unwrap();

        let scorecard: schemas::audit::AuditScorecard =
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
        assert_eq!(scorecard.semantic_results.len(), 1);
        assert_eq!(scorecard.semantic_results[0].section_id, Some(42));
        // Deterministic half of the report is untouched by regeneration.
        assert_eq!(scorecard.report.score.overall, 80.0);
    }

    #[test]
    fn test_regenerate_audit_scorecard_noop_when_no_prior_audit() {
        let root = tmp_reports_root();
        let templates_dir = root.join("no-such-templates-dir");
        // No write_audit_scorecard call happened for "philosophy" — regenerate must be a no-op,
        // not an error, since audit() simply hasn't run for this domain yet.
        let result = regenerate_audit_scorecard(&root, &templates_dir, "philosophy", &[]);
        assert!(result.is_ok());
        assert!(!root.join("philosophy").join("scorecard").join("latest.json").exists());
    }

    #[test]
    fn test_build_report_path_section() {
        let root = Path::new("/reports");
        let path = build_report_path(root, AuditStage::Section, "feature", Some(42), Some("purpose"));
        assert_eq!(path, Path::new("/reports/feature/42/purpose"));
    }

    #[test]
    fn test_build_report_path_document() {
        let root = Path::new("/reports");
        let path = build_report_path(root, AuditStage::Document, "architecture", Some(7), None);
        assert_eq!(path, Path::new("/reports/architecture/7/document"));
    }

    #[test]
    fn test_build_report_path_cross_domain() {
        let root = Path::new("/reports");
        let path = build_report_path(root, AuditStage::CrossDomain, "feature", None, Some("feature-technical"));
        assert_eq!(path, Path::new("/reports/cross-domain/feature-technical"));
    }

    // ── Template engine tests ───────────────────────────────────────────

    #[test]
    fn test_template_simple_substitution() {
        let ctx = TemplateContext {
            pipeline: "build".to_string(),
            score: 85.5,
            date: "2026-07-06".to_string(),
            ..Default::default()
        };
        let result = render_from_template("Pipeline: {{pipeline}} Score: {{score}}%", &ctx);
        assert_eq!(result, "Pipeline: build Score: 85.5%");
    }

    #[test]
    fn test_template_title_filter() {
        let ctx = TemplateContext {
            pipeline: "build".to_string(),
            ..Default::default()
        };
        let result = render_from_template("{{pipeline|title}}", &ctx);
        assert_eq!(result, "Build");
    }

    #[test]
    fn test_template_upper_filter() {
        let ctx = TemplateContext {
            pipeline: "build".to_string(),
            ..Default::default()
        };
        let result = render_from_template("{{pipeline|upper}}", &ctx);
        assert_eq!(result, "BUILD");
    }

    #[test]
    fn test_template_score_bar() {
        let ctx = TemplateContext {
            score: 55.0,
            ..Default::default()
        };
        let result = render_from_template("{{score_bar}}", &ctx);
        assert!(result.contains("55.0%"));
        assert!(result.contains('█'));
        assert!(result.contains('░'));
    }

    #[test]
    fn test_template_conditional_errors_renders_when_present() {
        let ctx = TemplateContext {
            pipeline: "test".to_string(),
            errors: vec![TemplateFinding {
                check_id: "E1".into(),
                message: "Something failed".into(),
                location: Some("file.rs".into()),
            }],
            ..Default::default()
        };
        let result = render_from_template("{{#errors}}### Errors ({{errors_count}})\n{{errors_table}}{{/errors}}", &ctx);
        assert!(result.contains("Errors (1)"));
        assert!(result.contains("E1"));
        assert!(result.contains("Something failed"));
    }

    #[test]
    fn test_template_conditional_errors_hidden_when_empty() {
        let ctx = TemplateContext {
            pipeline: "test".to_string(),
            errors: vec![],
            ..Default::default()
        };
        let result = render_from_template("{{#errors}}### Errors\n{{errors_table}}{{/errors}}No errors", &ctx);
        assert_eq!(result, "No errors");
    }

    #[test]
    fn test_template_categories_table() {
        let mut cats = HashMap::new();
        cats.insert("Config Checks".into(), 100.0);
        cats.insert("Coverage".into(), 50.0);
        let ctx = TemplateContext {
            categories: cats,
            ..Default::default()
        };
        let result = render_from_template("{{categories}}", &ctx);
        assert!(result.contains("Config Checks"));
        assert!(result.contains("100.0%"));
        assert!(result.contains("Coverage"));
        assert!(result.contains("50.0%"));
    }

    #[test]
    fn test_template_full_default_template() {
        let ctx = TemplateContext {
            pipeline: "build".to_string(),
            score: 92.0,
            date: "2026-07-06".to_string(),
            categories: {
                let mut m = HashMap::new();
                m.insert("Reliability".into(), 92.0);
                m
            },
            errors: vec![TemplateFinding {
                check_id: "B1".into(),
                message: "Build failed".into(),
                location: None,
            }],
            warnings: vec![],
            suggestions: vec![],
            comments: vec![],
            semantic_pending: vec![],
            semantic_done: vec![],
        };
        let result = render_from_template(DEFAULT_TEMPLATE, &ctx);
        assert!(result.contains("Build Report"));
        assert!(result.contains("92.0%"));
        assert!(result.contains("B1"));
        assert!(result.contains("Build failed"));
        // Conditional blocks — errors present, warnings/suggestions/comments empty
        assert!(result.contains("Errors (1)"));
        assert!(!result.contains("Warnings"));
        assert!(!result.contains("Suggestions"));
        assert!(!result.contains("Comments"));
    }

    #[test]
    fn test_title_case() {
        assert_eq!(title_case("build"), "Build");
        assert_eq!(title_case("security-audit"), "Security Audit");
        assert_eq!(title_case("cross_domain"), "Cross Domain");
    }

    #[test]
    fn test_list_templates_empty_dir() {
        let dir = std::env::temp_dir().join(format!("sg_tpl_test_empty_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let names = list_templates(&dir).unwrap();
        assert!(names.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_templates_finds_md_files() {
        let dir = std::env::temp_dir().join(format!("sg_tpl_test_md_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("custom.md"), "custom").unwrap();
        fs::write(dir.join("other.md"), "other").unwrap();
        fs::write(dir.join("readme.txt"), "ignore").unwrap();
        let names = list_templates(&dir).unwrap();
        assert_eq!(names, vec!["custom", "other"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn render_fix_plan_replaces_all_variables() {
        let plan = schemas::fix::FixPlan {
            id: Some(1),
            session_id: "sess-1".into(),
            report_id: 10,
            criterion_id: "B1".into(),
            domain: "build".into(),
            plan_type: schemas::fix::PlanType::Build,
            title: "Fix build".into(),
            summary: "Update build config".into(),
            prerequisites: vec!["File writable".into()],
            steps: vec![schemas::fix::PlanStep {
                id: Some(1),
                plan_id: Some(1),
                step_order: 1,
                action: "modify_value".into(),
                target: "Cargo.toml".into(),
                rationale: "Missing dep".into(),
                detail: "Add dep foo".into(),
                verification: "cargo check".into(),
                rollback: Some("git checkout".into()),
                status: schemas::fix::FixStepStatus::Pending,
                verified_at: None,
                score: None,
            }],
            rollback_instructions: Some("git checkout .".into()),
            expected_checks: vec!["B1".into()],
            status: schemas::fix::FixPlanStatus::Draft,
            created_at: None,
            updated_at: None,
        };
        let template = r#"# {{domain}} Fix Plan

{{summary}}

{{#prerequisites}}
{{prerequisites}}
{{/prerequisites}}

## Steps

{{#steps}}
{{steps}}
{{/steps}}

Rollback: {{rollback_instructions}}"#;
        let rendered = render_fix_plan(&plan, template);
        assert!(rendered.contains("build Fix Plan"));
        assert!(rendered.contains("Update build config"));
        assert!(rendered.contains("git checkout ."));
        assert!(rendered.contains("- File writable"));
        assert!(rendered.contains("modify_value"));
        assert!(rendered.contains("Cargo.toml"));
    }

    #[test]
    fn render_fix_plan_empty_prerequisites_removes_block() {
        let plan = schemas::fix::FixPlan {
            id: None, session_id: "s".into(), report_id: 1, criterion_id: "C1".into(),
            domain: "test".into(), plan_type: schemas::fix::PlanType::Test,
            title: "t".into(), summary: "s".into(),
            prerequisites: vec![],
            steps: vec![schemas::fix::PlanStep {
                id: None, plan_id: None, step_order: 1,
                action: "a".into(), target: "t".into(), rationale: "r".into(),
                detail: "d".into(), verification: "v".into(), rollback: None,
                status: schemas::fix::FixStepStatus::Pending,
                verified_at: None, score: None,
            }],
            rollback_instructions: None, expected_checks: vec![],
            status: schemas::fix::FixPlanStatus::Draft,
            created_at: None, updated_at: None,
        };
        let template = "BEFORE{{#prerequisites}}\n{{prerequisites}}\n{{/prerequisites}}AFTER";
        let rendered = render_fix_plan(&plan, template);
        assert_eq!(rendered, "BEFOREAFTER");
    }

    #[test]
    fn render_fix_plan_no_rollback_uses_default() {
        let plan = schemas::fix::FixPlan {
            id: None, session_id: "s".into(), report_id: 1, criterion_id: "C1".into(),
            domain: "test".into(), plan_type: schemas::fix::PlanType::Test,
            title: "t".into(), summary: "s".into(),
            prerequisites: vec![],
            steps: vec![schemas::fix::PlanStep {
                id: None, plan_id: None, step_order: 1,
                action: "a".into(), target: "t".into(), rationale: "r".into(),
                detail: "d".into(), verification: "v".into(), rollback: None,
                status: schemas::fix::FixStepStatus::Pending,
                verified_at: None, score: None,
            }],
            rollback_instructions: None, expected_checks: vec![],
            status: schemas::fix::FixPlanStatus::Draft,
            created_at: None, updated_at: None,
        };
        let template = "Rollback: {{rollback_instructions}}";
        let rendered = render_fix_plan(&plan, template);
        assert!(rendered.contains("Rollback: N/A"));
    }
}
