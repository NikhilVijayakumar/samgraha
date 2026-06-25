use serde::{Deserialize, Serialize};

pub type StandardId = String;
pub type StandardVersion = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardDefinition {
    pub id: StandardId,
    pub name: String,
    pub version: StandardVersion,
    pub domain: String,
    pub description: String,
    pub required_sections: Vec<RequiredSection>,
    pub prohibited_content: Vec<String>,
    pub relationships: Vec<StandardRelationship>,
    pub audit_rules: Vec<AuditRuleDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredSection {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub allows_children: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardRelationship {
    pub from_domain: String,
    pub to_domain: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRuleDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandardDeclaration {
    pub standard_id: StandardId,
    pub version: StandardVersion,
    pub extensions: Vec<String>,
}

impl StandardDefinition {
    pub fn section_is_required(&self, name: &str) -> bool {
        self.required_sections
            .iter()
            .any(|s| s.name == name && s.required)
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
];
