use anyhow::{Context, Result};
use schemas::audit::AuditStage;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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
    let report_dir = build_report_path(reports_root, stage, domain, document_id, section_type);
    fs::create_dir_all(&report_dir)
        .with_context(|| format!("Failed to create report dir: {:?}", report_dir))?;

    let history_dir = report_dir.join("history");
    fs::create_dir_all(&history_dir)
        .with_context(|| format!("Failed to create history dir: {:?}", history_dir))?;

    let latest_path = report_dir.join("latest.json");

    // Rotate existing latest.json to history
    if latest_path.exists() {
        let timestamp = chrono_now();
        let history_name = format!("{}-rev{}.json", timestamp, revision);
        let history_path = history_dir.join(&history_name);
        fs::rename(&latest_path, &history_path)
            .with_context(|| format!("Failed to rotate report to {:?}", history_path))?;
    }

    // Atomic write: write to temp file, then rename
    let tmp_path = report_dir.join("latest.json.tmp");
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
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub pipeline: String,
    pub score: f64,
    pub categories: HashMap<String, f64>,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub date: String,
    pub comments: Vec<TemplateComment>,
}

#[derive(Debug, Clone)]
pub struct TemplateFinding {
    pub check_id: String,
    pub message: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub collection_integrity_score: f64,
    pub collection_integrity_rating: String,
    pub structural_integrity_score: f64,
    pub structural_integrity_rating: String,
    pub consistency_score: f64,
    pub consistency_rating: String,
    pub cross_repo_score: f64,
    pub cross_repo_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

/// Required + optional section types from the Architecture Standard's
/// Required Sections table (`docs/raw/standards/architecture.md`).
const ARCHITECTURE_SECTION_TYPES: &[(&str, bool)] = &[
    ("system_overview", true),
    ("component_model", true),
    ("communication_paths", true),
    ("data_flow", true),
    ("security_considerations", true),
    ("purpose", false),
    ("rationale", false),
    ("constraints", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/architecture/*.md` file, transcribed once here
/// so the report can explain *why* a section scored the way it did without
/// making the reader open 11 separate rubric files. Update this list if
/// those files change.
const ARCHITECTURE_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Defines the architectural intent, scope, and key goals for the system — why the architecture exists and what it optimizes for.",
        &["Purpose is stated clearly with scope boundaries", "Primary goals are identified and prioritized", "Consistent with requirements and downstream sections"]),
    ("system_overview", "High-level description of the system and its environment — the entry point for understanding the architecture.",
        &["System purpose and architectural style are described", "Deployment context and external dependencies are identified", "Consistent with detailed sections"]),
    ("component_model", "Identifies the major structural elements of the system; each must have a clear responsibility, defined interfaces, and known dependencies.",
        &["All components identified", "Each has a clear, non-overlapping responsibility", "Interfaces between components documented"]),
    ("communication_paths", "How components interact: protocols, message formats, invocation patterns, and network topology.",
        &["All inter-component paths documented with protocol and direction", "Synchronization model and QoS defined per path", "Error handling / retry strategy documented"]),
    ("data_flow", "How data moves through the system: sources, transformations, storage, and consumption endpoints.",
        &["All major data flows identified with sources and sinks", "Processing semantics (sync/async, batch/stream) defined", "Transformations and storage boundaries documented"]),
    ("constraints", "Non-negotiable, system-wide properties of the design driven by architectural decisions.",
        &["Constraints are architectural, not implementation-specific", "Each constraint has a documented justification", "No contradictory constraints"]),
    ("security_considerations", "Threat model boundaries, trust zones, authentication/authorization flows, data protection, and compliance requirements.",
        &["Trust boundaries and attack surface enumerated", "Structured threat model (STRIDE/PASTA/OWASP) with mitigations", "Authentication and authorization described"]),
    ("observability", "The observability infrastructure itself — telemetry backend, correlation ID strategy, log retention, SLO monitoring.",
        &["Telemetry backend and correlation ID strategy documented", "Log aggregation pipeline with retention policy described", "SLO monitoring and on-call routing documented"]),
    ("operational_readiness", "Whether the system can be safely deployed, operated, scaled, and recovered by teams who didn't write it.",
        &["Deployment automation and promotion gating documented", "Rollback procedure defined with a time target", "Runbooks and RTO/RPO targets documented"]),
    ("traceability", "Maps architecture decisions and elements back to requirements, forward to implementation, and across sections.",
        &["Elements traceable to source requirements or decisions", "Cross-references between sections present and resolvable", "ADRs referenced by stable ID, not title only"]),
    ("generic", "Architecture-relevant context that doesn't fit a specialized section type.",
        &["Content is architecture-relevant, not implementation-specific", "Claims justified by evidence or reasoning", "No duplication of typed-section content"]),
];

/// Shared by every domain's `build_*_audit_standards_summary()` — each
/// domain just supplies its own `(semantic_type, intent, objectives)` table
/// transcribed from `docs/raw/audit-standards/<domain>/*.md`.
fn build_audit_standards_summary_for(
    table: &[(&str, &str, &[&str])],
) -> Vec<TeraAuditStandardSummary> {
    table
        .iter()
        .map(|(semantic_type, intent, objectives)| TeraAuditStandardSummary {
            semantic_type: semantic_type.to_string(),
            engineering_intent: intent.to_string(),
            top_objectives: objectives.iter().map(|s| s.to_string()).collect(),
        })
        .collect()
}

fn build_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(ARCHITECTURE_AUDIT_STANDARDS)
}

/// Shared by every domain's `build_*_section_compliance()` — each domain
/// supplies its own `domain` name and `(semantic_type, required)` table from
/// `docs/raw/standards/<domain>.md`'s Required Sections table.
fn build_section_compliance_for(
    store: &registry::RegistryStore,
    domain: &str,
    section_types: &[(&str, bool)],
) -> Vec<TeraSectionCompliance> {
    let types: Vec<&str> = section_types.iter().map(|(t, _)| *t).collect();
    let required_lookup: std::collections::HashMap<&str, bool> =
        section_types.iter().cloned().collect();

    store
        .count_section_type_coverage(domain, &types)
        .unwrap_or_default()
        .into_iter()
        .map(|(semantic_type, doc_count, total_docs)| {
            let required = *required_lookup.get(semantic_type.as_str()).unwrap_or(&false);
            let status = if total_docs == 0 {
                "Unknown".to_string()
            } else if doc_count == total_docs {
                "Complete".to_string()
            } else if doc_count == 0 {
                "Missing".to_string()
            } else {
                "Partial".to_string()
            };
            TeraSectionCompliance {
                semantic_type,
                required,
                doc_count,
                total_docs,
                status,
            }
        })
        .collect()
}

fn build_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "architecture", ARCHITECTURE_SECTION_TYPES)
}

/// Required + optional section types from the Vision Standard's Required
/// Sections table (`docs/raw/standards/vision.md`).
const VISION_SECTION_TYPES: &[(&str, bool)] = &[
    ("purpose", true),
    ("vision_statement", true),
    ("problem", true),
    ("solution", true),
    ("target_audience", true),
    ("pillars", false),
    ("philosophy", false),
    ("guiding_principles", false),
    ("success_criteria", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/vision/*.md` file — same purpose as
/// `ARCHITECTURE_AUDIT_STANDARDS` above.
const VISION_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Why the vision document exists and what it intends to accomplish — establishes scope, context, and motivation.",
        &["Purpose distinguishes document intent from product mission", "Scope boundaries explicitly stated", "Aligned with program-level objectives"]),
    ("vision_statement", "A concise, aspirational description of the future state the product enables — the north star for strategic decisions.",
        &["1-3 sentences describing a future state", "Free of implementation-specific language", "Differentiates from current status quo or alternatives"]),
    ("problem", "The real-world pain point, gap, or opportunity the vision addresses — grounds the vision in evidence.",
        &["Stated from user/stakeholder perspective with evidence", "Affected parties identified and scope bounded", "Current workarounds or alternatives acknowledged"]),
    ("solution", "The high-level approach to solving the identified problem, without descending into implementation detail.",
        &["Addresses all aspects of the stated problem", "Described at capability level without technology prescription", "Constraints and feasibility considerations acknowledged"]),
    ("target_audience", "Who will use, benefit from, or be affected by the envisioned system.",
        &["Primary and secondary audiences clearly distinguished with characteristics", "All stakeholders implied by the problem are covered", "Audience needs linked to the problem statement"]),
    ("pillars", "The core strategic pillars that underpin the vision — foundational areas of investment collectively realizing it.",
        &["3-5 distinct pillars with no overlap and clear vision linkage", "Each pillar includes rationale and thematic description", "Pillars are sequenced or prioritized appropriately"]),
    ("philosophy", "The core beliefs, values, and design worldview that inform the vision — the \"why behind the what.\"",
        &["Philosophy statements are domain-specific and substantive", "Each belief includes operational implications for decision-making", "Internally consistent with pillars and principles"]),
    ("guiding_principles", "Actionable rules that translate the vision and philosophy into concrete decision-making criteria.",
        &["Each principle is prescriptive with clear decision-making direction", "Principles are distinct, limited in number, and include rationale", "Address expected trade-off scenarios with ordering"]),
    ("success_criteria", "Measurable outcomes that determine whether the vision has been achieved — bridges aspiration to verifiable results.",
        &["Criteria are specific, measurable, and include clear targets", "Span multiple dimensions (user, business, technical)", "Include timeframes and state dimension priority for conflicts"]),
    ("traceability", "How the vision connects to downstream artifacts — requirements, design decisions, implementation plans.",
        &["Every vision element maps to at least one downstream artifact type", "Consistent identifiers and bidirectional traceability", "Maintenance process and tools documented"]),
];

fn build_vision_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(VISION_AUDIT_STANDARDS)
}

fn build_vision_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "vision", VISION_SECTION_TYPES)
}

/// Required + optional section types from the Design Standard's Required
/// Sections table (`docs/raw/standards/design.md`).
const DESIGN_SECTION_TYPES: &[(&str, bool)] = &[
    ("design_principles", true),
    ("ux_principles", true),
    ("accessibility", true),
    ("purpose", false),
    ("constraints", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/design/*.md` file.
const DESIGN_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("design_principles", "The foundational rules guiding visual and interaction decisions — ensure consistency and a unified product identity.",
        &["Principles explicitly defined and documented", "Consistently applied across all UI components", "No contradictory or overlapping principles"]),
    ("ux_principles", "Operationalizes human-centered design into actionable heuristics governing interaction patterns, information architecture, and user flows.",
        &["Documented and grounded in HCI best practices", "User flows define happy and error paths", "Consistency in navigation, labeling, and feedback"]),
    ("accessibility", "Ensures the product is usable by people with diverse abilities — WCAG compliance is both a legal requirement and a quality benchmark.",
        &["WCAG 2.1 AA compliance for all screens", "Keyboard navigation and focus management work", "ARIA labels and semantic structure correct"]),
    ("purpose", "The strategic intent behind visual and interaction decisions — every element must serve a clear functional or communicative goal.",
        &["Design purpose documented for each major screen", "Purpose traceable to user or business need", "No unexplained decorative elements"]),
    ("constraints", "The boundaries within which design decisions must operate — platform limitations, brand guidelines, technical dependencies, regulatory requirements.",
        &["All constraints documented and categorized", "Brand and platform constraints have defined sources", "Constraint conflicts identified with fallback strategies"]),
    ("traceability", "Ensures every design decision, component, and visual element can be linked back to a requirement, principle, or user need.",
        &["All design components traceable to requirements", "Bidirectional traceability matrix exists", "Design change log records rationale and affected requirements"]),
];

fn build_design_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(DESIGN_AUDIT_STANDARDS)
}

fn build_design_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "design", DESIGN_SECTION_TYPES)
}

/// README has no `Structural Compliance Matrix` equivalent — unlike every
/// other domain, it's a single repo-root file, not a `docs/raw/<domain>/`
/// collection with a semantic_type Required Sections table. The README
/// template drops that section entirely rather than rendering an empty one.
const README_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("generic", "Project overview, purpose, audience, and technology stack — must orient a new reader within seconds.",
        &["Project name and one-liner present", "Target audience identified", "Getting-started path completable in \u{2264}5 minutes; license stated"]),
    ("getting_started", "Guides a user from zero to running the project — step-by-step, copy-paste safe, resolving all dependencies.",
        &["Prerequisites listed with versions", "Copy-paste safe commands", "Verification step present after key commands"]),
    ("documentation", "Explains architecture, configuration, API, or usage — must be correct, navigable, and maintainable.",
        &["Accurate and reflects current code", "All links resolve", "Consistent terminology used"]),
];

fn build_readme_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(README_AUDIT_STANDARDS)
}

/// Required + optional section types from the Prototype Standard's Required
/// Sections table (`docs/raw/standards/prototype.md`).
const PROTOTYPE_SECTION_TYPES: &[(&str, bool)] = &[
    ("scope", true),
    ("mock_apis", true),
    ("data_model", true),
    ("purpose", false),
    ("constraints", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/prototype/*.md` file.
const PROTOTYPE_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("scope", "The boundary between what is simulated and what is real — over-scoping inflates cost, under-scoping produces misleading results.",
        &["In-scope and out-of-scope lists present", "Scope items map to the stated prototype purpose", "Fidelity level defined per scope item"]),
    ("mock_apis", "Faithful representation of real interface contracts without implementing production logic — mismatched mocks produce false confidence or false negatives.",
        &["Every external dependency has a mock or stub", "Mock responses match the real API schema", "Mock includes at least one error scenario"]),
    ("data_model", "A simplified representation of real domain entities sufficient to exercise the prototype scenario without unrealistic constraints.",
        &["Core entities and relationships are documented", "No PII, secrets, or production data in the model", "Seed or fixture data covers at least 2 scenarios"]),
    ("purpose", "The specific question a prototype answers or risk it mitigates — without it, fitness can't be evaluated.",
        &["Purpose explicitly stated in the entry point or README", "References a falsifiable question with explicit success/failure thresholds", "Stakeholder or audience identified"]),
    ("constraints", "Known limitations, assumptions, and guardrails — undisclosed constraints mislead evaluators into overgeneralizing results.",
        &["All known constraints and assumptions documented", "Each constraint includes an impact on result generalizability", "Constraints traceable to scope items or mock decisions"]),
    ("traceability", "Connects purpose, scope, mocks, and data model into a coherent, auditable chain.",
        &["Every artifact traces to a purpose or scope item", "No orphaned code, mocks, or data without a trace", "Traceability matrix or equivalent documented"]),
];

fn build_prototype_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(PROTOTYPE_AUDIT_STANDARDS)
}

fn build_prototype_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "prototype", PROTOTYPE_SECTION_TYPES)
}

/// Required + optional section types from the External Context Standard's
/// Required Sections table (`docs/raw/standards/external-context.md`).
const EXTERNAL_CONTEXT_SECTION_TYPES: &[(&str, bool)] = &[
    ("purpose", true),
    ("integration_contract", true),
    ("constraints", false),
    ("dependencies", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/external-context/*.md` file.
const EXTERNAL_CONTEXT_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Why the system depends on, integrates with, and is constrained by an external system — what role it plays.",
        &["Purpose stated for every external dependency", "Business justification documented", "Scope boundaries clearly defined"]),
    ("integration_contract", "The formal interface between the system and an external dependency — endpoints, schemas, protocol versions, auth, expected behaviors.",
        &["Contract exists for every external dependency", "API or protocol version is pinned", "Request/response schemas documented"]),
    ("constraints", "Limitations and boundaries an external system imposes on internal design and operation.",
        &["Constraints enumerated per external dependency", "Each constraint has a measurable threshold", "Constraint source attributed"]),
    ("dependencies", "External libraries, services, APIs, SDKs, and infrastructure the system relies on — versions, licenses, upgrade cadence, EOL risk.",
        &["All dependencies cataloged with pinned versions", "Dependency type and license recorded", "Runtime vs build-time separation"]),
    ("traceability", "Maps each dependency and integration point back to its authoritative source — vendor docs, SLAs, API specs.",
        &["Every dependency links to its authoritative source", "Integration contracts have bidirectional traceability", "External changes logged with internal impact notes"]),
];

fn build_external_context_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(EXTERNAL_CONTEXT_AUDIT_STANDARDS)
}

fn build_external_context_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "external-context", EXTERNAL_CONTEXT_SECTION_TYPES)
}

/// Required + optional section types from the Engineering Standard's
/// Required Sections table (`docs/raw/standards/engineering.md`).
const ENGINEERING_SECTION_TYPES: &[(&str, bool)] = &[
    ("guiding_principles", true),
    ("rationale", true),
    ("build_standards", true),
    ("testing_standards", true),
    ("purpose", false),
    ("code_standards", false),
    ("constraints", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/engineering/*.md` file.
const ENGINEERING_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("guiding_principles", "Architecture and code should demonstrably follow the project's stated design principles.",
        &["Design principles are explicitly documented and discoverable", "Code structure reflects the declared architectural patterns", "Violations of core principles are flagged and justified"]),
    ("rationale", "Every non-obvious engineering decision must be accompanied by an explicit rationale record.",
        &["Architectural and design decisions are documented with rationale", "Rationale includes rejected alternatives and why they were rejected", "Rationale distinguishes between reversible and irreversible decisions"]),
    ("build_standards", "The build pipeline must be reproducible, fast, secure, and produce identical artifacts from the same source.",
        &["Build is reproducible byte-for-byte from version-controlled source", "Dependencies are pinned to known versions with integrity hashes", "Supply-chain integrity is maintained"]),
    ("testing_standards", "Automated tests must be deterministic, isolated, fast, and provide meaningful coverage.",
        &["Tests are deterministic and isolated", "Test coverage meets project thresholds", "Security test coverage includes auth bypass, authorization escalation, and input injection paths"]),
    ("code_standards", "Code Standards define the coding conventions, style guides, and quality requirements that govern implementation.",
        &["Standards cover naming conventions, formatting, and file organization", "Static analysis and linting requirements are defined", "Review criteria align with stated standards"]),
    ("security_standards", "Engineering security standards govern secrets handling, cryptography, and supply-chain risk — enforced at the tooling level, not just policy.",
        &["Secrets management approach is documented, no hardcoded values", "Cryptographic algorithm selection is documented and current", "SAST and dependency vulnerability scanning are integrated into CI"]),
    ("purpose", "Every engineering artifact must carry an explicit, discoverable purpose statement.",
        &["Every module, service, and configuration has a declared purpose", "Purpose statements are specific and not generic boilerplate", "No zombie artifacts that run but serve no documented purpose"]),
    ("constraints", "Constraints document the deliberate boundaries and limitations that engineering decisions must respect.",
        &["All constraints have clear rationale explaining why they exist", "Constraints are categorized as hard or soft", "Constraint expiry or review conditions are documented"]),
    ("traceability", "Traceability ensures every requirement, design decision, implementation artifact, and test case is bidirectionally linked.",
        &["Every requirement maps to one or more implementation artifacts", "Every code module traces back to a documented requirement", "Test cases are linked to the requirements they verify"]),
];

fn build_engineering_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(ENGINEERING_AUDIT_STANDARDS)
}

fn build_engineering_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "engineering", ENGINEERING_SECTION_TYPES)
}

/// Required + optional section types from the Feature Standard's
/// Required Sections table (`docs/raw/standards/feature.md`).
const FEATURE_SECTION_TYPES: &[(&str, bool)] = &[
    ("purpose", true),
    ("functional_requirements", true),
    ("acceptance_criteria", true),
    ("business_rules", false),
    ("inputs", false),
    ("outputs", false),
    ("constraints", false),
    ("dependencies", false),
    ("non_goals", false),
    ("future_extensions", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/feature/*.md` file.
const FEATURE_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Purpose defines the feature's reason for existence and the problem it solves.",
        &["Purpose states the problem being solved", "Purpose identifies the target users or stakeholders", "Purpose is concise (not a requirements list)"]),
    ("functional_requirements", "Functional requirements describe what the system must do — complete, unambiguous, testable, and implementation-independent.",
        &["All functional requirements are enumerated without gaps", "Each requirement is testable", "Requirements are implementation-independent (describe WHAT not HOW)"]),
    ("business_rules", "Business rules encode domain logic, policies, calculations, and decision logic the system must enforce.",
        &["Each business rule is atomic", "Business rules are expressed declaratively", "Rules handle edge cases and exceptions"]),
    ("acceptance_criteria", "Acceptance criteria define conditions a feature must satisfy for stakeholder sign-off.",
        &["Each criterion is pass/fail testable", "Criteria are written from the user or stakeholder perspective", "Each criterion tests a single behavior"]),
    ("inputs", "Inputs define the data and triggers the system receives.",
        &["Every input has a defined source", "Input format and schema are specified", "Validation rules are documented"]),
    ("outputs", "Outputs define what the system produces or exposes.",
        &["Every output has a defined consumer or destination", "Output format and schema are specified", "Output frequency and timing are documented"]),
    ("constraints", "Constraints define boundaries the implementation must operate within.",
        &["Each constraint is specific and measurable", "Each constraint has a clear justification", "Constraints are not contradictory"]),
    ("dependencies", "Dependencies enumerate external systems, services, libraries, or teams the feature relies on.",
        &["Every dependency is identified with name and version", "Interface contract or integration point is documented", "Failure impact of each dependency is assessed"]),
    ("non_goals", "Non-goals explicitly state what the feature will NOT address.",
        &["Each non-goal is specific and testable as out-of-scope", "Non-goals are justified", "No overlap between non-goals and future extensions"]),
    ("future_extensions", "Future extensions document planned or possible enhancements beyond the current scope.",
        &["Extensions are clearly marked as out of current scope", "Each extension has a trigger or rationale", "Architectural impact of each extension is considered"]),
    ("traceability", "Traceability maps requirements to implementation artifacts, test cases, and acceptance criteria.",
        &["Every requirement is linked to at least one test case", "Every test case traces back to a requirement", "No orphaned requirements or test cases"]),
];

fn build_feature_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(FEATURE_AUDIT_STANDARDS)
}

fn build_feature_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "feature", FEATURE_SECTION_TYPES)
}

/// Required + optional section types from the Feature Technical Design
/// Standard's Required Sections table (`docs/raw/standards/feature-technical.md`).
const FEATURE_TECHNICAL_SECTION_TYPES: &[(&str, bool)] = &[
    ("purpose", true),
    ("participating_components", true),
    ("component_interactions", true),
    ("data_ownership", true),
    ("feature_specification", false),
    ("component_responsibilities", false),
    ("runtime_behavior", false),
    ("communication_paths", false),
    ("integration_points", false),
    ("external_dependencies", false),
    ("runtime_constraints", false),
    ("architectural_constraints", false),
    ("security_considerations", false),
    ("performance_considerations", false),
    ("failure_handling", false),
    ("extension_points", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/feature-technical/*.md` file.
const FEATURE_TECHNICAL_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Purpose defines the rationale, problem statement, and motivation for the feature from a technical perspective.",
        &["The technical problem being solved is clearly stated", "Engineering value is articulated", "The purpose is distinguishable from business goals"]),
    ("participating_components", "Participating components list every module, service, class, or sub-system that takes part in the feature.",
        &["Every component involved in the feature is listed", "The deployment unit for each component is identified", "Each component's role within the feature is described"]),
    ("component_interactions", "Component interactions describe how components invoke, pass data to, and depend on each other within the feature.",
        &["Every pairwise interaction between components is documented", "Interaction direction is specified", "No undocumented or implicit interactions exist"]),
    ("data_ownership", "Data ownership defines which component or service is the authoritative source for each data entity within the feature.",
        &["Every data entity has a designated owner component", "Write authority is clearly assigned", "Conflict resolution strategy for concurrent writes is documented"]),
    ("feature_specification", "The feature specification defines the technical boundaries, inputs, outputs, and behavioral contract of the entire feature.",
        &["Feature scope is clearly stated with inclusion and exclusion criteria", "Preconditions and postconditions are defined", "Behavioral contract leaves no ambiguity"]),
    ("component_responsibilities", "Component responsibilities define what each module, service, or class owns within the feature.",
        &["Every component has a documented responsibility statement", "Responsibilities are non-overlapping across components", "Components without clear responsibilities are identified"]),
    ("runtime_behavior", "Runtime behavior describes the feature's operational execution model.",
        &["Startup sequence and initialization steps are documented", "State transitions and their triggers are enumerated", "Threading or concurrency model is documented"]),
    ("communication_paths", "Communication paths describe the data flow topology across the feature.",
        &["Every distinct data flow path is documented from source to sink", "Delivery guarantees are defined", "Backpressure handling is documented for each path"]),
    ("integration_points", "Integration points document every boundary where the feature connects to external systems, other features, shared infrastructure, or platform services.",
        &["Every integration point is enumerated with its external system", "The interface contract is specified", "Error handling at each integration boundary is defined"]),
    ("external_dependencies", "External dependencies document all third-party libraries, services, APIs, databases, and infrastructure the feature relies on.",
        &["Every external dependency is listed with its name and version", "The purpose each dependency serves is documented", "Deprecated or unmaintained dependencies are flagged"]),
    ("runtime_constraints", "Runtime constraints specify measurable resource limits and environmental bounds the feature must operate within.",
        &["All resource constraints are enumerated with numeric thresholds", "Measurement units are specified", "Hard limits vs soft advisory limits are distinguished"]),
    ("architectural_constraints", "Architectural constraints define the non-negotiable design rules and patterns the feature must adhere to.",
        &["Every architectural constraint is explicitly stated", "Forbidden or disallowed patterns are enumerated", "Dependency direction is specified"]),
    ("security_considerations", "Security considerations document the threat model, security controls, and trust boundaries relevant to the feature.",
        &["Threat model documents threats with mitigations", "Authentication and authorization model is documented", "Input validation names specific attack vectors"]),
    ("performance_considerations", "Performance considerations document the non-functional performance requirements and expected behavior of the feature under various load conditions.",
        &["Latency targets are specified with percentile levels", "Throughput capacity is defined", "Resource utilization profile per transaction is documented"]),
    ("failure_handling", "Failure handling documents how the feature detects, responds to, and recovers from errors, exceptions, and degraded states.",
        &["All known failure modes are enumerated", "Retry policy is specified per failure mode", "Data consistency guarantees during failures are stated"]),
    ("extension_points", "Extension points document where the feature can be customized, extended, or integrated with by other features or external consumers.",
        &["Every extension point is enumerated with its interface signature", "Stability guarantees for each extension point are documented", "Default behavior when no extension is provided is defined"]),
    ("traceability", "Traceability documents the links between feature requirements, design decisions, implementation artifacts, and verification artifacts.",
        &["Every requirement is traceable to its implementation component(s)", "Tests are mapped to the requirements they verify", "Trace links are bidirectional"]),
    ("data_governance", "Features that handle data must classify that data, protect it appropriately, and comply with applicable regulations.",
        &["Data elements handled by the feature are enumerated and classified", "PII fields are minimized", "Retention period and deletion trigger are documented per data type"]),
    ("observability", "At the technical design level, observability must be designed in, not bolted on.",
        &["Instrumentation library and metric naming convention are specified", "SLO targets are defined", "Log schema with required fields and PII redaction is documented"]),
    ("versioning", "APIs, schemas, and message formats are contracts with callers — breaking changes without versioning destroy integrations silently.",
        &["Versioning strategy is defined with a breaking change definition", "Deprecation lifecycle is documented with notice period and migration guide", "Rollout strategy is specified with rollback gate criteria"]),
];

fn build_feature_technical_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(FEATURE_TECHNICAL_AUDIT_STANDARDS)
}

fn build_feature_technical_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "feature-technical", FEATURE_TECHNICAL_SECTION_TYPES)
}

/// Required + optional section types from the Feature Design Standard's
/// Required Sections table (`docs/raw/standards/feature-design.md`).
const FEATURE_DESIGN_SECTION_TYPES: &[(&str, bool)] = &[
    ("user_experience", true),
    ("workflow", true),
    ("states", true),
    ("purpose", false),
    ("non_goals", false),
    ("constraints", false),
    ("traceability", false),
];

/// Engineering Intent + top Audit Objectives from each
/// `docs/raw/audit-standards/feature-design/*.md` file.
const FEATURE_DESIGN_AUDIT_STANDARDS: &[(&str, &str, &[&str])] = &[
    ("purpose", "Purpose defines the core rationale for a feature: what problem it solves and why it exists.",
        &["Feature purpose is explicitly stated and unambiguous", "Purpose aligns with product strategy and user needs", "Purpose is distinguishable from implementation or solution details"]),
    ("user_experience", "User experience audit evaluates how a feature feels, flows, and responds from the user's perspective.",
        &["User flows are logical and match mental models", "Accessibility standards are met", "Error states and empty states communicate clearly"]),
    ("workflow", "Workflow audit examines the sequence of user actions and system responses required to accomplish a feature's goal.",
        &["Workflow steps are logically ordered and non-redundant", "All branching paths are defined", "Entry points and exit points are clearly identified"]),
    ("states", "States audit covers every visual and interactive state a UI component or screen can exhibit.",
        &["All component states are defined", "State transitions are documented", "Error states display actionable message and recovery path"]),
    ("constraints", "Constraints document the boundaries within which a feature must be designed and built.",
        &["All constraint types are enumerated", "Each constraint is specific and verifiable", "Constraints distinguish hard from soft boundaries"]),
    ("non_goals", "Non-goals explicitly define what a feature will not do, preventing scope creep and managing stakeholder expectations.",
        &["Non-goals are clearly stated and not disguised as missing features", "Each non-goal has a rationale explaining the exclusion", "Non-goals are distinguishable from goals"]),
    ("traceability", "Traceability ensures every design decision, requirement, and state can be linked back to the feature's purpose and forward to implementation artifacts.",
        &["Every design element is traceable to a stated requirement or purpose", "Design artifacts link to test cases or acceptance criteria", "Traceability is bidirectional"]),
];

fn build_feature_design_audit_standards_summary() -> Vec<TeraAuditStandardSummary> {
    build_audit_standards_summary_for(FEATURE_DESIGN_AUDIT_STANDARDS)
}

fn build_feature_design_section_compliance(store: &registry::RegistryStore) -> Vec<TeraSectionCompliance> {
    build_section_compliance_for(store, "feature-design", FEATURE_DESIGN_SECTION_TYPES)
}

fn build_architecture_context(
    session: registry::store::ArchitectureSessionInfo,
    report: registry::store::ArchitectureReportWithFindings,
    store: &registry::RegistryStore,
) -> ArchitectureTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The architecture documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The architecture documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The architecture documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    ArchitectureTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        collection_integrity_score: report.collection_integrity_score.unwrap_or(0.0),
        collection_integrity_rating: rating_word(report.collection_integrity_score.unwrap_or(0.0)).to_string(),
        structural_integrity_score: report.structural_integrity_score.unwrap_or(0.0),
        structural_integrity_rating: rating_word(report.structural_integrity_score.unwrap_or(0.0)).to_string(),
        consistency_score: report.consistency_score.unwrap_or(0.0),
        consistency_rating: rating_word(report.consistency_score.unwrap_or(0.0)).to_string(),
        cross_repo_score: report.cross_repo_score.unwrap_or(0.0),
        cross_repo_rating: rating_word(report.cross_repo_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_section_compliance(store),
        audit_standards: build_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_architecture_template(ctx: &ArchitectureTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct VisionTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub vision_content_score: f64,
    pub vision_content_rating: String,
    pub tech_independence_score: f64,
    pub tech_independence_rating: String,
    pub traceability_consistency_score: f64,
    pub traceability_consistency_rating: String,
    pub doc_quality_score: f64,
    pub doc_quality_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_vision_context(
    session: registry::store::VisionSessionInfo,
    report: registry::store::VisionReportWithFindings,
    store: &registry::RegistryStore,
) -> VisionTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The vision documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The vision documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The vision documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    VisionTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        vision_content_score: report.vision_content_score.unwrap_or(0.0),
        vision_content_rating: rating_word(report.vision_content_score.unwrap_or(0.0)).to_string(),
        tech_independence_score: report.tech_independence_score.unwrap_or(0.0),
        tech_independence_rating: rating_word(report.tech_independence_score.unwrap_or(0.0)).to_string(),
        traceability_consistency_score: report.traceability_consistency_score.unwrap_or(0.0),
        traceability_consistency_rating: rating_word(report.traceability_consistency_score.unwrap_or(0.0)).to_string(),
        doc_quality_score: report.doc_quality_score.unwrap_or(0.0),
        doc_quality_rating: rating_word(report.doc_quality_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_vision_section_compliance(store),
        audit_standards: build_vision_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_vision_template(ctx: &VisionTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub design_system_score: f64,
    pub design_system_rating: String,
    pub doc_quality_score: f64,
    pub doc_quality_rating: String,
    pub design_quality_score: f64,
    pub design_quality_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_design_context(
    session: registry::store::DesignSessionInfo,
    report: registry::store::DesignReportWithFindings,
    store: &registry::RegistryStore,
) -> DesignTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The design documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The design documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The design documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    DesignTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        design_system_score: report.design_system_score.unwrap_or(0.0),
        design_system_rating: rating_word(report.design_system_score.unwrap_or(0.0)).to_string(),
        doc_quality_score: report.doc_quality_score.unwrap_or(0.0),
        doc_quality_rating: rating_word(report.doc_quality_score.unwrap_or(0.0)).to_string(),
        design_quality_score: report.design_quality_score.unwrap_or(0.0),
        design_quality_rating: rating_word(report.design_quality_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_design_section_compliance(store),
        audit_standards: build_design_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_design_template(ctx: &DesignTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

/// No `section_compliance` field — README is a single repo-root file, not a
/// `docs/raw/<domain>/` collection, so the Structural Compliance Matrix
/// concept (used by every other domain) doesn't apply. See
/// `README_AUDIT_STANDARDS` above.
#[derive(Debug, Clone, Serialize)]
pub struct ReadmeTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub repo_introduction_score: f64,
    pub repo_introduction_rating: String,
    pub doc_navigation_score: f64,
    pub doc_navigation_rating: String,
    pub doc_quality_score: f64,
    pub doc_quality_rating: String,
    pub maintainability_score: f64,
    pub maintainability_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_readme_context(
    session: registry::store::ReadmeSessionInfo,
    report: registry::store::ReadmeReportWithFindings,
) -> ReadmeTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The README has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The README has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The README score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    ReadmeTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        repo_introduction_score: report.repo_introduction_score.unwrap_or(0.0),
        repo_introduction_rating: rating_word(report.repo_introduction_score.unwrap_or(0.0)).to_string(),
        doc_navigation_score: report.doc_navigation_score.unwrap_or(0.0),
        doc_navigation_rating: rating_word(report.doc_navigation_score.unwrap_or(0.0)).to_string(),
        doc_quality_score: report.doc_quality_score.unwrap_or(0.0),
        doc_quality_rating: rating_word(report.doc_quality_score.unwrap_or(0.0)).to_string(),
        maintainability_score: report.maintainability_score.unwrap_or(0.0),
        maintainability_rating: rating_word(report.maintainability_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        audit_standards: build_readme_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_readme_template(ctx: &ReadmeTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct PrototypeTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub product_validation_score: f64,
    pub product_validation_rating: String,
    pub runtime_validation_score: f64,
    pub runtime_validation_rating: String,
    pub engineering_validation_score: f64,
    pub engineering_validation_rating: String,
    pub validation_quality_score: f64,
    pub validation_quality_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_prototype_context(
    session: registry::store::PrototypeSessionInfo,
    report: registry::store::PrototypeReportWithFindings,
    store: &registry::RegistryStore,
) -> PrototypeTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The prototype validation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The prototype validation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The prototype validation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    PrototypeTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        product_validation_score: report.product_validation_score.unwrap_or(0.0),
        product_validation_rating: rating_word(report.product_validation_score.unwrap_or(0.0)).to_string(),
        runtime_validation_score: report.runtime_validation_score.unwrap_or(0.0),
        runtime_validation_rating: rating_word(report.runtime_validation_score.unwrap_or(0.0)).to_string(),
        engineering_validation_score: report.engineering_validation_score.unwrap_or(0.0),
        engineering_validation_rating: rating_word(report.engineering_validation_score.unwrap_or(0.0)).to_string(),
        validation_quality_score: report.validation_quality_score.unwrap_or(0.0),
        validation_quality_rating: rating_word(report.validation_quality_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_prototype_section_compliance(store),
        audit_standards: build_prototype_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_prototype_template(ctx: &PrototypeTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalContextTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub document_quality_score: f64,
    pub document_quality_rating: String,
    pub content_completeness_score: f64,
    pub content_completeness_rating: String,
    pub documentation_integrity_score: f64,
    pub documentation_integrity_rating: String,
    pub collection_quality_score: f64,
    pub collection_quality_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_external_context_context(
    session: registry::store::ExternalContextSessionInfo,
    report: registry::store::ExternalContextReportWithFindings,
    store: &registry::RegistryStore,
) -> ExternalContextTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The External Context documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The External Context documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The External Context documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    ExternalContextTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        document_quality_score: report.document_quality_score.unwrap_or(0.0),
        document_quality_rating: rating_word(report.document_quality_score.unwrap_or(0.0)).to_string(),
        content_completeness_score: report.content_completeness_score.unwrap_or(0.0),
        content_completeness_rating: rating_word(report.content_completeness_score.unwrap_or(0.0)).to_string(),
        documentation_integrity_score: report.documentation_integrity_score.unwrap_or(0.0),
        documentation_integrity_rating: rating_word(report.documentation_integrity_score.unwrap_or(0.0)).to_string(),
        collection_quality_score: report.collection_quality_score.unwrap_or(0.0),
        collection_quality_rating: rating_word(report.collection_quality_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_external_context_section_compliance(store),
        audit_standards: build_external_context_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_external_context_template(ctx: &ExternalContextTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineeringTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub engineering_coverage_score: f64,
    pub engineering_coverage_rating: String,
    pub documentation_quality_score: f64,
    pub documentation_quality_rating: String,
    pub traceability_consistency_score: f64,
    pub traceability_consistency_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_engineering_context(
    session: registry::store::EngineeringSessionInfo,
    report: registry::store::EngineeringReportWithFindings,
    store: &registry::RegistryStore,
) -> EngineeringTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The Engineering documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The Engineering documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The Engineering documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    EngineeringTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        engineering_coverage_score: report.engineering_coverage_score.unwrap_or(0.0),
        engineering_coverage_rating: rating_word(report.engineering_coverage_score.unwrap_or(0.0)).to_string(),
        documentation_quality_score: report.documentation_quality_score.unwrap_or(0.0),
        documentation_quality_rating: rating_word(report.documentation_quality_score.unwrap_or(0.0)).to_string(),
        traceability_consistency_score: report.traceability_consistency_score.unwrap_or(0.0),
        traceability_consistency_rating: rating_word(report.traceability_consistency_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_engineering_section_compliance(store),
        audit_standards: build_engineering_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_engineering_template(ctx: &EngineeringTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_definition_score: f64,
    pub feature_definition_rating: String,
    pub product_definition_score: f64,
    pub product_definition_rating: String,
    pub documentation_quality_score: f64,
    pub documentation_quality_rating: String,
    pub product_readiness_score: f64,
    pub product_readiness_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_feature_context(
    session: registry::store::FeatureSessionInfo,
    report: registry::store::FeatureReportWithFindings,
    store: &registry::RegistryStore,
) -> FeatureTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The Feature documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The Feature documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The Feature documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    FeatureTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        feature_definition_score: report.feature_definition_score.unwrap_or(0.0),
        feature_definition_rating: rating_word(report.feature_definition_score.unwrap_or(0.0)).to_string(),
        product_definition_score: report.product_definition_score.unwrap_or(0.0),
        product_definition_rating: rating_word(report.product_definition_score.unwrap_or(0.0)).to_string(),
        documentation_quality_score: report.documentation_quality_score.unwrap_or(0.0),
        documentation_quality_rating: rating_word(report.documentation_quality_score.unwrap_or(0.0)).to_string(),
        product_readiness_score: report.product_readiness_score.unwrap_or(0.0),
        product_readiness_rating: rating_word(report.product_readiness_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_feature_section_compliance(store),
        audit_standards: build_feature_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_feature_template(ctx: &FeatureTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureTechnicalTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: f64,
    pub feature_mapping_rating: String,
    pub technical_realization_score: f64,
    pub technical_realization_rating: String,
    pub documentation_quality_score: f64,
    pub documentation_quality_rating: String,
    pub implementation_readiness_score: f64,
    pub implementation_readiness_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_feature_technical_context(
    session: registry::store::FeatureTechnicalSessionInfo,
    report: registry::store::FeatureTechnicalReportWithFindings,
    store: &registry::RegistryStore,
) -> FeatureTechnicalTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The Feature Technical Design documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The Feature Technical Design documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The Feature Technical Design documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    FeatureTechnicalTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        feature_mapping_score: report.feature_mapping_score.unwrap_or(0.0),
        feature_mapping_rating: rating_word(report.feature_mapping_score.unwrap_or(0.0)).to_string(),
        technical_realization_score: report.technical_realization_score.unwrap_or(0.0),
        technical_realization_rating: rating_word(report.technical_realization_score.unwrap_or(0.0)).to_string(),
        documentation_quality_score: report.documentation_quality_score.unwrap_or(0.0),
        documentation_quality_rating: rating_word(report.documentation_quality_score.unwrap_or(0.0)).to_string(),
        implementation_readiness_score: report.implementation_readiness_score.unwrap_or(0.0),
        implementation_readiness_rating: rating_word(report.implementation_readiness_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_feature_technical_section_compliance(store),
        audit_standards: build_feature_technical_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_feature_technical_template(ctx: &FeatureTechnicalTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureDesignTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: f64,
    pub feature_mapping_rating: String,
    pub user_experience_score: f64,
    pub user_experience_rating: String,
    pub documentation_quality_score: f64,
    pub documentation_quality_rating: String,
    pub design_readiness_score: f64,
    pub design_readiness_rating: String,
    pub doc_scores: Vec<TeraDocScore>,
    pub validation_scores: Vec<TeraValidationScore>,
    pub section_compliance: Vec<TeraSectionCompliance>,
    pub audit_standards: Vec<TeraAuditStandardSummary>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_feature_design_context(
    session: registry::store::FeatureDesignSessionInfo,
    report: registry::store::FeatureDesignReportWithFindings,
    store: &registry::RegistryStore,
) -> FeatureDesignTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The Feature Design documentation has improved since the last audit.".to_string()
            } else if report.score < prev {
                "The Feature Design documentation has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The Feature Design documentation score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut doc_scores: Vec<TeraDocScore> = report.doc_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for d in &mut doc_scores {
        d.rating = rating_word(d.score).to_string();
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    FeatureDesignTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        feature_mapping_score: report.feature_mapping_score.unwrap_or(0.0),
        feature_mapping_rating: rating_word(report.feature_mapping_score.unwrap_or(0.0)).to_string(),
        user_experience_score: report.user_experience_score.unwrap_or(0.0),
        user_experience_rating: rating_word(report.user_experience_score.unwrap_or(0.0)).to_string(),
        documentation_quality_score: report.documentation_quality_score.unwrap_or(0.0),
        documentation_quality_rating: rating_word(report.documentation_quality_score.unwrap_or(0.0)).to_string(),
        design_readiness_score: report.design_readiness_score.unwrap_or(0.0),
        design_readiness_rating: rating_word(report.design_readiness_score.unwrap_or(0.0)).to_string(),
        doc_scores,
        validation_scores,
        section_compliance: build_feature_design_section_compliance(store),
        audit_standards: build_feature_design_audit_standards_summary(),
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_feature_design_template(ctx: &FeatureDesignTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

// Deterministic Runtime is cross-cutting (scans Architecture + Engineering
// together) rather than a single documentation collection with a Required
// Sections table, so it has no Structural Compliance Matrix or
// Audit Standard Rubrics section — those concepts don't apply here.
#[derive(Debug, Clone, Serialize)]
pub struct DeterministicRuntimeTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub runtime_model_score: f64,
    pub runtime_model_rating: String,
    pub engineering_principles_score: f64,
    pub engineering_principles_rating: String,
    pub runtime_integrity_score: f64,
    pub runtime_integrity_rating: String,
    pub validation_scores: Vec<TeraValidationScore>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_deterministic_runtime_context(
    session: registry::store::DeterministicRuntimeSessionInfo,
    report: registry::store::DeterministicRuntimeReportWithFindings,
) -> DeterministicRuntimeTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "The runtime model has become more deterministic since the last audit.".to_string()
            } else if report.score < prev {
                "The runtime model has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The runtime model score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    DeterministicRuntimeTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        runtime_model_score: report.runtime_model_score.unwrap_or(0.0),
        runtime_model_rating: rating_word(report.runtime_model_score.unwrap_or(0.0)).to_string(),
        engineering_principles_score: report.engineering_principles_score.unwrap_or(0.0),
        engineering_principles_rating: rating_word(report.engineering_principles_score.unwrap_or(0.0)).to_string(),
        runtime_integrity_score: report.runtime_integrity_score.unwrap_or(0.0),
        runtime_integrity_rating: rating_word(report.runtime_integrity_score.unwrap_or(0.0)).to_string(),
        validation_scores,
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_deterministic_runtime_template(ctx: &DeterministicRuntimeTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

// External Context Ownership is cross-cutting (scans External Context plus
// every referencing domain) rather than a single documentation collection
// with a Required Sections table, so — like Deterministic Runtime — it has
// no Structural Compliance Matrix or Audit Standard Rubrics section.
#[derive(Debug, Clone, Serialize)]
pub struct ExternalContextOwnershipTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub dependency_coverage_score: f64,
    pub dependency_coverage_rating: String,
    pub documentation_integration_score: f64,
    pub documentation_integration_rating: String,
    pub consistency_score: f64,
    pub consistency_rating: String,
    pub validation_scores: Vec<TeraValidationScore>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_external_context_ownership_context(
    session: registry::store::ExternalContextOwnershipSessionInfo,
    report: registry::store::ExternalContextOwnershipReportWithFindings,
) -> ExternalContextOwnershipTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "External Context ownership across the documentation ecosystem has improved since the last audit.".to_string()
            } else if report.score < prev {
                "External Context ownership across the documentation ecosystem has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The External Context ownership score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    ExternalContextOwnershipTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        dependency_coverage_score: report.dependency_coverage_score.unwrap_or(0.0),
        dependency_coverage_rating: rating_word(report.dependency_coverage_score.unwrap_or(0.0)).to_string(),
        documentation_integration_score: report.documentation_integration_score.unwrap_or(0.0),
        documentation_integration_rating: rating_word(report.documentation_integration_score.unwrap_or(0.0)).to_string(),
        consistency_score: report.consistency_score.unwrap_or(0.0),
        consistency_rating: rating_word(report.consistency_score.unwrap_or(0.0)).to_string(),
        validation_scores,
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_external_context_ownership_template(ctx: &ExternalContextOwnershipTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

// Implementation reads actual source code under the declared implementation
// folder rather than a docs/raw/*.md collection, so — like Deterministic
// Runtime and External Context Ownership — it has no Structural Compliance
// Matrix or Audit Standard Rubrics section.
#[derive(Debug, Clone, Serialize)]
pub struct ImplementationTeraContext {
    pub session_id: String,
    pub score: f64,
    pub rating: String,
    pub rating_description: String,
    pub previous_score: Option<f64>,
    pub score_change_display: String,
    pub trend_text: String,
    pub git_revision: String,
    pub created_at: String,
    pub engineering_readiness: String,
    pub architectural_conformance_score: f64,
    pub architectural_conformance_rating: String,
    pub feature_conformance_score: f64,
    pub feature_conformance_rating: String,
    pub engineering_conformance_score: f64,
    pub engineering_conformance_rating: String,
    pub documentation_integrity_score: f64,
    pub documentation_integrity_rating: String,
    pub implementation_quality_score: f64,
    pub implementation_quality_rating: String,
    pub validation_scores: Vec<TeraValidationScore>,
    pub critical_findings: Vec<TeraFindingItem>,
    pub major_findings: Vec<TeraFindingItem>,
    pub minor_findings: Vec<TeraFindingItem>,
    pub observations: Vec<TeraFindingItem>,
    pub recommendations: Vec<TeraRecommendationItem>,
    pub total_checks: usize,
}

fn build_implementation_context(
    session: registry::store::ImplementationSessionInfo,
    report: registry::store::ImplementationReportWithFindings,
) -> ImplementationTeraContext {
    let score_change_display = match report.previous_score {
        Some(prev) => {
            let diff = report.score - prev;
            if diff > 0.0 {
                format!("+{:.1} (improvement)", diff)
            } else if diff < 0.0 {
                format!("{:.1} (regression)", diff)
            } else {
                "0 (no change)".to_string()
            }
        }
        None => "N/A (baseline)".to_string(),
    };

    let trend_text = match report.previous_score {
        Some(prev) => {
            if report.score > prev {
                "Implementation conformance has improved since the last audit.".to_string()
            } else if report.score < prev {
                "Implementation conformance has regressed since the last audit. Review findings below.".to_string()
            } else {
                "The implementation conformance score is unchanged from the last audit.".to_string()
            }
        }
        None => String::new(),
    };

    let mut critical_findings = Vec::new();
    let mut major_findings = Vec::new();
    let mut minor_findings = Vec::new();
    let mut observations = Vec::new();

    for f in &report.findings {
        let item = TeraFindingItem {
            check_id: f.check_id.clone(),
            message: f.message.clone(),
            location: f.location.clone(),
            evidence_excerpt: f.evidence_excerpt.clone(),
            evidence_source: f.evidence_source.clone(),
        };
        match f.severity.as_str() {
            "critical" => critical_findings.push(item),
            "error" | "major" => major_findings.push(item),
            "warning" | "minor" => minor_findings.push(item),
            _ => observations.push(item),
        }
    }

    let mut validation_scores: Vec<TeraValidationScore> = report.validation_scores
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();
    for v in &mut validation_scores {
        v.rating = rating_word(v.score).to_string();
    }

    ImplementationTeraContext {
        session_id: report.session_id.clone(),
        score: report.score,
        rating: rating_word(report.score).to_string(),
        rating_description: rating_description(report.score).to_string(),
        previous_score: report.previous_score,
        score_change_display,
        trend_text,
        git_revision: report.git_revision.unwrap_or_default(),
        created_at: session.created_at.clone(),
        engineering_readiness: report.engineering_readiness.clone(),
        architectural_conformance_score: report.architectural_conformance_score.unwrap_or(0.0),
        architectural_conformance_rating: rating_word(report.architectural_conformance_score.unwrap_or(0.0)).to_string(),
        feature_conformance_score: report.feature_conformance_score.unwrap_or(0.0),
        feature_conformance_rating: rating_word(report.feature_conformance_score.unwrap_or(0.0)).to_string(),
        engineering_conformance_score: report.engineering_conformance_score.unwrap_or(0.0),
        engineering_conformance_rating: rating_word(report.engineering_conformance_score.unwrap_or(0.0)).to_string(),
        documentation_integrity_score: report.documentation_integrity_score.unwrap_or(0.0),
        documentation_integrity_rating: rating_word(report.documentation_integrity_score.unwrap_or(0.0)).to_string(),
        implementation_quality_score: report.implementation_quality_score.unwrap_or(0.0),
        implementation_quality_rating: rating_word(report.implementation_quality_score.unwrap_or(0.0)).to_string(),
        validation_scores,
        critical_findings,
        major_findings,
        minor_findings,
        observations,
        recommendations: report.recommendations.iter().map(|r| {
            let priority_label = match r.priority.as_str() {
                "P1" => "Critical",
                "P2" => "High",
                "P3" => "Medium",
                "P4" => "Low",
                _ => &r.priority,
            };
            TeraRecommendationItem {
                category: r.category.clone(),
                priority: priority_label.to_string(),
                description: r.description.clone(),
                file_path: r.file_path.clone(),
            }
        }).collect(),
        total_checks: report.findings.len(),
    }
}

fn render_implementation_template(ctx: &ImplementationTeraContext, template: &str) -> Result<String> {
    let tera = tera::Tera::default();
    let context = tera::Context::from_serialize(ctx)
        .map_err(|e| anyhow::anyhow!("Failed to serialize Tera context: {}", e))?;
    tera.render_str(template, &context, false)
        .map_err(|e| anyhow::anyhow!("Tera render error: {}", e))
}

// ── Per-Audit Template Contexts (Phase 8) ──────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct BuildTemplateContext {
    pub session_id: String,
    pub score: f64,
    pub date: String,
    pub git_revision: String,
    pub contract_name: String,
    pub declared_produces: String,
    pub artifact_table: String,
    pub execution_success: String,
    pub execution_output: String,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub improvements: String,
}

#[derive(Debug, Clone, Default)]
pub struct SecurityTemplateContext {
    pub session_id: String,
    pub score: f64,
    pub date: String,
    pub git_revision: String,
    pub secrets_scanned: String,
    pub secrets_found: String,
    pub runtime_checks: String,
    pub runtime_issues: String,
    pub high_risk_count: String,
    pub threat_summary: String,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub improvements: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConsistencyTemplateContext {
    pub session_id: String,
    pub score: f64,
    pub date: String,
    pub git_revision: String,
    pub vision_exists: String,
    pub architecture_exists: String,
    pub structure_score: String,
    pub naming_issues_table: String,
    pub cross_references: String,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub improvements: String,
}

#[derive(Debug, Clone, Default)]
pub struct CoverageTemplateContext {
    pub session_id: String,
    pub score: f64,
    pub date: String,
    pub git_revision: String,
    pub features_count: String,
    pub src_files_count: String,
    pub coverage_bar: String,
    pub uncovered_features_list: String,
    pub doc_types_table: String,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub improvements: String,
}

#[derive(Debug, Clone, Default)]
pub struct HelpTemplateContext {
    pub session_id: String,
    pub score: f64,
    pub date: String,
    pub git_revision: String,
    pub engineering_readiness: String,
    pub coverage_score: f64,
    pub navigation_score: f64,
    pub quality_score: f64,
    pub accuracy_score: f64,
    pub errors: Vec<TemplateFinding>,
    pub warnings: Vec<TemplateFinding>,
    pub suggestions: Vec<TemplateFinding>,
    pub improvements: String,
}

// ── High-Level Renderer ────────────────────────────────────────────────

fn build_build_context(store: &registry::RegistryStore) -> BuildTemplateContext {
    let sessions = store.query_build_sessions(1).unwrap_or_default();
    if let Some(s) = sessions.into_iter().next() {
        if let Ok(Some(r)) = store.get_build_report_with_findings(s.id) {
            let mut ctx = BuildTemplateContext {
                session_id: r.session_id.clone(),
                score: r.score,
                date: r.created_at,
                git_revision: r.git_revision.unwrap_or_default(),
                contract_name: r.contract_name.as_deref().unwrap_or("N/A").to_string(),
                declared_produces: r.declared_produces.as_deref().unwrap_or("N/A").to_string(),
                execution_success: r.execution_success
                    .map(|v| if v { "Yes" } else { "No" })
                    .unwrap_or("N/A").to_string(),
                execution_output: r.execution_output.as_deref().unwrap_or("N/A").to_string(),
                ..Default::default()
            };
            ctx.artifact_table = r.actual_artifacts.as_deref().unwrap_or("None").to_string();
            for f in r.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id,
                    message: f.message,
                    location: f.location,
                };
                match f.severity.as_str() {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            return ctx;
        }
    }
    BuildTemplateContext::default()
}

fn build_security_context(store: &registry::RegistryStore) -> SecurityTemplateContext {
    let sessions = store.query_security_sessions(1).unwrap_or_default();
    if let Some(s) = sessions.into_iter().next() {
        if let Ok(Some(r)) = store.get_security_report_with_findings(s.id) {
            let mut ctx = SecurityTemplateContext {
                session_id: r.session_id.clone(),
                score: r.score,
                date: r.created_at,
                git_revision: r.git_revision.unwrap_or_default(),
                secrets_scanned: r.secrets_scanned.to_string(),
                secrets_found: r.secrets_found.to_string(),
                runtime_checks: r.runtime_checks.to_string(),
                runtime_issues: r.runtime_issues.to_string(),
                high_risk_count: r.high_risk_findings.to_string(),
                threat_summary: r.threat_summary.unwrap_or_default(),
                ..Default::default()
            };
            for f in r.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id,
                    message: f.message,
                    location: f.location,
                };
                match f.severity.as_str() {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            return ctx;
        }
    }
    SecurityTemplateContext::default()
}

fn build_consistency_context(store: &registry::RegistryStore) -> ConsistencyTemplateContext {
    let sessions = store.query_consistency_sessions(1).unwrap_or_default();
    if let Some(s) = sessions.into_iter().next() {
        if let Ok(Some(r)) = store.get_consistency_report_with_findings(s.id) {
            let mut ctx = ConsistencyTemplateContext {
                session_id: r.session_id.clone(),
                score: r.score,
                date: r.created_at,
                git_revision: r.git_revision.unwrap_or_default(),
                vision_exists: if r.vision_exists { "Yes" } else { "No" }.to_string(),
                architecture_exists: if r.architecture_exists { "Yes" } else { "No" }.to_string(),
                structure_score: r.structure_score.map(|v| format!("{:.1}", v)).unwrap_or("N/A".to_string()),
                naming_issues_table: r.naming_issues.unwrap_or_default(),
                cross_references: r.cross_references.to_string(),
                ..Default::default()
            };
            for f in r.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id,
                    message: f.message,
                    location: f.location,
                };
                match f.severity.as_str() {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            return ctx;
        }
    }
    ConsistencyTemplateContext::default()
}

fn build_coverage_context(store: &registry::RegistryStore) -> CoverageTemplateContext {
    let sessions = store.query_coverage_sessions(1).unwrap_or_default();
    if let Some(s) = sessions.into_iter().next() {
        if let Ok(Some(r)) = store.get_coverage_report_with_findings(s.id) {
            let mut ctx = CoverageTemplateContext {
                session_id: r.session_id.clone(),
                score: r.score,
                date: r.created_at,
                git_revision: r.git_revision.unwrap_or_default(),
                features_count: r.features_count.to_string(),
                src_files_count: r.src_files_count.to_string(),
                coverage_bar: r.feature_coverage_pct
                    .map(|v| format!("{:.1}%", v))
                    .unwrap_or("N/A".to_string()),
                uncovered_features_list: r.uncovered_features.unwrap_or_default(),
                doc_types_table: r.doc_types_covered.unwrap_or_default(),
                ..Default::default()
            };
            for f in r.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id,
                    message: f.message,
                    location: f.location,
                };
                match f.severity.as_str() {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            return ctx;
        }
    }
    CoverageTemplateContext::default()
}

fn build_help_context(store: &registry::RegistryStore) -> HelpTemplateContext {
    let sessions = store.query_help_sessions(1).unwrap_or_default();
    if let Some(s) = sessions.into_iter().next() {
        if let Ok(Some(r)) = store.get_help_report_with_findings(s.id) {
            let mut ctx = HelpTemplateContext {
                session_id: r.session_id.clone(),
                score: r.score,
                date: r.created_at,
                git_revision: r.git_revision.unwrap_or_default(),
                engineering_readiness: r.engineering_readiness,
                coverage_score: r.coverage_score.unwrap_or(0.0),
                navigation_score: r.navigation_score.unwrap_or(0.0),
                quality_score: r.quality_score.unwrap_or(0.0),
                accuracy_score: r.accuracy_score.unwrap_or(0.0),
                ..Default::default()
            };
            for f in r.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id,
                    message: f.message,
                    location: f.location,
                };
                match f.severity.as_str() {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            return ctx;
        }
    }
    HelpTemplateContext::default()
}

/// Render a per-audit report from a template file + data.
/// Returns rendered markdown or an error if template is missing.
/// Queries the store for the latest session of the given audit type.
pub fn render_report(
    report_type: &str,
    templates_dir: &Path,
    store: &registry::RegistryStore,
) -> Result<String> {
    let template_file = if report_type == "help" {
        "product-guide-report.md"
    } else {
        &format!("{}-report.md", report_type)
    };
    let template_path = templates_dir.join(template_file);
    let template = fs::read_to_string(&template_path)?;
    match report_type {
        "build" => {
            let ctx = build_build_context(store);
            Ok(render_build_template(&ctx, &template))
        }
        "security" => {
            let ctx = build_security_context(store);
            Ok(render_security_template(&ctx, &template))
        }
        "consistency" => {
            let ctx = build_consistency_context(store);
            Ok(render_consistency_template(&ctx, &template))
        }
        "coverage" => {
            let ctx = build_coverage_context(store);
            Ok(render_coverage_template(&ctx, &template))
        }
        "help" => {
            let ctx = build_help_context(store);
            Ok(render_help_template(&ctx, &template))
        }
        "architecture" => {
            let sessions = store.query_architecture_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_architecture_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_architecture_context(s, r, store);
                    return render_architecture_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No architecture report available"))
        }
        "vision" => {
            let sessions = store.query_vision_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_vision_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_vision_context(s, r, store);
                    return render_vision_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No vision report available"))
        }
        "design" => {
            let sessions = store.query_design_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_design_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_design_context(s, r, store);
                    return render_design_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No design report available"))
        }
        "readme" => {
            let sessions = store.query_readme_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_readme_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_readme_context(s, r);
                    return render_readme_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No readme report available"))
        }
        "prototype" => {
            let sessions = store.query_prototype_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_prototype_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_prototype_context(s, r, store);
                    return render_prototype_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No prototype report available"))
        }
        "external-context" => {
            let sessions = store.query_external_context_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_external_context_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_external_context_context(s, r, store);
                    return render_external_context_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No external-context report available"))
        }
        "engineering" => {
            let sessions = store.query_engineering_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_engineering_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_engineering_context(s, r, store);
                    return render_engineering_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No engineering report available"))
        }
        "feature" => {
            let sessions = store.query_feature_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_feature_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_feature_context(s, r, store);
                    return render_feature_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No feature report available"))
        }
        "feature-technical" => {
            let sessions = store.query_feature_technical_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_feature_technical_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_feature_technical_context(s, r, store);
                    return render_feature_technical_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No feature-technical report available"))
        }
        "feature-design" => {
            let sessions = store.query_feature_design_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_feature_design_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_feature_design_context(s, r, store);
                    return render_feature_design_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No feature-design report available"))
        }
        "deterministic-runtime" => {
            let sessions = store.query_deterministic_runtime_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_deterministic_runtime_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_deterministic_runtime_context(s, r);
                    return render_deterministic_runtime_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No deterministic-runtime report available"))
        }
        "external-context-ownership" => {
            let sessions = store.query_external_context_ownership_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_external_context_ownership_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_external_context_ownership_context(s, r);
                    return render_external_context_ownership_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No external-context-ownership report available"))
        }
        "implementation" => {
            let sessions = store.query_implementation_sessions(1)?;
            if let Some(s) = sessions.into_iter().next() {
                let report = store.get_implementation_report_with_findings(s.id)?;
                if let Some(r) = report {
                    let ctx = build_implementation_context(s, r);
                    return render_implementation_template(&ctx, &template);
                }
            }
            Err(anyhow::anyhow!("No implementation report available"))
        }
        _ => Err(anyhow::anyhow!("Unknown report type: {}", report_type)),
    }
}

/// Render a per-audit report using in-memory pipeline report data (for --report flag in audit).
pub fn render_report_from_pipeline(
    report_type: &str,
    template: &str,
    report: &schemas::audit::PipelineReport,
) -> String {
    use schemas::audit::Severity;
    // Helper to classify an AuditFinding's severity
    let classify = |sev: &Severity| -> &str {
        match sev {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Suggestion => "suggestion",
        }
    };
    // Build per-audit context from the pipeline report
    match report_type {
        "build" => {
            let mut ctx = BuildTemplateContext {
                session_id: String::new(),
                score: report.score,
                date: chrono_now_iso(),
                contract_name: report.pipeline.as_str().to_string(),
                ..Default::default()
            };
            for f in &report.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id.clone(),
                    message: f.message.clone(),
                    location: f.location.clone(),
                };
                match classify(&f.severity) {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            render_build_template(&ctx, template)
        }
        "security" => {
            let mut ctx = SecurityTemplateContext {
                session_id: String::new(),
                score: report.score,
                date: chrono_now_iso(),
                ..Default::default()
            };
            for f in &report.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id.clone(),
                    message: f.message.clone(),
                    location: f.location.clone(),
                };
                match classify(&f.severity) {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            render_security_template(&ctx, template)
        }
        "consistency" => {
            let mut ctx = ConsistencyTemplateContext {
                session_id: String::new(),
                score: report.score,
                date: chrono_now_iso(),
                ..Default::default()
            };
            for f in &report.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id.clone(),
                    message: f.message.clone(),
                    location: f.location.clone(),
                };
                match classify(&f.severity) {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            render_consistency_template(&ctx, template)
        }
        "coverage" => {
            let mut ctx = CoverageTemplateContext {
                session_id: String::new(),
                score: report.score,
                date: chrono_now_iso(),
                ..Default::default()
            };
            for f in &report.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id.clone(),
                    message: f.message.clone(),
                    location: f.location.clone(),
                };
                match classify(&f.severity) {
                    "error" => ctx.errors.push(tf),
                    "warning" => ctx.warnings.push(tf),
                    _ => ctx.suggestions.push(tf),
                }
            }
            render_coverage_template(&ctx, template)
        }
        _ => {
            // Fallback: use old template engine
            let mut errors = Vec::new();
            let mut warnings = Vec::new();
            let mut suggestions = Vec::new();
            for f in &report.findings {
                let tf = TemplateFinding {
                    check_id: f.check_id.clone(),
                    message: f.message.clone(),
                    location: f.location.clone(),
                };
                match classify(&f.severity) {
                    "error" => errors.push(tf),
                    "warning" => warnings.push(tf),
                    _ => suggestions.push(tf),
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
            };
            render_from_template(template, &ctx)
        }
    }
}

fn chrono_now_iso() -> String {
    // Simple ISO-like timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

fn render_build_template(ctx: &BuildTemplateContext, template: &str) -> String {
    let mut t = template
        .replace("{{date}}", &ctx.date)
        .replace("{{score}}", &format!("{:.1}", ctx.score))
        .replace("{{score_bar}}", &render_score_bar(ctx.score))
        .replace("{{session_id}}", &ctx.session_id)
        .replace("{{git_revision}}", &ctx.git_revision)
        .replace("{{contract_name}}", &ctx.contract_name)
        .replace("{{declared_produces}}", &ctx.declared_produces)
        .replace("{{artifact_table}}", &ctx.artifact_table)
        .replace("{{execution_success}}", &ctx.execution_success)
        .replace("{{execution_output}}", &ctx.execution_output)
        .replace("{{errors_table}}", &render_finding_table(&ctx.errors))
        .replace("{{warnings_table}}", &render_finding_table(&ctx.warnings))
        .replace("{{suggestions_table}}", &render_finding_table(&ctx.suggestions))
        .replace("{{errors_count}}", &ctx.errors.len().to_string())
        .replace("{{warnings_count}}", &ctx.warnings.len().to_string())
        .replace("{{suggestions_count}}", &ctx.suggestions.len().to_string())
        .replace("{{improvements}}", &ctx.improvements);
    t = strip_conditional_blocks(&t, &ctx.errors, "errors");
    t = strip_conditional_blocks(&t, &ctx.warnings, "warnings");
    t = strip_conditional_blocks(&t, &ctx.suggestions, "suggestions");
    t = strip_conditional_blocks_str(&t, "execution_output", &ctx.execution_output);
    t
}

fn render_security_template(ctx: &SecurityTemplateContext, template: &str) -> String {
    let mut t = template
        .replace("{{date}}", &ctx.date)
        .replace("{{score}}", &format!("{:.1}", ctx.score))
        .replace("{{score_bar}}", &render_score_bar(ctx.score))
        .replace("{{session_id}}", &ctx.session_id)
        .replace("{{git_revision}}", &ctx.git_revision)
        .replace("{{secrets_scanned}}", &ctx.secrets_scanned)
        .replace("{{secrets_found}}", &ctx.secrets_found)
        .replace("{{runtime_checks}}", &ctx.runtime_checks)
        .replace("{{runtime_issues}}", &ctx.runtime_issues)
        .replace("{{high_risk_count}}", &ctx.high_risk_count)
        .replace("{{threat_summary}}", &ctx.threat_summary)
        .replace("{{errors_table}}", &render_finding_table(&ctx.errors))
        .replace("{{warnings_table}}", &render_finding_table(&ctx.warnings))
        .replace("{{suggestions_table}}", &render_finding_table(&ctx.suggestions))
        .replace("{{errors_count}}", &ctx.errors.len().to_string())
        .replace("{{warnings_count}}", &ctx.warnings.len().to_string())
        .replace("{{suggestions_count}}", &ctx.suggestions.len().to_string())
        .replace("{{improvements}}", &ctx.improvements);
    t = strip_conditional_blocks(&t, &ctx.errors, "errors");
    t = strip_conditional_blocks(&t, &ctx.warnings, "warnings");
    t = strip_conditional_blocks(&t, &ctx.suggestions, "suggestions");
    t
}

fn render_consistency_template(ctx: &ConsistencyTemplateContext, template: &str) -> String {
    let mut t = template
        .replace("{{date}}", &ctx.date)
        .replace("{{score}}", &format!("{:.1}", ctx.score))
        .replace("{{score_bar}}", &render_score_bar(ctx.score))
        .replace("{{session_id}}", &ctx.session_id)
        .replace("{{git_revision}}", &ctx.git_revision)
        .replace("{{vision_exists}}", &ctx.vision_exists)
        .replace("{{architecture_exists}}", &ctx.architecture_exists)
        .replace("{{structure_score}}", &ctx.structure_score)
        .replace("{{naming_issues_table}}", &ctx.naming_issues_table)
        .replace("{{cross_references}}", &ctx.cross_references)
        .replace("{{errors_table}}", &render_finding_table(&ctx.errors))
        .replace("{{warnings_table}}", &render_finding_table(&ctx.warnings))
        .replace("{{suggestions_table}}", &render_finding_table(&ctx.suggestions))
        .replace("{{errors_count}}", &ctx.errors.len().to_string())
        .replace("{{warnings_count}}", &ctx.warnings.len().to_string())
        .replace("{{suggestions_count}}", &ctx.suggestions.len().to_string())
        .replace("{{improvements}}", &ctx.improvements);
    t = strip_conditional_blocks(&t, &ctx.errors, "errors");
    t = strip_conditional_blocks(&t, &ctx.warnings, "warnings");
    t = strip_conditional_blocks(&t, &ctx.suggestions, "suggestions");
    t = strip_conditional_blocks_str(&t, "naming_issues_table", &ctx.naming_issues_table);
    t
}

fn render_coverage_template(ctx: &CoverageTemplateContext, template: &str) -> String {
    let mut t = template
        .replace("{{date}}", &ctx.date)
        .replace("{{score}}", &format!("{:.1}", ctx.score))
        .replace("{{score_bar}}", &render_score_bar(ctx.score))
        .replace("{{session_id}}", &ctx.session_id)
        .replace("{{git_revision}}", &ctx.git_revision)
        .replace("{{features_count}}", &ctx.features_count)
        .replace("{{src_files_count}}", &ctx.src_files_count)
        .replace("{{coverage_bar}}", &ctx.coverage_bar)
        .replace("{{uncovered_features_list}}", &ctx.uncovered_features_list)
        .replace("{{doc_types_table}}", &ctx.doc_types_table)
        .replace("{{errors_table}}", &render_finding_table(&ctx.errors))
        .replace("{{warnings_table}}", &render_finding_table(&ctx.warnings))
        .replace("{{suggestions_table}}", &render_finding_table(&ctx.suggestions))
        .replace("{{errors_count}}", &ctx.errors.len().to_string())
        .replace("{{warnings_count}}", &ctx.warnings.len().to_string())
        .replace("{{suggestions_count}}", &ctx.suggestions.len().to_string())
        .replace("{{improvements}}", &ctx.improvements);
    t = strip_conditional_blocks(&t, &ctx.errors, "errors");
    t = strip_conditional_blocks(&t, &ctx.warnings, "warnings");
    t = strip_conditional_blocks(&t, &ctx.suggestions, "suggestions");
    t = strip_conditional_blocks_str(&t, "uncovered_features_list", &ctx.uncovered_features_list);
    t
}

fn render_help_template(ctx: &HelpTemplateContext, template: &str) -> String {
    let mut t = template
        .replace("{{date}}", &ctx.date)
        .replace("{{score}}", &format!("{:.1}", ctx.score))
        .replace("{{score_bar}}", &render_score_bar(ctx.score))
        .replace("{{session_id}}", &ctx.session_id)
        .replace("{{git_revision}}", &ctx.git_revision)
        .replace("{{engineering_readiness}}", &ctx.engineering_readiness)
        .replace("{{coverage_score}}", &format!("{:.1}", ctx.coverage_score))
        .replace("{{navigation_score}}", &format!("{:.1}", ctx.navigation_score))
        .replace("{{quality_score}}", &format!("{:.1}", ctx.quality_score))
        .replace("{{accuracy_score}}", &format!("{:.1}", ctx.accuracy_score))
        .replace("{{errors_table}}", &render_finding_table(&ctx.errors))
        .replace("{{warnings_table}}", &render_finding_table(&ctx.warnings))
        .replace("{{suggestions_table}}", &render_finding_table(&ctx.suggestions))
        .replace("{{errors_count}}", &ctx.errors.len().to_string())
        .replace("{{warnings_count}}", &ctx.warnings.len().to_string())
        .replace("{{suggestions_count}}", &ctx.suggestions.len().to_string())
        .replace("{{improvements}}", &ctx.improvements);
    t = strip_conditional_blocks(&t, &ctx.errors, "errors");
    t = strip_conditional_blocks(&t, &ctx.warnings, "warnings");
    t = strip_conditional_blocks(&t, &ctx.suggestions, "suggestions");
    t
}

fn strip_conditional_blocks(template: &str, items: &[TemplateFinding], section: &str) -> String {
    let start = format!("{{{{#{}}}}}\n", section);
    let end = format!("\n{{{{/{}}}}}", section);
    if items.is_empty() {
        // Remove the entire block including surrounding whitespace
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
        // Remove only the marker tags, keep content
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

/// Legacy default template embedded for backward compatibility.
/// New code should use per-audit templates from the filesystem.
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

    #[test]
    fn real_help_template_renders_without_error() {
        // Reads the actual shipped `product-guide-report.md` and renders it with a
        // realistic context — this is the template `samgraha report help`
        // reads via `render_report()`'s `"help"` arm; previously there was
        // no such file and no such arm, so `report help` failed outright.
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/product-guide-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = HelpTemplateContext {
            session_id: "sess-1".into(),
            score: 43.6,
            date: "2026-07-06".into(),
            git_revision: "abc123".into(),
            engineering_readiness: "NOT_READY".into(),
            coverage_score: 14.3,
            navigation_score: 75.0,
            quality_score: 40.0,
            accuracy_score: 57.1,
            errors: vec![TemplateFinding { check_id: "PC1".into(), message: "missing docs".into(), location: Some("docs/raw/help/commands/".into()) }],
            warnings: vec![TemplateFinding { check_id: "PQ4".into(), message: "short page".into(), location: None }],
            suggestions: vec![],
            improvements: String::new(),
        };
        let rendered = render_help_template(&ctx, &template);

        assert!(!rendered.contains("{{"), "unresolved placeholder left in rendered output: {}", rendered);
        assert!(rendered.contains("43.6"));
        assert!(rendered.contains("NOT_READY"));
        assert!(rendered.contains("PC1"));
        assert!(rendered.contains("PQ4"));
        // No suggestions were supplied — the conditional block must be stripped, not rendered empty.
        assert!(!rendered.contains("Suggestions (0)"));
    }

    #[test]
    fn real_architecture_template_renders_without_error() {
        // Reads the actual shipped template (not a hardcoded string) and
        // renders it with a realistic context — catches Tera syntax errors
        // that only show up at render time, not at compile time.
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/architecture-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = ArchitectureTeraContext {
            session_id: "sess-1".into(),
            score: 82.5,
            rating: rating_word(82.5).to_string(),
            rating_description: rating_description(82.5).to_string(),
            previous_score: Some(80.0),
            score_change_display: "+2.5 (improvement)".into(),
            trend_text: "Improved since last audit.".into(),
            git_revision: "abc123".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "YES".into(),
            collection_integrity_score: 90.0,
            collection_integrity_rating: rating_word(90.0).to_string(),
            structural_integrity_score: 85.0,
            structural_integrity_rating: rating_word(85.0).to_string(),
            consistency_score: 80.0,
            consistency_rating: rating_word(80.0).to_string(),
            cross_repo_score: 70.0,
            cross_repo_rating: rating_word(70.0).to_string(),
            doc_scores: vec![TeraDocScore { name: "overview.md".into(), score: 90.0, rating: rating_word(90.0).to_string() }],
            validation_scores: vec![TeraValidationScore { id: "A1".into(), score: 100.0, rating: rating_word(100.0).to_string() }],
            section_compliance: build_section_compliance_fixture(),
            audit_standards: build_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![TeraFindingItem {
                check_id: "A9".into(),
                message: "Terminology drift between two documents".into(),
                location: Some("docs/raw/architecture/overview.md".into()),
                evidence_excerpt: Some("uses 'Repository' and 'Store' interchangeably".into()),
                evidence_source: Some("section_id=5, paragraph_index=2".into()),
            }],
            minor_findings: vec![],
            observations: vec![TeraFindingItem {
                check_id: "A4".into(),
                message: "No duplication detected".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            recommendations: vec![],
            total_checks: 13,
        };

        let rendered = render_architecture_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("82.5"));
        assert!(rendered.contains("Good")); // rating_word(82.5)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("uses 'Repository' and 'Store' interchangeably"));
        // Empty sections must not leave a dangling heading with no content
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Minor"));
    }

    fn build_section_compliance_fixture() -> Vec<TeraSectionCompliance> {
        vec![
            TeraSectionCompliance { semantic_type: "system_overview".into(), required: true, doc_count: 3, total_docs: 3, status: "Complete".into() },
            TeraSectionCompliance { semantic_type: "component_model".into(), required: true, doc_count: 1, total_docs: 3, status: "Partial".into() },
        ]
    }

    #[test]
    fn real_vision_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/vision-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = VisionTeraContext {
            session_id: "sess-2".into(),
            score: 88.0,
            rating: rating_word(88.0).to_string(),
            rating_description: rating_description(88.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "def456".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            vision_content_score: 90.0,
            vision_content_rating: rating_word(90.0).to_string(),
            tech_independence_score: 100.0,
            tech_independence_rating: rating_word(100.0).to_string(),
            traceability_consistency_score: 80.0,
            traceability_consistency_rating: rating_word(80.0).to_string(),
            doc_quality_score: 85.0,
            doc_quality_rating: rating_word(85.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "V6".into(), score: 100.0, rating: rating_word(100.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "purpose".into(), required: true, doc_count: 1, total_docs: 1, status: "Complete".into() },
                TeraSectionCompliance { semantic_type: "pillars".into(), required: false, doc_count: 0, total_docs: 1, status: "Missing".into() },
            ],
            audit_standards: build_vision_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![TeraFindingItem {
                check_id: "V4".into(),
                message: "Guiding Principles section not found".into(),
                location: Some("docs/raw/vision/vision.md".into()),
                evidence_excerpt: None,
                evidence_source: None,
            }],
            observations: vec![],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_vision_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("88.0"));
        assert!(rendered.contains("Good")); // rating_word(88.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("Guiding Principles section not found"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Major"));
    }

    #[test]
    fn build_vision_audit_standards_summary_covers_all_required_sections() {
        let summary = build_vision_audit_standards_summary();
        for (semantic_type, required) in VISION_SECTION_TYPES {
            if !required {
                continue;
            }
            assert!(
                summary.iter().any(|s| s.semantic_type == *semantic_type),
                "missing audit-standard summary for required vision section '{}'",
                semantic_type
            );
        }
    }

    #[test]
    fn real_design_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/design-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = DesignTeraContext {
            session_id: "sess-3".into(),
            score: 91.0,
            rating: rating_word(91.0).to_string(),
            rating_description: rating_description(91.0).to_string(),
            previous_score: Some(85.0),
            score_change_display: "+6.0 (improvement)".into(),
            trend_text: "The design documentation has improved since the last audit.".into(),
            git_revision: "abc789".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            design_system_score: 95.0,
            design_system_rating: rating_word(95.0).to_string(),
            doc_quality_score: 90.0,
            doc_quality_rating: rating_word(90.0).to_string(),
            design_quality_score: 88.0,
            design_quality_rating: rating_word(88.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "D4".into(), score: 100.0, rating: rating_word(100.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "design_principles".into(), required: true, doc_count: 1, total_docs: 1, status: "Complete".into() },
            ],
            audit_standards: build_design_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![],
            observations: vec![TeraFindingItem {
                check_id: "D8".into(),
                message: "Cross-repository reuse not mentioned".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_design_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("91.0"));
        assert!(rendered.contains("Very Good")); // rating_word(91.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("Cross-repository reuse not mentioned"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Major"));
    }

    #[test]
    fn build_design_audit_standards_summary_covers_all_required_sections() {
        let summary = build_design_audit_standards_summary();
        for (semantic_type, required) in DESIGN_SECTION_TYPES {
            if !required {
                continue;
            }
            assert!(
                summary.iter().any(|s| s.semantic_type == *semantic_type),
                "missing audit-standard summary for required design section '{}'",
                semantic_type
            );
        }
    }

    #[test]
    fn real_readme_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/readme-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = ReadmeTeraContext {
            session_id: "sess-4".into(),
            score: 72.0,
            rating: rating_word(72.0).to_string(),
            rating_description: rating_description(72.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "def123".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "NEEDS_WORK".into(),
            repo_introduction_score: 100.0,
            repo_introduction_rating: rating_word(100.0).to_string(),
            doc_navigation_score: 66.7,
            doc_navigation_rating: rating_word(66.7).to_string(),
            doc_quality_score: 60.0,
            doc_quality_rating: rating_word(60.0).to_string(),
            maintainability_score: 100.0,
            maintainability_rating: rating_word(100.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "R10".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            audit_standards: build_readme_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![TeraFindingItem {
                check_id: "R10".into(),
                message: "No Getting Started/Installation section found".into(),
                location: Some("README.md".into()),
                evidence_excerpt: None,
                evidence_source: None,
            }],
            minor_findings: vec![],
            observations: vec![],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_readme_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("72.0"));
        assert!(rendered.contains("Acceptable")); // rating_word(72.0)
        // No Structural Compliance Matrix *section* — the intro paragraph
        // explains why by name, so check for the numbered heading, not the phrase.
        assert!(!rendered.contains("## 4. Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("No Getting Started/Installation section found"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Minor"));
    }

    #[test]
    fn real_prototype_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/prototype-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = PrototypeTeraContext {
            session_id: "sess-5".into(),
            score: 68.0,
            rating: rating_word(68.0).to_string(),
            rating_description: rating_description(68.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "aaa111".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "NOT_READY".into(),
            product_validation_score: 75.0,
            product_validation_rating: rating_word(75.0).to_string(),
            runtime_validation_score: 50.0,
            runtime_validation_rating: rating_word(50.0).to_string(),
            engineering_validation_score: 100.0,
            engineering_validation_rating: rating_word(100.0).to_string(),
            validation_quality_score: 66.7,
            validation_quality_rating: rating_word(66.7).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "P6".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "mock_apis".into(), required: true, doc_count: 0, total_docs: 2, status: "Missing".into() },
            ],
            audit_standards: build_prototype_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![],
            observations: vec![],
            recommendations: vec![],
            total_checks: 15,
        };

        let rendered = render_prototype_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("68.0"));
        assert!(rendered.contains("— **Needs Improvement**")); // rating_word(68.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("mock_apis"));
    }

    #[test]
    fn real_external_context_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/external-context-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = ExternalContextTeraContext {
            session_id: "sess-6".into(),
            score: 84.0,
            rating: rating_word(84.0).to_string(),
            rating_description: rating_description(84.0).to_string(),
            previous_score: Some(80.0),
            score_change_display: "+4.0 (improvement)".into(),
            trend_text: "The External Context documentation has improved since the last audit.".into(),
            git_revision: "bbb222".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            document_quality_score: 90.0,
            document_quality_rating: rating_word(90.0).to_string(),
            content_completeness_score: 80.0,
            content_completeness_rating: rating_word(80.0).to_string(),
            documentation_integrity_score: 85.0,
            documentation_integrity_rating: rating_word(85.0).to_string(),
            collection_quality_score: 75.0,
            collection_quality_rating: rating_word(75.0).to_string(),
            doc_scores: vec![TeraDocScore { name: "sqlite.md".into(), score: 90.0, rating: rating_word(90.0).to_string() }],
            validation_scores: vec![TeraValidationScore { id: "EC5".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "purpose".into(), required: true, doc_count: 1, total_docs: 1, status: "Complete".into() },
            ],
            audit_standards: build_external_context_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![TeraFindingItem {
                check_id: "EC5".into(),
                message: "No Constraints section found".into(),
                location: Some("docs/raw/external-context/sqlite.md".into()),
                evidence_excerpt: None,
                evidence_source: None,
            }],
            observations: vec![],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_external_context_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("84.0"));
        assert!(rendered.contains("— **Good**")); // rating_word(84.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("No Constraints section found"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Major"));
    }

    #[test]
    fn real_engineering_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/engineering-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = EngineeringTeraContext {
            session_id: "sess-7".into(),
            score: 78.0,
            rating: rating_word(78.0).to_string(),
            rating_description: rating_description(78.0).to_string(),
            previous_score: Some(70.0),
            score_change_display: "+8.0 (improvement)".into(),
            trend_text: "The Engineering documentation has improved since the last audit.".into(),
            git_revision: "ccc333".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "NEEDS_WORK".into(),
            engineering_coverage_score: 71.4,
            engineering_coverage_rating: rating_word(71.4).to_string(),
            documentation_quality_score: 100.0,
            documentation_quality_rating: rating_word(100.0).to_string(),
            traceability_consistency_score: 50.0,
            traceability_consistency_rating: rating_word(50.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "E2".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "guiding_principles".into(), required: true, doc_count: 1, total_docs: 1, status: "Complete".into() },
            ],
            audit_standards: build_engineering_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![TeraFindingItem {
                check_id: "E2".into(),
                message: "Repository Structure section missing".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            minor_findings: vec![],
            observations: vec![],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_engineering_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("78.0"));
        assert!(rendered.contains("— **Acceptable**")); // rating_word(78.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("Repository Structure section missing"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Minor"));
    }

    #[test]
    fn real_feature_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/feature-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = FeatureTeraContext {
            session_id: "sess-8".into(),
            score: 91.0,
            rating: rating_word(91.0).to_string(),
            rating_description: rating_description(91.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "ddd444".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            feature_definition_score: 100.0,
            feature_definition_rating: rating_word(100.0).to_string(),
            product_definition_score: 87.5,
            product_definition_rating: rating_word(87.5).to_string(),
            documentation_quality_score: 100.0,
            documentation_quality_rating: rating_word(100.0).to_string(),
            product_readiness_score: 50.0,
            product_readiness_rating: rating_word(50.0).to_string(),
            doc_scores: vec![TeraDocScore { name: "authentication.md".into(), score: 91.0, rating: rating_word(91.0).to_string() }],
            validation_scores: vec![TeraValidationScore { id: "F14".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "acceptance_criteria".into(), required: true, doc_count: 19, total_docs: 19, status: "Complete".into() },
            ],
            audit_standards: build_feature_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![],
            observations: vec![TeraFindingItem {
                check_id: "F14".into(),
                message: "No Future Extensions or Non-Goals section found".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            recommendations: vec![],
            total_checks: 14,
        };

        let rendered = render_feature_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("91.0"));
        assert!(rendered.contains("— **Very Good**")); // rating_word(91.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("No Future Extensions or Non-Goals section found"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Major"));
    }

    #[test]
    fn real_feature_technical_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/feature-technical-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = FeatureTechnicalTeraContext {
            session_id: "sess-9".into(),
            score: 82.0,
            rating: rating_word(82.0).to_string(),
            rating_description: rating_description(82.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "eee555".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "NEEDS_WORK".into(),
            feature_mapping_score: 80.0,
            feature_mapping_rating: rating_word(80.0).to_string(),
            technical_realization_score: 87.5,
            technical_realization_rating: rating_word(87.5).to_string(),
            documentation_quality_score: 75.0,
            documentation_quality_rating: rating_word(75.0).to_string(),
            implementation_readiness_score: 100.0,
            implementation_readiness_rating: rating_word(100.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![TeraValidationScore { id: "FT2".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            section_compliance: vec![
                TeraSectionCompliance { semantic_type: "participating_components".into(), required: true, doc_count: 20, total_docs: 20, status: "Complete".into() },
            ],
            audit_standards: build_feature_technical_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![TeraFindingItem {
                check_id: "FT2".into(),
                message: "1 Feature Specification(s) have no corresponding Feature Technical Design document".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            minor_findings: vec![],
            observations: vec![],
            recommendations: vec![],
            total_checks: 15,
        };

        let rendered = render_feature_technical_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("82.0"));
        assert!(rendered.contains("— **Good**")); // rating_word(82.0)
        assert!(rendered.contains("Structural Compliance Matrix"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("no corresponding Feature Technical Design document"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Minor"));
    }

    #[test]
    fn real_feature_design_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/feature-design-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = FeatureDesignTeraContext {
            session_id: "sess-10".into(),
            score: 100.0,
            rating: rating_word(100.0).to_string(),
            rating_description: rating_description(100.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "fff666".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            feature_mapping_score: 100.0,
            feature_mapping_rating: rating_word(100.0).to_string(),
            user_experience_score: 100.0,
            user_experience_rating: rating_word(100.0).to_string(),
            documentation_quality_score: 100.0,
            documentation_quality_rating: rating_word(100.0).to_string(),
            design_readiness_score: 100.0,
            design_readiness_rating: rating_word(100.0).to_string(),
            doc_scores: vec![],
            validation_scores: vec![],
            section_compliance: vec![],
            audit_standards: build_feature_design_audit_standards_summary(),
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![],
            observations: vec![TeraFindingItem {
                check_id: "FD4".into(),
                message: "No reference to External Context found".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            recommendations: vec![],
            total_checks: 15,
        };

        let rendered = render_feature_design_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("— **Excellent**")); // rating_word(100.0)
        assert!(rendered.contains("feature-design-validation.md"));
        assert!(rendered.contains("Audit Standard Rubrics"));
        assert!(rendered.contains("No reference to External Context found"));
        assert!(!rendered.contains("### Critical"));
        assert!(!rendered.contains("### Major"));
    }

    #[test]
    fn real_deterministic_runtime_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/deterministic-runtime-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = DeterministicRuntimeTeraContext {
            session_id: "sess-11".into(),
            score: 88.0,
            rating: rating_word(88.0).to_string(),
            rating_description: rating_description(88.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "aaa777".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "NEEDS_WORK".into(),
            runtime_model_score: 100.0,
            runtime_model_rating: rating_word(100.0).to_string(),
            engineering_principles_score: 75.0,
            engineering_principles_rating: rating_word(75.0).to_string(),
            runtime_integrity_score: 83.3,
            runtime_integrity_rating: rating_word(83.3).to_string(),
            validation_scores: vec![TeraValidationScore { id: "S7".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![TeraFindingItem {
                check_id: "S7".into(),
                message: "No cache strategy documented".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            observations: vec![],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_deterministic_runtime_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("88.0"));
        assert!(rendered.contains("— **Good**")); // rating_word(88.0)
        assert!(rendered.contains("No cache strategy documented"));
        assert!(!rendered.contains("## 4. Structural Compliance Matrix"));
        assert!(!rendered.contains("### Critical"));
    }

    #[test]
    fn real_external_context_ownership_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/external-context-ownership-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = ExternalContextOwnershipTeraContext {
            session_id: "sess-12".into(),
            score: 90.0,
            rating: rating_word(90.0).to_string(),
            rating_description: rating_description(90.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "bbb888".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            dependency_coverage_score: 75.0,
            dependency_coverage_rating: rating_word(75.0).to_string(),
            documentation_integration_score: 100.0,
            documentation_integration_rating: rating_word(100.0).to_string(),
            consistency_score: 100.0,
            consistency_rating: rating_word(100.0).to_string(),
            validation_scores: vec![TeraValidationScore { id: "EC1".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            critical_findings: vec![],
            major_findings: vec![],
            minor_findings: vec![],
            observations: vec![TeraFindingItem {
                check_id: "EC1".into(),
                message: "Exhaustive external-dependency discovery requires manifest parsing".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            recommendations: vec![],
            total_checks: 12,
        };

        let rendered = render_external_context_ownership_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("90.0"));
        assert!(rendered.contains("— **Very Good**")); // rating_word(90.0)
        assert!(rendered.contains("requires manifest parsing"));
        assert!(!rendered.contains("## 4. Structural Compliance Matrix"));
        assert!(!rendered.contains("### Critical"));
    }

    #[test]
    fn real_implementation_template_renders_without_error() {
        let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/raw/report-templates/implementation-report.md");
        let template = fs::read_to_string(&template_path)
            .unwrap_or_else(|e| panic!("failed to read {:?}: {}", template_path, e));

        let ctx = ImplementationTeraContext {
            session_id: "sess-13".into(),
            score: 92.0,
            rating: rating_word(92.0).to_string(),
            rating_description: rating_description(92.0).to_string(),
            previous_score: None,
            score_change_display: "N/A (baseline)".into(),
            trend_text: String::new(),
            git_revision: "ccc999".into(),
            created_at: "2026-07-06".into(),
            engineering_readiness: "READY".into(),
            architectural_conformance_score: 100.0,
            architectural_conformance_rating: rating_word(100.0).to_string(),
            feature_conformance_score: 100.0,
            feature_conformance_rating: rating_word(100.0).to_string(),
            engineering_conformance_score: 66.7,
            engineering_conformance_rating: rating_word(66.7).to_string(),
            documentation_integrity_score: 100.0,
            documentation_integrity_rating: rating_word(100.0).to_string(),
            implementation_quality_score: 100.0,
            implementation_quality_rating: rating_word(100.0).to_string(),
            validation_scores: vec![TeraValidationScore { id: "I9".into(), score: 0.0, rating: rating_word(0.0).to_string() }],
            critical_findings: vec![],
            major_findings: vec![TeraFindingItem {
                check_id: "I9".into(),
                message: "1 crate(s) exist in the implementation but are never named in Engineering documentation".into(),
                location: None,
                evidence_excerpt: None,
                evidence_source: None,
            }],
            minor_findings: vec![],
            observations: vec![],
            recommendations: vec![],
            total_checks: 15,
        };

        let rendered = render_implementation_template(&ctx, &template)
            .unwrap_or_else(|e| panic!("template failed to render: {}", e));

        assert!(rendered.contains("92.0"));
        assert!(rendered.contains("— **Very Good**")); // rating_word(92.0)
        assert!(rendered.contains("never named in Engineering documentation"));
        assert!(!rendered.contains("## 4. Structural Compliance Matrix"));
        assert!(!rendered.contains("### Critical"));
    }

    #[test]
    fn build_audit_standards_summary_covers_all_required_sections() {
        // Only the 5 *required* section types are asserted here — "rationale"
        // (optional, per standards/architecture.md) has no corresponding
        // docs/raw/audit-standards/architecture/rationale.md file today. That's
        // a pre-existing gap in the docs, not something to paper over with a
        // fabricated entry; it just means "rationale" won't get a rubric
        // summary in the report until that file is written.
        let summary = build_audit_standards_summary();
        for (semantic_type, required) in ARCHITECTURE_SECTION_TYPES {
            if !required {
                continue;
            }
            assert!(
                summary.iter().any(|s| s.semantic_type == *semantic_type),
                "missing audit-standard summary for required section '{}'",
                semantic_type
            );
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
