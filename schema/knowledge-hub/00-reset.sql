-- Reset: drop all tables in reverse dependency order.
-- Run before a fresh load; safe to execute on an empty database.

DROP TABLE IF EXISTS workflow_phase_dependencies;
DROP TABLE IF EXISTS workflow_phases;
DROP TABLE IF EXISTS workflow_use_cases;
DROP TABLE IF EXISTS plan_generation_inputs;
DROP TABLE IF EXISTS validation_rules;
DROP TABLE IF EXISTS plan_scenarios;
DROP TABLE IF EXISTS plan_settings;
DROP TABLE IF EXISTS score_bands;
DROP TABLE IF EXISTS calculation_inputs;
DROP TABLE IF EXISTS calculation_rules;
DROP TABLE IF EXISTS standard_docs;
DROP TABLE IF EXISTS templates;
DROP TABLE IF EXISTS rule_evidence_params;
DROP TABLE IF EXISTS rules;
DROP TABLE IF EXISTS script_runs;
DROP TABLE IF EXISTS script_checks;
DROP TABLE IF EXISTS section_catalog;
DROP TABLE IF EXISTS domain_relationships;
DROP TABLE IF EXISTS relationship_types;
DROP TABLE IF EXISTS domains;
DROP TABLE IF EXISTS standards;
DROP TABLE IF EXISTS systems;
