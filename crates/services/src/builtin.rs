use registry::RegistryStore;
use std::path::PathBuf;
use std::sync::Arc;

/// Built-in knowledge domains and the .db filename each ships as, adjacent to
/// the running binary (mcp.exe / cli.exe).
pub const BUILTIN_DOMAINS: &[(&str, &str)] = &[("standards", "standards.db"), ("help", "help.db")];

/// Load built-in knowledge databases shipped next to the running binary.
/// Read-only secondary sources — missing or corrupt files are skipped, never fatal.
pub fn load_builtin_stores() -> Vec<(String, Arc<RegistryStore>)> {
    let binary_dir = match std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)) {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    BUILTIN_DOMAINS
        .iter()
        .filter_map(|(domain, filename)| {
            let db_path = binary_dir.join(filename);
            if !db_path.exists() {
                return None;
            }
            match RegistryStore::open(&db_path) {
                Ok(store) => {
                    tracing::info!("Loaded built-in {domain} knowledge store ({})", db_path.display());
                    Some((domain.to_string(), Arc::new(store)))
                }
                Err(e) => {
                    tracing::warn!("Failed to open built-in {domain} store: {e}");
                    None
                }
            }
        })
        .collect()
}
