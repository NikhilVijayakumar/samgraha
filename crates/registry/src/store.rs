use crate::migration::MIGRATIONS;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use schemas::audit::AuditFinding;
use schemas::document::{Document, DocumentSection};
use schemas::enrichment::EnrichmentArtifact;
use schemas::registry::{
    BuildMetadata, GlossaryEntry, RegistryMetadata, RegistryStatus, Relationship,
};
use schemas::search::{SemanticSection, SectionQuery, SectionQueryResponse};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

pub struct RegistryStore {
    pub conn: Connection,
    _path: PathBuf,
}

impl RegistryStore {
    pub fn path_str(&self) -> Option<&str> {
        self._path.to_str().filter(|s| *s != ":memory:")
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)
            .context(format!("Failed to open registry at {}", path.display()))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let mut store = Self { conn, _path: path };
        store.run_migrations()?;
        store.update_build_metadata()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let mut store = Self {
            conn,
            _path: PathBuf::from(":memory:"),
        };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&mut self) -> Result<()> {
        let current_version: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _schema_version",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        for (i, migration) in MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i64;
            if version > current_version {
                self.conn.execute_batch(migration)?;
                self.conn.execute(
                    "INSERT INTO _schema_version (version) VALUES (?1)",
                    params![version],
                )?;
                info!("Applied migration V{}", version);
            }
        }
        Ok(())
    }

    fn update_build_metadata(&self) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('version', ?1)",
            params!["0.1.0"],
        )?;
        Ok(())
    }

    // --- Document operations ---

    pub fn insert_document(&self, doc: &Document) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO documents (id, path, hash, standard, title, body, metadata, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                doc.id,
                doc.path.as_str(),
                doc.hash,
                doc.standard,
                doc.title,
                doc.body,
                serde_json::to_string(&doc.metadata)?,
                doc.created_at,
                doc.updated_at,
            ],
        )?;
        Ok(doc.id)
    }

    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, created_at, updated_at FROM documents WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                metadata: serde_json::from_str(&row.get::<_, String>(6)?)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                sections: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, created_at, updated_at FROM documents WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                metadata: serde_json::from_str(&row.get::<_, String>(6)?)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                sections: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, created_at, updated_at FROM documents ORDER BY id",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: row.get(5)?,
                metadata: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                sections: Vec::new(),
            })
        })?;

        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }
        Ok(documents)
    }

    pub fn document_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn document_exists(&self, path: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_document_hash(&self, path: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT hash FROM documents WHERE path = ?1",
            params![path],
            |row| row.get(0),
        );
        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_document(&self, id: i64) -> Result<bool> {
        let affected = self
            .conn
            .execute("DELETE FROM documents WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    // --- Section operations ---

    pub fn insert_document_sections(&self, doc_id: i64, sections: &[DocumentSection]) -> Result<()> {
        self.conn.execute(
            "DELETE FROM document_sections WHERE document_id = ?1",
            params![doc_id],
        )?;
        self.insert_sections_recursive(doc_id, sections, 0)?;
        Ok(())
    }

    fn insert_sections_recursive(
        &self,
        doc_id: i64,
        sections: &[DocumentSection],
        order_offset: usize,
    ) -> Result<usize> {
        let mut order = order_offset;
        for section in sections {
            self.conn.execute(
                "INSERT INTO document_sections (document_id, semantic_type, canonical_name, content, required, section_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    doc_id,
                    section.semantic_type,
                    section.heading,
                    section.body,
                    section.required as i64,
                    order as i64,
                ],
            )?;
            order += 1;
            order = self.insert_sections_recursive(doc_id, &section.subsections, order)?;
        }
        Ok(order)
    }

    pub fn get_sections_by_type(&self, query: &SectionQuery) -> Result<SectionQueryResponse> {
        let start = Instant::now();
        let mut stmt = self.conn.prepare(
            "SELECT ds.id, ds.document_id, d.title, d.path, d.standard,
                    ds.semantic_type, ds.canonical_name, ds.content, ds.required, ds.section_order
             FROM document_sections ds
             JOIN documents d ON ds.document_id = d.id
             WHERE ds.semantic_type = ?1
               AND (?2 IS NULL OR d.standard = ?2)
             ORDER BY ds.section_order
             LIMIT ?3",
        )?;

        let rows = stmt.query_map(
            params![
                query.semantic_type,
                query.domain,
                query.max_results as i64,
            ],
            |row| {
                Ok(SemanticSection {
                    id: row.get(0)?,
                    document_id: row.get(1)?,
                    document_title: row.get(2)?,
                    document_path: row.get(3)?,
                    standard: row.get(4)?,
                    semantic_type: row.get(5)?,
                    canonical_name: row.get(6)?,
                    content: row.get(7)?,
                    required: row.get::<_, i64>(8)? != 0,
                    section_order: row.get(9)?,
                })
            },
        )?;

        let mut sections = Vec::new();
        for row in rows {
            sections.push(row?);
        }
        let total_count = sections.len();

        Ok(SectionQueryResponse {
            sections,
            total_count,
            semantic_type: query.semantic_type.clone(),
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    // --- Relationship operations ---

    pub fn insert_relationship(&self, rel: &Relationship) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO relationships (source_id, target_id, rel_type, metadata) VALUES (?1, ?2, ?3, ?4)",
            params![rel.source_id, rel.target_id, rel.rel_type, serde_json::to_string(&rel.metadata)?],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_relationships(&self, rels: &[Relationship]) -> Result<()> {
        for rel in rels {
            self.insert_relationship(rel)?;
        }
        Ok(())
    }

    pub fn clear_relationships(&self) -> Result<()> {
        self.conn.execute("DELETE FROM relationships", [])?;
        Ok(())
    }

    pub fn get_relationships_for_document(&self, doc_id: i64) -> Result<Vec<Relationship>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_id, target_id, rel_type, metadata FROM relationships WHERE source_id = ?1 OR target_id = ?1",
        )?;
        let rows = stmt.query_map(params![doc_id], |row| {
            Ok(Relationship {
                id: row.get(0)?,
                source_id: row.get(1)?,
                target_id: row.get(2)?,
                rel_type: row.get(3)?,
                metadata: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            })
        })?;
        let mut rels = Vec::new();
        for row in rows {
            rels.push(row?);
        }
        Ok(rels)
    }

    // --- Audit operations ---

    pub fn insert_audit_result(&self, finding: &AuditFinding) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO audit_results (document_id, check_id, severity, message, location)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                finding.document_id,
                finding.check_id,
                finding.severity.to_string(),
                finding.message,
                finding.location,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_audit_findings(&self, findings: &[AuditFinding]) -> Result<()> {
        for f in findings {
            if f.document_id.is_none() {
                continue;
            }
            self.insert_audit_result(f)?;
        }
        Ok(())
    }

    pub fn clear_audit_results(&self) -> Result<()> {
        self.conn.execute("DELETE FROM audit_results", [])?;
        Ok(())
    }

    pub fn get_audit_findings(&self, document_id: i64) -> Result<Vec<AuditFinding>> {
        let mut stmt = self.conn.prepare(
            "SELECT check_id, severity, message, location, document_id FROM audit_results WHERE document_id = ?1",
        )?;
        let rows = stmt.query_map(params![document_id], |row| {
            Ok(AuditFinding {
                check_id: row.get(0)?,
                severity: schemas::audit::Severity::from_str(&row.get::<_, String>(1)?),
                message: row.get(2)?,
                location: row.get(3)?,
                document_id: row.get(4)?,
                provider: "registry".into(),
            })
        })?;
        let mut findings = Vec::new();
        for row in rows {
            findings.push(row?);
        }
        Ok(findings)
    }

    // --- Enrichment operations ---

    pub fn insert_enrichment(&self, artifact: &EnrichmentArtifact) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO enrichment (document_id, artifact_type, content, provider, model)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                artifact.document_id,
                artifact.artifact_type.to_string(),
                artifact.content,
                artifact.provider,
                artifact.model,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_enrichments(&self, artifacts: &[EnrichmentArtifact]) -> Result<()> {
        for a in artifacts {
            self.insert_enrichment(a)?;
        }
        Ok(())
    }

    pub fn clear_enrichments(&self) -> Result<()> {
        self.conn.execute("DELETE FROM enrichment", [])?;
        Ok(())
    }

    // --- Glossary operations ---

    pub fn insert_glossary_entry(&self, entry: &GlossaryEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO glossary (term, definition, source_document_id)
             VALUES (?1, ?2, ?3)",
            params![entry.term, entry.definition, entry.source_document_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_glossary_entries(&self, entries: &[GlossaryEntry]) -> Result<()> {
        for e in entries {
            self.insert_glossary_entry(e)?;
        }
        Ok(())
    }

    // --- Integrity ---

    pub fn check_integrity(&self) -> Result<RegistryMetadata> {
        let integrity_ok: bool = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| {
                let result: String = row.get(0)?;
                Ok(result == "ok")
            })
            .unwrap_or(false);

        let doc_count = self.document_count().unwrap_or(0);
        let status = if integrity_ok {
            RegistryStatus::Valid
        } else {
            RegistryStatus::Corrupt
        };

        let metadata = RegistryMetadata {
            version: "0.1.0".to_string(),
            repository: String::new(),
            document_count: doc_count,
            build_timestamp: chrono_now(),
            compiler_version: "0.1.0".to_string(),
            integrity_hash: None,
            status,
        };
        Ok(metadata)
    }

    // --- Build metadata ---

    pub fn set_build_metadata(&self, meta: &BuildMetadata) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('compiler_version', ?1)",
            params![meta.compiler_version],
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('build_version', ?1)",
            params![meta.build_version],
        )?;
        self.conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('build_timestamp', ?1)",
            params![meta.build_timestamp],
        )?;
        Ok(())
    }

    pub fn get_build_metadata(&self) -> Result<BuildMetadata> {
        let compiler_version = self.get_meta_value("compiler_version").unwrap_or_default();
        let build_version = self.get_meta_value("build_version").unwrap_or_default();
        let build_timestamp = self.get_meta_value("build_timestamp").unwrap_or_default();
        Ok(BuildMetadata {
            document_hashes: HashMap::new(),
            artifact_hashes: HashMap::new(),
            compiler_version,
            build_version,
            build_timestamp,
            enrichment_version: None,
            audit_version: None,
        })
    }

    fn get_meta_value(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM build_metadata WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::document::{DocumentMetadata, DocumentPath};

    fn create_test_doc(id: i64) -> Document {
        Document {
            id,
            path: DocumentPath(PathBuf::from(format!("test_{}.md", id))),
            hash: "abc123".into(),
            standard: "architecture".into(),
            title: format!("Test Document {}", id),
            body: "# Test\n\nContent".into(),
            metadata: DocumentMetadata::default(),
            sections: Vec::new(),
            created_at: "now".into(),
            updated_at: "now".into(),
        }
    }

    #[test]
    fn test_insert_and_retrieve_document() {
        let store = RegistryStore::open_in_memory().unwrap();
        let doc = create_test_doc(1);
        store.insert_document(&doc).unwrap();

        let retrieved = store.get_document(1).unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Document 1");
        assert_eq!(retrieved.standard, "architecture");
    }

    #[test]
    fn test_document_count() {
        let store = RegistryStore::open_in_memory().unwrap();
        assert_eq!(store.document_count().unwrap(), 0);
        store.insert_document(&create_test_doc(1)).unwrap();
        assert_eq!(store.document_count().unwrap(), 1);
        store.insert_document(&create_test_doc(2)).unwrap();
        assert_eq!(store.document_count().unwrap(), 2);
    }

    #[test]
    fn test_integrity_check() {
        let store = RegistryStore::open_in_memory().unwrap();
        let meta = store.check_integrity().unwrap();
        assert_eq!(meta.status, RegistryStatus::Valid);
    }

    #[test]
    fn test_get_all_documents() {
        let store = RegistryStore::open_in_memory().unwrap();
        store.insert_document(&create_test_doc(1)).unwrap();
        store.insert_document(&create_test_doc(2)).unwrap();
        let docs = store.get_all_documents().unwrap();
        assert_eq!(docs.len(), 2);
    }
}
