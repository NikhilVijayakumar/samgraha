use anyhow::{Context, Result};
use schemas::audit::AuditStage;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
}
