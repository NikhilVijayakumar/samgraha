use serde::{Deserialize, Serialize};

/// Represents the identity of a Knowledge System (parsed from a system.toml).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSystemIdentity {
    /// The unique identifier of the system (e.g., "dev", "academic")
    pub id: String,
    /// The human-readable name of the system (e.g., "Software Development")
    pub name: String,
    /// The version of the system (default: "1.0.0")
    #[serde(default = "default_knowledge_system_version")]
    pub version: String,
    /// A description of the system's purpose (optional)
    #[serde(default)]
    pub description: String,
}

fn default_knowledge_system_version() -> String {
    "1.0.0".to_string()
}
