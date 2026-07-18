use crate::StandardRegistry;
use anyhow::{bail, Context, Result};
use rusqlite::Connection;
use schemas::audit::{CalculationInput, CalculationRule, ScoreBand, ScoringConfig, ValidationRule};
use schemas::standard::{AuditRuleDef, SectionDefinition, StandardDefinition, StandardDoc, StandardRelationship, PlanSetting, PlanScenario, ScriptCheck};
use std::collections::HashMap;

/// Must match `SCHEMA_VERSION` in `schema/knowledge-hub/knowledge-hub-loader.py`.
/// Bump both together whenever a table is added/removed/changes shape.
pub const EXPECTED_SCHEMA_VERSION: i64 = 2;

/// Reject a standards.db whose `PRAGMA user_version` doesn't match what this
/// build of samgraha understands — a version mismatch means the DB was
/// written by a different schema shape (e.g. before/after the 09/10/13-15
/// runtime-table removal), and reading it would otherwise fail with a
/// confusing "no such table"/"no such column" deep inside a query instead of
/// a clear message up front. `0` (SQLite's default when never set) means the
/// DB predates version tracking entirely — same treatment, reject with a
/// message that says so rather than a generic mismatch.
pub fn check_schema_version(conn: &Connection) -> Result<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version == 0 {
        bail!(
            "standards.db has no schema version set (PRAGMA user_version = 0) — \
             it predates version tracking. Re-register from the source knowledge-hub \
             directory with the current loader to get a versioned DB."
        );
    }
    if version != EXPECTED_SCHEMA_VERSION {
        bail!(
            "standards.db schema version {} does not match what this build expects ({}) — \
             re-register from the source knowledge-hub directory with the current loader.",
            version,
            EXPECTED_SCHEMA_VERSION
        );
    }
    Ok(())
}

/// Load semantic rubrics from the `templates` table — rows where
/// `kind = 'audit_report'` and `audit_bucket = 'semantic'` hold full
/// rubric markdown keyed by `"{domain_key}/{section_semantic_type}"`.
pub fn load_semantic_rubrics(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut rubrics = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT d.key, sc.semantic_type, t.content
         FROM templates t
         JOIN domains d ON t.domain_id = d.id
         LEFT JOIN section_catalog sc ON t.section_catalog_id = sc.id
         WHERE t.kind = 'audit_report' AND t.audit_bucket = 'semantic'",
    )?;
    let rows = stmt.query_map([], |row| {
        let domain_key: String = row.get(0)?;
        let semantic_type: String = row.get(1)?;
        let content: String = row.get(2)?;
        Ok((domain_key, semantic_type, content))
    })?;
    for row in rows {
        let (domain_key, semantic_type, content) = row?;
        let key = format!("{}/{}", domain_key, semantic_type);
        rubrics.insert(key, content);
    }
    Ok(rubrics)
}

/// Load scoring configuration from `calculation_rules`, `calculation_inputs`,
/// and `score_bands` tables.
pub fn load_scoring_config(conn: &Connection) -> Result<ScoringConfig> {
    let mut calc_rules = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT bucket, calculation_method, formula FROM calculation_rules",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CalculationRule {
            bucket: row.get(0)?,
            calculation_method: row.get(1)?,
            formula: row.get(2)?,
        })
    })?;
    for row in rows {
        calc_rules.push(row?);
    }

    // Was `WHERE cr.bucket = 'final_score'` — a literal bucket-name string
    // that broke silently the moment bucket names stopped being a fixed
    // hardcoded set (knowledge-hub-loader.py's Pass 7 now derives bucket
    // names from each calculation file's own path, e.g.
    // "summary/final_score.yaml" -> "summary_final_score", not "final_score").
    // `calculation_method` is the actual semantic marker for "this is the
    // weighted-sum bucket" — a name-agnostic property every standard's
    // final-score-shaped bucket shares, not a name every standard has to
    // spell the same way.
    let mut calc_inputs = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT ci.name, ci.weight FROM calculation_inputs ci
         JOIN calculation_rules cr ON ci.calculation_rule_id = cr.id
         WHERE cr.calculation_method = 'weighted_sum' ORDER BY ci.sort_order",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CalculationInput {
            name: row.get(0)?,
            weight: row.get(1)?,
        })
    })?;
    for row in rows {
        calc_inputs.push(row?);
    }

    let mut bands = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT rating, min_score, max_score FROM score_bands ORDER BY sort_order",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ScoreBand {
            rating: row.get(0)?,
            min_score: row.get(1)?,
            max_score: row.get(2)?,
        })
    })?;
    for row in rows {
        bands.push(row?);
    }

    let mut validation_rules = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT check_key, name, description, rule, severity, invalidate_audit
         FROM validation_rules ORDER BY sort_order",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ValidationRule {
            check_key: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            rule: row.get(3)?,
            severity: row.get(4)?,
            invalidate_audit: row.get(5)?,
        })
    })?;
    for row in rows {
        validation_rules.push(row?);
    }

    Ok(ScoringConfig {
        calculation_rules: calc_rules,
        calculation_inputs: calc_inputs,
        score_bands: bands,
        validation_rules,
    })
}

