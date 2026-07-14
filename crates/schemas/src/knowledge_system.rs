use serde::{Deserialize, Serialize};

/// Represents the identity of a Knowledge System (parsed from a system.toml).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeSystemIdentity {
    /// The unique identifier of the system (e.g., "dev", "academic")
    pub id: String,
    /// The human-readable name of the system (e.g., "Software Development")
    pub name: String,
    /// The version of the system
    pub version: String,
    /// A description of the system's purpose
    pub description: String,
}
