use crate::migration::KNOWLEDGE_MIGRATIONS;
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use schemas::audit::{
    AuditFinding, AuditStage, FindingStatus, GateResult, SectionChangedResult,
    SemanticReport,
};
use schemas::document::{Document, DocumentBody, DocumentSection};
use schemas::enrichment::EnrichmentArtifact;
use schemas::graph::{EdgeType, GraphEdge, GraphNode, KnowledgeGraph};
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

    pub fn get_audit_knowledge(&self, domain: &str, section_type: &str) -> Result<String> {
        let knowledge_path = PathBuf::from("docs/raw/audit-standards")
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
