use std::collections::HashMap;
use schemas::standard::{StandardDeclaration, StandardDefinition, AuditRuleDef};
use crate::builtin;

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
        self.standards
            .values()
            .map(|s| s.domain.as_str())
            .collect()
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
