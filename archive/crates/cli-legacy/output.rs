use schemas::manifest::CachedRepoMetadata;
use serde::Serialize;
use schemas::audit::{AuditReport, PipelineReport};
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

pub fn render_audit_report(report: &AuditReport) -> String {
    use std::collections::BTreeMap;

    let domain = report.domain.as_deref().unwrap_or("all");
    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Error))
        .collect();
    let warnings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Warning))
        .collect();
    let suggestions: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Suggestion))
        .collect();

    let mut out = String::new();
    out.push_str("# Saṃgraha Audit Report\n\n");
    out.push_str(&format!("**Date:** {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    out.push_str(&format!("**Domain:** {}\n", domain));
    out.push_str(&format!("**Provider:** {}\n", report.provider));
    out.push_str(&format!("**Score:** {:.1}%\n", report.score.overall));
    out.push_str(&format!("**Readiness:** {}\n\n", report.readiness));

    out.push_str("## Summary\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!(
        "| Documents Checked | {} |\n",
        report.score.documents_checked
    ));
    out.push_str(&format!(
        "| Documents Passed | {} |\n",
        report.score.documents_passed
    ));
    out.push_str(&format!("| Errors | {} |\n", errors.len()));
    out.push_str(&format!("| Warnings | {} |\n", warnings.len()));
    out.push_str(&format!("| Suggestions | {} |\n", suggestions.len()));

    if !report.score.categories.is_empty() {
        out.push_str("\n## Category Scores\n\n");
        out.push_str("| Standard | Score |\n");
        out.push_str("|---|---|\n");
        let sorted: BTreeMap<_, _> = report.score.categories.iter().collect();
        for (std, score) in &sorted {
            out.push_str(&format!("| {} | {:.1}% |\n", std, score));
        }
    }

    if !errors.is_empty() {
        out.push_str("\n## Errors\n\n");
        out.push_str("| Check | Document | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &errors {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                f.check_id, loc, f.message
            ));
        }
    }

    if !warnings.is_empty() {
        out.push_str("\n## Warnings\n\n");
        out.push_str("| Check | Document | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &warnings {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                f.check_id, loc, f.message
            ));
        }
    }

    if !suggestions.is_empty() {
        out.push_str("\n## Suggestions\n\n");
        out.push_str("| Check | Document | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &suggestions {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!(
                "| {} | {} | {} |\n",
                f.check_id, loc, f.message
            ));
        }
    }

    out.push_str(&format!(
        "\n---\n*Report ID: {}*\n",
        chrono::Local::now().format("audit-%Y%m%d-%H%M%S")
    ));
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
    if !info.builtin_stores.is_empty() {
        out.push_str(&format!("Built-in:    {}\n", info.builtin_stores.join(", ")));
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

pub fn render_registry_list(entries: &[CachedRepoMetadata], format: &OutputFormat) -> String {
    if matches!(format, OutputFormat::Json) {
        return serde_json::to_string_pretty(entries).unwrap_or_default();
    }
    let mut out = String::new();
    out.push_str(&format!("Registered repositories ({})\n", entries.len()));
    out.push_str(&format!("{:-<80}\n", ""));
    for entry in entries {
        out.push_str(&format!(
            "  {} ({}) — rev {} | exports: {} | audit: {}\n",
            entry.repository.id,
            entry.repository.uuid.to_string().split('-').next().unwrap_or("?"),
            entry.revision,
            entry.exports.join(", "),
            entry.audit,
        ));
    }
    out
}

pub fn render_pipeline_report(report: &PipelineReport) -> String {
    use std::collections::BTreeMap;

    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Error))
        .collect();
    let warnings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Warning))
        .collect();
    let suggestions: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.severity, schemas::audit::Severity::Suggestion))
        .collect();

    let mut out = String::new();
    out.push_str("# Saṃgraha Pipeline Report\n\n");
    out.push_str(&format!("**Date:** {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    out.push_str(&format!("**Pipeline:** {}\n", report.pipeline.as_str()));
    out.push_str(&format!("**Score:** {:.1}%\n\n", report.score));

    if !report.categories.is_empty() {
        out.push_str("## Category Scores\n\n");
        out.push_str("| Category | Score |\n");
        out.push_str("|---|---|\n");
        let sorted: BTreeMap<_, _> = report.categories.iter().collect();
        for (cat, score) in &sorted {
            out.push_str(&format!("| {} | {:.1}% |\n", cat, score));
        }
    }

    if !errors.is_empty() {
        out.push_str("\n## Errors\n\n");
        out.push_str("| Check | Location | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &errors {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!("| {} | {} | {} |\n", f.check_id, loc, f.message));
        }
    }

    if !warnings.is_empty() {
        out.push_str("\n## Warnings\n\n");
        out.push_str("| Check | Location | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &warnings {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!("| {} | {} | {} |\n", f.check_id, loc, f.message));
        }
    }

    if !suggestions.is_empty() {
        out.push_str("\n## Suggestions\n\n");
        out.push_str("| Check | Location | Message |\n");
        out.push_str("|---|---|---|\n");
        for f in &suggestions {
            let loc = f.location.as_deref().unwrap_or("-");
            out.push_str(&format!("| {} | {} | {} |\n", f.check_id, loc, f.message));
        }
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
