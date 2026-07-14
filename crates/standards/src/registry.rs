use crate::loader::StandardLoader;
use anyhow::{Context, Result};
use schemas::audit::ScoringConfig;
use schemas::standard::{AuditRuleDef, StandardDeclaration, StandardDefinition, StandardDoc, PlanSetting, PlanScenario, ScriptCheck};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StandardRegistry {
    standards: HashMap<String, StandardDefinition>,
    rubrics: HashMap<String, String>,
    scoring: ScoringConfig,
    plan_settings: Vec<PlanSetting>,
    plan_scenarios: Vec<PlanScenario>,
    script_checks: Vec<ScriptCheck>,
    standard_docs: HashMap<String, StandardDoc>,
}

impl StandardRegistry {
    pub fn new() -> Self {
        Self {
            standards: HashMap::new(),
            rubrics: HashMap::new(),
            scoring: ScoringConfig::default(),
            plan_settings: Vec::new(),
            plan_scenarios: Vec::new(),
            script_checks: Vec::new(),
            standard_docs: HashMap::new(),
        }
    }

    /// Seeds exactly one Rust-native standard: `help`. Every other domain
    /// (readme, vision, architecture, ...) comes from the knowledge-hub DB
    /// via `from_standards_db_and_overrides()` and this returns empty for
    /// them — `help` is the one deliberate exception.
    ///
    /// `help` documents samgraha itself (its own CLI/MCP usage), not any
    /// user-registered documentation-standard system — it has zero
    /// dependency on knowledge-hub by design (same reasoning `docs/knowledge-hub/`
    /// has zero dependency on samgraha: the direction only goes one way).
    /// A repo picks whichever knowledge-hub system it wants, or none, and
    /// `help` is unaffected either way. Minimal on purpose — no required
    /// sections, no rules — just enough for `compile`'s `validate_config`
    /// to find *something* registered under "help" so `docs/raw/product-guide`
    /// (this repo's own help content, compiled by `scripts/build-release.ps1`
    /// into `bin/help.db`) can compile without needing a whole knowledge-hub
    /// system just to validate itself.
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register(StandardDefinition {
            id: "help".to_string(),
            name: "Help".to_string(),
            version: "1.0.0".to_string(),
            domain: "help".to_string(),
            description: "Samgraha's own usage documentation — no structural requirements.".to_string(),
            required_sections: Vec::new(),
            prohibited_content: Vec::new(),
            relationships: Vec::new(),
            audit_rules: Vec::new(),
            profiles: Vec::new(),
        });
        registry.register(StandardDefinition {
            id: "standards".to_string(),
            name: "Documentation Standards".to_string(),
            version: "1.0.0".to_string(),
            domain: "standards".to_string(),
            description: "Documentation-standards system — structural requirements for domain specs.".to_string(),
            required_sections: Vec::new(),
            prohibited_content: Vec::new(),
            relationships: Vec::new(),
            audit_rules: Vec::new(),
            profiles: Vec::new(),
        });
        registry
    }

    /// Open a knowledge-hub SQLite database at `{repo_root}/.samgraha/standards.db`
    /// and project it onto `StandardDefinition` structs, then layer any
    /// repo-supplied `StandardDefinition` JSON/TOML overrides on top.
    ///
    /// If the DB file does not exist, returns an empty registry with only
    /// repo-supplied overrides (if any).
    pub fn from_standards_db_and_overrides(repo_root: &Path) -> Result<Self> {
        Self::from_standards_db_and_overrides_with_system(repo_root, None)
    }

    /// Same as `from_standards_db_and_overrides` but accepts an explicit
    /// `system_name` to select a non-default documentation standard system
    /// (from `samgraha.toml [repository.documentation] standard_system`).
    pub fn from_standards_db_and_overrides_with_system(
        repo_root: &Path,
        system_name: Option<&str>,
    ) -> Result<Self> {
        let db_path = repo_root.join(".samgraha").join("standards.db");
        if db_path.exists() {
            let conn = rusqlite::Connection::open(&db_path)
                .with_context(|| format!("Failed to open knowledge-hub DB at {}", db_path.display()))?;
            let mut registry = crate::db_reader::from_standards_db(&conn, system_name)?;
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
        for (key, content) in other.rubrics {
            self.rubrics.entry(key).or_insert(content);
        }
        for (key, doc) in other.standard_docs {
            self.standard_docs.entry(key).or_insert(doc);
        }
        // Keep own scoring if both have one (first wins).
        if self.scoring.calculation_rules.is_empty() && !other.scoring.calculation_rules.is_empty() {
            self.scoring = other.scoring;
        }
    }

    /// Set the semantic audit rubrics (loaded from `templates` table).
    pub fn set_rubrics(&mut self, rubrics: HashMap<String, String>) {
        self.rubrics = rubrics;
    }

    /// Retrieve the semantic audit rubric for a domain/section_type pair.
    /// Returns `Err` when no rubric exists — callers treat that as "skip"
    /// (same semantics as the old file-based `get_audit_knowledge`).
    pub fn get_audit_knowledge(&self, domain: &str, section_type: &str) -> Result<&str> {
        let key = format!("{}/{}", domain, section_type);
        self.rubrics
            .get(&key)
            .map(|s| s.as_str())
            .with_context(|| format!("No audit knowledge for {}/{}", domain, section_type))
    }

    /// Retrieve the scoring configuration.
    pub fn scoring_config(&self) -> &ScoringConfig {
        &self.scoring
    }

    /// Set the scoring configuration (loaded from DB).
    pub fn set_scoring(&mut self, scoring: ScoringConfig) {
        self.scoring = scoring;
    }

    pub fn plan_settings(&self) -> &[PlanSetting] {
        &self.plan_settings
    }

    pub fn set_plan_settings(&mut self, settings: Vec<PlanSetting>) {
        self.plan_settings = settings;
    }

    pub fn plan_scenarios(&self) -> &[PlanScenario] {
        &self.plan_scenarios
    }

    pub fn set_plan_scenarios(&mut self, scenarios: Vec<PlanScenario>) {
        self.plan_scenarios = scenarios;
    }

    pub fn script_checks(&self) -> &[ScriptCheck] {
        &self.script_checks
    }

    pub fn set_script_checks(&mut self, checks: Vec<ScriptCheck>) {
        self.script_checks = checks;
    }

    /// The human-readable documentation-standards spec for a domain — the
    /// prose behind `StandardDefinition`'s structural rules/sections.
    pub fn get_standard_doc(&self, domain: &str) -> Option<&StandardDoc> {
        self.standard_docs.get(domain)
    }

    pub fn set_standard_docs(&mut self, docs: HashMap<String, StandardDoc>) {
        self.standard_docs = docs;
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
    fn with_builtins_and_overrides_has_only_help_when_no_custom_dir() {
        let repo_root = std::env::temp_dir().join(format!(
            "samgraha-standards-test-empty-{}",
            std::process::id()
        ));
        let registry = StandardRegistry::with_builtins_and_overrides(&repo_root).unwrap();
        // No knowledge-hub-DB-backed builtins — only the two Rust-native
        // exceptions: `help` and `standards`.
        assert_eq!(registry.all().len(), 2);
        assert!(registry.has_standard("help"));
        assert!(registry.has_standard("standards"));
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
