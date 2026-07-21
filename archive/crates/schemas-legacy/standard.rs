use serde::{Deserialize, Serialize};

pub type StandardId = String;
pub type StandardVersion = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardDefinition {
    pub id: StandardId,
    pub name: String,
    pub version: StandardVersion,
    pub domain: String,
    pub description: String,
    pub required_sections: Vec<SectionDefinition>,
    pub prohibited_content: Vec<String>,
    pub relationships: Vec<StandardRelationship>,
    pub audit_rules: Vec<AuditRuleDef>,
    /// `kind = 'semantic'` rules for this domain (`evidence_type:
    /// "llm_judgment"`, prompt/rubric text in `params`) — a rule here isn't
    /// scored by any Rust code (that needs an LLM); `SemanticAuditProvider`
    /// turns each into a review-task-eligible finding, picked up by the same
    /// `semantic_review` bundling `AuditFramework::execute` already does for
    /// findings tagged `provider: "semantic"`.
    #[serde(default)]
    pub semantic_rules: Vec<AuditRuleDef>,
    pub profiles: Vec<ProfileDef>,
    /// This domain's tier within its standard's `domains` table (e.g.
    /// python_hackathon's "infrastructure" is tier 1) — `None` for the two
    /// builtin non-DB standards (`help`, `standards`), which have no tiers.
    #[serde(default)]
    pub tier: Option<i32>,
    /// Per-domain content kind: `"documentation"` (full generate→store→assemble
    /// pipeline, content persisted for reuse in later audit/fix passes) or
    /// `"code"` (the calling agent generates and writes the target file directly;
    /// `store_generated_content` is never called, `assemble` never runs, nothing
    /// persists to DB). From `domains.content_kind`.
    #[serde(default = "default_content_kind")]
    pub content_kind: String,
    /// Per-standard generation granularity: `"document"` (one task, full
    /// upstream context), `"section"` (one task per section, respecting
    /// dependencies), or `"hybrid"` (section tasks first, then a whole-document
    /// coherence pass). From `standards.generation_granularity`.
    #[serde(default = "default_generation_granularity")]
    pub generation_granularity: String,
    /// Section-level dependency edges for this domain. Each entry is
    /// `(section_catalog_id, depends_on_section_id)` — used by `generate`
    /// to order section-level generation tasks and by `store_generated_content`
    /// to reject writes for sections whose dependencies haven't been stored yet.
    /// From `section_dependencies` table.
    #[serde(default)]
    pub section_dependencies: Vec<(i64, i64)>,
}

/// A semantic section definition within a Documentation Standard.
/// `canonical_name` is the heading as it appears in well-formed documents.
/// `aliases` are alternate heading spellings matched case-insensitively.
/// `semantic_type` is a stable snake_case identifier used for storage and querying.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionDefinition {
    pub canonical_name: String,
    pub semantic_type: String,
    pub aliases: Vec<String>,
    pub required: bool,
    pub description: String,
    /// Database row ID from `section_catalog.id` — bridges `section_dependencies`
    /// (which uses catalog IDs) to the semantic_type strings handlers work with.
    /// `None` for builtin standards that don't have a knowledge-hub DB.
    #[serde(default)]
    pub section_catalog_id: Option<i64>,
}

/// Retained for backward compatibility — alias for SectionDefinition.
pub type RequiredSection = SectionDefinition;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileDef {
    pub name: String,
    pub description: String,
    pub include_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardRelationship {
    pub from_domain: String,
    pub to_domain: String,
    pub relationship: String,
    /// Within the same tier, `from_domain` must complete before `to_domain`
    /// starts (`domain_relationships.enforce_order`) — the one documented
    /// exception in base_dev's own tiers (External Context before
    /// Engineering); most edges leave this false and rely only on tier
    /// ordering itself.
    pub enforce_order: bool,
    /// This relationship type's tier-gating rule
    /// (`relationship_types.tier_gating`: "strict" blocks tier advancement
    /// until `from_domain` clears, "none" is informational-only, e.g. a
    /// citation or a non-mandatory soft alignment).
    pub tier_gating_strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditRuleDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: String,
    /// Evidence-check type — drives generic dispatch in DeterministicAuditProvider.
    /// Values: "section_presence" | "keyword_absence" | "content_check" |
    /// "cross_reference" | "word_count" | "script_result" | "llm_judgment" | ...
    pub evidence_type: String,
    /// Target for section-level checks: the heading text to look for.
    /// Empty for document-scope checks.
    pub scope: String,
    /// Weight for weighted scoring (higher = more important).
    pub weight: f64,
    /// Whether failing this rule is always an error regardless of score.
    pub mandatory: bool,
    /// Evidence-extractor parameters (key-value pairs from rule_evidence_params).
    pub params: std::collections::HashMap<String, String>,
}

/// The human-readable documentation-standards spec for a domain — the prose
/// a domain's rules/templates/section-catalog are derived from, as opposed
/// to `StandardDefinition`'s structural metadata. From the `standard_docs`
/// table (`schema/knowledge-hub/17-standard_docs.sql`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardDoc {
    pub domain: String,
    pub title: String,
    pub content: String,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardDeclaration {
    pub standard_id: StandardId,
    pub version: StandardVersion,
    pub extensions: Vec<String>,
}

pub fn profile_def(name: &str, description: &str, sections: &[&str]) -> ProfileDef {
    ProfileDef {
        name: name.into(),
        description: description.into(),
        include_sections: sections.iter().map(|s| s.to_string()).collect(),
    }
}

fn default_content_kind() -> String {
    "documentation".to_string()
}

fn default_generation_granularity() -> String {
    "section".to_string()
}

impl StandardDefinition {
    pub fn section_is_required(&self, name: &str) -> bool {
        self.required_sections
            .iter()
            .any(|s| s.canonical_name == name && s.required)
    }

    /// Case-insensitive alias match. Returns the first matching SectionDefinition.
    pub fn find_section_type(&self, heading: &str) -> Option<&SectionDefinition> {
        let h = heading.trim().to_lowercase();
        self.required_sections.iter().find(|s| {
            s.canonical_name.to_lowercase() == h
                || s.aliases.iter().any(|a| a.to_lowercase() == h)
        })
    }

    pub fn profile(&self, name: &str) -> Option<&ProfileDef> {
        let lower = name.to_lowercase();
        self.profiles.iter().find(|p| p.name.to_lowercase() == lower)
    }

    pub fn sections_for_profile(&self, name: &str) -> Vec<&SectionDefinition> {
        match self.profile(name) {
            Some(prof) => self
                .required_sections
                .iter()
                .filter(|s| prof.include_sections.contains(&s.semantic_type))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Returns section definitions where required=true that are missing from the given headings.
    pub fn missing_required(&self, headings: &[String]) -> Vec<&SectionDefinition> {
        let lower: Vec<String> = headings.iter().map(|h| h.to_lowercase()).collect();
        self.required_sections
            .iter()
            .filter(|s| s.required)
            .filter(|s| {
                let cn = s.canonical_name.to_lowercase();
                let matched = lower.contains(&cn)
                    || s.aliases.iter().any(|a| lower.contains(&a.to_lowercase()));
                !matched
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanSetting {
    pub threshold_rating: String,
    pub max_iterations: i32,
    pub fallback: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanScenario {
    pub repo_state: String,
    pub doc_state: String,
    pub tier: i32,
    pub step: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptCheck {
    pub check_name: String,
    pub domain_id: Option<String>,
    pub category: Option<String>,
    pub timeout_seconds: i32,
    pub requires_network: bool,
    pub result_schema: String,
    pub description: Option<String>,
}
