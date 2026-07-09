use crate::builtin;
use crate::loader::StandardLoader;
use anyhow::Result;
use schemas::standard::{AuditRuleDef, StandardDeclaration, StandardDefinition};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StandardRegistry {
    standards: HashMap<String, StandardDefinition>,
}

impl StandardRegistry {
    pub fn new() -> Self {
        Self {
            standards: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        for std in builtin::all_builtin_standards() {
            registry.register(std);
        }
        registry
    }

    /// Built-ins, then any repo-supplied `StandardDefinition` JSON/TOML
    /// files under `{repo_root}/.samgraha/standards/` layered on top —
    /// a repo-supplied standard overrides a built-in of the same
    /// `id@version` (register() always overwrites); anything new is added
    /// alongside. Missing directory behaves exactly like `with_builtins()`.
    pub fn with_builtins_and_overrides(repo_root: &Path) -> Result<Self> {
        let mut registry = Self::with_builtins();
        let custom = StandardLoader::discover_from_path(&repo_root.join(".samgraha").join("standards"))?;
        for std in custom {
            registry.register(std);
        }
        Ok(registry)
    }

    pub fn register(&mut self, standard: StandardDefinition) {
        let key = format!("{}@{}", standard.id, standard.version);
        self.standards.insert(key, standard);
    }

    pub fn get(&self, id: &str, version: &str) -> Option<&StandardDefinition> {
        let key = format!("{id}@{version}");
        self.standards.get(&key)
    }

    pub fn get_by_declaration(&self, decl: &StandardDeclaration) -> Option<&StandardDefinition> {
        self.get(&decl.standard_id, &decl.version)
    }

    pub fn get_by_domain(&self, domain: &str) -> Option<&StandardDefinition> {
        self.standards.values().find(|s| s.domain == domain)
    }

    pub fn all(&self) -> Vec<&StandardDefinition> {
        self.standards.values().collect()
    }

    pub fn get_audit_rules(&self, domain: &str) -> Vec<&AuditRuleDef> {
        self.standards
            .values()
            .filter(|s| s.domain == domain)
            .flat_map(|s| s.audit_rules.iter())
            .collect()
    }

    pub fn has_standard(&self, domain: &str) -> bool {
        self.standards.values().any(|s| s.domain == domain)
    }

    pub fn domains(&self) -> Vec<&str> {
        self.standards.values().map(|s| s.domain.as_str()).collect()
    }

    pub fn merge(&mut self, other: StandardRegistry) {
        for (key, std) in other.standards {
            self.standards.entry(key).or_insert(std);
        }
    }
}

impl Default for StandardRegistry {
    fn default() -> Self {
        Self::with_builtins()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_builtins_and_overrides_returns_builtins_when_no_custom_dir() {
        let repo_root = std::env::temp_dir().join(format!(
            "samgraha-standards-test-empty-{}",
            std::process::id()
        ));
        let registry = StandardRegistry::with_builtins_and_overrides(&repo_root).unwrap();
        assert!(registry.has_standard("architecture"));
        assert_eq!(
            registry.get("architecture", "1.0.0").unwrap().required_sections.len(),
            StandardRegistry::with_builtins()
                .get("architecture", "1.0.0")
                .unwrap()
                .required_sections
                .len()
        );
    }

    #[test]
    fn with_builtins_and_overrides_lets_repo_override_a_builtin() {
        // Regression: StandardRegistry was always with_builtins() — a repo
        // had no way to supply its own StandardDefinition even though
        // StandardLoader::discover_from_path existed for exactly this.
        let repo_root = std::env::temp_dir().join(format!(
            "samgraha-standards-test-override-{}",
            std::process::id()
        ));
        let standards_dir = repo_root.join(".samgraha").join("standards");
        std::fs::create_dir_all(&standards_dir).unwrap();
        std::fs::write(
            standards_dir.join("architecture.json"),
            r#"{
                "id": "architecture",
                "name": "Custom Architecture Standard",
                "version": "1.0.0",
                "domain": "architecture",
                "description": "Repo-specific override",
                "required_sections": [],
                "prohibited_content": [],
                "relationships": [],
                "audit_rules": [],
                "profiles": []
            }"#,
        )
        .unwrap();

        let registry = StandardRegistry::with_builtins_and_overrides(&repo_root).unwrap();
        let overridden = registry.get("architecture", "1.0.0").unwrap();
        assert_eq!(overridden.name, "Custom Architecture Standard");
        assert!(overridden.required_sections.is_empty());
        // Unrelated builtins remain untouched.
        assert!(registry.has_standard("feature"));

        std::fs::remove_dir_all(&repo_root).ok();
    }
}
