use serde::Serialize;
use schemas::audit::AuditReport;
use schemas::compilation::CompilationResult;
use schemas::search::{SearchResponse, SectionQueryResponse};
use services::runtime::runtime::RuntimeInfo;
use services::WorkspaceBuildResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn format_output<T: Serialize>(data: &T, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(data).unwrap_or_default(),
        OutputFormat::Text => serde_json::to_string_pretty(data).unwrap_or_default(),
    }
}

pub fn render_compile(result: &CompilationResult, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(result).unwrap_or_default();
    }
    let status = if result.success { "ok" } else { "FAILED" };
    let mut out = format!(
        "Compilation {}: {} found, {} compiled, {} failed, {} skipped ({}ms)\n",
        status,
        result.documents_found,
        result.documents_processed,
        result.documents_failed,
        result.documents_skipped,
        result.duration_ms,
    );
    if let Some(path) = &result.registry_path {
        out.push_str(&format!("Registry: {}\n", path));
    }
    for e in &result.errors {
        let loc = e.path.as_deref().unwrap_or("?");
        out.push_str(&format!("  error [{}] {} — {}\n", e.error_type, loc, e.message));
    }
    for w in &result.warnings {
        out.push_str(&format!("  warn  {}\n", w));
    }
    out
}

pub fn render_search(resp: &SearchResponse, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(resp).unwrap_or_default();
    }
    if resp.results.is_empty() {
        return format!("No results for \"{}\"\n", resp.query);
    }
    let headers = ["Title", "Domain", "Score", "Snippet"];
    let rows: Vec<Vec<String>> = resp
        .results
        .iter()
        .map(|r| {
            let snippet = r
                .snippet
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();
            vec![
                truncate(&r.title, 40),
                r.domain.clone(),
                format!("{:.2}", r.score),
                snippet,
            ]
        })
        .collect();
    let mut out = format_table(&headers, &rows);
    out.push_str(&format!(
        "\n{} result(s) in {}ms\n",
        resp.total_count, resp.duration_ms
    ));
    out
}

pub fn render_audit(report: &AuditReport, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(report).unwrap_or_default();
    }
    let domain = report.domain.as_deref().unwrap_or("all");
    let errors = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Error))
        .count();
    let warnings = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Warning))
        .count();
    let suggestions = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Suggestion))
        .count();

    let mut out = format!(
        "Audit: {} | Score: {:.1}% | Readiness: {}\n",
        domain, report.score.overall, report.readiness
    );
    out.push_str(&format!(
        "Checked: {} docs | {} errors, {} warnings, {} suggestions\n",
        report.score.documents_checked, errors, warnings, suggestions
    ));

    if !report.findings.is_empty() {
        out.push('\n');
        for f in &report.findings {
            let loc = f.location.as_deref().unwrap_or("");
            let sev = match f.severity {
                schemas::audit::Severity::Error => "ERROR",
                schemas::audit::Severity::Warning => "WARN ",
                schemas::audit::Severity::Suggestion => "HINT ",
            };
            out.push_str(&format!(
                "  {} [{}] {} {}\n",
                sev, f.check_id, f.message, loc
            ));
        }
    }
    out
}

pub fn render_info(info: &RuntimeInfo, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(info).unwrap_or_default();
    }
    let mut out = format!("Repository:  {}\n", info.repository);
    out.push_str(&format!("Registry:    {}\n", info.registry_path));
    out.push_str(&format!("Documents:   {}\n", info.document_count));
    if !info.standards.is_empty() {
        out.push_str(&format!("Standards:   {}\n", info.standards.join(", ")));
    }
    out
}

pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }
    let mut out = String::new();
    for (i, header) in headers.iter().enumerate() {
        if i > 0 {
            out.push_str("  ");
        }
        out.push_str(&format!("{:width$}", header, width = col_widths[i]));
    }
    out.push('\n');
    for &width in &col_widths {
        out.push_str(&"-".repeat(width));
        out.push_str("  ");
    }
    out.push('\n');
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i > 0 {
                out.push_str("  ");
            }
            out.push_str(&format!("{:width$}", cell, width = col_widths[i]));
        }
        out.push('\n');
    }
    out
}

pub fn render_workspace_compile(result: &WorkspaceBuildResult, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(&serde_json::json!({
            "workspace": result.workspace_name,
            "total_documents": result.total_documents,
            "total_errors": result.total_errors,
            "repositories": result.repository_results.iter().map(|(name, r)| {
                serde_json::json!({
                    "name": name,
                    "success": r.success,
                    "documents_processed": r.documents_processed,
                    "documents_failed": r.documents_failed,
                })
            }).collect::<Vec<_>>(),
        })).unwrap_or_default();
    }
    let status = if result.total_errors == 0 { "ok" } else { "FAILED" };
    let mut out = format!(
        "Workspace \"{}\": {} — {} docs, {} errors\n",
        result.workspace_name, status, result.total_documents, result.total_errors
    );
    for (name, r) in &result.repository_results {
        let r_status = if r.success { "ok" } else { "FAIL" };
        out.push_str(&format!(
            "  {:4} {}  ({} docs, {} failed)\n",
            r_status, name, r.documents_processed, r.documents_failed
        ));
    }
    out
}

pub fn render_sections(resp: &SectionQueryResponse, format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(resp).unwrap_or_default();
    }
    if resp.sections.is_empty() {
        return format!("No sections found for type \"{}\"\n", resp.semantic_type);
    }
    let mut out = format!(
        "{} section(s) of type \"{}\" ({}ms)\n\n",
        resp.total_count, resp.semantic_type, resp.duration_ms
    );
    for s in &resp.sections {
        out.push_str(&format!("--- {} [{}]\n", s.document_title, s.standard));
        out.push_str(&format!("    heading: {}\n", s.canonical_name));
        if !s.content.is_empty() {
            let preview: String = s.content.lines().take(3).collect::<Vec<_>>().join(" ").chars().take(120).collect();
            out.push_str(&format!("    {}\n", preview));
        }
        out.push('\n');
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
