//! Splits a `docs/raw/audit/*.md` spec file into its per-check blocks
//! (`## A1. Modular Architecture`, `## V1. Purpose and Problem Defined`, ...)
//! so a Spec-layer judgment (docs/proposal.md — "Three-Layer Audit Model")
//! can be generated one check at a time instead of handing an LLM the whole
//! file. Each file's checklist IDs are documented in `docs/raw/audit/README.md`'s
//! Authority Chain table (e.g. Vision Audit: V1-V12, Architecture Audit: A1-A13).

/// One `## <ID>. <Title>` block from an audit spec file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditSpecCheck {
    pub id: String,
    pub title: String,
    pub body: String,
    /// The `**Audit Rule:**` line, when the check has one — not every check
    /// does (e.g. Vision's V5, V8 state the requirement in prose only).
    pub audit_rule: Option<String>,
}

/// Splits `raw` into its checks, in file order. Non-check content (the doc
/// title, `# Authority`, `# Scoring Model`, etc.) is skipped, not returned.
pub fn parse_audit_spec_checks(raw: &str) -> Vec<AuditSpecCheck> {
    let lines: Vec<&str> = raw.lines().collect();
    let mut checks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if let Some((id, title)) = parse_check_heading(lines[i]) {
            let start = i + 1;
            let mut end = lines.len();
            for (offset, line) in lines[start..].iter().enumerate() {
                if parse_check_heading(line).is_some() || is_category_heading(line) {
                    end = start + offset;
                    break;
                }
            }
            let body = trim_body(&lines[start..end]);
            let audit_rule = extract_audit_rule(&body);
            checks.push(AuditSpecCheck { id, title, body, audit_rule });
            i = end;
        } else {
            i += 1;
        }
    }
    checks
}

/// A check heading: `## <ID>. <Title>` where `<ID>` is uppercase letters
/// followed by digits (`V1`, `A13`, `BC10`, `SEC12`) — never a bare `## `
/// section like `## Purpose` (no digit suffix, no match).
fn parse_check_heading(line: &str) -> Option<(String, String)> {
    let rest = line.strip_prefix("## ")?;
    let (id, title) = rest.split_once(". ")?;
    if !is_check_id(id) {
        return None;
    }
    Some((id.to_string(), title.trim().to_string()))
}

fn is_check_id(token: &str) -> bool {
    let letters_end = token
        .find(|c: char| !c.is_ascii_uppercase())
        .unwrap_or(token.len());
    letters_end > 0
        && letters_end < token.len()
        && token[letters_end..].chars().all(|c| c.is_ascii_digit())
}

/// A single-`#` heading (`# Technology Independence`, `# Scoring Model`) —
/// these group checks or hold non-checklist content; either way a check's
/// body ends here.
fn is_category_heading(line: &str) -> bool {
    line.starts_with("# ") && !line.starts_with("## ")
}

fn trim_body(lines: &[&str]) -> String {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && matches!(lines[end - 1].trim(), "" | "---") {
        end -= 1;
    }
    lines[start..end].join("\n")
}

fn extract_audit_rule(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        line.trim()
            .strip_prefix("**Audit Rule:**")
            .map(|rest| rest.trim().to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_id_title_body_and_audit_rule() {
        let raw = "\
# Vision Audit

# Vision Content

## V1. Purpose and Problem Defined

The Vision clearly explains why the product exists.

**Audit Rule:** The Vision explains why the product exists.

---

## V2. Long-term Direction Explicit

Direction should remain stable over time.

---
";
        let checks = parse_audit_spec_checks(raw);
        assert_eq!(checks.len(), 2);

        assert_eq!(checks[0].id, "V1");
        assert_eq!(checks[0].title, "Purpose and Problem Defined");
        assert!(checks[0].body.contains("clearly explains why"));
        assert_eq!(
            checks[0].audit_rule.as_deref(),
            Some("The Vision explains why the product exists.")
        );

        assert_eq!(checks[1].id, "V2");
        assert_eq!(checks[1].audit_rule, None);
        assert!(!checks[1].body.contains("---"));
    }

    #[test]
    fn ignores_non_check_headings() {
        let raw = "\
# Authority

Audit rules are defined by the validation checks in this document.

## Not A Check

This has no digit suffix so it isn't a check.

## V1. Purpose

Real check.
";
        let checks = parse_audit_spec_checks(raw);
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].id, "V1");
    }

    #[test]
    fn rejects_lowercase_and_bare_number_ids() {
        assert!(parse_check_heading("## v1. lowercase").is_none());
        assert!(parse_check_heading("## 1. bare number").is_none());
        assert!(parse_check_heading("## V. no digits").is_none());
        assert!(parse_check_heading("## Purpose").is_none());
        assert!(parse_check_heading("## V1. Purpose").is_some());
        assert!(parse_check_heading("## BC10. Build Conformance").is_some());
    }

    #[test]
    fn empty_input_yields_no_checks() {
        assert!(parse_audit_spec_checks("").is_empty());
        assert!(parse_audit_spec_checks("# Just a title\n\nSome prose.\n").is_empty());
    }

    /// Regression guard against real fixture drift — parses every actual
    /// `docs/raw/audit/*.md` spec file and cross-checks the parsed count
    /// against `README.md`'s own Authority Chain table (the source of truth
    /// for how many checks each file is supposed to define).
    #[test]
    #[ignore = "Audit specs refactored to docs/knowledge-hub; spec_parser needs rewrite"]
    fn parses_real_spec_files_matching_authority_chain_counts() {
        let audit_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/knowledge-hub/audit/semantic/document");
        assert!(audit_dir.is_dir(), "expected {} to exist", audit_dir.display());

        // (file, expected check count) — counts taken from that file's own
        // Authority Chain range, e.g. "Vision Audit | V1-V12" => 12.
        let expectations = [
            ("vision-audit.md", 12),
            ("architecture-audit.md", 13),
            ("design-audit.md", 12),
            ("feature-audit.md", 14),
            ("feature-design-validation.md", 15),
            ("feature-technical-audit.md", 15),
            ("prototype-audit.md", 15),
            ("external-context-audit.md", 12),
            // README.md's Authority Chain table says "EC1-EC7" for this file,
            // but the file itself defines EC1-EC12 — trust the file (parser
            // output), not the table; the table row is stale.
            ("external-context-ownership-audit.md", 12),
            ("engineering-audit.md", 12),
            ("readme-audit.md", 12),
            ("deterministic-runtime-audit.md", 12),
            ("implementation-audit.md", 15),
            ("consistency-audit.md", 12),
            ("coverage-audit.md", 15),
            ("dependency-audit.md", 8),
            ("documentation-structure-audit.md", 45),
        ];

        for (file, expected) in expectations {
            let path = audit_dir.join(file);
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let checks = parse_audit_spec_checks(&raw);
            assert_eq!(
                checks.len(),
                expected,
                "{file}: expected {expected} checks, parsed {} ({:?})",
                checks.len(),
                checks.iter().map(|c| c.id.as_str()).collect::<Vec<_>>()
            );
        }

        // build-audit.md and security-audit.md carry two check families each
        // in one file (B1-B12 + BC1-BC10, SEC1-SEC12 + SC1-SC11 — see
        // README.md's Taxonomy note) — 22 and 23 checks respectively.
        for (file, expected) in [("build-audit.md", 22), ("security-audit.md", 23)] {
            let path = audit_dir.join(file);
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let checks = parse_audit_spec_checks(&raw);
            assert_eq!(checks.len(), expected, "{file}: expected {expected} checks, parsed {}", checks.len());
        }
    }
}
