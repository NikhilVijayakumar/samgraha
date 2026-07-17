/// Knowledge registry migrations — create `knowledge.db` tables.
pub const KNOWLEDGE_MIGRATIONS: &[&str] = &[V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12, V13, V14, V15, V16, V17, V18, V19, V20, V21, V22, V23, V24, V25, V26, V27, V28, V29, V30, V31, V32, V33];

/// Repository registry migrations — create `registry.db` tables.
pub const REGISTRY_MIGRATIONS: &[&str] = &[REG_V1, REG_V2];

/// Backward compat alias.
pub const MIGRATIONS: &[&str] = KNOWLEDGE_MIGRATIONS;

const V1: &str = "
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL,
    standard TEXT NOT NULL,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_documents_standard ON documents(standard);
CREATE INDEX IF NOT EXISTS idx_documents_path ON documents(path);
CREATE INDEX IF NOT EXISTS idx_documents_hash ON documents(hash);

CREATE TABLE IF NOT EXISTS relationships (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES documents(id),
    target_id INTEGER NOT NULL REFERENCES documents(id),
    rel_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);

CREATE TABLE IF NOT EXISTS audit_results (
    id INTEGER PRIMARY KEY,
    document_id INTEGER REFERENCES documents(id),
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    location TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_document ON audit_results(document_id);

CREATE TABLE IF NOT EXISTS glossary (
    id INTEGER PRIMARY KEY,
    term TEXT NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    source_document_id INTEGER REFERENCES documents(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS enrichment (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    artifact_type TEXT NOT NULL,
    content TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_enrichment_document ON enrichment(document_id);

CREATE TABLE IF NOT EXISTS build_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const V2: &str = "
CREATE TABLE IF NOT EXISTS search_index (
    term TEXT NOT NULL,
    document_id INTEGER NOT NULL REFERENCES documents(id),
    frequency INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (term, document_id)
);

CREATE INDEX IF NOT EXISTS idx_search_term ON search_index(term);
";

const V3: &str = "
CREATE TABLE IF NOT EXISTS knowledge_packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    manifest TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

const V4: &str = "
CREATE TABLE IF NOT EXISTS document_sections (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    semantic_type TEXT NOT NULL,
    canonical_name TEXT NOT NULL,
    content TEXT NOT NULL,
    required INTEGER NOT NULL DEFAULT 0,
    section_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sections_semantic_type ON document_sections(semantic_type);
CREATE INDEX IF NOT EXISTS idx_sections_document_id ON document_sections(document_id);
CREATE INDEX IF NOT EXISTS idx_sections_type_doc ON document_sections(semantic_type, document_id);
";

const V5: &str = "
ALTER TABLE document_sections ADD COLUMN parent_id INTEGER REFERENCES document_sections(id);

CREATE INDEX IF NOT EXISTS idx_sections_parent_id ON document_sections(parent_id);
";

const V6: &str = "
CREATE TABLE IF NOT EXISTS graph_nodes (
    urn TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE,
    title TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_document ON graph_nodes(document_id);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes(node_type);

CREATE TABLE IF NOT EXISTS graph_edges (
    id INTEGER PRIMARY KEY,
    source_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    target_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON graph_edges(source_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON graph_edges(target_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_type ON graph_edges(edge_type);
";

const V7: &str = "
ALTER TABLE documents ADD COLUMN quality TEXT NOT NULL DEFAULT '{}';
";

const V8: &str = "
-- V8: Make graph_nodes.document_id nullable (sub-item nodes don't belong to a document)
DROP TABLE IF EXISTS graph_edges;
DROP TABLE IF EXISTS graph_nodes;

CREATE TABLE IF NOT EXISTS graph_nodes (
    urn TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE,
    title TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_document ON graph_nodes(document_id);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes(node_type);

CREATE TABLE IF NOT EXISTS graph_edges (
    id INTEGER PRIMARY KEY,
    source_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    target_urn TEXT NOT NULL REFERENCES graph_nodes(urn) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON graph_edges(source_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON graph_edges(target_urn);
CREATE INDEX IF NOT EXISTS idx_graph_edges_type ON graph_edges(edge_type);
";

const V9: &str = "
-- V9: ON DELETE CASCADE on all FK constraints referencing documents(id).
--      Enables INSERT OR REPLACE on documents to cascade-clean child rows.
--      Recreates tables using CREATE+INSERT+DROP+RENAME pattern.

CREATE TABLE IF NOT EXISTS relationships_v9 (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    target_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    rel_type TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}'
);
INSERT INTO relationships_v9 SELECT * FROM relationships;
DROP TABLE IF EXISTS relationships;
ALTER TABLE relationships_v9 RENAME TO relationships;
CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);

CREATE TABLE IF NOT EXISTS audit_results_v9 (
    id INTEGER PRIMARY KEY,
    document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE,
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    location TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO audit_results_v9 SELECT * FROM audit_results;
DROP TABLE IF EXISTS audit_results;
ALTER TABLE audit_results_v9 RENAME TO audit_results;
CREATE INDEX IF NOT EXISTS idx_audit_document ON audit_results(document_id);

CREATE TABLE IF NOT EXISTS glossary_v9 (
    id INTEGER PRIMARY KEY,
    term TEXT NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    source_document_id INTEGER REFERENCES documents(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO glossary_v9 SELECT * FROM glossary;
DROP TABLE IF EXISTS glossary;
ALTER TABLE glossary_v9 RENAME TO glossary;

CREATE TABLE IF NOT EXISTS enrichment_v9 (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    artifact_type TEXT NOT NULL,
    content TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
INSERT INTO enrichment_v9 SELECT * FROM enrichment;
DROP TABLE IF EXISTS enrichment;
ALTER TABLE enrichment_v9 RENAME TO enrichment;
CREATE INDEX IF NOT EXISTS idx_enrichment_document ON enrichment(document_id);

CREATE TABLE IF NOT EXISTS search_index_v9 (
    term TEXT NOT NULL,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    frequency INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (term, document_id)
);
INSERT INTO search_index_v9 SELECT * FROM search_index;
DROP TABLE IF EXISTS search_index;
ALTER TABLE search_index_v9 RENAME TO search_index;
CREATE INDEX IF NOT EXISTS idx_search_term ON search_index(term);
";

const V10: &str = "
-- V10: Add hash column to document_sections for incremental change detection
ALTER TABLE document_sections ADD COLUMN hash TEXT NOT NULL DEFAULT '';
";

const V11: &str = "
-- V11: semantic_reports table for semantic audit report storage
CREATE TABLE IF NOT EXISTS semantic_reports (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    report_uuid     TEXT    NOT NULL UNIQUE,
    stage           TEXT    NOT NULL,
    domain          TEXT    NOT NULL,
    document_id     INTEGER,
    section_id      INTEGER,
    score           INTEGER NOT NULL DEFAULT 0,
    findings        TEXT    NOT NULL DEFAULT '[]',
    strategy        TEXT,
    document_revision INTEGER,
    document_hash   TEXT,
    created_at      TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_semantic_reports_stage ON semantic_reports(stage);
CREATE INDEX IF NOT EXISTS idx_semantic_reports_domain ON semantic_reports(domain);
CREATE INDEX IF NOT EXISTS idx_semantic_reports_document ON semantic_reports(document_id);
CREATE INDEX IF NOT EXISTS idx_semantic_reports_section ON semantic_reports(section_id);
";

const V12: &str = "
-- V12: section_audit_hashes for fast incremental skip lookup
CREATE TABLE IF NOT EXISTS section_audit_hashes (
    section_id  INTEGER NOT NULL,
    hash        TEXT    NOT NULL,
    report_id   INTEGER NOT NULL,
    checked_at  TEXT    NOT NULL,
    PRIMARY KEY (section_id)
);
";

const V13: &str = "
-- V13: Pipeline report storage for Phase 7 — store facts, render views on demand
CREATE TABLE IF NOT EXISTS pipeline_reports (
    id INTEGER PRIMARY KEY,
    pipeline TEXT NOT NULL,
    score REAL NOT NULL,
    categories TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    session_id TEXT NOT NULL,
    git_revision TEXT,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE IF NOT EXISTS pipeline_findings (
    id INTEGER PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES pipeline_reports(id) ON DELETE CASCADE,
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    location TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    comment TEXT
);

CREATE INDEX IF NOT EXISTS idx_pipeline_findings_report ON pipeline_findings(report_id);
CREATE INDEX IF NOT EXISTS idx_pipeline_findings_severity ON pipeline_findings(severity);

CREATE TABLE IF NOT EXISTS report_comments (
    id INTEGER PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES pipeline_reports(id) ON DELETE CASCADE,
    finding_id INTEGER REFERENCES pipeline_findings(id) ON DELETE SET NULL,
    author TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_report_comments_report ON report_comments(report_id);
";

const V14: &str = "
-- V14: Per-audit report tables (Phase 8) — each audit type gets its own domain-specific schema.
--       Shared concerns (findings, evidence, summaries, improvements) use polymorphic tables.
--       Old pipeline_reports/findings/comments tables remain untouched.

CREATE TABLE IF NOT EXISTS build_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'build',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    contract_name TEXT,
    declared_produces TEXT,
    actual_artifacts TEXT,
    artifact_freshness TEXT,
    execution_success INTEGER,
    execution_output TEXT,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE IF NOT EXISTS security_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'security',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    secrets_scanned INTEGER DEFAULT 0,
    secrets_found INTEGER DEFAULT 0,
    runtime_checks INTEGER DEFAULT 0,
    runtime_issues INTEGER DEFAULT 0,
    high_risk_findings INTEGER DEFAULT 0,
    threat_summary TEXT,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE IF NOT EXISTS consistency_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'consistency',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    vision_exists INTEGER DEFAULT 0,
    architecture_exists INTEGER DEFAULT 0,
    structure_score REAL,
    naming_issues TEXT,
    cross_references INTEGER DEFAULT 0,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE IF NOT EXISTS coverage_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'coverage',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    features_count INTEGER DEFAULT 0,
    src_files_count INTEGER DEFAULT 0,
    feature_coverage_pct REAL,
    uncovered_features TEXT,
    doc_types_covered TEXT,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE IF NOT EXISTS report_findings (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    location TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_findings_type ON report_findings(report_type, report_id);

CREATE TABLE IF NOT EXISTS report_evidence (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    finding_id INTEGER REFERENCES report_findings(id),
    key TEXT NOT NULL,
    value TEXT,
    source TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_evidence_type ON report_evidence(report_type, report_id);

CREATE TABLE IF NOT EXISTS report_summaries (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    summary_text TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_summaries_type ON report_summaries(report_type, report_id);

CREATE TABLE IF NOT EXISTS report_improvements (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    suggestion TEXT NOT NULL,
    priority TEXT DEFAULT 'medium',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_improvements_type ON report_improvements(report_type, report_id);

-- Rename report_improvements to report_recommendations for clarity
CREATE TABLE IF NOT EXISTS report_recommendations (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    priority TEXT NOT NULL DEFAULT 'P3',
    category TEXT NOT NULL,
    description TEXT NOT NULL,
    file_path TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_recommendations_type ON report_recommendations(report_type, report_id);
";

const V15: &str = "
-- V15: In-depth architecture reports (Phase 9).
--       Dedicated table with domain-specific scoring dimensions, per-document scores,
--       per-validation (A1-A13) scores, and trend tracking.

CREATE TABLE IF NOT EXISTS architecture_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    collection_integrity_score REAL,
    structural_integrity_score REAL,
    consistency_score REAL,
    cross_repo_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V16: &str = "
-- V16: Vision report storage — mirrors architecture_reports' shape (score,
--       4 category scores, doc/validation scores, finding counts, trend
--       tracking) with Vision's own category names.

CREATE TABLE IF NOT EXISTS vision_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    vision_content_score REAL,
    tech_independence_score REAL,
    traceability_consistency_score REAL,
    doc_quality_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V17: &str = "
-- V17: Design report storage — mirrors architecture_reports'/vision_reports'
--       shape with Design's own category names (Design System /
--       Documentation Quality / Design Quality).

CREATE TABLE IF NOT EXISTS design_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    design_system_score REAL,
    doc_quality_score REAL,
    design_quality_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V18: &str = "
-- V18: README report storage — mirrors the other domain report tables with
--       README's own category names (Repository Introduction / Documentation
--       Navigation / Documentation Quality / Maintainability).

CREATE TABLE IF NOT EXISTS readme_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    repo_introduction_score REAL,
    doc_navigation_score REAL,
    doc_quality_score REAL,
    maintainability_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V19: &str = "
-- V19: Prototype report storage — mirrors the other domain report tables
--       with Prototype's own category names (Product Validation / Runtime
--       Validation / Engineering Validation / Validation Quality).

CREATE TABLE IF NOT EXISTS prototype_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    product_validation_score REAL,
    runtime_validation_score REAL,
    engineering_validation_score REAL,
    validation_quality_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V20: &str = "
-- V20: External Context report storage — mirrors the other domain report
--       tables with its own category names (Document Quality / Content
--       Completeness / Documentation Integrity / Collection Quality).

CREATE TABLE IF NOT EXISTS external_context_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    document_quality_score REAL,
    content_completeness_score REAL,
    documentation_integrity_score REAL,
    collection_quality_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V21: &str = "
-- V21: Engineering report storage — mirrors the other domain report
--       tables with its own category names (Engineering Coverage /
--       Documentation Quality / Traceability and Consistency).

CREATE TABLE IF NOT EXISTS engineering_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    engineering_coverage_score REAL,
    documentation_quality_score REAL,
    traceability_consistency_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V22: &str = "
-- V22: Feature report storage — mirrors the other domain report tables
--       with its own category names (Feature Definition / Product
--       Definition / Documentation Quality / Product Readiness).

CREATE TABLE IF NOT EXISTS feature_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    feature_definition_score REAL,
    product_definition_score REAL,
    documentation_quality_score REAL,
    product_readiness_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V23: &str = "
-- V23: Feature Technical Design report storage — mirrors the other domain
--       report tables with its own category names (Feature Mapping /
--       Technical Realization / Documentation Quality / Implementation
--       Readiness).

CREATE TABLE IF NOT EXISTS feature_technical_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    feature_mapping_score REAL,
    technical_realization_score REAL,
    documentation_quality_score REAL,
    implementation_readiness_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V24: &str = "
-- V24: Feature Design report storage — mirrors the other domain report
--       tables with its own category names (Feature Mapping / User
--       Experience / Documentation Quality / Design Readiness).

CREATE TABLE IF NOT EXISTS feature_design_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    feature_mapping_score REAL,
    user_experience_score REAL,
    documentation_quality_score REAL,
    design_readiness_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V25: &str = "
-- V25: Deterministic Runtime report storage — mirrors the other domain
--       report tables with its own category names (Runtime Model /
--       Engineering Principles / Runtime Integrity). This audit is
--       cross-cutting (scans Architecture + Engineering docs together)
--       rather than a single documentation collection, so it has no
--       document-scores concept of its own beyond the shared doc_scores column.

CREATE TABLE IF NOT EXISTS deterministic_runtime_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    runtime_model_score REAL,
    engineering_principles_score REAL,
    runtime_integrity_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V26: &str = "
-- V26: External Context Ownership report storage — mirrors the other
--       domain report tables with its own category names (Dependency
--       Coverage / Documentation Integration / Consistency). Distinct
--       from external_context_reports: this audit cross-checks External
--       Context usage across the whole documentation ecosystem rather
--       than auditing the External Context collection in isolation.

CREATE TABLE IF NOT EXISTS external_context_ownership_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    dependency_coverage_score REAL,
    documentation_integration_score REAL,
    consistency_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V27: &str = "
-- V27: Implementation Conformance report storage — mirrors the other
--       domain report tables with its own category names (Architectural
--       Conformance / Feature Conformance / Engineering Conformance /
--       Documentation Integrity / Implementation Quality). Distinct from
--       every other domain: this audit reads actual source code under
--       the declared implementation folder, not just docs/raw/*.md.

CREATE TABLE IF NOT EXISTS implementation_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NO',
    architectural_conformance_score REAL,
    feature_conformance_score REAL,
    engineering_conformance_score REAL,
    documentation_integrity_score REAL,
    implementation_quality_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V28: &str = "
-- V28: Audit-Fix Pipeline tables — fix sessions, attempts, plans, and plan steps.
-- Each fix_session tracks one finding remediation from first attempt through
-- pass or human-review escalation. fix_plans and fix_plan_steps store the
-- generated plan; fix_attempts records each verification loop iteration.

CREATE TABLE IF NOT EXISTS fix_sessions (
    id INTEGER PRIMARY KEY,
    report_id INTEGER NOT NULL,
    report_type TEXT NOT NULL,
    criterion_id TEXT NOT NULL,
    finding_json TEXT NOT NULL,
    domain TEXT NOT NULL,
    plan_type TEXT NOT NULL,
    target_file TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    status TEXT NOT NULL DEFAULT 'in_progress',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fix_attempts (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES fix_sessions(id),
    attempt INTEGER NOT NULL,
    plan_id INTEGER REFERENCES fix_plans(id),
    plan_type TEXT NOT NULL,
    score REAL,
    check_scores TEXT,
    passed INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fix_plans (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    criterion_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    plan_type TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    prerequisites TEXT,
    steps TEXT NOT NULL,
    rollback_instructions TEXT,
    expected_checks TEXT NOT NULL DEFAULT '[]',
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS fix_plan_steps (
    id INTEGER PRIMARY KEY,
    plan_id INTEGER NOT NULL REFERENCES fix_plans(id),
    step_order INTEGER NOT NULL,
    action TEXT NOT NULL,
    target TEXT NOT NULL,
    rationale TEXT NOT NULL,
    detail TEXT NOT NULL,
    verification TEXT NOT NULL,
    rollback TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    verified_at TEXT,
    score REAL
);
";

const REG_V1: &str = "
-- REG_V1 — repository registry tables for `.samgraha/registry.db`.
-- Stores cached dependency metadata in a single `repository_cache` table,
-- indexed by UUID for fast lookup during dependency resolution.
-- Supersedes the Phase 1-5 JSON file approach (`.samgraha/dependencies/*.meta.json`).
-- The cache is disposable — fully rebuildable from dependency manifests.
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS repository_cache (
    id TEXT PRIMARY KEY,
    uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    repository_root TEXT NOT NULL,
    knowledge_db TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 0,
    exports TEXT NOT NULL DEFAULT '[]',
    audit TEXT NOT NULL DEFAULT 'PASS',
    last_sync TEXT NOT NULL,
    expires TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_repo_cache_uuid ON repository_cache(uuid);
";

/// REG_V2 — add `dependencies` column for cached transitive dependency names.
/// Enables offline resolution without reading dependency manifests on cache hit.
const REG_V2: &str = "
ALTER TABLE repository_cache ADD COLUMN dependencies TEXT NOT NULL DEFAULT '[]';
";

/// V29 — Project planner tables for phasewise workflow orchestration.
const V29: &str = "
CREATE TABLE IF NOT EXISTS project_plans (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    case_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    current_phase TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS project_phases (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES project_plans(id),
    phase_number INTEGER NOT NULL,
    name TEXT NOT NULL,
    phase_type TEXT NOT NULL,
    domains TEXT NOT NULL,
    pipeline_ids TEXT NOT NULL,
    dependencies TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at TEXT,
    completed_at TEXT,
    result_json TEXT,
    UNIQUE(plan_id, phase_number)
);

CREATE INDEX IF NOT EXISTS idx_project_phases_plan ON project_phases(plan_id);
CREATE INDEX IF NOT EXISTS idx_project_phases_status ON project_phases(status);
";

const V30: &str = "
CREATE TABLE IF NOT EXISTS help_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NOT_READY',
    coverage_score REAL,
    navigation_score REAL,
    quality_score REAL,
    accuracy_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);

CREATE TABLE IF NOT EXISTS repository_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const V31: &str = "
-- V31: Documentation Structure report storage — mirrors the other domain
--       report tables. Seven categories per docs/proposal.md: Structural
--       Integrity, Mapping Consistency, Atomicity Enforcement,
--       Cross-Document Alignment, Name Preservation, Implementation
--       Traceability, Generation Compliance.

CREATE TABLE IF NOT EXISTS documentation_structure_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL UNIQUE,
    score REAL NOT NULL,
    previous_score REAL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    engineering_readiness TEXT NOT NULL DEFAULT 'NOT_READY',
    structural_integrity_score REAL,
    mapping_consistency_score REAL,
    atomicity_enforcement_score REAL,
    cross_document_alignment_score REAL,
    name_preservation_score REAL,
    implementation_traceability_score REAL,
    generation_compliance_score REAL,
    doc_scores TEXT,
    validation_scores TEXT,
    finding_counts TEXT DEFAULT '{\"critical\":0,\"major\":0,\"minor\":0,\"observations\":0}'
);
";

const V32: &str = "
-- V32: Spec-layer (docs/raw/audit/*.md checklist) report storage for
--       pipeline audits, plus a cross-layer summary rollup. See
--       docs/proposal.md — 'Three-Layer Audit Model' — for the design.
--       Mirrors semantic_reports (V11) but keyed by pipeline + check_id
--       instead of domain + section_id, since Spec-layer checks (A1-A13,
--       V1-V12, ...) judge a whole collection, not one section.

CREATE TABLE IF NOT EXISTS pipeline_semantic_reports (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    report_uuid     TEXT    NOT NULL UNIQUE,
    pipeline        TEXT    NOT NULL,
    check_id        TEXT    NOT NULL,
    score           INTEGER NOT NULL DEFAULT 0,
    findings        TEXT    NOT NULL DEFAULT '[]',
    git_revision    TEXT,
    created_at      TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_pipeline_semantic_reports_pipeline ON pipeline_semantic_reports(pipeline);
CREATE INDEX IF NOT EXISTS idx_pipeline_semantic_reports_check ON pipeline_semantic_reports(pipeline, check_id);

-- One row per (target, generated-at). The three *_report_ref columns are
-- nullable bookkeeping tags, not enforced foreign keys — same convention
-- report_findings.report_id already uses — because a target may have only
-- 1 or 2 of the 3 layers available (see docs/proposal.md §2's matrix).
CREATE TABLE IF NOT EXISTS summary_reports (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type             TEXT    NOT NULL,
    target_name             TEXT    NOT NULL,
    deterministic_report_ref TEXT,
    standard_report_ref     TEXT,
    spec_report_ref         TEXT,
    overall_score           REAL,
    readiness               TEXT,
    created_at              TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_summary_reports_target ON summary_reports(target_type, target_name);
";

const V33: &str = "
-- V33: Archive for standard-driven (YAML pipeline) audit runs. Every
--      standard/YAML pipeline audit is stateless today (result returned to
--      the MCP caller, never persisted) — this is the first archive for
--      that path. `model` is self-reported by the calling agent (MCP has
--      no protocol-level way to learn which LLM is driving a client), so
--      it can be NULL; grouping by it (leaderboards, agreement across
--      repeat runs) is a query concern, not enforced here.
CREATE TABLE IF NOT EXISTS standard_audit_runs (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    standard        TEXT    NOT NULL,
    pipeline        TEXT    NOT NULL,
    model           TEXT,
    score           REAL    NOT NULL,
    report          TEXT    NOT NULL,
    git_revision    TEXT,
    created_at      TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_standard_audit_runs_standard ON standard_audit_runs(standard, pipeline);
CREATE INDEX IF NOT EXISTS idx_standard_audit_runs_model ON standard_audit_runs(model);
";
