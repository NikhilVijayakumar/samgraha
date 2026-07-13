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
    pub profiles: Vec<ProfileDef>,
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

pub const BUILTIN_DOMAINS: &[&str] = &[
    "readme",
    "vision",
    "philosophy",
    "architecture",
    "feature",
    "feature-design",
    "feature-technical",
    "design",
    "engineering",
    "external-context",
    "prototype",
    "help",
    "standards",
];
