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
                raw: String::new(),
            },
            doc_standard: ParsedDocStandard {
                raw: String::new(),
            },
            feedback: Vec::new(),
            domain: domain.to_string(),
        })
    }

    pub fn invalidate(&self, _target_path: &Path) {
        // Clear all cache when any target is written.
        // Fine for v1 — single-session, single-domain usage.
        self.cache.borrow_mut().clear();
    }

    fn get_or_load(&self, domain: &str) -> Result<CachedFiles> {
        let mut cache = self.cache.borrow_mut();
        if let Some(cached) = cache.get(domain) {
            return Ok(CachedFiles {
                audit_spec_raw: cached.audit_spec_raw.clone(),
            });
        }

        let audit_spec_path = self.audit_spec_path(domain);

        // Degrade gracefully instead of hard-failing on a missing spec.
        // `docs/raw/audit/*.md` used to be samgraha's own copy of each
        // check's requirement text; per the documentation-cleanup pass,
        // that content is now the owning system's concern, not something
        // samgraha ships. A missing/empty spec means planner.rs's per-step
        // `rationale` falls back to its own generic wording (see
        // planner.rs's `unwrap_or_else` fallbacks) instead of quoting the
        // check's specific requirement — a plan still gets generated, just
        // a less specific one. Matches `read_file_optional`'s own already-
        // tolerant design (empty string, not an error, for a missing file);
        // this function used to immediately undo that tolerance by bailing
        // right after.
        let audit_spec_raw = read_file_optional(&audit_spec_path)
            .context(format!("Missing audit spec for domain '{}'", domain))?;

        let files = CachedFiles {
            audit_spec_raw,
        };
        cache.insert(domain.to_string(), CachedFiles {
            audit_spec_raw: files.audit_spec_raw.clone(),
        });
        Ok(files)
    }

    /// `docs/raw/audit/*.md` is written per **pipeline**, not per domain —
    /// its checklist (A1-A13, V1-V12, ...) judges a whole collection, which
    /// is pipeline-scoped even for the 11 names that are also a domain (see
    /// docs/proposal.md §3). The caller still passes the same identifier it
    /// uses for domain-scoped lookups — `pipeline_name` here is that same
    /// string reused under its pipeline meaning, not a different value.
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
}

fn read_file_optional(path: &Path) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path)
            .context(format!("Failed to read {}", path.display()))
    } else {
        Ok(String::new())
    }
}
