use crate::migration::KNOWLEDGE_MIGRATIONS;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use schemas::audit::{
    AuditFinding, AuditStage, FindingStatus, GateResult, SectionChangedResult,
    SemanticReport,
};
use schemas::document::{Document, DocumentBody, DocumentSection};
use schemas::fix::{
    FixAttempt, FixPlan, FixPlanStatus, FixSession, FixStepStatus, PlanStep, PlanType,
    SessionStatus,
};
use schemas::enrichment::EnrichmentArtifact;
use schemas::graph::{EdgeType, GraphEdge, GraphNode, KnowledgeGraph};
use schemas::registry::{
    BuildMetadata, GlossaryEntry, RegistryMetadata, RegistryStatus, Relationship,
};
use schemas::search::{SemanticSection, SectionQuery, SectionQueryResponse};
use serde::Serialize;
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

        for (i, migration) in KNOWLEDGE_MIGRATIONS.iter().enumerate() {
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
        let body_json = serde_json::to_string(&doc.body)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO documents (id, path, hash, standard, title, body, metadata, quality, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                doc.id,
                doc.path.as_str(),
                doc.hash,
                doc.standard,
                doc.title,
                body_json,
                serde_json::to_string(&doc.metadata)?,
                serde_json::to_string(&doc.quality)?,
                doc.created_at,
                doc.updated_at,
            ],
        )?;
        Ok(doc.id)
    }

    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, quality, created_at, updated_at FROM documents WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: serde_json::from_str(&row.get::<_, String>(5)?)?,
                metadata: serde_json::from_str(&row.get::<_, String>(6)?)?,
                provenance: None,
                quality: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_document_by_path(&self, path: &str) -> Result<Option<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, quality, created_at, updated_at FROM documents WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: serde_json::from_str(&row.get::<_, String>(5)?)?,
                metadata: serde_json::from_str(&row.get::<_, String>(6)?)?,
                provenance: None,
                quality: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_documents(&self) -> Result<Vec<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, quality, created_at, updated_at FROM documents ORDER BY id",
        )?;

        let rows = stmt.query_map([], |row| {
            let body_str: String = row.get(5)?;
            Ok(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: serde_json::from_str(&body_str).unwrap_or_else(|_| DocumentBody::Generic {
                    raw: body_str,
                    sections: Vec::new(),
                }),
                metadata: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                provenance: None,
                quality: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
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
                "INSERT INTO document_sections (document_id, semantic_type, canonical_name, content, required, section_order, hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    doc_id,
                    section.semantic_type,
                    section.heading,
                    section.body,
                    section.required as i64,
                    order as i64,
                    section.hash,
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
                    ds.semantic_type, ds.canonical_name, ds.content, ds.required, ds.section_order, ds.hash
             FROM document_sections ds
             JOIN documents d ON ds.document_id = d.id
             WHERE ds.semantic_type = ?1
               AND (?2 IS NULL OR d.standard = ?2)
               AND (?3 IS NULL OR ds.document_id = ?3)
             ORDER BY ds.section_order
             LIMIT ?4",
        )?;

        let rows = stmt.query_map(
            params![
                query.semantic_type,
                query.domain,
                query.document_id,
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
                    hash: row.get(10)?,
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
                stage: None,
                section_id: None,
                confidence: None,
                evidence: None,
                status: None,
                strategy: None,
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

    // --- Graph operations ---

    pub fn insert_graph(&self, graph: &KnowledgeGraph) -> Result<()> {
        for node in &graph.nodes {
            self.conn.execute(
                "INSERT OR REPLACE INTO graph_nodes (urn, node_type, document_id, title) VALUES (?1, ?2, ?3, ?4)",
                params![node.urn.as_str(), node.node_type, node.document_id, node.title],
            )?;
        }
        for edge in &graph.edges {
            self.conn.execute(
                "INSERT OR IGNORE INTO graph_edges (source_urn, target_urn, edge_type, metadata) VALUES (?1, ?2, ?3, ?4)",
                params![
                    edge.source_urn.as_str(),
                    edge.target_urn.as_str(),
                    edge.edge_type.as_str(),
                    serde_json::to_string(&edge.metadata)?,
                ],
            )?;
        }
        Ok(())
    }

    pub fn clear_graph(&self) -> Result<()> {
        self.conn.execute("DELETE FROM graph_edges", [])?;
        self.conn.execute("DELETE FROM graph_nodes", [])?;
        Ok(())
    }

    pub fn get_graph_for_document(&self, document_id: i64) -> Result<KnowledgeGraph> {
        let mut node_stmt = self.conn.prepare(
            "SELECT urn, node_type, document_id, title FROM graph_nodes WHERE document_id = ?1",
        )?;
        let nodes: Vec<GraphNode> = node_stmt.query_map(params![document_id], |row| {
            Ok(GraphNode {
                urn: row.get::<_, String>(0)?.into(),
                node_type: row.get(1)?,
                document_id: row.get(2)?,
                title: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        let urns: Vec<String> = nodes.iter().map(|n: &GraphNode| n.urn.as_str().to_string()).collect();
        let mut edges = Vec::new();
        if !urns.is_empty() {
            let placeholders: Vec<String> = urns.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
            let sql = format!(
                "SELECT source_urn, target_urn, edge_type, metadata FROM graph_edges WHERE source_urn IN ({}) OR target_urn IN ({})",
                placeholders.join(","), placeholders.join(",")
            );
            let mut edge_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
            for urn in &urns {
                edge_params.push(Box::new(urn.clone()));
            }
            for urn in &urns {
                edge_params.push(Box::new(urn.clone()));
            }
            let mut edge_stmt = self.conn.prepare(&sql)?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> = edge_params.iter().map(|p| p.as_ref()).collect();
            let edge_rows = edge_stmt.query_map(param_refs.as_slice(), |row| {
                Ok(GraphEdge {
                    source_urn: row.get::<_, String>(0)?.into(),
                    target_urn: row.get::<_, String>(1)?.into(),
                    edge_type: EdgeType::from_str(&row.get::<_, String>(2)?),
                    metadata: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or_default(),
                })
            })?;
            for row in edge_rows {
                if let Ok(edge) = row {
                    edges.push(edge);
                }
            }
        }

        Ok(KnowledgeGraph { nodes, edges })
    }

    pub fn get_all_graph_nodes(&self) -> Result<Vec<GraphNode>> {
        let mut stmt = self.conn.prepare("SELECT urn, node_type, document_id, title FROM graph_nodes ORDER BY urn")?;
        let nodes = stmt.query_map([], |row| {
            Ok(GraphNode {
                urn: row.get::<_, String>(0)?.into(),
                node_type: row.get(1)?,
                document_id: row.get(2)?,
                title: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(nodes)
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

    pub fn get_revision(&self) -> Result<u64> {
        self
            .get_meta_value("revision")
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| anyhow::anyhow!("No revision stored"))
    }

    pub fn set_revision(&self, revision: u64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO build_metadata (key, value) VALUES ('revision', ?1)",
            params![revision.to_string()],
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

    // ── Semantic Audit: Read Methods ────────────────────────────────────────

    pub fn get_domains(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT standard FROM documents ORDER BY standard",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut domains = Vec::new();
        for row in rows {
            domains.push(row?);
        }
        Ok(domains)
    }

    pub fn get_documents_by_domain(&self, domain: &str) -> Result<Vec<Document>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, hash, standard, title, body, metadata, quality, created_at, updated_at
             FROM documents WHERE standard = ?1 ORDER BY id",
        )?;
        let rows = stmt.query_map(params![domain], |row| {
            let body_str: String = row.get(5)?;
            Ok(Document {
                id: row.get(0)?,
                path: schemas::document::DocumentPath(PathBuf::from(row.get::<_, String>(1)?)),
                hash: row.get(2)?,
                standard: row.get(3)?,
                title: row.get(4)?,
                body: serde_json::from_str(&body_str).unwrap_or_else(|_| DocumentBody::Generic {
                    raw: body_str,
                    sections: Vec::new(),
                }),
                metadata: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                provenance: None,
                quality: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }
        Ok(documents)
    }

    pub fn get_section_by_id(&self, section_id: i64) -> Result<Option<SemanticSection>> {
        let mut stmt = self.conn.prepare(
            "SELECT ds.id, ds.document_id, d.title, d.path, d.standard,
                    ds.semantic_type, ds.canonical_name, ds.content, ds.required, ds.section_order, ds.hash
             FROM document_sections ds
             JOIN documents d ON ds.document_id = d.id
             WHERE ds.id = ?1",
        )?;
        let mut rows = stmt.query(params![section_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(SemanticSection {
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
                hash: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// All sections for one document, any semantic_type — unlike
    /// `get_sections_by_type`, which requires a single type filter.
    pub fn get_all_sections_for_document(&self, document_id: i64) -> Result<Vec<SemanticSection>> {
        let mut stmt = self.conn.prepare(
            "SELECT ds.id, ds.document_id, d.title, d.path, d.standard,
                    ds.semantic_type, ds.canonical_name, ds.content, ds.required, ds.section_order, ds.hash
             FROM document_sections ds
             JOIN documents d ON ds.document_id = d.id
             WHERE ds.document_id = ?1
             ORDER BY ds.section_order",
        )?;

        let rows = stmt.query_map(params![document_id], |row| {
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
                hash: row.get(10)?,
            })
        })?;

        let mut sections = Vec::new();
        for row in rows {
            sections.push(row?);
        }
        Ok(sections)
    }

    pub fn get_audit_knowledge(&self, repo_root: &Path, domain: &str, section_type: &str) -> Result<String> {
        let knowledge_path = repo_root
            .join("docs/raw/audit-standards")
            .join(domain)
            .join(format!("{}.md", section_type));
        let content = std::fs::read_to_string(&knowledge_path)
            .with_context(|| format!("Audit knowledge not found: {:?}", knowledge_path))?;
        Ok(content)
    }

    pub fn get_section_changed(&self, section_id: i64) -> Result<SectionChangedResult> {
        // Get current hash from document_sections
        let current_hash: Option<String> = self
            .conn
            .query_row(
                "SELECT hash FROM document_sections WHERE id = ?1",
                params![section_id],
                |row| row.get(0),
            )
            .ok();

        let current_hash = match current_hash {
            Some(h) => h,
            None => {
                return Ok(SectionChangedResult {
                    changed: true,
                    previous_report_id: None,
                });
            }
        };

        // Get stored hash from section_audit_hashes
        let stored: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT hash, report_id FROM section_audit_hashes WHERE section_id = ?1",
                params![section_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .ok();

        match stored {
            Some((stored_hash, report_id)) if stored_hash == current_hash => {
                Ok(SectionChangedResult {
                    changed: false,
                    previous_report_id: Some(report_id),
                })
            }
            _ => Ok(SectionChangedResult {
                changed: true,
                previous_report_id: None,
            }),
        }
    }

    pub fn check_gate(&self, stage: AuditStage, document_id: Option<i64>) -> Result<GateResult> {
        let stage_str = match stage {
            AuditStage::Deterministic => "deterministic",
            AuditStage::Section => "section",
            AuditStage::Document => "document",
            AuditStage::CrossDomain => "cross_domain",
        };

        match stage {
            AuditStage::Deterministic => {
                // Check audit_results for ERROR-severity findings
                let count: i64 = match document_id {
                    Some(did) => self.conn.query_row(
                        "SELECT COUNT(*) FROM audit_results WHERE document_id = ?1 AND severity = 'error'",
                        params![did],
                        |row| row.get(0),
                    ).unwrap_or(0),
                    None => self.conn.query_row(
                        "SELECT COUNT(*) FROM audit_results WHERE severity = 'error'",
                        [],
                        |row| row.get(0),
                    ).unwrap_or(0),
                };
                if count > 0 {
                    Ok(GateResult {
                        blocked: true,
                        reason: Some(format!("{} ERROR-severity deterministic findings", count)),
                        blocking_ids: Vec::new(),
                    })
                } else {
                    Ok(GateResult { blocked: false, reason: None, blocking_ids: Vec::new() })
                }
            }
            _ => {
                // Check semantic_reports for ERROR-severity findings
                let (where_clause, doc_param): (&str, Option<i64>) = match document_id {
                    Some(did) => ("AND document_id = ?2", Some(did)),
                    None => ("", None),
                };

                let sql = format!(
                    "SELECT COUNT(*) FROM semantic_reports
                     WHERE stage = ?1 {} AND score < 100",
                    where_clause
                );

                let count: i64 = match doc_param {
                    Some(did) => self.conn.query_row(
                        &sql,
                        params![stage_str, did],
                        |row| row.get(0),
                    ).unwrap_or(0),
                    None => self.conn.query_row(
                        &sql,
                        params![stage_str],
                        |row| row.get(0),
                    ).unwrap_or(0),
                };

                if count > 0 {
                    Ok(GateResult {
                        blocked: true,
                        reason: Some(format!("{} {} reports not converged (score < 100)", count, stage_str)),
                        blocking_ids: Vec::new(),
                    })
                } else {
                    Ok(GateResult { blocked: false, reason: None, blocking_ids: Vec::new() })
                }
            }
        }
    }

    pub fn get_audit_report(
        &self,
        domain: &str,
        document_id: Option<i64>,
        section_id: Option<i64>,
        stage: AuditStage,
    ) -> Result<Option<SemanticReport>> {
        let stage_str = match stage {
            AuditStage::Deterministic => "deterministic",
            AuditStage::Section => "section",
            AuditStage::Document => "document",
            AuditStage::CrossDomain => "cross_domain",
        };

        let mut sql = String::from(
            "SELECT report_uuid, stage, domain, document_id, section_id, score, findings, strategy, document_revision, document_hash, created_at
             FROM semantic_reports WHERE domain = ?1 AND stage = ?2",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(domain.to_string()));
        param_values.push(Box::new(stage_str.to_string()));

        if let Some(did) = document_id {
            sql.push_str(" AND document_id = ?3");
            param_values.push(Box::new(did));
        }
        if let Some(sid) = section_id {
            sql.push_str(" AND section_id = ?4");
            param_values.push(Box::new(sid));
        }
        sql.push_str(" ORDER BY id DESC LIMIT 1");

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let mut rows = stmt.query(param_refs.as_slice())?;

        if let Some(row) = rows.next()? {
            let findings_str: String = row.get(6)?;
            let findings: Vec<schemas::audit::AuditFinding> =
                serde_json::from_str(&findings_str).unwrap_or_default();
            Ok(Some(SemanticReport {
                report_id: row.get(0)?,
                stage: stage,
                domain: row.get(2)?,
                document_id: row.get(3)?,
                section_id: row.get(4)?,
                score: row.get(5)?,
                findings,
                strategy: row.get(7)?,
                document_revision: row.get(8)?,
                document_hash: row.get(9)?,
                created_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// All stored semantic reports for a domain, latest revision only per
    /// `(stage, document_id, section_id)` — unlike `get_audit_report`, which returns a single
    /// row for one exact key, this powers scorecard regeneration where every pending task
    /// across the domain needs to be checked against what's already been judged.
    pub fn get_semantic_reports_by_domain(&self, domain: &str) -> Result<Vec<SemanticReport>> {
        let mut stmt = self.conn.prepare(
            "SELECT report_uuid, stage, domain, document_id, section_id, score, findings, strategy, document_revision, document_hash, created_at
             FROM semantic_reports WHERE domain = ?1 ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![domain], |row| {
            let stage_str: String = row.get(1)?;
            let stage = match stage_str.as_str() {
                "deterministic" => AuditStage::Deterministic,
                "section" => AuditStage::Section,
                "document" => AuditStage::Document,
                _ => AuditStage::CrossDomain,
            };
            let findings_str: String = row.get(6)?;
            let findings: Vec<AuditFinding> = serde_json::from_str(&findings_str).unwrap_or_default();
            Ok(SemanticReport {
                report_id: row.get(0)?,
                stage,
                domain: row.get(2)?,
                document_id: row.get(3)?,
                section_id: row.get(4)?,
                score: row.get(5)?,
                findings,
                strategy: row.get(7)?,
                document_revision: row.get(8)?,
                document_hash: row.get(9)?,
                created_at: row.get(10)?,
            })
        })?;

        let mut seen: std::collections::HashSet<(String, Option<i64>, Option<i64>)> = std::collections::HashSet::new();
        let mut out = Vec::new();
        for row in rows {
            let report = row?;
            let key = (format!("{:?}", report.stage), report.document_id, report.section_id);
            if seen.insert(key) {
                out.push(report);
            }
        }
        Ok(out)
    }

    // ── Semantic Audit: Write Methods ───────────────────────────────────────

    fn store_report(&self, report: &SemanticReport) -> Result<i64> {
        let stage_str = match report.stage {
            AuditStage::Deterministic => "deterministic",
            AuditStage::Section => "section",
            AuditStage::Document => "document",
            AuditStage::CrossDomain => "cross_domain",
        };
        let findings_json = serde_json::to_string(&report.findings)?;
        self.conn.execute(
            "INSERT INTO semantic_reports (report_uuid, stage, domain, document_id, section_id, score, findings, strategy, document_revision, document_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                report.report_id,
                stage_str,
                report.domain,
                report.document_id,
                report.section_id,
                report.score,
                findings_json,
                report.strategy,
                report.document_revision,
                report.document_hash,
                report.created_at,
            ],
        )?;
        let id = self.conn.last_insert_rowid();

        // Update section_audit_hashes if this is a section report
        if report.stage == AuditStage::Section {
            if let Some(section_id) = report.section_id {
                if let Some(doc_hash) = &report.document_hash {
                    self.conn.execute(
                        "INSERT OR REPLACE INTO section_audit_hashes (section_id, hash, report_id, checked_at)
                         VALUES (?1, ?2, ?3, datetime('now'))",
                        params![section_id, doc_hash, id],
                    )?;
                }
            }
        }

        Ok(id)
    }

    pub fn store_section_report(&self, report: &SemanticReport) -> Result<i64> {
        self.store_report(report)
    }

    pub fn store_document_report(&self, report: &SemanticReport) -> Result<i64> {
        self.store_report(report)
    }

    pub fn store_cross_domain_report(&self, report: &SemanticReport) -> Result<i64> {
        self.store_report(report)
    }

    // ── Spec-Layer (Pipeline Checklist) Reports ─────────────────────────────

    pub fn store_pipeline_check_report(&self, report: &schemas::audit::PipelineCheckReport) -> Result<i64> {
        let findings_json = serde_json::to_string(&report.findings)?;
        self.conn.execute(
            "INSERT INTO pipeline_semantic_reports (report_uuid, pipeline, check_id, score, findings, git_revision, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                report.report_id,
                report.pipeline,
                report.check_id,
                report.score,
                findings_json,
                report.git_revision,
                report.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_pipeline_check_report(
        &self,
        pipeline: &str,
        check_id: &str,
    ) -> Result<Option<schemas::audit::PipelineCheckReport>> {
        let mut stmt = self.conn.prepare(
            "SELECT report_uuid, pipeline, check_id, score, findings, git_revision, created_at
             FROM pipeline_semantic_reports WHERE pipeline = ?1 AND check_id = ?2
             ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(params![pipeline, check_id])?;

        if let Some(row) = rows.next()? {
            let findings_str: String = row.get(4)?;
            let findings: Vec<schemas::audit::AuditFinding> =
                serde_json::from_str(&findings_str).unwrap_or_default();
            Ok(Some(schemas::audit::PipelineCheckReport {
                report_id: row.get(0)?,
                pipeline: row.get(1)?,
                check_id: row.get(2)?,
                score: row.get(3)?,
                findings,
                git_revision: row.get(5)?,
                created_at: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Pipeline counterpart to `check_gate`'s Section/Document/CrossDomain
    /// branch — blocks while any of this pipeline's judged checks scored
    /// below 100. Kept separate from `check_gate`/`AuditStage` rather than
    /// adding a `Pipeline` variant there: Spec-layer checks have no
    /// section/document/cross_domain distinction, so reusing that enum would
    /// force every exhaustive `AuditStage` match to grow a meaningless arm.
    pub fn check_pipeline_gate(&self, pipeline: &str) -> Result<GateResult> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM pipeline_semantic_reports WHERE pipeline = ?1 AND score < 100",
            params![pipeline],
            |row| row.get(0),
        ).unwrap_or(0);

        if count > 0 {
            Ok(GateResult {
                blocked: true,
                reason: Some(format!("{} '{}' spec-layer checks not converged (score < 100)", count, pipeline)),
                blocking_ids: Vec::new(),
            })
        } else {
            Ok(GateResult { blocked: false, reason: None, blocking_ids: Vec::new() })
        }
    }

    /// Averages the *latest* judged score per check_id for a pipeline —
    /// not a flat average over every row, which would let a re-judged
    /// check's stale earlier scores skew the result (same append-only shape
    /// `check_pipeline_gate` has to account for). `None` means no check has
    /// been judged yet, not a score of 0.
    pub fn get_pipeline_spec_score(&self, pipeline: &str) -> Result<Option<f64>> {
        let mut stmt = self.conn.prepare(
            "SELECT score FROM pipeline_semantic_reports p1
             WHERE pipeline = ?1 AND id = (
                 SELECT MAX(id) FROM pipeline_semantic_reports p2
                 WHERE p2.pipeline = p1.pipeline AND p2.check_id = p1.check_id
             )",
        )?;
        let scores: Vec<i64> = stmt
            .query_map(params![pipeline], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if scores.is_empty() {
            Ok(None)
        } else {
            Ok(Some(scores.iter().sum::<i64>() as f64 / scores.len() as f64))
        }
    }

    pub fn store_summary_report(&self, report: &schemas::audit::SummaryReport) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO summary_reports (target_type, target_name, deterministic_report_ref, standard_report_ref, spec_report_ref, overall_score, readiness, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                report.target_type,
                report.target_name,
                report.deterministic_score.map(|s| s.to_string()),
                report.standard_score.map(|s| s.to_string()),
                report.spec_score.map(|s| s.to_string()),
                report.overall_score,
                report.readiness.to_string(),
                report.created_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_finding_status(
        &self,
        report_id: i64,
        criterion_id: &str,
        status: FindingStatus,
    ) -> Result<()> {
        // Load existing findings, update the matching criterion, save
        let findings_json: String = self
            .conn
            .query_row(
                "SELECT findings FROM semantic_reports WHERE id = ?1",
                params![report_id],
                |row| row.get(0),
            )
            .map_err(|_| anyhow::anyhow!("Report not found: {}", report_id))?;

        let mut findings: Vec<schemas::audit::AuditFinding> =
            serde_json::from_str(&findings_json)?;

        for finding in &mut findings {
            if finding.check_id == criterion_id {
                finding.status = Some(status.clone());
            }
        }

        let updated_json = serde_json::to_string(&findings)?;
        self.conn.execute(
            "UPDATE semantic_reports SET findings = ?1 WHERE id = ?2",
            params![updated_json, report_id],
        )?;
        Ok(())
    }

    // ── Per-Audit Report Operations (Phase 8) ────────────────────────────

    // ═══════════════════════════════════════════════════════════════════════
    // Build Reports
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insert_build_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        contract_name: Option<&str>,
        declared_produces: Option<&str>,
        actual_artifacts: Option<&str>,
        artifact_freshness: Option<&str>,
        execution_success: Option<bool>,
        execution_output: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO build_reports (session_id, pipeline, score, git_revision, contract_name, declared_produces, actual_artifacts, artifact_freshness, execution_success, execution_output)
             VALUES (?1, 'build', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session_id, score, git_revision, contract_name,
                declared_produces, actual_artifacts, artifact_freshness,
                execution_success.map(|v| v as i64), execution_output,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        self.insert_report_findings("build", report_id, findings)?;
        Ok(report_id)
    }

    pub fn query_build_sessions(&self, limit: usize) -> Result<Vec<BuildSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'build' AND rf.report_id = br.id AND rf.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'build' AND rf.report_id = br.id AND rf.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'build' AND rf.report_id = br.id AND rf.severity = 'suggestion'), 0) as sug_count
             FROM build_reports br ORDER BY br.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(BuildSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                git_revision: row.get(3)?,
                created_at: row.get(4)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(5)? as usize,
                    warnings: row.get::<_, i64>(6)? as usize,
                    suggestions: row.get::<_, i64>(7)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_build_report_with_findings(&self, report_id: i64) -> Result<Option<BuildReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, contract_name, declared_produces, actual_artifacts, artifact_freshness, execution_success, execution_output
             FROM build_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("build", report_id)?;
        Ok(Some(BuildReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            git_revision: row.get(3)?,
            created_at: row.get(4)?,
            contract_name: row.get(5)?,
            declared_produces: row.get(6)?,
            actual_artifacts: row.get(7)?,
            artifact_freshness: row.get(8)?,
            execution_success: row.get::<_, Option<i64>>(9)?.map(|v| v != 0),
            execution_output: row.get(10)?,
            findings,
        }))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Security Reports
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insert_security_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        secrets_scanned: i64,
        secrets_found: i64,
        runtime_checks: i64,
        runtime_issues: i64,
        high_risk_findings: i64,
        threat_summary: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO security_reports (session_id, pipeline, score, git_revision, secrets_scanned, secrets_found, runtime_checks, runtime_issues, high_risk_findings, threat_summary)
             VALUES (?1, 'security', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session_id, score, git_revision, secrets_scanned, secrets_found,
                runtime_checks, runtime_issues, high_risk_findings, threat_summary,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        self.insert_report_findings("security", report_id, findings)?;
        Ok(report_id)
    }

    pub fn query_security_sessions(&self, limit: usize) -> Result<Vec<SecuritySessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, secrets_scanned, secrets_found, runtime_checks, runtime_issues, high_risk_findings,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'security' AND rf.report_id = sr.id AND rf.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'security' AND rf.report_id = sr.id AND rf.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'security' AND rf.report_id = sr.id AND rf.severity = 'suggestion'), 0) as sug_count
             FROM security_reports sr ORDER BY sr.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(SecuritySessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                git_revision: row.get(3)?,
                created_at: row.get(4)?,
                secrets_scanned: row.get(5)?,
                secrets_found: row.get(6)?,
                runtime_checks: row.get(7)?,
                runtime_issues: row.get(8)?,
                high_risk_findings: row.get(9)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(10)? as usize,
                    warnings: row.get::<_, i64>(11)? as usize,
                    suggestions: row.get::<_, i64>(12)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_security_report_with_findings(&self, report_id: i64) -> Result<Option<SecurityReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, secrets_scanned, secrets_found, runtime_checks, runtime_issues, high_risk_findings, threat_summary
             FROM security_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("security", report_id)?;
        Ok(Some(SecurityReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            git_revision: row.get(3)?,
            created_at: row.get(4)?,
            secrets_scanned: row.get(5)?,
            secrets_found: row.get(6)?,
            runtime_checks: row.get(7)?,
            runtime_issues: row.get(8)?,
            high_risk_findings: row.get(9)?,
            threat_summary: row.get(10)?,
            findings,
        }))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Consistency Reports
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insert_consistency_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        vision_exists: bool,
        architecture_exists: bool,
        structure_score: Option<f64>,
        naming_issues: Option<&str>,
        cross_references: i64,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO consistency_reports (session_id, pipeline, score, git_revision, vision_exists, architecture_exists, structure_score, naming_issues, cross_references)
             VALUES (?1, 'consistency', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session_id, score, git_revision,
                vision_exists as i64, architecture_exists as i64,
                structure_score, naming_issues, cross_references,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        self.insert_report_findings("consistency", report_id, findings)?;
        Ok(report_id)
    }

    pub fn query_consistency_sessions(&self, limit: usize) -> Result<Vec<ConsistencySessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, vision_exists, architecture_exists, structure_score, cross_references,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'consistency' AND rf.report_id = cr.id AND rf.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'consistency' AND rf.report_id = cr.id AND rf.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'consistency' AND rf.report_id = cr.id AND rf.severity = 'suggestion'), 0) as sug_count
             FROM consistency_reports cr ORDER BY cr.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ConsistencySessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                git_revision: row.get(3)?,
                created_at: row.get(4)?,
                vision_exists: row.get::<_, i64>(5)? != 0,
                architecture_exists: row.get::<_, i64>(6)? != 0,
                structure_score: row.get(7)?,
                cross_references: row.get::<_, i64>(8)? as usize,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(9)? as usize,
                    warnings: row.get::<_, i64>(10)? as usize,
                    suggestions: row.get::<_, i64>(11)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_consistency_report_with_findings(&self, report_id: i64) -> Result<Option<ConsistencyReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, vision_exists, architecture_exists, structure_score, naming_issues, cross_references
             FROM consistency_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("consistency", report_id)?;
        Ok(Some(ConsistencyReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            git_revision: row.get(3)?,
            created_at: row.get(4)?,
            vision_exists: row.get::<_, i64>(5)? != 0,
            architecture_exists: row.get::<_, i64>(6)? != 0,
            structure_score: row.get(7)?,
            naming_issues: row.get(8)?,
            cross_references: row.get::<_, i64>(9)? as usize,
            findings,
        }))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Coverage Reports
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insert_coverage_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        features_count: i64,
        src_files_count: i64,
        feature_coverage_pct: Option<f64>,
        uncovered_features: Option<&str>,
        doc_types_covered: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO coverage_reports (session_id, pipeline, score, git_revision, features_count, src_files_count, feature_coverage_pct, uncovered_features, doc_types_covered)
             VALUES (?1, 'coverage', ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session_id, score, git_revision,
                features_count, src_files_count, feature_coverage_pct,
                uncovered_features, doc_types_covered,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        self.insert_report_findings("coverage", report_id, findings)?;
        Ok(report_id)
    }

    pub fn query_coverage_sessions(&self, limit: usize) -> Result<Vec<CoverageSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, features_count, src_files_count, feature_coverage_pct,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'coverage' AND rf.report_id = cr.id AND rf.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'coverage' AND rf.report_id = cr.id AND rf.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings rf WHERE rf.report_type = 'coverage' AND rf.report_id = cr.id AND rf.severity = 'suggestion'), 0) as sug_count
             FROM coverage_reports cr ORDER BY cr.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(CoverageSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                git_revision: row.get(3)?,
                created_at: row.get(4)?,
                features_count: row.get::<_, i64>(5)? as usize,
                src_files_count: row.get::<_, i64>(6)? as usize,
                feature_coverage_pct: row.get(7)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(8)? as usize,
                    warnings: row.get::<_, i64>(9)? as usize,
                    suggestions: row.get::<_, i64>(10)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_coverage_report_with_findings(&self, report_id: i64) -> Result<Option<CoverageReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, git_revision, created_at, features_count, src_files_count, feature_coverage_pct, uncovered_features, doc_types_covered
             FROM coverage_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("coverage", report_id)?;
        Ok(Some(CoverageReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            git_revision: row.get(3)?,
            created_at: row.get(4)?,
            features_count: row.get::<_, i64>(5)? as usize,
            src_files_count: row.get::<_, i64>(6)? as usize,
            feature_coverage_pct: row.get(7)?,
            uncovered_features: row.get(8)?,
            doc_types_covered: row.get(9)?,
            findings,
        }))
    }

    // ── Architecture (Phase 9) ──────────────────────────────────────────

    pub fn insert_architecture_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        collection_integrity_score: Option<f64>,
        structural_integrity_score: Option<f64>,
        consistency_score: Option<f64>,
        cross_repo_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO architecture_reports (session_id, score, git_revision, previous_score, engineering_readiness, collection_integrity_score, structural_integrity_score, consistency_score, cross_repo_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                collection_integrity_score, structural_integrity_score,
                consistency_score, cross_repo_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("architecture", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_architecture_sessions(&self, limit: usize) -> Result<Vec<ArchitectureSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.session_id, a.score, a.previous_score, a.git_revision, a.created_at,
                    a.engineering_readiness, a.collection_integrity_score,
                    a.structural_integrity_score, a.consistency_score, a.cross_repo_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'architecture' AND f.report_id = a.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'architecture' AND f.report_id = a.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'architecture' AND f.report_id = a.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM architecture_reports a ORDER BY a.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ArchitectureSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                collection_integrity_score: row.get(7)?,
                structural_integrity_score: row.get(8)?,
                consistency_score: row.get(9)?,
                cross_repo_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_architecture_report_with_findings(&self, report_id: i64) -> Result<Option<ArchitectureReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, collection_integrity_score,
                    structural_integrity_score, consistency_score, cross_repo_score,
                    doc_scores, validation_scores, finding_counts
             FROM architecture_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("architecture", report_id)?;
        Ok(Some(ArchitectureReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            collection_integrity_score: row.get(7)?,
            structural_integrity_score: row.get(8)?,
            consistency_score: row.get(9)?,
            cross_repo_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("architecture", report_id)?,
        }))
    }

    // ── Vision ───────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_vision_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        vision_content_score: Option<f64>,
        tech_independence_score: Option<f64>,
        traceability_consistency_score: Option<f64>,
        doc_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO vision_reports (session_id, score, git_revision, previous_score, engineering_readiness, vision_content_score, tech_independence_score, traceability_consistency_score, doc_quality_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                vision_content_score, tech_independence_score,
                traceability_consistency_score, doc_quality_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("vision", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_vision_sessions(&self, limit: usize) -> Result<Vec<VisionSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT v.id, v.session_id, v.score, v.previous_score, v.git_revision, v.created_at,
                    v.engineering_readiness, v.vision_content_score,
                    v.tech_independence_score, v.traceability_consistency_score, v.doc_quality_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'vision' AND f.report_id = v.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'vision' AND f.report_id = v.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'vision' AND f.report_id = v.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM vision_reports v ORDER BY v.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(VisionSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                vision_content_score: row.get(7)?,
                tech_independence_score: row.get(8)?,
                traceability_consistency_score: row.get(9)?,
                doc_quality_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_vision_report_with_findings(&self, report_id: i64) -> Result<Option<VisionReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, vision_content_score,
                    tech_independence_score, traceability_consistency_score, doc_quality_score,
                    doc_scores, validation_scores, finding_counts
             FROM vision_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("vision", report_id)?;
        Ok(Some(VisionReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            vision_content_score: row.get(7)?,
            tech_independence_score: row.get(8)?,
            traceability_consistency_score: row.get(9)?,
            doc_quality_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("vision", report_id)?,
        }))
    }

    // ── Design ───────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_design_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        design_system_score: Option<f64>,
        doc_quality_score: Option<f64>,
        design_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO design_reports (session_id, score, git_revision, previous_score, engineering_readiness, design_system_score, doc_quality_score, design_quality_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                design_system_score, doc_quality_score, design_quality_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("design", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_design_sessions(&self, limit: usize) -> Result<Vec<DesignSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.session_id, d.score, d.previous_score, d.git_revision, d.created_at,
                    d.engineering_readiness, d.design_system_score,
                    d.doc_quality_score, d.design_quality_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'design' AND f.report_id = d.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'design' AND f.report_id = d.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'design' AND f.report_id = d.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM design_reports d ORDER BY d.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(DesignSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                design_system_score: row.get(7)?,
                doc_quality_score: row.get(8)?,
                design_quality_score: row.get(9)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(10)? as usize,
                    warnings: row.get::<_, i64>(11)? as usize,
                    suggestions: row.get::<_, i64>(12)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_design_report_with_findings(&self, report_id: i64) -> Result<Option<DesignReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, design_system_score,
                    doc_quality_score, design_quality_score,
                    doc_scores, validation_scores, finding_counts
             FROM design_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("design", report_id)?;
        Ok(Some(DesignReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            design_system_score: row.get(7)?,
            doc_quality_score: row.get(8)?,
            design_quality_score: row.get(9)?,
            doc_scores: row.get(10)?,
            validation_scores: row.get(11)?,
            finding_counts: row.get(12)?,
            findings,
            recommendations: self.query_recommendations("design", report_id)?,
        }))
    }

    // ── README ───────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_readme_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        repo_introduction_score: Option<f64>,
        doc_navigation_score: Option<f64>,
        doc_quality_score: Option<f64>,
        maintainability_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO readme_reports (session_id, score, git_revision, previous_score, engineering_readiness, repo_introduction_score, doc_navigation_score, doc_quality_score, maintainability_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                repo_introduction_score, doc_navigation_score, doc_quality_score, maintainability_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("readme", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_readme_sessions(&self, limit: usize) -> Result<Vec<ReadmeSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.session_id, r.score, r.previous_score, r.git_revision, r.created_at,
                    r.engineering_readiness, r.repo_introduction_score,
                    r.doc_navigation_score, r.doc_quality_score, r.maintainability_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'readme' AND f.report_id = r.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'readme' AND f.report_id = r.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'readme' AND f.report_id = r.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM readme_reports r ORDER BY r.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ReadmeSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                repo_introduction_score: row.get(7)?,
                doc_navigation_score: row.get(8)?,
                doc_quality_score: row.get(9)?,
                maintainability_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_readme_report_with_findings(&self, report_id: i64) -> Result<Option<ReadmeReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, repo_introduction_score,
                    doc_navigation_score, doc_quality_score, maintainability_score,
                    doc_scores, validation_scores, finding_counts
             FROM readme_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("readme", report_id)?;
        Ok(Some(ReadmeReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            repo_introduction_score: row.get(7)?,
            doc_navigation_score: row.get(8)?,
            doc_quality_score: row.get(9)?,
            maintainability_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("readme", report_id)?,
        }))
    }

    // ── Help ─────────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_help_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        coverage_score: Option<f64>,
        navigation_score: Option<f64>,
        quality_score: Option<f64>,
        accuracy_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO help_reports (session_id, score, git_revision, previous_score, engineering_readiness, coverage_score, navigation_score, quality_score, accuracy_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                coverage_score, navigation_score, quality_score, accuracy_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("help", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_help_sessions(&self, limit: usize) -> Result<Vec<HelpSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.session_id, r.score, r.previous_score, r.git_revision, r.created_at,
                    r.engineering_readiness, r.coverage_score,
                    r.navigation_score, r.quality_score, r.accuracy_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'help' AND f.report_id = r.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'help' AND f.report_id = r.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'help' AND f.report_id = r.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM help_reports r ORDER BY r.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HelpSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                coverage_score: row.get(7)?,
                navigation_score: row.get(8)?,
                quality_score: row.get(9)?,
                accuracy_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_help_report_with_findings(&self, report_id: i64) -> Result<Option<HelpReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, coverage_score,
                    navigation_score, quality_score, accuracy_score,
                    doc_scores, validation_scores, finding_counts
             FROM help_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("help", report_id)?;
        Ok(Some(HelpReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            coverage_score: row.get(7)?,
            navigation_score: row.get(8)?,
            quality_score: row.get(9)?,
            accuracy_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("help", report_id)?,
        }))
    }

    // ── Prototype ────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_prototype_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        product_validation_score: Option<f64>,
        runtime_validation_score: Option<f64>,
        engineering_validation_score: Option<f64>,
        validation_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO prototype_reports (session_id, score, git_revision, previous_score, engineering_readiness, product_validation_score, runtime_validation_score, engineering_validation_score, validation_quality_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                product_validation_score, runtime_validation_score,
                engineering_validation_score, validation_quality_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("prototype", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_prototype_sessions(&self, limit: usize) -> Result<Vec<PrototypeSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.session_id, p.score, p.previous_score, p.git_revision, p.created_at,
                    p.engineering_readiness, p.product_validation_score,
                    p.runtime_validation_score, p.engineering_validation_score, p.validation_quality_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'prototype' AND f.report_id = p.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'prototype' AND f.report_id = p.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'prototype' AND f.report_id = p.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM prototype_reports p ORDER BY p.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(PrototypeSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                product_validation_score: row.get(7)?,
                runtime_validation_score: row.get(8)?,
                engineering_validation_score: row.get(9)?,
                validation_quality_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_prototype_report_with_findings(&self, report_id: i64) -> Result<Option<PrototypeReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, product_validation_score,
                    runtime_validation_score, engineering_validation_score, validation_quality_score,
                    doc_scores, validation_scores, finding_counts
             FROM prototype_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("prototype", report_id)?;
        Ok(Some(PrototypeReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            product_validation_score: row.get(7)?,
            runtime_validation_score: row.get(8)?,
            engineering_validation_score: row.get(9)?,
            validation_quality_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("prototype", report_id)?,
        }))
    }

    // ── External Context ─────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_external_context_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        document_quality_score: Option<f64>,
        content_completeness_score: Option<f64>,
        documentation_integrity_score: Option<f64>,
        collection_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO external_context_reports (session_id, score, git_revision, previous_score, engineering_readiness, document_quality_score, content_completeness_score, documentation_integrity_score, collection_quality_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                document_quality_score, content_completeness_score,
                documentation_integrity_score, collection_quality_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("external-context", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_external_context_sessions(&self, limit: usize) -> Result<Vec<ExternalContextSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.document_quality_score,
                    e.content_completeness_score, e.documentation_integrity_score, e.collection_quality_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM external_context_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ExternalContextSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                document_quality_score: row.get(7)?,
                content_completeness_score: row.get(8)?,
                documentation_integrity_score: row.get(9)?,
                collection_quality_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_external_context_report_with_findings(&self, report_id: i64) -> Result<Option<ExternalContextReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, document_quality_score,
                    content_completeness_score, documentation_integrity_score, collection_quality_score,
                    doc_scores, validation_scores, finding_counts
             FROM external_context_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("external-context", report_id)?;
        Ok(Some(ExternalContextReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            document_quality_score: row.get(7)?,
            content_completeness_score: row.get(8)?,
            documentation_integrity_score: row.get(9)?,
            collection_quality_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("external-context", report_id)?,
        }))
    }

    // ── Engineering ───────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_engineering_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        engineering_coverage_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        traceability_consistency_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO engineering_reports (session_id, score, git_revision, previous_score, engineering_readiness, engineering_coverage_score, documentation_quality_score, traceability_consistency_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                engineering_coverage_score, documentation_quality_score,
                traceability_consistency_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("engineering", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_engineering_sessions(&self, limit: usize) -> Result<Vec<EngineeringSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.engineering_coverage_score,
                    e.documentation_quality_score, e.traceability_consistency_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'engineering' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'engineering' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'engineering' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM engineering_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(EngineeringSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                engineering_coverage_score: row.get(7)?,
                documentation_quality_score: row.get(8)?,
                traceability_consistency_score: row.get(9)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(10)? as usize,
                    warnings: row.get::<_, i64>(11)? as usize,
                    suggestions: row.get::<_, i64>(12)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_engineering_report_with_findings(&self, report_id: i64) -> Result<Option<EngineeringReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, engineering_coverage_score,
                    documentation_quality_score, traceability_consistency_score,
                    doc_scores, validation_scores, finding_counts
             FROM engineering_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("engineering", report_id)?;
        Ok(Some(EngineeringReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            engineering_coverage_score: row.get(7)?,
            documentation_quality_score: row.get(8)?,
            traceability_consistency_score: row.get(9)?,
            doc_scores: row.get(10)?,
            validation_scores: row.get(11)?,
            finding_counts: row.get(12)?,
            findings,
            recommendations: self.query_recommendations("engineering", report_id)?,
        }))
    }

    // ── Feature ───────────────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_feature_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_definition_score: Option<f64>,
        product_definition_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        product_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feature_reports (session_id, score, git_revision, previous_score, engineering_readiness, feature_definition_score, product_definition_score, documentation_quality_score, product_readiness_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                feature_definition_score, product_definition_score,
                documentation_quality_score, product_readiness_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("feature", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_feature_sessions(&self, limit: usize) -> Result<Vec<FeatureSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.feature_definition_score,
                    e.product_definition_score, e.documentation_quality_score, e.product_readiness_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM feature_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(FeatureSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                feature_definition_score: row.get(7)?,
                product_definition_score: row.get(8)?,
                documentation_quality_score: row.get(9)?,
                product_readiness_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_feature_report_with_findings(&self, report_id: i64) -> Result<Option<FeatureReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, feature_definition_score,
                    product_definition_score, documentation_quality_score, product_readiness_score,
                    doc_scores, validation_scores, finding_counts
             FROM feature_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("feature", report_id)?;
        Ok(Some(FeatureReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            feature_definition_score: row.get(7)?,
            product_definition_score: row.get(8)?,
            documentation_quality_score: row.get(9)?,
            product_readiness_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("feature", report_id)?,
        }))
    }

    // ── Feature Technical ───────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_feature_technical_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_mapping_score: Option<f64>,
        technical_realization_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        implementation_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feature_technical_reports (session_id, score, git_revision, previous_score, engineering_readiness, feature_mapping_score, technical_realization_score, documentation_quality_score, implementation_readiness_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                feature_mapping_score, technical_realization_score,
                documentation_quality_score, implementation_readiness_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("feature-technical", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_feature_technical_sessions(&self, limit: usize) -> Result<Vec<FeatureTechnicalSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.feature_mapping_score,
                    e.technical_realization_score, e.documentation_quality_score, e.implementation_readiness_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-technical' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-technical' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-technical' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM feature_technical_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(FeatureTechnicalSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                feature_mapping_score: row.get(7)?,
                technical_realization_score: row.get(8)?,
                documentation_quality_score: row.get(9)?,
                implementation_readiness_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_feature_technical_report_with_findings(&self, report_id: i64) -> Result<Option<FeatureTechnicalReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, feature_mapping_score,
                    technical_realization_score, documentation_quality_score, implementation_readiness_score,
                    doc_scores, validation_scores, finding_counts
             FROM feature_technical_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("feature-technical", report_id)?;
        Ok(Some(FeatureTechnicalReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            feature_mapping_score: row.get(7)?,
            technical_realization_score: row.get(8)?,
            documentation_quality_score: row.get(9)?,
            implementation_readiness_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("feature-technical", report_id)?,
        }))
    }

    // ── Feature Design ───────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_feature_design_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        feature_mapping_score: Option<f64>,
        user_experience_score: Option<f64>,
        documentation_quality_score: Option<f64>,
        design_readiness_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feature_design_reports (session_id, score, git_revision, previous_score, engineering_readiness, feature_mapping_score, user_experience_score, documentation_quality_score, design_readiness_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                feature_mapping_score, user_experience_score,
                documentation_quality_score, design_readiness_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("feature-design", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_feature_design_sessions(&self, limit: usize) -> Result<Vec<FeatureDesignSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.feature_mapping_score,
                    e.user_experience_score, e.documentation_quality_score, e.design_readiness_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-design' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-design' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'feature-design' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM feature_design_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(FeatureDesignSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                feature_mapping_score: row.get(7)?,
                user_experience_score: row.get(8)?,
                documentation_quality_score: row.get(9)?,
                design_readiness_score: row.get(10)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(11)? as usize,
                    warnings: row.get::<_, i64>(12)? as usize,
                    suggestions: row.get::<_, i64>(13)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_feature_design_report_with_findings(&self, report_id: i64) -> Result<Option<FeatureDesignReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, feature_mapping_score,
                    user_experience_score, documentation_quality_score, design_readiness_score,
                    doc_scores, validation_scores, finding_counts
             FROM feature_design_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("feature-design", report_id)?;
        Ok(Some(FeatureDesignReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            feature_mapping_score: row.get(7)?,
            user_experience_score: row.get(8)?,
            documentation_quality_score: row.get(9)?,
            design_readiness_score: row.get(10)?,
            doc_scores: row.get(11)?,
            validation_scores: row.get(12)?,
            finding_counts: row.get(13)?,
            findings,
            recommendations: self.query_recommendations("feature-design", report_id)?,
        }))
    }

    // ── Deterministic Runtime ────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_deterministic_runtime_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        runtime_model_score: Option<f64>,
        engineering_principles_score: Option<f64>,
        runtime_integrity_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO deterministic_runtime_reports (session_id, score, git_revision, previous_score, engineering_readiness, runtime_model_score, engineering_principles_score, runtime_integrity_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                runtime_model_score, engineering_principles_score, runtime_integrity_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("deterministic-runtime", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_deterministic_runtime_sessions(&self, limit: usize) -> Result<Vec<DeterministicRuntimeSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.runtime_model_score,
                    e.engineering_principles_score, e.runtime_integrity_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'deterministic-runtime' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'deterministic-runtime' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'deterministic-runtime' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM deterministic_runtime_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(DeterministicRuntimeSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                runtime_model_score: row.get(7)?,
                engineering_principles_score: row.get(8)?,
                runtime_integrity_score: row.get(9)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(10)? as usize,
                    warnings: row.get::<_, i64>(11)? as usize,
                    suggestions: row.get::<_, i64>(12)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_deterministic_runtime_report_with_findings(&self, report_id: i64) -> Result<Option<DeterministicRuntimeReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, runtime_model_score,
                    engineering_principles_score, runtime_integrity_score,
                    doc_scores, validation_scores, finding_counts
             FROM deterministic_runtime_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("deterministic-runtime", report_id)?;
        Ok(Some(DeterministicRuntimeReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            runtime_model_score: row.get(7)?,
            engineering_principles_score: row.get(8)?,
            runtime_integrity_score: row.get(9)?,
            doc_scores: row.get(10)?,
            validation_scores: row.get(11)?,
            finding_counts: row.get(12)?,
            findings,
            recommendations: self.query_recommendations("deterministic-runtime", report_id)?,
        }))
    }

    // ── External Context Ownership ───────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_external_context_ownership_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        dependency_coverage_score: Option<f64>,
        documentation_integration_score: Option<f64>,
        consistency_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO external_context_ownership_reports (session_id, score, git_revision, previous_score, engineering_readiness, dependency_coverage_score, documentation_integration_score, consistency_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                dependency_coverage_score, documentation_integration_score, consistency_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("external-context-ownership", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_external_context_ownership_sessions(&self, limit: usize) -> Result<Vec<ExternalContextOwnershipSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.dependency_coverage_score,
                    e.documentation_integration_score, e.consistency_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context-ownership' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context-ownership' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'external-context-ownership' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM external_context_ownership_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ExternalContextOwnershipSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                dependency_coverage_score: row.get(7)?,
                documentation_integration_score: row.get(8)?,
                consistency_score: row.get(9)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(10)? as usize,
                    warnings: row.get::<_, i64>(11)? as usize,
                    suggestions: row.get::<_, i64>(12)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_external_context_ownership_report_with_findings(&self, report_id: i64) -> Result<Option<ExternalContextOwnershipReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, dependency_coverage_score,
                    documentation_integration_score, consistency_score,
                    doc_scores, validation_scores, finding_counts
             FROM external_context_ownership_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("external-context-ownership", report_id)?;
        Ok(Some(ExternalContextOwnershipReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            dependency_coverage_score: row.get(7)?,
            documentation_integration_score: row.get(8)?,
            consistency_score: row.get(9)?,
            doc_scores: row.get(10)?,
            validation_scores: row.get(11)?,
            finding_counts: row.get(12)?,
            findings,
            recommendations: self.query_recommendations("external-context-ownership", report_id)?,
        }))
    }

    // ── Documentation Structure ───────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_documentation_structure_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        structural_integrity_score: Option<f64>,
        mapping_consistency_score: Option<f64>,
        atomicity_enforcement_score: Option<f64>,
        cross_document_alignment_score: Option<f64>,
        name_preservation_score: Option<f64>,
        implementation_traceability_score: Option<f64>,
        generation_compliance_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO documentation_structure_reports (session_id, score, git_revision, previous_score, engineering_readiness, structural_integrity_score, mapping_consistency_score, atomicity_enforcement_score, cross_document_alignment_score, name_preservation_score, implementation_traceability_score, generation_compliance_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                structural_integrity_score, mapping_consistency_score, atomicity_enforcement_score,
                cross_document_alignment_score, name_preservation_score,
                implementation_traceability_score, generation_compliance_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("documentation-structure", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_documentation_structure_sessions(&self, limit: usize) -> Result<Vec<DocumentationStructureSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.session_id, d.score, d.previous_score, d.git_revision, d.created_at,
                    d.engineering_readiness, d.structural_integrity_score, d.mapping_consistency_score,
                    d.atomicity_enforcement_score, d.cross_document_alignment_score,
                    d.name_preservation_score, d.implementation_traceability_score,
                    d.generation_compliance_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'documentation-structure' AND f.report_id = d.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'documentation-structure' AND f.report_id = d.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'documentation-structure' AND f.report_id = d.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM documentation_structure_reports d ORDER BY d.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(DocumentationStructureSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                structural_integrity_score: row.get(7)?,
                mapping_consistency_score: row.get(8)?,
                atomicity_enforcement_score: row.get(9)?,
                cross_document_alignment_score: row.get(10)?,
                name_preservation_score: row.get(11)?,
                implementation_traceability_score: row.get(12)?,
                generation_compliance_score: row.get(13)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(14)? as usize,
                    warnings: row.get::<_, i64>(15)? as usize,
                    suggestions: row.get::<_, i64>(16)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_documentation_structure_report_with_findings(&self, report_id: i64) -> Result<Option<DocumentationStructureReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, structural_integrity_score, mapping_consistency_score,
                    atomicity_enforcement_score, cross_document_alignment_score,
                    name_preservation_score, implementation_traceability_score,
                    generation_compliance_score, doc_scores, validation_scores, finding_counts
             FROM documentation_structure_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("documentation-structure", report_id)?;
        Ok(Some(DocumentationStructureReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            structural_integrity_score: row.get(7)?,
            mapping_consistency_score: row.get(8)?,
            atomicity_enforcement_score: row.get(9)?,
            cross_document_alignment_score: row.get(10)?,
            name_preservation_score: row.get(11)?,
            implementation_traceability_score: row.get(12)?,
            generation_compliance_score: row.get(13)?,
            doc_scores: row.get(14)?,
            validation_scores: row.get(15)?,
            finding_counts: row.get(16)?,
            findings,
            recommendations: self.query_recommendations("documentation-structure", report_id)?,
        }))
    }

    // ── Implementation ───────────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    pub fn insert_implementation_report(
        &self,
        score: f64,
        session_id: &str,
        git_revision: Option<&str>,
        previous_score: Option<f64>,
        engineering_readiness: &str,
        architectural_conformance_score: Option<f64>,
        feature_conformance_score: Option<f64>,
        engineering_conformance_score: Option<f64>,
        documentation_integrity_score: Option<f64>,
        implementation_quality_score: Option<f64>,
        doc_scores: Option<&str>,
        validation_scores: Option<&str>,
        finding_counts: Option<&str>,
        findings: &[schemas::audit::AuditFinding],
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO implementation_reports (session_id, score, git_revision, previous_score, engineering_readiness, architectural_conformance_score, feature_conformance_score, engineering_conformance_score, documentation_integrity_score, implementation_quality_score, doc_scores, validation_scores, finding_counts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                session_id, score, git_revision, previous_score, engineering_readiness,
                architectural_conformance_score, feature_conformance_score,
                engineering_conformance_score, documentation_integrity_score,
                implementation_quality_score,
                doc_scores, validation_scores, finding_counts,
            ],
        )?;
        let report_id = self.conn.last_insert_rowid();
        if !findings.is_empty() {
            self.insert_report_findings("implementation", report_id, findings)?;
        }
        Ok(report_id)
    }

    pub fn query_implementation_sessions(&self, limit: usize) -> Result<Vec<ImplementationSessionInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.session_id, e.score, e.previous_score, e.git_revision, e.created_at,
                    e.engineering_readiness, e.architectural_conformance_score,
                    e.feature_conformance_score, e.engineering_conformance_score,
                    e.documentation_integrity_score, e.implementation_quality_score,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'implementation' AND f.report_id = e.id AND f.severity = 'error'), 0) as err_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'implementation' AND f.report_id = e.id AND f.severity = 'warning'), 0) as warn_count,
                    COALESCE((SELECT COUNT(*) FROM report_findings f WHERE f.report_type = 'implementation' AND f.report_id = e.id AND f.severity = 'suggestion'), 0) as sug_count
             FROM implementation_reports e ORDER BY e.id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ImplementationSessionInfo {
                id: row.get(0)?,
                session_id: row.get(1)?,
                score: row.get(2)?,
                previous_score: row.get(3)?,
                git_revision: row.get(4)?,
                created_at: row.get(5)?,
                engineering_readiness: row.get(6)?,
                architectural_conformance_score: row.get(7)?,
                feature_conformance_score: row.get(8)?,
                engineering_conformance_score: row.get(9)?,
                documentation_integrity_score: row.get(10)?,
                implementation_quality_score: row.get(11)?,
                finding_counts: FindingCounts {
                    errors: row.get::<_, i64>(12)? as usize,
                    warnings: row.get::<_, i64>(13)? as usize,
                    suggestions: row.get::<_, i64>(14)? as usize,
                },
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    pub fn get_implementation_report_with_findings(&self, report_id: i64) -> Result<Option<ImplementationReportWithFindings>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, score, previous_score, git_revision, created_at,
                    engineering_readiness, architectural_conformance_score,
                    feature_conformance_score, engineering_conformance_score,
                    documentation_integrity_score, implementation_quality_score,
                    doc_scores, validation_scores, finding_counts
             FROM implementation_reports WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![report_id])?;
        let Some(row) = rows.next()? else { return Ok(None); };
        let findings = self.query_findings("implementation", report_id)?;
        Ok(Some(ImplementationReportWithFindings {
            id: row.get(0)?,
            session_id: row.get(1)?,
            score: row.get(2)?,
            previous_score: row.get(3)?,
            git_revision: row.get(4)?,
            created_at: row.get(5)?,
            engineering_readiness: row.get(6)?,
            architectural_conformance_score: row.get(7)?,
            feature_conformance_score: row.get(8)?,
            engineering_conformance_score: row.get(9)?,
            documentation_integrity_score: row.get(10)?,
            implementation_quality_score: row.get(11)?,
            doc_scores: row.get(12)?,
            validation_scores: row.get(13)?,
            finding_counts: row.get(14)?,
            findings,
            recommendations: self.query_recommendations("implementation", report_id)?,
        }))
    }

    // ── Recommendations (Phase 9) ───────────────────────────────────────

    pub fn insert_recommendations(&self, report_type: &str, report_id: i64, recommendations: &[ReportRecommendation]) -> Result<()> {
        for r in recommendations {
            self.conn.execute(
                "INSERT INTO report_recommendations (report_type, report_id, priority, category, description, file_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![report_type, report_id, r.priority, r.category, r.description, r.file_path],
            )?;
        }
        Ok(())
    }

    pub fn query_recommendations(&self, report_type: &str, report_id: i64) -> Result<Vec<ReportRecommendation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, priority, category, description, file_path
             FROM report_recommendations WHERE report_type = ?1 AND report_id = ?2
             ORDER BY
                CASE priority
                    WHEN 'P1' THEN 1
                    WHEN 'P2' THEN 2
                    WHEN 'P3' THEN 3
                    ELSE 4
                END",
        )?;
        let rows = stmt.query_map(params![report_type, report_id], |row| {
            Ok(ReportRecommendation {
                id: row.get(0)?,
                priority: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                file_path: row.get(4)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Shared Report Operations (Phase 8) ──────────────────────────────

    pub fn insert_report_findings(&self, report_type: &str, report_id: i64, findings: &[schemas::audit::AuditFinding]) -> Result<()> {
        for f in findings {
            self.conn.execute(
                "INSERT INTO report_findings (report_type, report_id, check_id, severity, message, location)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![report_type, report_id, f.check_id, f.severity.to_string(), f.message, f.location],
            )?;
            // Evidence was previously dropped here even though AuditFinding.evidence
            // carries it (semantic providers set it; deterministic providers leave it
            // None). Persist it when present so the read path / report templates can
            // show it — see StoredFinding.evidence_excerpt / query_findings below.
            if let Some(evidence) = &f.evidence {
                let finding_id = self.conn.last_insert_rowid();
                let source = format!("section_id={}, paragraph_index={}", evidence.section_id, evidence.paragraph_index);
                self.insert_report_evidence(
                    report_type,
                    report_id,
                    Some(finding_id),
                    "excerpt",
                    Some(&evidence.excerpt),
                    Some(&source),
                )?;
            }
        }
        Ok(())
    }

    pub fn query_findings(&self, report_type: &str, report_id: i64) -> Result<Vec<StoredFinding>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.check_id, f.severity, f.message, f.location, f.status,
                    e.value, e.source
             FROM report_findings f
             LEFT JOIN report_evidence e
                    ON e.finding_id = f.id AND e.key = 'excerpt'
             WHERE f.report_type = ?1 AND f.report_id = ?2 ORDER BY f.id",
        )?;
        let findings = stmt.query_map(params![report_type, report_id], |row| {
            Ok(StoredFinding {
                id: row.get(0)?,
                check_id: row.get(1)?,
                severity: row.get(2)?,
                message: row.get(3)?,
                location: row.get(4)?,
                status: row.get(5)?,
                evidence_excerpt: row.get(6)?,
                evidence_source: row.get(7)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        Ok(findings)
    }

    pub fn update_finding_status_by_id(&self, finding_id: i64, status: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE report_findings SET status = ?1 WHERE id = ?2",
            params![status, finding_id],
        )?;
        Ok(())
    }

    /// For each semantic section type, how many documents in `domain` contain
    /// it versus the total document count in that domain. Used to check a
    /// documentation collection against a standard's Required Sections table
    /// (e.g. `docs/raw/documentation-standards/04-architecture-standards.md`) without re-deriving that
    /// list here — the caller passes the section types to check.
    pub fn count_section_type_coverage(
        &self,
        domain: &str,
        semantic_types: &[&str],
    ) -> Result<Vec<(String, usize, usize)>> {
        let total_docs: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE standard = ?1",
            params![domain],
            |row| row.get(0),
        )?;

        let mut out = Vec::with_capacity(semantic_types.len());
        for semantic_type in semantic_types {
            let with_type: usize = self.conn.query_row(
                "SELECT COUNT(DISTINCT d.id) FROM documents d
                 JOIN document_sections ds ON ds.document_id = d.id
                 WHERE d.standard = ?1 AND ds.semantic_type = ?2",
                params![domain, semantic_type],
                |row| row.get(0),
            )?;
            out.push((semantic_type.to_string(), with_type, total_docs));
        }
        Ok(out)
    }

    pub fn insert_report_evidence(&self, report_type: &str, report_id: i64, finding_id: Option<i64>, key: &str, value: Option<&str>, source: Option<&str>) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO report_evidence (report_type, report_id, finding_id, key, value, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![report_type, report_id, finding_id, key, value, source],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_report_summary(&self, report_type: &str, report_id: i64, summary_text: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO report_summaries (report_type, report_id, summary_text)
             VALUES (?1, ?2, ?3)",
            params![report_type, report_id, summary_text],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_report_improvement(&self, report_type: &str, report_id: i64, category: &str, suggestion: &str, priority: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO report_improvements (report_type, report_id, category, suggestion, priority)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![report_type, report_id, category, suggestion, priority],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    // ── Fix Pipeline (Phase 1) ────────────────────────────────────────────────

    pub fn insert_fix_session(&self, session: &FixSession) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO fix_sessions (report_id, report_type, criterion_id, finding_json, domain, plan_type, target_file, attempt_count, max_attempts, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                session.report_id,
                session.report_type,
                session.criterion_id,
                session.finding_json,
                session.domain,
                session.plan_type.as_str(),
                session.target_file,
                session.attempt_count,
                session.max_attempts,
                session.status.as_str(),
                session.created_at.as_deref().unwrap_or(""),
                session.updated_at.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_fix_session(&self, session: &FixSession) -> Result<()> {
        self.conn.execute(
            "UPDATE fix_sessions SET
                plan_type = ?1, attempt_count = ?2, status = ?3, updated_at = ?4
             WHERE id = ?5",
            params![
                session.plan_type.as_str(),
                session.attempt_count,
                session.status.as_str(),
                chrono_now(),
                session.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_fix_session(&self, id: i64) -> Result<Option<FixSession>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, report_id, report_type, criterion_id, finding_json, domain, plan_type, target_file, attempt_count, max_attempts, status, created_at, updated_at
             FROM fix_sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(FixSession {
                id: Some(row.get(0)?),
                report_id: row.get(1)?,
                report_type: row.get(2)?,
                criterion_id: row.get(3)?,
                finding_json: row.get(4)?,
                domain: row.get(5)?,
                plan_type: PlanType::from_str(&row.get::<_, String>(6)?)
                    .unwrap_or(PlanType::Documentation),
                target_file: row.get(7)?,
                attempt_count: row.get(8)?,
                max_attempts: row.get(9)?,
                status: SessionStatus::from_str(&row.get::<_, String>(10)?)
                    .unwrap_or(SessionStatus::InProgress),
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn insert_fix_plan(&self, plan: &FixPlan) -> Result<i64> {
        let steps_json = serde_json::to_string(&plan.steps)?;
        let prereqs_json = serde_json::to_string(&plan.prerequisites)?;
        self.conn.execute(
            "INSERT INTO fix_plans (session_id, report_id, criterion_id, domain, plan_type, title, summary, prerequisites, steps, rollback_instructions, expected_checks, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                plan.session_id,
                plan.report_id,
                plan.criterion_id,
                plan.domain,
                plan.plan_type.as_str(),
                plan.title,
                plan.summary,
                prereqs_json,
                steps_json,
                plan.rollback_instructions,
                serde_json::to_string(&plan.expected_checks)?,
                plan.status.as_str(),
                plan.created_at.as_deref().unwrap_or(""),
                plan.updated_at.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_fix_plan(&self, id: i64) -> Result<Option<FixPlan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, report_id, criterion_id, domain, plan_type, title, summary, prerequisites, steps, rollback_instructions, expected_checks, status, created_at, updated_at
             FROM fix_plans WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let steps_str: String = row.get(9)?;
            let prereqs_str: String = row.get(8)?;
            let ec_str: String = row.get(11)?;
            Ok(Some(FixPlan {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                report_id: row.get(2)?,
                criterion_id: row.get(3)?,
                domain: row.get(4)?,
                plan_type: PlanType::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or(PlanType::Documentation),
                title: row.get(6)?,
                summary: row.get(7)?,
                prerequisites: serde_json::from_str(&prereqs_str).unwrap_or_default(),
                steps: serde_json::from_str(&steps_str).unwrap_or_default(),
                rollback_instructions: row.get(10)?,
                expected_checks: serde_json::from_str(&ec_str).unwrap_or_default(),
                status: FixPlanStatus::from_str(&row.get::<_, String>(12)?)
                    .unwrap_or(FixPlanStatus::Draft),
                created_at: Some(row.get(13)?),
                updated_at: Some(row.get(14)?),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_fix_plan_status(&self, plan_id: i64, status: &FixPlanStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE fix_plans SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), chrono_now(), plan_id],
        )?;
        Ok(())
    }

    pub fn insert_fix_plan_step(&self, step: &PlanStep) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO fix_plan_steps (plan_id, step_order, action, target, rationale, detail, verification, rollback, status, verified_at, score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                step.plan_id,
                step.step_order as i64,
                step.action,
                step.target,
                step.rationale,
                step.detail,
                step.verification,
                step.rollback,
                step.status.as_str(),
                step.verified_at,
                step.score,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_fix_plan_step(&self, step: &PlanStep) -> Result<()> {
        self.conn.execute(
            "UPDATE fix_plan_steps SET status = ?1, verified_at = ?2, score = ?3 WHERE id = ?4",
            params![
                step.status.as_str(),
                step.verified_at,
                step.score,
                step.id,
            ],
        )?;
        Ok(())
    }

    pub fn get_fix_plan_steps(&self, plan_id: i64) -> Result<Vec<PlanStep>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, step_order, action, target, rationale, detail, verification, rollback, status, verified_at, score
             FROM fix_plan_steps WHERE plan_id = ?1 ORDER BY step_order",
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(PlanStep {
                id: Some(row.get(0)?),
                plan_id: Some(row.get(1)?),
                step_order: row.get::<_, i64>(2)? as usize,
                action: row.get(3)?,
                target: row.get(4)?,
                rationale: row.get(5)?,
                detail: row.get(6)?,
                verification: row.get(7)?,
                rollback: row.get(8)?,
                status: FixStepStatus::from_str(&row.get::<_, String>(9)?)
                    .unwrap_or(FixStepStatus::Pending),
                verified_at: row.get(10)?,
                score: row.get(11)?,
            })
        })?;
        let mut steps = Vec::new();
        for row in rows {
            steps.push(row?);
        }
        Ok(steps)
    }

    pub fn insert_fix_attempt(&self, attempt: &FixAttempt) -> Result<i64> {
        let check_scores_json = attempt.check_scores.as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default())
            .unwrap_or_default();
        self.conn.execute(
            "INSERT INTO fix_attempts (session_id, attempt, plan_id, plan_type, score, check_scores, passed, error_message, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                attempt.session_id,
                attempt.attempt,
                attempt.plan_id,
                attempt.plan_type.as_str(),
                attempt.score,
                check_scores_json,
                attempt.passed as i32,
                attempt.error_message,
                attempt.created_at.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_fix_attempts(&self, session_id: i64) -> Result<Vec<FixAttempt>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, attempt, plan_id, plan_type, score, check_scores, passed, error_message, created_at
             FROM fix_attempts WHERE session_id = ?1 ORDER BY attempt",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let cs_str: Option<String> = row.get(6).ok();
            let check_scores = cs_str
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default();
            Ok(FixAttempt {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                attempt: row.get(2)?,
                plan_id: row.get(3)?,
                plan_type: PlanType::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or(PlanType::Documentation),
                score: row.get(5)?,
                check_scores: Some(check_scores),
                passed: row.get::<_, i32>(7)? != 0,
                error_message: row.get(8)?,
                created_at: Some(row.get(9)?),
            })
        })?;
        let mut attempts = Vec::new();
        for row in rows {
            attempts.push(row?);
        }
        Ok(attempts)
    }

    /// List fix sessions ordered by creation time descending.
    pub fn query_fix_sessions(&self, limit: usize, offset: usize) -> Result<Vec<FixSession>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, report_id, report_type, criterion_id, finding_json, domain, plan_type, target_file, attempt_count, max_attempts, status, created_at, updated_at
             FROM fix_sessions ORDER BY id DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(FixSession {
                id: Some(row.get(0)?),
                report_id: row.get(1)?,
                report_type: row.get(2)?,
                criterion_id: row.get(3)?,
                finding_json: row.get(4)?,
                domain: row.get(5)?,
                plan_type: PlanType::from_str(&row.get::<_, String>(6)?)
                    .unwrap_or(PlanType::Documentation),
                target_file: row.get(7)?,
                attempt_count: row.get(8)?,
                max_attempts: row.get(9)?,
                status: SessionStatus::from_str(&row.get::<_, String>(10)?)
                    .unwrap_or(SessionStatus::InProgress),
                created_at: Some(row.get(11)?),
                updated_at: Some(row.get(12)?),
            })
        })?;
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }

    /// List fix plans for a given fix session (by fix_sessions.id).
    pub fn query_fix_plans_by_session(&self, fix_session_id: i64) -> Result<Vec<FixPlan>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT fp.id, fp.session_id, fp.report_id, fp.criterion_id, fp.domain, fp.plan_type, fp.title, fp.summary, fp.prerequisites, fp.steps, fp.rollback_instructions, fp.expected_checks, fp.status, fp.created_at, fp.updated_at
             FROM fix_plans fp
             JOIN fix_attempts fa ON fa.plan_id = fp.id
             WHERE fa.session_id = ?1
             ORDER BY fp.id DESC",
        )?;
        let rows = stmt.query_map(params![fix_session_id], |row| {
            let prereq_str: String = row.get(8)?;
            let steps_str: String = row.get(9)?;
            let checks_str: String = row.get(11)?;
            let prerequisites: Vec<String> = serde_json::from_str(&prereq_str).unwrap_or_default();
            let steps: Vec<PlanStep> = serde_json::from_str(&steps_str).unwrap_or_default();
            let expected_checks: Vec<String> = serde_json::from_str(&checks_str).unwrap_or_default();
            Ok(FixPlan {
                id: Some(row.get(0)?),
                session_id: row.get(1)?,
                report_id: row.get(2)?,
                criterion_id: row.get(3)?,
                domain: row.get(4)?,
                plan_type: PlanType::from_str(&row.get::<_, String>(5)?)
                    .unwrap_or(PlanType::Documentation),
                title: row.get(6)?,
                summary: row.get(7)?,
                prerequisites,
                steps,
                rollback_instructions: row.get(10)?,
                expected_checks,
                status: FixPlanStatus::from_str(&row.get::<_, String>(12)?)
                    .unwrap_or(FixPlanStatus::Draft),
                created_at: Some(row.get(13)?),
                updated_at: Some(row.get(14)?),
            })
        })?;
        let mut plans = Vec::new();
        for row in rows {
            plans.push(row?);
        }
        Ok(plans)
    }

    // ── Repository Metadata (Product Guide Phase 1.5) ─────────────────────

    pub fn upsert_repository_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO repository_metadata (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_repository_metadata(&self) -> Result<std::collections::HashMap<String, String>> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM repository_metadata")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (k, v) = row?;
            map.insert(k, v);
        }
        Ok(map)
    }

    // ── Project Plan CRUD (Phase 9 — Planner) ─────────────────────────────

    pub fn insert_plan(&self, plan: &schemas::ProjectPlan) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO project_plans (id, title, case_type, status, current_phase, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                plan.id,
                plan.title,
                plan.case_type.to_string(),
                plan.status.to_string(),
                plan.current_phase,
                plan.created_at,
                plan.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_plan(&self, id: &str) -> Result<Option<schemas::ProjectPlan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, case_type, status, current_phase, created_at, updated_at FROM project_plans WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(schemas::ProjectPlan {
                id: row.get(0)?,
                title: row.get(1)?,
                case_type: schemas::ProjectCase::from_str(&row.get::<_, String>(2)?)
                    .unwrap_or(schemas::ProjectCase::DocAudit),
                status: schemas::PlanStatus::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(schemas::PlanStatus::Active),
                current_phase: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_plans(&self) -> Result<Vec<schemas::ProjectPlan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, case_type, status, current_phase, created_at, updated_at FROM project_plans ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(schemas::ProjectPlan {
                id: row.get(0)?,
                title: row.get(1)?,
                case_type: schemas::ProjectCase::from_str(&row.get::<_, String>(2)?)
                    .unwrap_or(schemas::ProjectCase::DocAudit),
                status: schemas::PlanStatus::from_str(&row.get::<_, String>(3)?)
                    .unwrap_or(schemas::PlanStatus::Active),
                current_phase: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;
        let mut plans = Vec::new();
        for row in rows {
            plans.push(row?);
        }
        Ok(plans)
    }

    pub fn update_plan_status(&self, id: &str, status: &schemas::PlanStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE project_plans SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status.to_string(), id],
        )?;
        Ok(())
    }

    pub fn update_plan_current_phase(&self, id: &str, phase_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE project_plans SET current_phase = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![phase_id, id],
        )?;
        Ok(())
    }

    pub fn insert_phase(&self, phase: &schemas::ProjectPhase) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO project_phases (id, plan_id, phase_number, name, phase_type, domains, pipeline_ids, dependencies, status, started_at, completed_at, result_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                phase.id,
                phase.plan_id,
                phase.phase_number,
                phase.name,
                phase.phase_type.to_string(),
                serde_json::to_string(&phase.domains).unwrap_or_default(),
                serde_json::to_string(&phase.pipeline_ids).unwrap_or_default(),
                serde_json::to_string(&phase.dependencies).unwrap_or_default(),
                phase.status.to_string(),
                phase.started_at,
                phase.completed_at,
                phase.result_json,
            ],
        )?;
        Ok(())
    }

    pub fn get_phases(&self, plan_id: &str) -> Result<Vec<schemas::ProjectPhase>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, phase_number, name, phase_type, domains, pipeline_ids, dependencies, status, started_at, completed_at, result_json
             FROM project_phases WHERE plan_id = ?1 ORDER BY phase_number",
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            let domains_str: String = row.get(5)?;
            let pipelines_str: String = row.get(6)?;
            let deps_str: String = row.get(7)?;
            Ok(schemas::ProjectPhase {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                phase_number: row.get(2)?,
                name: row.get(3)?,
                phase_type: schemas::PhaseType::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or(schemas::PhaseType::Audit),
                domains: serde_json::from_str(&domains_str).unwrap_or_default(),
                pipeline_ids: serde_json::from_str(&pipelines_str).unwrap_or_default(),
                dependencies: serde_json::from_str(&deps_str).unwrap_or_default(),
                status: schemas::PhaseStatus::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or(schemas::PhaseStatus::Pending),
                started_at: row.get(9)?,
                completed_at: row.get(10)?,
                result_json: row.get(11)?,
            })
        })?;
        let mut phases = Vec::new();
        for row in rows {
            phases.push(row?);
        }
        Ok(phases)
    }

    /// Atomically transition a phase from `pending` to `in_progress`.
    /// Returns `false` (no-op) if the phase was not `pending` — guards
    /// against two concurrent `execute_phase` calls both starting the same
    /// phase.
    pub fn try_start_phase(&self, id: &str) -> Result<bool> {
        let started = chrono::Utc::now().to_rfc3339();
        let changed = self.conn.execute(
            "UPDATE project_phases SET status = 'in_progress', started_at = ?1 WHERE id = ?2 AND status = 'pending'",
            params![started, id],
        )?;
        Ok(changed > 0)
    }

    pub fn update_phase_status(&self, id: &str, status: &schemas::PhaseStatus) -> Result<()> {
        let (started, completed) = match status {
            schemas::PhaseStatus::InProgress => (Some(chrono::Utc::now().to_rfc3339()), None),
            schemas::PhaseStatus::Completed | schemas::PhaseStatus::Failed => (None, Some(chrono::Utc::now().to_rfc3339())),
            _ => (None, None),
        };
        self.conn.execute(
            "UPDATE project_phases SET status = ?1, started_at = COALESCE(?2, started_at), completed_at = COALESCE(?3, completed_at) WHERE id = ?4",
            params![status.to_string(), started, completed, id],
        )?;
        Ok(())
    }

    pub fn update_phase_result(&self, id: &str, result_json: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE project_phases SET result_json = ?1 WHERE id = ?2",
            params![result_json, id],
        )?;
        Ok(())
    }

    pub fn get_pending_phase(&self, plan_id: &str) -> Result<Option<schemas::ProjectPhase>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, phase_number, name, phase_type, domains, pipeline_ids, dependencies, status, started_at, completed_at, result_json
             FROM project_phases WHERE plan_id = ?1 AND status = 'pending' ORDER BY phase_number LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![plan_id], |row| {
            let domains_str: String = row.get(5)?;
            let pipelines_str: String = row.get(6)?;
            let deps_str: String = row.get(7)?;
            Ok(schemas::ProjectPhase {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                phase_number: row.get(2)?,
                name: row.get(3)?,
                phase_type: schemas::PhaseType::from_str(&row.get::<_, String>(4)?)
                    .unwrap_or(schemas::PhaseType::Audit),
                domains: serde_json::from_str(&domains_str).unwrap_or_default(),
                pipeline_ids: serde_json::from_str(&pipelines_str).unwrap_or_default(),
                dependencies: serde_json::from_str(&deps_str).unwrap_or_default(),
                status: schemas::PhaseStatus::from_str(&row.get::<_, String>(8)?)
                    .unwrap_or(schemas::PhaseStatus::Pending),
                started_at: row.get(9)?,
                completed_at: row.get(10)?,
                result_json: row.get(11)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }
}

// ── Per-Audit Report Types (Phase 8) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FindingCounts {
    pub errors: usize,
    pub warnings: usize,
    pub suggestions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredFinding {
    pub id: i64,
    pub check_id: String,
    pub severity: String,
    pub message: String,
    pub location: Option<String>,
    pub status: String,
    /// Captured excerpt backing this finding, when the provider that raised
    /// it recorded one (semantic providers do; deterministic providers
    /// currently don't — this is `None` for those, not a bug).
    pub evidence_excerpt: Option<String>,
    /// Where the excerpt came from, e.g. "section_id=5, paragraph_index=0".
    pub evidence_source: Option<String>,
}

// ── Build ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct BuildSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub finding_counts: FindingCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub contract_name: Option<String>,
    pub declared_produces: Option<String>,
    pub actual_artifacts: Option<String>,
    pub artifact_freshness: Option<String>,
    pub execution_success: Option<bool>,
    pub execution_output: Option<String>,
    pub findings: Vec<StoredFinding>,
}

// ── Security ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SecuritySessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub secrets_scanned: i64,
    pub secrets_found: i64,
    pub runtime_checks: i64,
    pub runtime_issues: i64,
    pub high_risk_findings: i64,
    pub finding_counts: FindingCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub secrets_scanned: i64,
    pub secrets_found: i64,
    pub runtime_checks: i64,
    pub runtime_issues: i64,
    pub high_risk_findings: i64,
    pub threat_summary: Option<String>,
    pub findings: Vec<StoredFinding>,
}

// ── Consistency ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ConsistencySessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub vision_exists: bool,
    pub architecture_exists: bool,
    pub structure_score: Option<f64>,
    pub cross_references: usize,
    pub finding_counts: FindingCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConsistencyReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub vision_exists: bool,
    pub architecture_exists: bool,
    pub structure_score: Option<f64>,
    pub naming_issues: Option<String>,
    pub cross_references: usize,
    pub findings: Vec<StoredFinding>,
}

// ── Coverage ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct CoverageSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub features_count: usize,
    pub src_files_count: usize,
    pub feature_coverage_pct: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub features_count: usize,
    pub src_files_count: usize,
    pub feature_coverage_pct: Option<f64>,
    pub uncovered_features: Option<String>,
    pub doc_types_covered: Option<String>,
    pub findings: Vec<StoredFinding>,
}

// ── Architecture (Phase 9) ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectureSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub collection_integrity_score: Option<f64>,
    pub structural_integrity_score: Option<f64>,
    pub consistency_score: Option<f64>,
    pub cross_repo_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportRecommendation {
    pub id: i64,
    pub priority: String,
    pub category: String,
    pub description: String,
    pub file_path: Option<String>,
}

#[derive(Serialize)]
pub struct ArchitectureReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub collection_integrity_score: Option<f64>,
    pub structural_integrity_score: Option<f64>,
    pub consistency_score: Option<f64>,
    pub cross_repo_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VisionSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub vision_content_score: Option<f64>,
    pub tech_independence_score: Option<f64>,
    pub traceability_consistency_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct VisionReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub vision_content_score: Option<f64>,
    pub tech_independence_score: Option<f64>,
    pub traceability_consistency_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DesignSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub design_system_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub design_quality_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct DesignReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub design_system_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub design_quality_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReadmeSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub repo_introduction_score: Option<f64>,
    pub doc_navigation_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub maintainability_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct ReadmeReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub repo_introduction_score: Option<f64>,
    pub doc_navigation_score: Option<f64>,
    pub doc_quality_score: Option<f64>,
    pub maintainability_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelpSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub coverage_score: Option<f64>,
    pub navigation_score: Option<f64>,
    pub quality_score: Option<f64>,
    pub accuracy_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct HelpReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub coverage_score: Option<f64>,
    pub navigation_score: Option<f64>,
    pub quality_score: Option<f64>,
    pub accuracy_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrototypeSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub product_validation_score: Option<f64>,
    pub runtime_validation_score: Option<f64>,
    pub engineering_validation_score: Option<f64>,
    pub validation_quality_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct PrototypeReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub product_validation_score: Option<f64>,
    pub runtime_validation_score: Option<f64>,
    pub engineering_validation_score: Option<f64>,
    pub validation_quality_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalContextSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub document_quality_score: Option<f64>,
    pub content_completeness_score: Option<f64>,
    pub documentation_integrity_score: Option<f64>,
    pub collection_quality_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct ExternalContextReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub document_quality_score: Option<f64>,
    pub content_completeness_score: Option<f64>,
    pub documentation_integrity_score: Option<f64>,
    pub collection_quality_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineeringSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub engineering_coverage_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub traceability_consistency_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct EngineeringReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub engineering_coverage_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub traceability_consistency_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_definition_score: Option<f64>,
    pub product_definition_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub product_readiness_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct FeatureReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_definition_score: Option<f64>,
    pub product_definition_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub product_readiness_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureTechnicalSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: Option<f64>,
    pub technical_realization_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub implementation_readiness_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct FeatureTechnicalReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: Option<f64>,
    pub technical_realization_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub implementation_readiness_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FeatureDesignSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: Option<f64>,
    pub user_experience_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub design_readiness_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct FeatureDesignReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub feature_mapping_score: Option<f64>,
    pub user_experience_score: Option<f64>,
    pub documentation_quality_score: Option<f64>,
    pub design_readiness_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeterministicRuntimeSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub runtime_model_score: Option<f64>,
    pub engineering_principles_score: Option<f64>,
    pub runtime_integrity_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct DeterministicRuntimeReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub runtime_model_score: Option<f64>,
    pub engineering_principles_score: Option<f64>,
    pub runtime_integrity_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExternalContextOwnershipSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub dependency_coverage_score: Option<f64>,
    pub documentation_integration_score: Option<f64>,
    pub consistency_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct ExternalContextOwnershipReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub dependency_coverage_score: Option<f64>,
    pub documentation_integration_score: Option<f64>,
    pub consistency_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentationStructureSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub structural_integrity_score: Option<f64>,
    pub mapping_consistency_score: Option<f64>,
    pub atomicity_enforcement_score: Option<f64>,
    pub cross_document_alignment_score: Option<f64>,
    pub name_preservation_score: Option<f64>,
    pub implementation_traceability_score: Option<f64>,
    pub generation_compliance_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct DocumentationStructureReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub structural_integrity_score: Option<f64>,
    pub mapping_consistency_score: Option<f64>,
    pub atomicity_enforcement_score: Option<f64>,
    pub cross_document_alignment_score: Option<f64>,
    pub name_preservation_score: Option<f64>,
    pub implementation_traceability_score: Option<f64>,
    pub generation_compliance_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImplementationSessionInfo {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub architectural_conformance_score: Option<f64>,
    pub feature_conformance_score: Option<f64>,
    pub engineering_conformance_score: Option<f64>,
    pub documentation_integrity_score: Option<f64>,
    pub implementation_quality_score: Option<f64>,
    pub finding_counts: FindingCounts,
}

#[derive(Serialize)]
pub struct ImplementationReportWithFindings {
    pub id: i64,
    pub session_id: String,
    pub score: f64,
    pub previous_score: Option<f64>,
    pub git_revision: Option<String>,
    pub created_at: String,
    pub engineering_readiness: String,
    pub architectural_conformance_score: Option<f64>,
    pub feature_conformance_score: Option<f64>,
    pub engineering_conformance_score: Option<f64>,
    pub documentation_integrity_score: Option<f64>,
    pub implementation_quality_score: Option<f64>,
    pub doc_scores: Option<String>,
    pub validation_scores: Option<String>,
    pub finding_counts: Option<String>,
    pub findings: Vec<StoredFinding>,
    pub recommendations: Vec<ReportRecommendation>,
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
    use schemas::document::{DocumentBody, DocumentMetadata, DocumentPath};
    use schemas::quality::ObjectStatistics;

    fn create_test_doc(id: i64) -> Document {
        Document {
            id,
            path: DocumentPath(PathBuf::from(format!("test_{}.md", id))),
            hash: "abc123".into(),
            standard: "architecture".into(),
            title: format!("Test Document {}", id),
            body: DocumentBody::Generic {
                raw: "# Test\n\nContent".into(),
                sections: Vec::new(),
            },
            metadata: DocumentMetadata::default(),
            provenance: None,
            quality: ObjectStatistics::default(),
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
    fn test_count_section_type_coverage() {
        let store = RegistryStore::open_in_memory().unwrap();
        store.insert_document(&create_test_doc(1)).unwrap();
        store.insert_document(&create_test_doc(2)).unwrap();

        let section = DocumentSection {
            heading: "System Overview".into(),
            semantic_type: "system_overview".into(),
            level: 1,
            body: "...".into(),
            required: true,
            source_span: None,
            subsections: Vec::new(),
            hash: "h1".into(),
        };
        // Only document 1 has a system_overview section; document 2 has none.
        store.insert_document_sections(1, std::slice::from_ref(&section)).unwrap();

        let coverage = store
            .count_section_type_coverage("architecture", &["system_overview", "component_model"])
            .unwrap();

        let system_overview = coverage.iter().find(|(t, _, _)| t == "system_overview").unwrap();
        assert_eq!(system_overview.1, 1); // 1 doc has it
        assert_eq!(system_overview.2, 2); // 2 docs total

        let component_model = coverage.iter().find(|(t, _, _)| t == "component_model").unwrap();
        assert_eq!(component_model.1, 0); // no doc has it
        assert_eq!(component_model.2, 2);
    }

    #[test]
    fn test_finding_evidence_round_trips_through_insert_and_query() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = vec![schemas::audit::AuditFinding {
            check_id: "C1".into(),
            severity: schemas::audit::Severity::Warning,
            message: "Purpose section is generic".into(),
            location: Some("docs/raw/architecture/overview.md".into()),
            document_id: None,
            provider: "semantic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: Some(schemas::audit::Evidence {
                section_id: 5,
                paragraph_index: 0,
                sentence: None,
                excerpt: "This system does things.".into(),
            }),
            status: None,
            strategy: None,
        }];
        store.insert_report_findings("architecture", 1, &findings).unwrap();

        let stored = store.query_findings("architecture", 1).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].evidence_excerpt.as_deref(), Some("This system does things."));
        assert!(stored[0].evidence_source.as_deref().unwrap().contains("section_id=5"));
    }

    #[test]
    fn test_pipeline_check_report_round_trip_and_gate() {
        let store = RegistryStore::open_in_memory().unwrap();

        let report = schemas::audit::PipelineCheckReport {
            report_id: "arch-a1-1".into(),
            pipeline: "architecture".into(),
            check_id: "A1".into(),
            score: 70,
            findings: vec![schemas::audit::AuditFinding {
                check_id: "A1".into(),
                severity: schemas::audit::Severity::Warning,
                message: "Terminology drifts between two architecture docs".into(),
                location: None,
                document_id: None,
                provider: "semantic".into(),
                stage: None,
                section_id: None,
                confidence: Some(0.8),
                evidence: None,
                status: None,
                strategy: None,
            }],
            git_revision: Some("deadbeef".into()),
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let id = store.store_pipeline_check_report(&report).unwrap();
        assert!(id > 0);

        // Below 100 → gate blocks.
        let blocked = store.check_pipeline_gate("architecture").unwrap();
        assert!(blocked.blocked);

        let fetched = store.get_pipeline_check_report("architecture", "A1").unwrap().unwrap();
        assert_eq!(fetched.score, 70);
        assert_eq!(fetched.findings.len(), 1);
        assert_eq!(fetched.git_revision.as_deref(), Some("deadbeef"));

        // A different pipeline's checks don't affect this one's report/gate.
        assert!(store.get_pipeline_check_report("vision", "V1").unwrap().is_none());
        assert!(!store.check_pipeline_gate("vision").unwrap().blocked);

        // Re-judging A1 at 100 adds a second row rather than replacing the
        // first — same append-only shape `semantic_reports`/`check_gate`
        // already use for domain audits, so the gate still counts the
        // earlier score=70 row and stays blocked.
        let converged = schemas::audit::PipelineCheckReport {
            report_id: "arch-a1-2".into(),
            score: 100,
            findings: vec![],
            ..report
        };
        store.store_pipeline_check_report(&converged).unwrap();
        assert!(store.check_pipeline_gate("architecture").unwrap().blocked);
    }

    #[test]
    fn test_pipeline_spec_score_averages_latest_per_check_not_every_row() {
        let store = RegistryStore::open_in_memory().unwrap();

        assert_eq!(store.get_pipeline_spec_score("architecture").unwrap(), None);

        let make = |report_id: &str, check_id: &str, score: i64| schemas::audit::PipelineCheckReport {
            report_id: report_id.into(),
            pipeline: "architecture".into(),
            check_id: check_id.into(),
            score,
            findings: vec![],
            git_revision: None,
            created_at: "2026-01-01T00:00:00Z".into(),
        };

        store.store_pipeline_check_report(&make("r1", "A1", 60)).unwrap();
        store.store_pipeline_check_report(&make("r2", "A2", 100)).unwrap();
        // A1 re-judged higher — the stale 60 must not count toward the average.
        store.store_pipeline_check_report(&make("r3", "A1", 100)).unwrap();

        let score = store.get_pipeline_spec_score("architecture").unwrap().unwrap();
        assert!((score - 100.0).abs() < 0.01, "expected avg(A1=100, A2=100) = 100, got {score}");
    }

    #[test]
    fn test_summary_report_round_trips() {
        let store = RegistryStore::open_in_memory().unwrap();
        let report = schemas::audit::SummaryReport {
            target_type: "pipeline".into(),
            target_name: "architecture".into(),
            deterministic_score: Some(85.0),
            standard_score: None,
            spec_score: Some(70.0),
            overall_score: 77.5,
            readiness: schemas::audit::ReadinessAssessment::Engineering,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let id = store.store_summary_report(&report).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_finding_without_evidence_has_none() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = vec![schemas::audit::AuditFinding {
            check_id: "C2".into(),
            severity: schemas::audit::Severity::Error,
            message: "Deterministic check failed".into(),
            location: None,
            document_id: None,
            provider: "deterministic".into(),
            stage: None,
            section_id: None,
            confidence: None,
            evidence: None,
            status: None,
            strategy: None,
        }];
        store.insert_report_findings("architecture", 1, &findings).unwrap();

        let stored = store.query_findings("architecture", 1).unwrap();
        assert_eq!(stored.len(), 1);
        assert!(stored[0].evidence_excerpt.is_none());
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

    #[test]
    fn test_get_domains_reflects_actual_documents() {
        let store = RegistryStore::open_in_memory().unwrap();
        assert_eq!(store.get_domains().unwrap(), Vec::<String>::new());

        let mut doc = create_test_doc(1);
        doc.standard = "audit".into();
        store.insert_document(&doc).unwrap();
        store.insert_document(&create_test_doc(2)).unwrap(); // standard: "architecture"

        let domains = store.get_domains().unwrap();
        assert_eq!(domains, vec!["architecture".to_string(), "audit".to_string()]);
    }

    // ── Per-Audit Report Tests (Phase 8) ───────────────────────────────

    fn make_findings() -> Vec<schemas::audit::AuditFinding> {
        vec![
            schemas::audit::AuditFinding {
                check_id: "B1".into(),
                severity: schemas::audit::Severity::Error,
                message: "Build failed".into(),
                location: Some("src/main.rs".into()),
                document_id: None, provider: "pipeline".into(),
                stage: None, section_id: None,
                confidence: None, evidence: None, status: None, strategy: None,
            },
            schemas::audit::AuditFinding {
                check_id: "B2".into(),
                severity: schemas::audit::Severity::Warning,
                message: "Missing artifact".into(),
                location: None,
                document_id: None, provider: "pipeline".into(),
                stage: None, section_id: None,
                confidence: None, evidence: None, status: None, strategy: None,
            },
        ]
    }

    #[test]
    fn test_insert_and_query_build_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = make_findings();

        let report_id = store.insert_build_report(
            80.0, "session-1", Some("abc123"),
            Some("my-contract"), Some("[\"target/release/myapp\"]"),
            Some("[\"target/release/myapp\"]"), Some("{\"target/release/myapp\":\"fresh\"}"),
            Some(true), Some("Build completed"),
            &findings,
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_build_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].score, 80.0);
        assert_eq!(sessions[0].finding_counts.errors, 1);
        assert_eq!(sessions[0].finding_counts.warnings, 1);

        let stored = store.get_build_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(stored.contract_name.as_deref(), Some("my-contract"));
        assert_eq!(stored.findings.len(), 2);
        assert_eq!(stored.findings[0].check_id, "B1");
        assert_eq!(stored.findings[0].severity, "error");
        assert_eq!(stored.findings[1].check_id, "B2");
        assert_eq!(stored.findings[1].severity, "warning");
    }

    /// Regression test: `insert_help_report`'s INSERT statement listed a
    /// `finding_counts` column that the V30 migration's `help_reports` table
    /// didn't define — this would fail at runtime with "no column named
    /// finding_counts" on the very first call, invisible to `cargo check`
    /// since rusqlite doesn't validate SQL text at compile time.
    #[test]
    fn test_insert_and_query_help_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = make_findings();

        let report_id = store.insert_help_report(
            85.0, "session-help-1", Some("abc123"), None,
            "READY",
            Some(90.0), Some(80.0), Some(85.0), Some(85.0),
            None, None, None,
            &findings,
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_help_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].score, 85.0);
        assert_eq!(sessions[0].finding_counts.errors, 1);
        assert_eq!(sessions[0].finding_counts.warnings, 1);

        let stored = store.get_help_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(stored.engineering_readiness, "READY");
        assert_eq!(stored.findings.len(), 2);
    }

    #[test]
    fn test_insert_and_query_security_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let report_id = store.insert_security_report(
            75.0, "session-sec-1", None,
            100, 3, 20, 1, 1, Some("Medium risk — 1 secret found"),
            &[],
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_security_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].secrets_scanned, 100);
        assert_eq!(sessions[0].secrets_found, 3);

        let stored = store.get_security_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(stored.threat_summary.as_deref(), Some("Medium risk — 1 secret found"));
        assert_eq!(stored.high_risk_findings, 1);
    }

    #[test]
    fn test_insert_and_query_consistency_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let report_id = store.insert_consistency_report(
            90.0, "session-con-1", None,
            true, true, Some(85.0), Some("[{\"issue\":\"naming\"}]"), 12,
            &[],
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_consistency_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].vision_exists);
        assert_eq!(sessions[0].cross_references, 12);

        let stored = store.get_consistency_report_with_findings(report_id).unwrap().unwrap();
        assert!(stored.vision_exists);
        assert!(stored.architecture_exists);
    }

    #[test]
    fn test_insert_and_query_coverage_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let report_id = store.insert_coverage_report(
            65.0, "session-cov-1", None,
            20, 10, Some(50.0), Some("[\"feat-1\"]"), Some("{\"standard\":true}"),
            &[],
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_coverage_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].features_count, 20);
        assert_eq!(sessions[0].src_files_count, 10);

        let stored = store.get_coverage_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(stored.features_count, 20);
        assert_eq!(stored.src_files_count, 10);
        assert!((stored.feature_coverage_pct.unwrap() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_insert_and_query_architecture_report() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = make_findings();
        let doc_scores = r#"[{"name":"API Architecture","score":88.0},{"name":"Data Flow","score":92.0}]"#;
        let validation_scores = r#"[{"id":"A1","score":95.0},{"id":"A2","score":80.0}]"#;
        let report_id = store.insert_architecture_report(
            85.5, "session-arch-1", None, None,
            "YES",
            Some(90.0), Some(85.0), Some(80.0), Some(75.0),
            Some(doc_scores), Some(validation_scores), None,
            &findings,
        ).unwrap();
        assert!(report_id > 0);

        let sessions = store.query_architecture_sessions(10).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].score as i64, 85);
        assert_eq!(sessions[0].engineering_readiness, "YES");

        let stored = store.get_architecture_report_with_findings(report_id).unwrap().unwrap();
        assert!((stored.score - 85.5).abs() < 0.01);
        assert_eq!(stored.findings.len(), 2);
        assert_eq!(sessions[0].finding_counts.errors, 1);
        assert_eq!(sessions[0].finding_counts.warnings, 1);
        assert_eq!(sessions[0].finding_counts.suggestions, 0);
        assert!(stored.doc_scores.as_deref().unwrap().contains("API Architecture"));
        assert!(stored.validation_scores.as_deref().unwrap().contains("A1"));
    }

    #[test]
    fn test_report_findings_update_status() {
        let store = RegistryStore::open_in_memory().unwrap();
        let findings = make_findings();
        let report_id = store.insert_build_report(
            100.0, "session-status", None,
            None, None, None, None, None, None,
            &findings,
        ).unwrap();

        let stored = store.get_build_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(stored.findings[0].status, "open");

        store.update_finding_status_by_id(stored.findings[0].id, "accepted").unwrap();

        let updated = store.get_build_report_with_findings(report_id).unwrap().unwrap();
        assert_eq!(updated.findings[0].status, "accepted");
    }

    #[test]
    fn test_query_sessions_empty() {
        let store = RegistryStore::open_in_memory().unwrap();
        assert!(store.query_build_sessions(10).unwrap().is_empty());
        assert!(store.query_security_sessions(10).unwrap().is_empty());
        assert!(store.query_consistency_sessions(10).unwrap().is_empty());
        assert!(store.query_coverage_sessions(10).unwrap().is_empty());
        assert!(store.query_architecture_sessions(10).unwrap().is_empty());
    }

    #[test]
    fn test_insert_and_query_fix_session() {
        let store = RegistryStore::open_in_memory().unwrap();
        let session = FixSession {
            id: None, report_id: 1, report_type: "build".into(),
            criterion_id: "B1".into(), finding_json: r#"{"check":"B1"}"#.into(),
            domain: "build".into(), plan_type: PlanType::Build,
            target_file: Some("Cargo.toml".into()),
            attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        };
        let id = store.insert_fix_session(&session).unwrap();
        assert!(id > 0);

        let fetched = store.get_fix_session(id).unwrap().unwrap();
        assert_eq!(fetched.report_id, 1);
        assert_eq!(fetched.criterion_id, "B1");
        assert_eq!(fetched.domain, "build");
        assert_eq!(fetched.plan_type, PlanType::Build);

        let sessions = store.query_fix_sessions(10, 0).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].domain, "build");
    }

    #[test]
    fn test_insert_and_query_fix_plan() {
        let store = RegistryStore::open_in_memory().unwrap();
        let step = PlanStep {
            id: None, plan_id: None, step_order: 1,
            action: "modify".into(), target: "Cargo.toml".into(),
            rationale: "Missing dep".into(), detail: "Add dep".into(),
            verification: "cargo check".into(), rollback: Some("git checkout".into()),
            status: FixStepStatus::Pending, verified_at: None, score: None,
        };
        let plan = FixPlan {
            id: None, session_id: "sess-1".into(), report_id: 1,
            criterion_id: "B1".into(), domain: "build".into(),
            plan_type: PlanType::Build, title: "Fix build".into(),
            summary: "Update Cargo.toml".into(),
            prerequisites: vec!["File writable".into()],
            steps: vec![step],
            rollback_instructions: Some("git checkout".into()),
            expected_checks: vec!["B1".into()],
            status: FixPlanStatus::Draft, created_at: None, updated_at: None,
        };
        let id = store.insert_fix_plan(&plan).unwrap();
        assert!(id > 0);

        let fetched = store.get_fix_plan(id).unwrap().unwrap();
        assert_eq!(fetched.criterion_id, "B1");
        assert_eq!(fetched.steps.len(), 1);
        assert_eq!(fetched.prerequisites.len(), 1);
        assert_eq!(fetched.expected_checks, vec!["B1"]);

        // Plans are linked via fix_attempts: insert session first, then attempt
        let sess_id = store.insert_fix_session(&FixSession {
            id: None, report_id: 1, report_type: "build".into(),
            criterion_id: "B1".into(), finding_json: "{}".into(),
            domain: "build".into(), plan_type: PlanType::Build,
            target_file: None, attempt_count: 0, max_attempts: 3,
            status: SessionStatus::InProgress, created_at: None, updated_at: None,
        }).unwrap();
        // Insert attempt linking session to plan
        store.insert_fix_attempt(&FixAttempt {
            id: None, session_id: sess_id, attempt: 0, plan_id: Some(id),
            plan_type: PlanType::Build, score: Some(9.0),
            check_scores: None, passed: true, error_message: None, created_at: None,
        }).unwrap();
        let plans = store.query_fix_plans_by_session(sess_id).unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].title, "Fix build");
    }

    #[test]
    fn test_fix_session_pagination() {
        let store = RegistryStore::open_in_memory().unwrap();
        for i in 0..5 {
            let session = FixSession {
                id: None, report_id: i, report_type: "build".into(),
                criterion_id: format!("B{}", i), finding_json: "{}".into(),
                domain: "build".into(), plan_type: PlanType::Build,
                target_file: None, attempt_count: 0, max_attempts: 3,
                status: SessionStatus::InProgress, created_at: None, updated_at: None,
            };
            store.insert_fix_session(&session).unwrap();
        }

        let page1 = store.query_fix_sessions(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        let page2 = store.query_fix_sessions(2, 2).unwrap();
        assert_eq!(page2.len(), 2);

        let page3 = store.query_fix_sessions(2, 4).unwrap();
        assert_eq!(page3.len(), 1);
    }

    // ── Repository Metadata Tests ─────────────────────────────────────────

    #[test]
    fn test_repository_metadata_upsert_and_get() {
        let store = RegistryStore::open_in_memory().unwrap();
        store.upsert_repository_metadata("repo_name", "samgraha").unwrap();
        store.upsert_repository_metadata("source_dir", "crates").unwrap();
        // Upsert of an existing key must update, not duplicate.
        store.upsert_repository_metadata("repo_name", "samgraha-renamed").unwrap();

        let meta = store.get_repository_metadata().unwrap();
        assert_eq!(meta.get("repo_name").map(String::as_str), Some("samgraha-renamed"));
        assert_eq!(meta.get("source_dir").map(String::as_str), Some("crates"));
        assert_eq!(meta.len(), 2);
    }

    // ── Project Plan CRUD Tests ──────────────────────────────────────────

    #[test]
    fn test_insert_and_get_plan() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-1".into(),
            title: "Test Plan".into(),
            case_type: schemas::ProjectCase::DocAudit,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();
        let fetched = store.get_plan("plan-1").unwrap().unwrap();
        assert_eq!(fetched.title, "Test Plan");
        assert_eq!(fetched.case_type, schemas::ProjectCase::DocAudit);
    }

    #[test]
    fn test_insert_and_get_phases() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-2".into(),
            title: "Phase Test".into(),
            case_type: schemas::ProjectCase::NewProject,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();

        let phase = schemas::ProjectPhase {
            id: "phase-1".into(),
            plan_id: "plan-2".into(),
            phase_number: 1,
            name: "Audit".into(),
            phase_type: schemas::PhaseType::Audit,
            domains: vec!["build".into()],
            pipeline_ids: vec!["build".into()],
            dependencies: vec![],
            status: schemas::PhaseStatus::Pending,
            started_at: None,
            completed_at: None,
            result_json: None,
        };
        store.insert_phase(&phase).unwrap();

        let phases = store.get_phases("plan-2").unwrap();
        assert_eq!(phases.len(), 1);
        assert_eq!(phases[0].name, "Audit");
        assert_eq!(phases[0].phase_number, 1);
    }

    #[test]
    fn test_list_plans() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-list-1".into(),
            title: "List Test".into(),
            case_type: schemas::ProjectCase::BuildAudit,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();
        let plans = store.list_plans().unwrap();
        assert_eq!(plans.len(), 1);
    }

    #[test]
    fn test_update_plan_status() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-status".into(),
            title: "Status Test".into(),
            case_type: schemas::ProjectCase::BuildAudit,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();
        store.update_plan_status("plan-status", &schemas::PlanStatus::Completed).unwrap();
        let fetched = store.get_plan("plan-status").unwrap().unwrap();
        assert_eq!(fetched.status, schemas::PlanStatus::Completed);
    }

    #[test]
    fn test_get_pending_phase() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-pending".into(),
            title: "Pending Test".into(),
            case_type: schemas::ProjectCase::NewProject,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();

        let phase1 = schemas::ProjectPhase {
            id: "p1".into(), plan_id: "plan-pending".into(),
            phase_number: 1, name: "Phase 1".into(),
            phase_type: schemas::PhaseType::Audit,
            domains: vec!["build".into()], pipeline_ids: vec!["build".into()],
            dependencies: vec![], status: schemas::PhaseStatus::Completed,
            started_at: None, completed_at: None, result_json: None,
        };
        let phase2 = schemas::ProjectPhase {
            id: "p2".into(), plan_id: "plan-pending".into(),
            phase_number: 2, name: "Phase 2".into(),
            phase_type: schemas::PhaseType::Fix,
            domains: vec!["build".into()], pipeline_ids: vec!["build".into()],
            dependencies: vec![], status: schemas::PhaseStatus::Pending,
            started_at: None, completed_at: None, result_json: None,
        };
        store.insert_phase(&phase1).unwrap();
        store.insert_phase(&phase2).unwrap();

        let pending = store.get_pending_phase("plan-pending").unwrap().unwrap();
        assert_eq!(pending.phase_number, 2);
        assert_eq!(pending.name, "Phase 2");
    }

    #[test]
    fn test_try_start_phase_guards_concurrent_start() {
        let store = RegistryStore::open_in_memory().unwrap();
        let plan = schemas::ProjectPlan {
            id: "plan-race".into(),
            title: "Race Test".into(),
            case_type: schemas::ProjectCase::BuildAudit,
            status: schemas::PlanStatus::Active,
            current_phase: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        store.insert_plan(&plan).unwrap();
        let phase = schemas::ProjectPhase {
            id: "phase-race".into(), plan_id: "plan-race".into(),
            phase_number: 1, name: "Audit".into(),
            phase_type: schemas::PhaseType::Audit,
            domains: vec!["build".into()], pipeline_ids: vec!["build".into()],
            dependencies: vec![], status: schemas::PhaseStatus::Pending,
            started_at: None, completed_at: None, result_json: None,
        };
        store.insert_phase(&phase).unwrap();

        // First caller wins the race.
        assert!(store.try_start_phase("phase-race").unwrap());
        // A second overlapping caller must not also "win" — the phase is no
        // longer pending.
        assert!(!store.try_start_phase("phase-race").unwrap());
    }

    #[test]
    fn get_audit_knowledge_reads_from_repo_root_not_cwd() {
        // Regression: get_audit_knowledge used to resolve
        // "docs/raw/audit-standards/..." as a bare relative path against the
        // process's current working directory instead of the target repo —
        // an MCP server launched from anywhere other than the audited repo
        // silently read the wrong repo's (or no) knowledge.
        let store = RegistryStore::open_in_memory().unwrap();
        let repo_root = std::env::temp_dir().join(format!(
            "samgraha-audit-knowledge-test-{}",
            std::process::id()
        ));
        let knowledge_dir = repo_root.join("docs/raw/audit-standards/architecture");
        std::fs::create_dir_all(&knowledge_dir).unwrap();
        std::fs::write(knowledge_dir.join("purpose.md"), "expected content").unwrap();

        let content = store
            .get_audit_knowledge(&repo_root, "architecture", "purpose")
            .unwrap();
        assert_eq!(content, "expected content");

        std::fs::remove_dir_all(&repo_root).ok();
    }
}
