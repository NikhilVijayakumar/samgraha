use crate::loader::StandardLoader;
use anyhow::{Context, Result};
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

    /// Empty registry — all standards now come from the knowledge-hub DB
    /// via `from_standards_db_and_overrides()`. Kept for backward compat
    /// with the `Default` trait and call sites that need an empty registry.
    pub fn with_builtins() -> Self {
        Self::new()
    }

    /// Open a knowledge-hub SQLite database at `{repo_root}/.samgraha/knowledge.db`
    /// and project it onto `StandardDefinition` structs via `db_reader`, then
    /// layer any repo-supplied `StandardDefinition` JSON/TOML files from
    /// `{repo_root}/.samgraha/standards/` on top.
    ///
    /// If the DB file does not exist, returns an empty registry with only
    /// repo-supplied overrides (if any).
    pub fn from_standards_db_and_overrides(repo_root: &Path) -> Result<Self> {
        let db_path = repo_root.join(".samgraha").join("knowledge.db");
        if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)
                .with_context(|| format!("Failed to open knowledge-hub DB at {}", db_path.display()))?;
            let mut registry = crate::db_reader::from_standards_db(&conn)?;
            let custom = StandardLoader::discover_from_path(&repo_root.join(".samgraha").join("standards"))?;
            for std in custom {
                registry.register(std);
            }
            Ok(registry)
        } else {
            tracing::info!(
                "No knowledge-hub DB at {}; no standards loaded",
                db_path.display()
            );
            Self::with_builtins_and_overrides(repo_root)
        }
    }

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
    fn with_builtins_and_overrides_returns_empty_when_no_custom_dir() {
        let repo_root = std::env::temp_dir().join(format!(
            "samgraha-standards-test-empty-{}",
            std::process::id()
        ));
        let registry = StandardRegistry::with_builtins_and_overrides(&repo_root).unwrap();
        // No builtins anymore — registry is empty without a DB or custom overrides.
        assert!(registry.all().is_empty());
    }

    #[test]
    fn with_builtins_and_overrides_lets_repo_supply_custom_standard() {
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
        let custom = registry.get("architecture", "1.0.0").unwrap();
        assert_eq!(custom.name, "Custom Architecture Standard");

        std::fs::remove_dir_all(&repo_root).ok();
    }
}
