use crate::fix::types::{
    ParsedAuditSpec, ParsedAuditStandard, ParsedDocStandard, PlanningContext,
};
use anyhow::{Context, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

struct CachedFiles {
    audit_spec_raw: String,
    audit_standard_raw: String,
    doc_standard_raw: String,
}

pub struct PlanningContextBuilder {
    repo_root: PathBuf,
    cache: RefCell<HashMap<String, CachedFiles>>,
}

impl PlanningContextBuilder {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn build(
        &self,
        domain: &str,
        target_path: &Path,
    ) -> Result<PlanningContext> {
        let target_content = if target_path.exists() {
            fs::read_to_string(target_path)
                .context(format!("Failed to read target: {}", target_path.display()))?
        } else {
            String::new()
        };

        let cached = self.get_or_load(domain)?;

        Ok(PlanningContext {
            target_path: target_path.to_path_buf(),
            target_content,
            audit_spec: ParsedAuditSpec {
                raw: cached.audit_spec_raw,
            },
            audit_standard: ParsedAuditStandard {
                raw: cached.audit_standard_raw,
            },
            doc_standard: ParsedDocStandard {
                raw: cached.doc_standard_raw,
            },
            feedback: Vec::new(),
            domain: domain.to_string(),
        })
    }

    pub fn invalidate(&self, target_path: &Path) {
        let target_str = target_path.to_string_lossy().to_string();
        self.cache.borrow_mut().retain(|_, v| {
            let audit_contains = v.audit_spec_raw.contains(&target_str);
            let standard_contains = v.doc_standard_raw.contains(&target_str);
            !audit_contains && !standard_contains
        });
        // For simplicity, clear all cache when any target is written.
        // Fine for v1 — single-session, single-domain usage.
        self.cache.borrow_mut().clear();
    }

    fn get_or_load(&self, domain: &str) -> Result<CachedFiles> {
        let mut cache = self.cache.borrow_mut();
        if let Some(cached) = cache.get(domain) {
            return Ok(CachedFiles {
                audit_spec_raw: cached.audit_spec_raw.clone(),
                audit_standard_raw: cached.audit_standard_raw.clone(),
                doc_standard_raw: cached.doc_standard_raw.clone(),
            });
        }

        let audit_spec_path = self.audit_spec_path(domain);
        let audit_standard_dir = self.audit_standard_path(domain);
        let doc_standard_path = self.doc_standard_path(domain);

        let audit_spec_raw = read_file_optional(&audit_spec_path)
            .context(format!("Missing audit spec for domain '{}'", domain))?;
        if audit_spec_raw.trim().is_empty() {
            anyhow::bail!(
                "Audit spec for domain '{}' is missing or empty at {} — the fix pipeline \
                 cannot plan a fix without knowing what the check requires",
                domain,
                audit_spec_path.display()
            );
        }

        let doc_standard_raw = read_file_optional(&doc_standard_path)
            .context(format!("Missing doc standard for domain '{}'", domain))?;
        if doc_standard_raw.trim().is_empty() {
            tracing::warn!(
                "No doc standard on disk for domain '{}' ({}) — planning context for this \
                 domain is missing one of its three layers; fix quality may be degraded",
                domain,
                doc_standard_path.display()
            );
        }

        let audit_standard_raw = read_audit_standard_dir(&audit_standard_dir)
            .context(format!("Missing or empty audit-standards dir for domain '{}'", domain))?;
        if audit_standard_raw.trim().is_empty() {
            tracing::warn!(
                "No audit-standard rubric on disk for domain '{}' ({}) — planning context for \
                 this domain is missing one of its three layers; fix quality may be degraded",
                domain,
                audit_standard_dir.display()
            );
        }

        let files = CachedFiles {
            audit_spec_raw,
            audit_standard_raw,
            doc_standard_raw,
        };
        cache.insert(domain.to_string(), CachedFiles {
            audit_spec_raw: files.audit_spec_raw.clone(),
            audit_standard_raw: files.audit_standard_raw.clone(),
            doc_standard_raw: files.doc_standard_raw.clone(),
        });
        Ok(files)
    }

    /// `docs/raw/audit/*.md` is written per **pipeline**, not per domain —
    /// its checklist (A1-A13, V1-V12, ...) judges a whole collection, which
    /// is pipeline-scoped even for the 11 names that are also a domain (see
    /// docs/proposal.md §3). The caller still passes the same identifier it
    /// uses for `audit_standard_path`/`doc_standard_path` (those two really
    /// are domain-scoped) — `pipeline_name` here is that same string reused
    /// under its pipeline meaning, not a different value.
    /// Reads `docs/raw/audit/{pipeline_name}-audit.md` verbatim — the
    /// Spec-layer source for `build_pipeline_semantic_review` (see
    /// docs/proposal.md §6.1). Reuses this builder's path resolution
    /// (including the feature-design exception) instead of duplicating it.
    pub fn read_audit_spec(&self, pipeline_name: &str) -> Result<String> {
        read_file_optional(&self.audit_spec_path(pipeline_name))
    }

    fn audit_spec_path(&self, pipeline_name: &str) -> PathBuf {
        let file_name = match pipeline_name {
            "feature-design" => "feature-design-validation.md".to_string(),
            other => format!("{}-audit.md", other),
        };
        self.repo_root.join("docs/raw/audit").join(&file_name)
    }

    fn audit_standard_path(&self, domain: &str) -> PathBuf {
        self.repo_root.join("docs/raw/audit-standards").join(domain)
    }

    /// Standard docs carry an ordering prefix (`01-vision-standards.md`), not a bare
    /// `{domain}.md` name, so fall back to scanning the directory for a file whose
    /// stem — minus a leading `NN-` and trailing `-standards` — matches the domain.
    fn doc_standard_path(&self, domain: &str) -> PathBuf {
        let dir = self.repo_root.join("docs/raw/documentation-standards");
        let exact = dir.join(format!("{}.md", domain));
        if exact.exists() {
            return exact;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
                let stripped = stem
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('-')
                    .trim_end_matches("-standards");
                if stripped == domain {
                    return path;
                }
            }
        }
        exact
    }
}

fn read_file_optional(path: &Path) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path)
            .context(format!("Failed to read {}", path.display()))
    } else {
        Ok(String::new())
    }
}

fn read_audit_standard_dir(dir: &Path) -> Result<String> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(String::new());
    }
    let mut combined = String::new();
    let mut entries: Vec<_> = fs::read_dir(dir)
        .context(format!("Failed to read directory {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in &entries {
        let content = fs::read_to_string(entry.path())
            .context(format!("Failed to read {}", entry.path().display()))?;
        combined.push_str(&content);
        combined.push('\n');
    }
    Ok(combined)
}
