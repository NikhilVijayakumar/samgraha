use anyhow::Result;
use schemas::ProjectCase;
use schemas::standard::{PlanScenario, PlanSetting, StandardRelationship};
use standards::StandardRegistry;
use std::collections::HashMap;
use std::path::Path;

/// The data `StandardWorkflowPlanner` needs — all of it already loaded by
/// `StandardRegistry` from `standards.db` (`plan_scenarios`/`plan_settings`,
/// loader Pass 8), not re-read from any file here.
pub struct StandardWorkflowContext {
    pub plan_settings: Option<PlanSetting>,
    pub plan_scenarios: Vec<PlanScenario>,
    /// Tier -> domain keys, from `StandardDefinition.tier`/`.domain`
    /// (`domains.tier`, loader Pass 1) — lets a generated Audit/Fix phase
    /// name the actual domains it covers instead of leaving it empty.
    pub domains_by_tier: HashMap<i32, Vec<String>>,
    /// Every domain_relationships edge across the standard (loader Pass 1) —
    /// `StandardWorkflowPlanner` only actually uses the `enforce_order` ones,
    /// but keeping the full set here rather than pre-filtering means a
    /// future consumer doesn't need this struct's shape to change to reach
    /// the rest (e.g. `tier_gating_strict` for something other than phase
    /// ordering).
    pub relationships: Vec<StandardRelationship>,
}

/// Describes what the project currently has — drives phase generation.
pub struct ProjectContext {
    /// Which goal the user selected.
    pub case: ProjectCase,
    /// True when docs/raw/ has at least some markdown files.
    pub has_docs: bool,
    /// Known domains that have been compiled.
    pub compiled_domains: Vec<String>,
    /// Audit pipelines that have been run, mapped to their latest score.
    pub existing_scores: HashMap<String, f64>,
    /// Populated only for `ProjectCase::Standard` (when a registry was
    /// supplied to `detect()`) — `plan_scenarios` is keyed by
    /// (repo_state, doc_state, tier, step), so picking the right subset
    /// needs `has_docs` (doc_state) and repo_state (existing vs new,
    /// detected below) alongside it.
    pub standard: Option<StandardWorkflowContext>,
    /// "existing" if this repo already has samgraha state (`.samgraha/manifest.json`),
    /// "new" otherwise — mirrors `plan_scenarios.repo_state`'s two values.
    pub repo_state: String,
}

impl ProjectContext {
    pub fn detect(root: &Path, case: &ProjectCase) -> Result<Self> {
        Self::detect_with_registry(root, case, None)
    }

    pub fn detect_with_registry(
        root: &Path,
        case: &ProjectCase,
        standard_registry: Option<&StandardRegistry>,
    ) -> Result<Self> {
        let raw_dir = root.join("docs").join("raw");
        let has_docs = raw_dir.is_dir() && Self::has_md_files(&raw_dir);
        let repo_state = if root.join(".samgraha").join("manifest.json").is_file() {
            "existing"
        } else {
            "new"
        }
        .to_string();

        let standard = match (case, standard_registry) {
            (ProjectCase::Standard, Some(registry)) => {
                let mut domains_by_tier: HashMap<i32, Vec<String>> = HashMap::new();
                let mut relationships: Vec<StandardRelationship> = Vec::new();
                for std in registry.all() {
                    if let Some(tier) = std.tier {
                        domains_by_tier.entry(tier).or_default().push(std.domain.clone());
                    }
                    relationships.extend(std.relationships.iter().cloned());
                }
                Some(StandardWorkflowContext {
                    plan_settings: registry.plan_settings().first().cloned(),
                    plan_scenarios: registry.plan_scenarios().to_vec(),
                    domains_by_tier,
                    relationships,
                })
            }
            _ => None,
        };

        Ok(Self {
            case: case.clone(),
            has_docs,
            compiled_domains: Vec::new(),
            existing_scores: HashMap::new(),
            standard,
            repo_state,
        })
    }

    fn has_md_files(dir: &Path) -> bool {
        std::fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
            })
            .unwrap_or(false)
    }
}
