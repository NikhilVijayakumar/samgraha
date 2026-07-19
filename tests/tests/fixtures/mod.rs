#![allow(dead_code)]
use schemas::document::{Document, DocumentBody, DocumentMetadata, DocumentPath};
use schemas::quality::ObjectStatistics;
use std::path::PathBuf;

pub fn sample_document(id: i64, standard: &str, title: &str, body: &str) -> Document {
    Document {
        id,
        path: DocumentPath(PathBuf::from(format!("docs/{}/{}.md", standard, title))),
        hash: compute_hash(body),
        standard: standard.to_string(),
        title: title.to_string(),
        body: DocumentBody::Generic {
            raw: body.to_string(),
            sections: Vec::new(),
        },
        metadata: DocumentMetadata::default(),
        provenance: None,
        quality: ObjectStatistics::default(),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn compute_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn sample_documents() -> Vec<Document> {
    vec![
        sample_document(
            1,
            "architecture",
            "System Overview",
            "# System Overview\n\n## Purpose\n\nThis document describes the system.\n\n## Components\n\n- Compiler\n- Registry\n- Runtime",
        ),
        sample_document(
            2,
            "architecture",
            "Component Model",
            "# Component Model\n\n## Purpose\n\nDefines components.\n\n## Responsibilities\n\nEach component has one responsibility.",
        ),
        sample_document(
            3,
            "feature",
            "Knowledge Compilation",
            "# Knowledge Compilation\n\n## Purpose\n\nTransform documentation into knowledge.\n\n## Requirements\n\n- Must be deterministic\n- Must work offline",
        ),
        sample_document(
            4,
            "feature",
            "Knowledge Search",
            "# Knowledge Search\n\n## Purpose\n\nSearch compiled knowledge.\n\n## Requirements\n\n- Full-text search\n- Progressive retrieval",
        ),
    ]
}

/// Create a temporary directory with `.samgraha/standards.db` populated
/// with a minimal but complete knowledge-hub schema. Returns the temp dir
/// path (caller is responsible for cleanup, or use `TempDir` if tempfile
/// is added later).
///
/// The DB contains: 1 system (default), 1 standard ("documentation-standards"),
/// domains for architecture + feature + vision + engineering + readme +
/// design + external-context + prototype + help + philosophy +
/// feature-design + feature-technical + standards, section_catalog entries
/// for architecture, deterministic rules for architecture (A1–A4) and
/// vision (vis-001–vis-003), and a couple of domain_relationships.
pub fn create_test_standards_db() -> PathBuf {
    let tmp = std::env::temp_dir().join(format!(
        "samgraha-standards-fixture-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let samgraha_dir = tmp.join(".samgraha");
    std::fs::create_dir_all(&samgraha_dir).unwrap();
    let db_path = samgraha_dir.join("standards.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();

    // Create the full knowledge-hub schema.
    conn.execute_batch(
        "CREATE TABLE systems (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            is_default INTEGER NOT NULL DEFAULT 0
        );
        CREATE UNIQUE INDEX ux_systems_one_default ON systems(is_default) WHERE is_default = 1;

        CREATE TABLE standards (
            id INTEGER PRIMARY KEY,
            system_id INTEGER NOT NULL REFERENCES systems(id),
            name TEXT NOT NULL,
            version TEXT NOT NULL,
            description TEXT,
            generation_granularity TEXT NOT NULL DEFAULT 'section',
            UNIQUE(system_id, name, version)
        );

        CREATE TABLE domains (
            id INTEGER PRIMARY KEY,
            standard_id INTEGER NOT NULL REFERENCES standards(id),
            key TEXT NOT NULL,
            name TEXT NOT NULL,
            tier INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            description TEXT,
            content_kind TEXT NOT NULL DEFAULT 'documentation',
            UNIQUE(standard_id, key)
        );

        CREATE TABLE section_catalog (
            id INTEGER PRIMARY KEY,
            domain_id INTEGER NOT NULL REFERENCES domains(id),
            semantic_type TEXT NOT NULL,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            mandatory INTEGER NOT NULL DEFAULT 1,
            aliases TEXT,
            UNIQUE(domain_id, semantic_type)
        );

        CREATE TABLE rules (
            id INTEGER PRIMARY KEY,
            standard_id INTEGER NOT NULL,
            domain_id INTEGER NOT NULL,
            section_catalog_id INTEGER,
            rule_key TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind IN ('deterministic','semantic')),
            scope TEXT NOT NULL CHECK (scope IN ('document','section')),
            description TEXT NOT NULL,
            condition TEXT NOT NULL,
            message TEXT NOT NULL,
            severity TEXT NOT NULL CHECK (severity IN ('error','warning','suggestion','info')),
            weight REAL NOT NULL DEFAULT 1.0,
            mandatory INTEGER NOT NULL DEFAULT 0,
            evidence_type TEXT NOT NULL,
            script_check_id INTEGER,
            is_fallback INTEGER NOT NULL DEFAULT 0,
            section_catalog_key INTEGER GENERATED ALWAYS AS (COALESCE(section_catalog_id, 0)) VIRTUAL,
            UNIQUE(standard_id, domain_id, section_catalog_key, scope, kind, rule_key)
        );

        CREATE TABLE relationship_types (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            tier_gating TEXT NOT NULL CHECK (tier_gating IN ('strict','none'))
        );

        CREATE TABLE domain_relationships (
            id INTEGER PRIMARY KEY,
            standard_id INTEGER NOT NULL,
            from_domain_id INTEGER NOT NULL,
            to_domain_id INTEGER NOT NULL,
            relationship_type_id INTEGER NOT NULL,
            mutual INTEGER NOT NULL DEFAULT 0,
            enforce_order INTEGER NOT NULL DEFAULT 0,
            note TEXT,
            UNIQUE(standard_id, from_domain_id, to_domain_id, relationship_type_id)
        );

        CREATE TABLE rule_evidence_params (
            id          INTEGER PRIMARY KEY,
            rule_id     INTEGER NOT NULL REFERENCES rules(id),
            param_key   TEXT NOT NULL,
            param_value TEXT NOT NULL,
            sort_order  INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE calculation_rules (
            id                    INTEGER PRIMARY KEY,
            standard_id           INTEGER NOT NULL REFERENCES standards(id),
            bucket                TEXT NOT NULL,
            calculation_method    TEXT NOT NULL,
            scope                 TEXT CHECK (scope IS NULL OR scope IN ('document','section')),
            formula               TEXT NOT NULL,
            rollup                TEXT,
            tolerance_method      TEXT,
            tolerance_k           REAL,
            tolerance_floor       REAL,
            tolerance_scope       TEXT,
            min_samples           INTEGER,
            fallback_scope        TEXT,
            fallback_min_samples  INTEGER,
            note                  TEXT,
            UNIQUE(standard_id, bucket)
        );

        CREATE TABLE calculation_inputs (
            id                   INTEGER PRIMARY KEY,
            calculation_rule_id  INTEGER NOT NULL REFERENCES calculation_rules(id),
            name                 TEXT NOT NULL,
            weight               REAL NOT NULL,
            sort_order           INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE score_bands (
            id           INTEGER PRIMARY KEY,
            standard_id  INTEGER NOT NULL REFERENCES standards(id),
            rating       TEXT NOT NULL,
            min_score    REAL NOT NULL,
            max_score    REAL NOT NULL,
            sort_order   INTEGER NOT NULL DEFAULT 0,
            UNIQUE(standard_id, rating)
        );

        CREATE TABLE section_dependencies (
            id INTEGER PRIMARY KEY,
            standard_id INTEGER NOT NULL,
            domain_id INTEGER NOT NULL,
            section_catalog_id INTEGER NOT NULL,
            depends_on_section_id INTEGER NOT NULL,
            UNIQUE(standard_id, section_catalog_id, depends_on_section_id)
        );",
    )
    .unwrap();

    // Insert default system + documentation-standards standard.
    conn.execute(
        "INSERT INTO systems (name, description, is_default) VALUES ('samgraha-documentation', 'Documentation knowledge hub', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO standards (system_id, name, version, description) VALUES (1, 'documentation-standards', '1.0.0', 'Samgraha documentation standards')",
        [],
    )
    .unwrap();

    // Insert all 13 domains.
    let domains = vec![
        ("architecture", "Architecture", 2),
        ("feature", "Feature", 2),
        ("vision", "Vision", 1),
        ("engineering", "Engineering", 2),
        ("readme", "Readme", 2),
        ("design", "Design", 2),
        ("external-context", "External Context", 3),
        ("prototype", "Prototype", 2),
        ("help", "Help", 2),
        ("philosophy", "Philosophy", 1),
        ("feature-design", "Feature Design", 3),
        ("feature-technical", "Feature Technical", 3),
        ("standards", "Standards", 2),
    ];
    for (i, (key, name, tier)) in domains.iter().enumerate() {
        conn.execute(
            "INSERT INTO domains (standard_id, key, name, tier, sort_order) VALUES (1, ?1, ?2, ?3, ?4)",
            rusqlite::params![key, name, tier, i as i64],
        )
        .unwrap();
    }

    // Architecture section_catalog (matching the old builtin).
    let arch_domain_id = 1i64; // architecture is first
    let arch_sections = vec![
        ("purpose", "Purpose", true),
        ("system_overview", "System Overview", true),
        ("component_model", "Component Model", true),
        ("communication_paths", "Communication", true),
        ("data_flow", "Data Flow", true),
        ("security_considerations", "Security", true),
        ("rationale", "Rationale", false),
        ("constraints", "Constraints", false),
        ("traceability", "Traceability", false),
    ];
    for (i, (sem, name, mand)) in arch_sections.iter().enumerate() {
        conn.execute(
            "INSERT INTO section_catalog (domain_id, semantic_type, name, sort_order, mandatory) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![arch_domain_id, sem, name, i as i64, *mand as i64],
        )
        .unwrap();
    }

    // Architecture deterministic rules.
    let arch_rules = vec![
        ("A1", "Has overview", "Architecture must include system overview", "error", "section_presence", Some(2i64)),
        ("A2", "Has component model", "Architecture must define component responsibilities", "error", "section_presence", Some(3i64)),
        ("A3", "No implementation details", "Architecture must avoid implementation specifics", "warning", "keyword_absence", None),
        ("A4", "Has security", "Architecture must address security", "warning", "section_presence", Some(6i64)),
    ];
    for (key, desc, msg, sev, ev_type, sc_id) in arch_rules {
        if let Some(section_id) = sc_id {
            conn.execute(
                "INSERT INTO rules (standard_id, domain_id, section_catalog_id, rule_key, kind, scope, description, condition, message, severity, evidence_type) VALUES (1, 1, ?1, ?2, 'deterministic', 'section', ?3, '', ?4, ?5, ?6)",
                rusqlite::params![section_id, key, desc, msg, sev, ev_type],
            ).unwrap();
        } else {
            conn.execute(
                "INSERT INTO rules (standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type) VALUES (1, 1, ?1, 'deterministic', 'document', ?2, '', ?3, ?4, ?5)",
                rusqlite::params![key, desc, msg, sev, ev_type],
            ).unwrap();
        }
    }

    // Vision section_catalog.
    let vision_domain_id = 3i64; // vision is third
    let vision_sections = vec![
        ("purpose", "Purpose", true),
        ("vision_statement", "Vision", true),
        ("problem", "Problem", true),
        ("solution", "Solution", true),
        ("target_audience", "Target Audience", true),
        ("success_criteria", "Success Criteria", false),
        ("traceability", "Traceability", false),
    ];
    for (i, (sem, name, mand)) in vision_sections.iter().enumerate() {
        conn.execute(
            "INSERT INTO section_catalog (domain_id, semantic_type, name, sort_order, mandatory) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![vision_domain_id, sem, name, i as i64, *mand as i64],
        )
        .unwrap();
    }

    // Vision deterministic rules.
    let vision_rules = vec![
        ("vis-001", "Has purpose", "Vision must include product purpose", "error", "section_presence", Some(1i64)),
        ("vis-002", "Has audience", "Vision must define target audience", "warning", "section_presence", Some(5i64)),
        ("vis-003", "No implementation", "Vision must not contain implementation details", "warning", "keyword_absence", None),
    ];
    for (key, desc, msg, sev, ev_type, sc_id) in vision_rules {
        if let Some(section_id) = sc_id {
            conn.execute(
                "INSERT INTO rules (standard_id, domain_id, section_catalog_id, rule_key, kind, scope, description, condition, message, severity, evidence_type) VALUES (1, 3, ?1, ?2, 'deterministic', 'section', ?3, '', ?4, ?5, ?6)",
                rusqlite::params![section_id, key, desc, msg, sev, ev_type],
            ).unwrap();
        } else {
            conn.execute(
                "INSERT INTO rules (standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type) VALUES (1, 3, ?1, 'deterministic', 'document', ?2, '', ?3, ?4, ?5)",
                rusqlite::params![key, desc, msg, sev, ev_type],
            ).unwrap();
        }
    }

    // Relationship types.
    conn.execute(
        "INSERT INTO relationship_types (name, description, tier_gating) VALUES ('derives', 'Derived from', 'none')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO relationship_types (name, description, tier_gating) VALUES ('guides', 'Guides', 'none')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO relationship_types (name, description, tier_gating) VALUES ('implements', 'Implements', 'none')",
        [],
    )
    .unwrap();

    // Domain relationships.
    conn.execute(
        "INSERT INTO domain_relationships (standard_id, from_domain_id, to_domain_id, relationship_type_id) VALUES (1, 1, 2, 1)",
        [], // architecture -> feature (derives)
    )
    .unwrap();
    conn.execute(
        "INSERT INTO domain_relationships (standard_id, from_domain_id, to_domain_id, relationship_type_id) VALUES (1, 3, 2, 1)",
        [], // vision -> feature (derives)
    )
    .unwrap();
    conn.execute(
        "INSERT INTO domain_relationships (standard_id, from_domain_id, to_domain_id, relationship_type_id) VALUES (1, 1, 4, 2)",
        [], // architecture -> engineering (guides)
    )
    .unwrap();

    // Default scoring config for tests.
    conn.execute(
        "INSERT INTO calculation_rules (standard_id, bucket, calculation_method, scope, formula) VALUES (1, 'deterministic_whole', 'weighted_pass_rate', 'document', '100 * sum(weight where passed) / sum(all weights)')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO calculation_rules (standard_id, bucket, calculation_method, scope, formula) VALUES (1, 'deterministic_section', 'weighted_pass_rate', 'section', '100 * sum(weight where passed) / sum(all weights)')",
        [],
    ).unwrap();
    conn.execute(
        "INSERT INTO calculation_rules (standard_id, bucket, calculation_method, formula) VALUES (1, 'final_score', 'weighted_sum', '0.25 * dw + 0.25 * ds + 0.25 * sw + 0.25 * ss')",
        [],
    ).unwrap();
    conn.execute("INSERT INTO calculation_inputs (calculation_rule_id, name, weight, sort_order) VALUES (3, 'deterministic_whole', 25.0, 1)", []).unwrap();
    conn.execute("INSERT INTO calculation_inputs (calculation_rule_id, name, weight, sort_order) VALUES (3, 'deterministic_section', 25.0, 2)", []).unwrap();
    conn.execute("INSERT INTO calculation_inputs (calculation_rule_id, name, weight, sort_order) VALUES (3, 'semantic_whole', 25.0, 3)", []).unwrap();
    conn.execute("INSERT INTO calculation_inputs (calculation_rule_id, name, weight, sort_order) VALUES (3, 'semantic_section', 25.0, 4)", []).unwrap();
    let bands = vec![
        ("Excellent", 95.0, 100.0),
        ("Very Good", 90.0, 94.99),
        ("Good", 80.0, 89.99),
        ("Acceptable", 70.0, 79.99),
        ("Needs Improvement", 0.0, 69.99),
    ];
    for (i, (rating, min, max)) in bands.iter().enumerate() {
        conn.execute(
            "INSERT INTO score_bands (standard_id, rating, min_score, max_score, sort_order) VALUES (1, ?1, ?2, ?3, ?4)",
            rusqlite::params![rating, min, max, i as i64],
        ).unwrap();
    }

    tmp
}