pub fn load_plan_settings(conn: &Connection, standard_id: i64) -> Result<Vec<PlanSetting>> {
    let mut settings = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT threshold_rating, max_iterations, fallback, note FROM plan_settings WHERE standard_id = ?"
    )?;
    let rows = stmt.query_map([standard_id], |row| {
        Ok(PlanSetting {
            threshold_rating: row.get(0)?,
            max_iterations: row.get(1)?,
            fallback: row.get(2)?,
            note: row.get(3)?,
        })
    })?;
    for row in rows {
        settings.push(row?);
    }
    Ok(settings)
}

pub fn load_plan_scenarios(conn: &Connection, standard_id: i64) -> Result<Vec<PlanScenario>> {
    let mut scenarios = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT repo_state, doc_state, tier, step, content FROM plan_scenarios WHERE standard_id = ?"
    )?;
    let rows = stmt.query_map([standard_id], |row| {
        Ok(PlanScenario {
            repo_state: row.get(0)?,
            doc_state: row.get(1)?,
            tier: row.get(2)?,
            step: row.get(3)?,
            content: row.get(4)?,
        })
    })?;
    for row in rows {
        scenarios.push(row?);
    }
    Ok(scenarios)
}

/// Load the `standard_docs` table — the human-readable documentation-standards
/// spec content, one row per domain, keyed by the domain's `key`.
pub fn load_standard_docs(conn: &Connection, standard_id: i64) -> Result<HashMap<String, StandardDoc>> {
    let mut docs = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT d.key, sd.title, sd.content, sd.source_file
         FROM standard_docs sd
         JOIN domains d ON sd.domain_id = d.id
         WHERE d.standard_id = ?",
    )?;
    let rows = stmt.query_map([standard_id], |row| {
        let domain: String = row.get(0)?;
        Ok((
            domain.clone(),
            StandardDoc {
                domain,
                title: row.get(1)?,
                content: row.get(2)?,
                source_file: row.get(3)?,
            },
        ))
    })?;
    for row in rows {
        let (domain, doc) = row?;
        docs.insert(domain, doc);
    }
    Ok(docs)
}

pub fn load_script_checks(conn: &Connection, standard_id: i64) -> Result<Vec<ScriptCheck>> {
    let mut checks = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT sc.check_name, d.key as domain_key, sc.category, sc.timeout_seconds, sc.requires_network, sc.result_schema, sc.description 
         FROM script_checks sc 
         LEFT JOIN domains d ON sc.domain_id = d.id 
         WHERE sc.standard_id = ?"
    )?;
    let rows = stmt.query_map([standard_id], |row| {
        let req_network: i32 = row.get(4)?;
        Ok(ScriptCheck {
            check_name: row.get(0)?,
            domain_id: row.get(1)?,
            category: row.get(2)?,
            timeout_seconds: row.get(3)?,
            requires_network: req_network != 0,
            result_schema: row.get(5)?,
            description: row.get(6)?,
        })
    })?;
    for row in rows {
        checks.push(row?);
    }
    Ok(checks)
}

