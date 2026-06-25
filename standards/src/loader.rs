use std::path::Path;
use anyhow::{Result, Context};
use schemas::standard::{StandardDefinition, StandardDeclaration};
use crate::StandardRegistry;

pub struct StandardLoader;

impl StandardLoader {
    pub fn load_from_declarations(
        decls: &[StandardDeclaration],
        registry: &StandardRegistry,
    ) -> Result<Vec<StandardDefinition>> {
        let mut loaded = Vec::new();
        for decl in decls {
            match registry.get_by_declaration(decl) {
                Some(std) => loaded.push(std.clone()),
                None => anyhow::bail!("Standard {}@{} not found", decl.standard_id, decl.version),
            }
        }
        Ok(loaded)
    }

    pub fn discover_from_path(path: &Path) -> Result<Vec<StandardDefinition>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let mut standards = Vec::new();
        let entries = std::fs::read_dir(path)
            .context("Failed to read standards directory")?;
        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().map_or(false, |e| e == "json" || e == "toml") {
                let content = std::fs::read_to_string(&file_path)
                    .context(format!("Failed to read {}", file_path.display()))?;
                if let Ok(std) = serde_json::from_str::<StandardDefinition>(&content) {
                    standards.push(std);
                } else if let Ok(std) = toml::from_str::<StandardDefinition>(&content) {
                    standards.push(std);
                }
            }
        }
        Ok(standards)
    }
}
