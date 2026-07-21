use anyhow::Result;
use registry::RegistryStore;
use std::path::Path;

/// Open the `help.db` shipped next to the running binary (`common::env::mcp_dir()`).
/// Registry-crate schema (`Document`/`domain = 'help'`), same shape as a repo's
/// own `.samgraha/knowledge.db` — that's what makes a plain document merge into
/// the local repo DB possible on sync. Read-only source; missing or corrupt is
/// not fatal, just means nothing to sync yet.
pub fn open_help_store() -> Option<RegistryStore> {
    let path = common::env::mcp_dir().join("help.db");
    if !path.exists() {
        return None;
    }
    match RegistryStore::open(&path) {
        Ok(store) => Some(store),
        Err(e) => {
            tracing::warn!("Failed to open help store at {}: {e}", path.display());
            None
        }
    }
}

/// Sync `help.db`'s documents into `root`'s local `.samgraha/knowledge.db`
/// under the `help` domain. Full replace — help content is never locally
/// authored, so dropping and re-inserting is safe and matches how
/// `standards.db` sync already behaves (whole-file copy).
///
/// Matches rows by *path*, never by the source db's raw `id` — ids are only
/// unique per-store, so literally reusing help.db's ids on `insert_document`
/// (an `INSERT OR REPLACE` keyed on the `id` primary key) could silently
/// clobber an unrelated local document that happens to share that same id.
/// Returns the number of documents synced (0 if no `help.db` is shipped).
pub fn sync_help_into_local(root: &Path) -> Result<usize> {
    let Some(help_store) = open_help_store() else {
        return Ok(0);
    };
    let help_docs = help_store.get_all_documents()?;

    let local_db_path = root.join(".samgraha").join("knowledge.db");
    if let Some(parent) = local_db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let local = RegistryStore::open(&local_db_path)?;

    for existing in local.get_documents_by_domain("help")? {
        local.delete_document(existing.id)?;
    }

    let mut next_id: i64 = local
        .conn
        .query_row("SELECT COALESCE(MAX(id), 0) FROM documents", [], |r| r.get(0))
        .unwrap_or(0);

    let mut synced = 0usize;
    for mut doc in help_docs {
        next_id += 1;
        doc.id = next_id;
        doc.standard = "help".to_string();
        local.insert_document(&doc)?;
        synced += 1;
    }
    Ok(synced)
}