/// Load a `StandardRegistry` from a `schema/knowledge-hub`-shaped SQLite
/// database, projecting rows onto the existing `StandardDefinition` structs.
/// All deterministic rules survive (Phase 4); semantic rules are skipped.
///
/// `system_name`: when `Some`, selects the system by name; when `None`,
/// falls back to the system marked `is_default = 1`. This corresponds to
/// `samgraha.toml [repository.documentation] standard_system = "..."`.
pub fn from_standards_db(conn: &Connection, system_name: Option<&str>) -> Result<StandardRegistry> {
    let mut registry = StandardRegistry::new();

    // Find the standard for the requested (or default) system.
    let standard_id: i64 = if let Some(name) = system_name {
        conn.query_row(
            "SELECT s.id FROM standards s
             JOIN systems sys ON s.system_id = sys.id
             WHERE sys.name = ? AND s.name = 'documentation-standards'
             LIMIT 1",
            [name],
            |row| row.get(0),
        )
        .with_context(|| format!("No documentation-standards found for system '{}' in DB", name))?
    } else {
        conn.query_row(
            "SELECT s.id FROM standards s
             JOIN systems sys ON s.system_id = sys.id
             WHERE sys.is_default = 1 AND s.name = 'documentation-standards'
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .context("No default documentation-standards found in DB")?
    };

    let standard_version: String = conn
        .query_row(
            "SELECT version FROM standards WHERE id = ?",
            [standard_id],
            |row| row.get(0),
        )?;

    // Load all domains for this standard.
    let mut stmt_domains = conn.prepare(
        "SELECT id, key, name, description, tier FROM domains
         WHERE standard_id = ? ORDER BY sort_order",
    )?;

    let domains: Vec<(i64, String, String, String, i64)> = stmt_domains
        .query_map([standard_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, Option<String>>(3)?
                    .unwrap_or_default(),
                row.get(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to query domains")?;

    // Pre-load all section_catalog rows, grouped by domain_id.
    let mut sections_by_domain: std::collections::HashMap<i64, Vec<SectionDefinition>> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT domain_id, name, semantic_type, aliases, mandatory
             FROM section_catalog WHERE domain_id IN (SELECT id FROM domains WHERE standard_id = ?)
             ORDER BY sort_order",
        )?;
        let rows = stmt.query_map([standard_id], |row| {
            let domain_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let semantic_type: String = row.get(2)?;
            let aliases_raw: Option<String> = row.get(3)?;
            let mandatory: bool = row.get(4)?;
            Ok((domain_id, name, semantic_type, aliases_raw, mandatory))
        })?;
        for row in rows {
            let (domain_id, name, semantic_type, aliases_raw, mandatory) = row?;
            let aliases = aliases_raw
                .map(|a| {
                    a.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            sections_by_domain.entry(domain_id).or_default().push(
                SectionDefinition {
                    canonical_name: name,
                    semantic_type,
                    aliases,
                    required: mandatory,
                    description: String::new(), // no description column in section_catalog
                },
            );
        }
    }

    // Pre-load rule_evidence_params, grouped by rule_id.
    let mut params_by_rule: std::collections::HashMap<i64, std::collections::HashMap<String, String>> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT rule_id, param_key, param_value FROM rule_evidence_params ORDER BY rule_id, sort_order",
        )?;
        let rows = stmt.query_map([], |row| {
            let rule_id: i64 = row.get(0)?;
            let key: String = row.get(1)?;
            let value: String = row.get(2)?;
            Ok((rule_id, key, value))
        })?;
        for row in rows {
            let (rule_id, key, value) = row?;
            // rule_evidence_params stores one row per value for multi-value
            // params (e.g. a rule's evidence.paths: [a, b] list) sharing the
            // same param_key, ordered by sort_order — every consumer of
            // AuditRuleDef.params (providers.rs's evidence-type match arms)
            // already expects a single comma-joined string it can .split(',')
            // on, so accumulate here rather than overwrite (plain `insert`
            // silently kept only the last row for any multi-value param).
            let params = params_by_rule.entry(rule_id).or_default();
            params
                .entry(key)
                .and_modify(|v: &mut String| {
                    v.push(',');
                    v.push_str(&value);
                })
                .or_insert(value);
        }
    }

    // Pre-load rules (both kinds), grouped by domain_id and split by kind.
    // Was `WHERE r.kind = 'deterministic'` — semantic rows (kind='semantic',
    // evidence_type='llm_judgment', matching audit/semantic/document/*.yaml's
    // real shape) were silently never loaded into anything, so nothing could
    // ever execute them. Same `rules` table, same evidence_type/params shape
    // either kind — no new query needed, just no longer filtering one kind
    // out.
    let mut rules_by_domain: std::collections::HashMap<i64, Vec<AuditRuleDef>> =
        std::collections::HashMap::new();
    let mut semantic_rules_by_domain: std::collections::HashMap<i64, Vec<AuditRuleDef>> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.domain_id, r.rule_key, r.description, r.severity,
                    r.evidence_type, r.scope, r.section_catalog_id,
                    r.weight, r.mandatory, r.kind,
                    sc.name AS section_name
             FROM rules r
             LEFT JOIN section_catalog sc ON r.section_catalog_id = sc.id
             WHERE r.standard_id = ?
             ORDER BY r.domain_id, r.rule_key",
        )?;
        let rows = stmt.query_map([standard_id], |row| {
            let rule_id: i64 = row.get(0)?;
            let domain_id: i64 = row.get(1)?;
            let rule_key: String = row.get(2)?;
            let description: String = row.get(3)?;
            let severity: String = row.get(4)?;
            let evidence_type: String = row.get(5)?;
            let _scope: String = row.get(6)?;
            let section_catalog_id: Option<i64> = row.get(7)?;
            let weight: f64 = row.get(8)?;
            let mandatory: bool = row.get(9)?;
            let kind: String = row.get(10)?;
            let section_name: Option<String> = row.get(11)?;
            Ok((
                rule_id, domain_id, rule_key, description, severity,
                evidence_type, _scope, section_catalog_id, weight, mandatory, kind, section_name,
            ))
        })?;
        for row in rows {
            let (
                rule_id, domain_id, rule_key, description, severity,
                evidence_type, _scope, _section_catalog_id, weight, mandatory, kind, section_name,
            ) = row?;

            // For section_presence checks, scope = heading text from section_catalog.
            // For all others, scope is the raw DB scope value.
            let scope = if evidence_type == "section_presence" {
                section_name.unwrap_or_default()
            } else {
                _scope
            };

            let params = params_by_rule.remove(&rule_id).unwrap_or_default();

            let rule = AuditRuleDef {
                id: rule_key.clone(),
                name: description.clone(),
                description,
                severity,
                evidence_type,
                scope,
                weight,
                mandatory,
                params,
            };

            let target = if kind == "semantic" { &mut semantic_rules_by_domain } else { &mut rules_by_domain };
            target.entry(domain_id).or_default().push(rule);
        }
    }

    // Pre-load relationships, grouped by from_domain_id.
    let mut rels_by_domain: std::collections::HashMap<i64, Vec<StandardRelationship>> =
        std::collections::HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT dr.from_domain_id,
                    from_d.key AS from_key,
                    to_d.key AS to_key,
                    rt.name AS rel_name,
                    dr.enforce_order,
                    rt.tier_gating
             FROM domain_relationships dr
             JOIN domains from_d ON dr.from_domain_id = from_d.id
             JOIN domains to_d ON dr.to_domain_id = to_d.id
             JOIN relationship_types rt ON dr.relationship_type_id = rt.id
             WHERE dr.standard_id = ?",
        )?;
        let rows = stmt.query_map([standard_id], |row| {
            let from_domain_id: i64 = row.get(0)?;
            let from_key: String = row.get(1)?;
            let to_key: String = row.get(2)?;
            let rel_name: String = row.get(3)?;
            let enforce_order: bool = row.get(4)?;
            let tier_gating: String = row.get(5)?;
            Ok((from_domain_id, from_key, to_key, rel_name, enforce_order, tier_gating))
        })?;
        for row in rows {
            let (from_domain_id, from_key, to_key, rel_name, enforce_order, tier_gating) = row?;
            rels_by_domain
                .entry(from_domain_id)
                .or_default()
                .push(StandardRelationship {
                    from_domain: from_key,
                    to_domain: to_key,
                    relationship: rel_name,
                    enforce_order,
                    tier_gating_strict: tier_gating == "strict",
                });
        }
    }

    // Assemble one StandardDefinition per domain.
    for (domain_id, domain_key, domain_name, domain_desc, domain_tier) in &domains {
        let required_sections = sections_by_domain
            .remove(domain_id)
            .unwrap_or_default();
        let audit_rules = rules_by_domain.remove(domain_id).unwrap_or_default();
        let semantic_rules = semantic_rules_by_domain.remove(domain_id).unwrap_or_default();
        let relationships = rels_by_domain.remove(domain_id).unwrap_or_default();

        let std = StandardDefinition {
            id: domain_key.clone(),
            name: domain_name.clone(),
            version: standard_version.clone(),
            domain: domain_key.clone(),
            description: domain_desc.clone(),
            required_sections,
            prohibited_content: Vec::new(), // not in knowledge-hub schema
            relationships,
            audit_rules,
            semantic_rules,
            profiles: Vec::new(), // not in knowledge-hub schema
            tier: Some(*domain_tier as i32),
        };
        registry.register(std);
    }

    // Load semantic rubrics from templates table.
    match load_semantic_rubrics(conn) {
        Ok(rubrics) => {
            let count = rubrics.len();
            registry.set_rubrics(rubrics);
            tracing::info!("Loaded {} semantic rubrics from templates", count);
        }
        Err(e) => {
            tracing::warn!("Failed to load semantic rubrics: {}", e);
        }
    }

    // Load scoring configuration.
    match load_scoring_config(conn) {
        Ok(scoring) => {
            let count = scoring.calculation_rules.len();
            registry.set_scoring(scoring);
            tracing::info!("Loaded scoring config ({} calculation rules)", count);
        }
        Err(e) => {
            tracing::warn!("Failed to load scoring config: {}", e);
        }
    }

    match load_plan_settings(conn, standard_id) {
        Ok(settings) => {
            let count = settings.len();
            registry.set_plan_settings(settings);
            tracing::info!("Loaded {} plan settings", count);
        }
        Err(e) => tracing::warn!("Failed to load plan settings: {}", e),
    }

    match load_plan_scenarios(conn, standard_id) {
        Ok(scenarios) => {
            let count = scenarios.len();
            registry.set_plan_scenarios(scenarios);
            tracing::info!("Loaded {} plan scenarios", count);
        }
        Err(e) => tracing::warn!("Failed to load plan scenarios: {}", e),
    }

    match load_script_checks(conn, standard_id) {
        Ok(checks) => {
            let count = checks.len();
            registry.set_script_checks(checks);
            tracing::info!("Loaded {} script checks", count);
        }
        Err(e) => tracing::warn!("Failed to load script checks: {}", e),
    }

    match load_standard_docs(conn, standard_id) {
        Ok(docs) => {
            let count = docs.len();
            registry.set_standard_docs(docs);
            tracing::info!("Loaded {} standard docs", count);
        }
        Err(e) => tracing::warn!("Failed to load standard docs: {}", e),
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full pipeline test: create an in-memory DB with known data, load via
    /// from_standards_db(), verify all projections.
    #[test]
    fn test_from_standards_db_projection() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE systems (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                description TEXT, is_default INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE standards (
                id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                UNIQUE(system_id, name, version)
            );
            CREATE TABLE domains (
                id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL,
                key TEXT NOT NULL, name TEXT NOT NULL, tier INTEGER NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0, description TEXT,
                UNIQUE(standard_id, key)
            );
            CREATE TABLE relationship_types (
                id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL,
                name TEXT NOT NULL, tier_gating TEXT NOT NULL,
                description TEXT, UNIQUE(standard_id, name)
            );
            CREATE TABLE domain_relationships (
                id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL,
                from_domain_id INTEGER NOT NULL, to_domain_id INTEGER NOT NULL,
                relationship_type_id INTEGER INTEGER NOT NULL,
                mutual INTEGER NOT NULL DEFAULT 0,
                enforce_order INTEGER NOT NULL DEFAULT 0, note TEXT,
                UNIQUE(standard_id, from_domain_id, to_domain_id, relationship_type_id)
            );
            CREATE TABLE section_catalog (
                id INTEGER PRIMARY KEY, domain_id INTEGER NOT NULL,
                semantic_type TEXT NOT NULL, name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                mandatory INTEGER NOT NULL DEFAULT 1, aliases TEXT,
                UNIQUE(domain_id, semantic_type)
            );
            CREATE TABLE rules (
                id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL,
                domain_id INTEGER NOT NULL, section_catalog_id INTEGER,
                rule_key TEXT NOT NULL, kind TEXT NOT NULL,
                scope TEXT NOT NULL, description TEXT NOT NULL,
                condition TEXT NOT NULL, message TEXT NOT NULL,
                severity TEXT NOT NULL, weight REAL NOT NULL DEFAULT 1.0,
                mandatory INTEGER NOT NULL DEFAULT 0,
                evidence_type TEXT NOT NULL, script_check_id INTEGER,
                is_fallback INTEGER NOT NULL DEFAULT 0,
                section_catalog_key INTEGER GENERATED ALWAYS AS (COALESCE(section_catalog_id, 0)) VIRTUAL,
                UNIQUE(standard_id, domain_id, section_catalog_key, scope, kind, rule_key)
            );
            CREATE TABLE rule_evidence_params (
                id INTEGER PRIMARY KEY,
                rule_id INTEGER NOT NULL,
                param_key TEXT NOT NULL,
                param_value TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0
            );",
        )
        .unwrap();

        // Seed data.
        conn.execute("INSERT INTO systems (id, name, is_default) VALUES (1, 'test', 1)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO standards (id, system_id, name, version, description) VALUES (1, 1, 'documentation-standards', '1.0.0', 'Test standard')",
            [],
        )
        .unwrap();

        // Two domains.
        conn.execute(
            "INSERT INTO domains (id, standard_id, key, name, tier, description) VALUES (1, 1, 'vision', 'Vision', 1, 'Product vision')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO domains (id, standard_id, key, name, tier, description) VALUES (2, 1, 'architecture', 'Architecture', 2, 'System architecture')",
            [],
        )
        .unwrap();

        // Sections for vision.
        conn.execute(
            "INSERT INTO section_catalog (id, domain_id, semantic_type, name, mandatory) VALUES (1, 1, 'purpose', 'Purpose', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO section_catalog (id, domain_id, semantic_type, name, mandatory) VALUES (2, 1, 'problem', 'Problem', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO section_catalog (id, domain_id, semantic_type, name, mandatory) VALUES (3, 1, 'constraints', 'Constraints', 0)",
            [],
        )
        .unwrap();

        // Relationship types.
        conn.execute(
            "INSERT INTO relationship_types (id, standard_id, name, tier_gating) VALUES (1, 1, 'derives', 'strict')",
            [],
        )
        .unwrap();

        // Relationship: vision derives architecture.
        conn.execute(
            "INSERT INTO domain_relationships (id, standard_id, from_domain_id, to_domain_id, relationship_type_id) VALUES (1, 1, 1, 2, 1)",
            [],
        )
        .unwrap();

        // Rules for vision.
        // section_presence → has_section (with section name)
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, section_catalog_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (1, 1, 1, 1, 'vis-001', 'deterministic', 'section', 'Has purpose', 'section exists', 'Must have purpose', 'error', 'section_presence')",
            [],
        )
        .unwrap();
        // keyword_absence → no_implementation
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (2, 1, 1, 'vis-002', 'deterministic', 'document', 'No implementation', 'keywords absent', 'No impl details', 'warning', 'keyword_absence')",
            [],
        )
        .unwrap();
        // content_check → kept (no longer dropped), with evidence params
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (3, 1, 1, 'vis-003', 'deterministic', 'document', 'Has overview', 'content present', 'Needs overview', 'error', 'content_check')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO rule_evidence_params (rule_id, param_key, param_value, sort_order) VALUES (3, 'keywords', 'overview,introduction,summary', 0)",
            [],
        )
        .unwrap();
        // section_presence without section_catalog_id → has_section with empty scope
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (4, 1, 1, 'vis-004', 'deterministic', 'document', 'Document exists', 'file present', 'Must exist', 'error', 'section_presence')",
            [],
        )
        .unwrap();
        // semantic rule → not loaded (kind='semantic')
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (5, 1, 1, 'vis-sem-001', 'semantic', 'document', 'Quality check', 'llm judges', 'Be good', 'warning', 'llm_judgment')",
            [],
        )
        .unwrap();

        // Rules for architecture: section_presence only.
        conn.execute(
            "INSERT INTO rules (id, standard_id, domain_id, rule_key, kind, scope, description, condition, message, severity, evidence_type)
             VALUES (6, 1, 2, 'arch-001', 'deterministic', 'document', 'Arch exists', 'file present', 'Must exist', 'error', 'section_presence')",
            [],
        )
        .unwrap();

        // Load from DB.
        let registry = from_standards_db(&conn, None).unwrap();

        // Should have 2 domains.
        let all = registry.all();
        assert_eq!(all.len(), 2, "Expected 2 domains, got {}", all.len());

        // Vision domain.
        let vision = registry.get("vision", "1.0.0").expect("vision not found");
        assert_eq!(vision.domain, "vision");
        assert_eq!(vision.name, "Vision");
        assert_eq!(vision.description, "Product vision");
        assert_eq!(vision.version, "1.0.0");
        assert_eq!(vision.required_sections.len(), 3);
        // Verify sections by semantic_type (order-independent).
        let section_types: Vec<&str> = vision.required_sections.iter().map(|s| s.semantic_type.as_str()).collect();
        assert!(section_types.contains(&"purpose"));
        assert!(section_types.contains(&"problem"));
        assert!(section_types.contains(&"constraints"));
        let purpose_section = vision.required_sections.iter().find(|s| s.semantic_type == "purpose").unwrap();
        assert_eq!(purpose_section.canonical_name, "Purpose");
        assert!(purpose_section.required);
        let constraints_section = vision.required_sections.iter().find(|s| s.semantic_type == "constraints").unwrap();
        assert!(!constraints_section.required);

        // Vision rules: all 4 deterministic survive (content_check no longer dropped), semantic dropped.
        assert_eq!(vision.audit_rules.len(), 4, "Expected 4 rules for vision, got {}", vision.audit_rules.len());
        let rule_ids: Vec<&str> = vision.audit_rules.iter().map(|r| r.id.as_str()).collect();
        assert!(rule_ids.contains(&"vis-001"));
        assert!(rule_ids.contains(&"vis-002"));
        assert!(rule_ids.contains(&"vis-003"));
        assert!(rule_ids.contains(&"vis-004"));

        // Check the section_presence rule has scope from section_catalog.
        let purpose_rule = vision.audit_rules.iter().find(|r| r.id == "vis-001").unwrap();
        assert_eq!(purpose_rule.evidence_type, "section_presence");
        assert_eq!(purpose_rule.scope, "Purpose");

        // Check the keyword_absence rule has scope from db.
        let no_impl_rule = vision.audit_rules.iter().find(|r| r.id == "vis-002").unwrap();
        assert_eq!(no_impl_rule.evidence_type, "keyword_absence");
        assert_eq!(no_impl_rule.scope, "document");

        // Check section_presence without section_catalog_id.
        let doc_rule = vision.audit_rules.iter().find(|r| r.id == "vis-004").unwrap();
        assert_eq!(doc_rule.evidence_type, "section_presence");
        assert_eq!(doc_rule.scope, ""); // no section catalog entry

        // Check content_check rule is kept with params.
        let cc_rule = vision.audit_rules.iter().find(|r| r.id == "vis-003").unwrap();
        assert_eq!(cc_rule.evidence_type, "content_check");
        assert_eq!(cc_rule.params.get("keywords").unwrap(), "overview,introduction,summary");

        // Relationships.
        assert_eq!(vision.relationships.len(), 1);
        assert_eq!(vision.relationships[0].from_domain, "vision");
        assert_eq!(vision.relationships[0].to_domain, "architecture");
        assert_eq!(vision.relationships[0].relationship, "derives");

        // Architecture domain.
        let arch = registry.get("architecture", "1.0.0").expect("architecture not found");
        assert_eq!(arch.audit_rules.len(), 1);
        assert_eq!(arch.required_sections.len(), 0);
        assert_eq!(arch.relationships.len(), 0);

        // No prohibited_content or profiles from DB.
        assert!(vision.prohibited_content.is_empty());
        assert!(vision.profiles.is_empty());
    }

    /// Verify load_semantic_rubrics reads templates with kind='audit_report'
    /// and audit_bucket='semantic', keyed by "{domain}/{semantic_type}".
    #[test]
    fn test_load_semantic_rubrics() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE systems (
                id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                description TEXT, is_default INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE standards (
                id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                UNIQUE(system_id, name, version)
            );
            CREATE TABLE domains (
                id INTEGER PRIMARY KEY, standard_id INTEGER NOT NULL,
                key TEXT NOT NULL, name TEXT NOT NULL, tier INTEGER NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0, description TEXT,
                UNIQUE(standard_id, key)
            );
            CREATE TABLE section_catalog (
                id INTEGER PRIMARY KEY, domain_id INTEGER NOT NULL,
                semantic_type TEXT NOT NULL, name TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                mandatory INTEGER NOT NULL DEFAULT 1, aliases TEXT,
                UNIQUE(domain_id, semantic_type)
            );
            CREATE TABLE templates (
                id INTEGER PRIMARY KEY,
                standard_id INTEGER NOT NULL,
                domain_id INTEGER NOT NULL,
                section_catalog_id INTEGER,
                kind TEXT NOT NULL,
                audit_bucket TEXT,
                scope TEXT NOT NULL,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                source_file TEXT,
                section_catalog_key INTEGER GENERATED ALWAYS AS (COALESCE(section_catalog_id, 0)) VIRTUAL,
                audit_bucket_key TEXT GENERATED ALWAYS AS (COALESCE(audit_bucket, '')) VIRTUAL,
                UNIQUE(standard_id, domain_id, section_catalog_key, kind, audit_bucket_key, scope)
            );",
        )
        .unwrap();

        conn.execute("INSERT INTO systems (id, name, is_default) VALUES (1, 'test', 1)", []).unwrap();
        conn.execute("INSERT INTO standards (id, system_id, name, version) VALUES (1, 1, 'documentation-standards', '1.0.0')", []).unwrap();
        conn.execute("INSERT INTO domains (id, standard_id, key, name, tier) VALUES (1, 1, 'vision', 'Vision', 1)", []).unwrap();
        conn.execute("INSERT INTO section_catalog (id, domain_id, semantic_type, name) VALUES (1, 1, 'purpose', 'Purpose')", []).unwrap();

        // Semantic rubric template — should be loaded.
        conn.execute(
            "INSERT INTO templates (standard_id, domain_id, section_catalog_id, kind, audit_bucket, scope, name, content)
             VALUES (1, 1, 1, 'audit_report', 'semantic', 'section', 'Vision Purpose Rubric', '# Rubric content')",
            [],
        ).unwrap();
        // Generation template — should NOT be loaded.
        conn.execute(
            "INSERT INTO templates (standard_id, domain_id, section_catalog_id, kind, audit_bucket, scope, name, content)
             VALUES (1, 1, 1, 'generation', NULL, 'section', 'Gen Template', 'gen content')",
            [],
        ).unwrap();
        // Deterministic audit template — should NOT be loaded.
        conn.execute(
            "INSERT INTO templates (standard_id, domain_id, kind, audit_bucket, scope, name, content)
             VALUES (1, 1, 'audit_report', 'deterministic', 'document', 'Det Template', 'det content')",
            [],
        ).unwrap();

        let rubrics = load_semantic_rubrics(&conn).unwrap();
        assert_eq!(rubrics.len(), 1);
        assert!(rubrics.contains_key("vision/purpose"));
        assert_eq!(rubrics["vision/purpose"], "# Rubric content");
    }
}
