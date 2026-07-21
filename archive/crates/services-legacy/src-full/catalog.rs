use anyhow::{Context, Result};
use registry::RegistryStore;
use std::path::Path;

/// Result of populating standard catalogs into knowledge.db.
#[derive(Debug, Default)]
pub struct CatalogPopulateResult {
    pub assets_inserted: usize,
    pub data_tables_inserted: bool,
}

/// A single catalog entry to insert into `standard_assets`.
#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub purpose: String,
}

/// Configuration for custom data tables (maps to `custom_data_tables`).
#[derive(Debug, Clone)]
pub struct DataTablesConfig {
    pub owner_script: String,
    pub prefix: String,
    pub purpose: String,
    pub tables: Vec<String>,
}

/// Populate `standard_assets` and `custom_data_tables` catalogs in
/// `knowledge.db` for a given standard. Called from both
/// `handle_register_standard` (adapter.rs) and `sync_knowledge_system`
/// (init.rs) to keep the logic in one place.
///
/// - Deletes existing rows for this standard before inserting (idempotent).
/// - `catalog`: assets to insert into `standard_assets` (may be empty).
/// - `data_tables`: if present, inserts a row into `custom_data_tables`.
pub fn populate_standard_catalogs(
    knowledge_db: &Path,
    standard_name: &str,
    catalog: &[CatalogEntry],
    data_tables: Option<&DataTablesConfig>,
) -> Result<CatalogPopulateResult> {
    let store = RegistryStore::open(knowledge_db)
        .context("Failed to open knowledge.db for catalog populate")?;
    let db = &store.conn;

    let mut result = CatalogPopulateResult::default();

    // Clear existing rows for this standard (idempotent re-populate).
    db.execute(
        "DELETE FROM standard_assets WHERE standard = ?1",
        rusqlite::params![standard_name],
    )?;
    db.execute(
        "DELETE FROM custom_data_tables WHERE standard = ?1",
        rusqlite::params![standard_name],
    )?;

    // Insert catalog entries into standard_assets.
    for entry in catalog {
        db.execute(
            "INSERT INTO standard_assets (standard, name, kind, path, purpose)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                standard_name,
                entry.name,
                entry.kind,
                entry.path,
                entry.purpose,
            ],
        )?;
        result.assets_inserted += 1;
    }

    // Insert custom data tables metadata.
    if let Some(dt) = data_tables {
        for table_name in &dt.tables {
            db.execute(
                "INSERT INTO custom_data_tables
                    (standard, table_name, purpose, owner_script)
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    standard_name,
                    table_name,
                    dt.purpose,
                    dt.owner_script,
                ],
            )?;
        }
        result.data_tables_inserted = !dt.tables.is_empty();
    }

    Ok(result)
}
