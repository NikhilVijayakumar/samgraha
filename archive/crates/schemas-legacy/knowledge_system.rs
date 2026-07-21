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

#[cfg(test)]
mod tests {
    use super::*;

    /// Optimization 10: `system.toml` parsing had no dedicated test — this
    /// round-trips the proposal's own example (docs/crates-refactor-proposal.md §3).
    #[test]
    fn parses_proposal_example_system_toml() {
        let toml_str = r#"
            id = "dev"
            name = "Software Development"
            version = "1.0"
            description = "General software engineering documentation."
        "#;
        let identity: KnowledgeSystemIdentity = toml::from_str(toml_str).unwrap();
        assert_eq!(identity.id, "dev");
        assert_eq!(identity.name, "Software Development");
        assert_eq!(identity.version, "1.0");
        assert_eq!(identity.description, "General software engineering documentation.");
    }

    #[test]
    fn version_and_description_default_when_omitted() {
        let toml_str = "id = \"academic\"\nname = \"Academic Publishing\"\n";
        let identity: KnowledgeSystemIdentity = toml::from_str(toml_str).unwrap();
        assert_eq!(identity.version, "1.0.0");
        assert_eq!(identity.description, "");
    }

    #[test]
    fn missing_required_id_is_rejected() {
        let toml_str = "name = \"No Id\"\n";
        assert!(toml::from_str::<KnowledgeSystemIdentity>(toml_str).is_err());
    }
}
