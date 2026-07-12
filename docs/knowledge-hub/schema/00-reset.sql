-- Reset: drop all tables in reverse dependency order.
-- Run before a fresh load; safe to execute on an empty database.

DROP TABLE IF EXISTS scores;
DROP TABLE IF EXISTS audit_results;
DROP TABLE IF EXISTS rules;
DROP TABLE IF EXISTS sections;
DROP TABLE IF EXISTS documents;
DROP TABLE IF EXISTS standards;
DROP TABLE IF EXISTS systems;
