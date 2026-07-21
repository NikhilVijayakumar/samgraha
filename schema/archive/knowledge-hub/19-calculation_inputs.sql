-- Calculation inputs: the weighted-sum inputs for a calculation_rules row
-- (currently only final_score has these — its four 25-point buckets), as
-- rows instead of an inline list.

CREATE TABLE calculation_inputs (
    id                   INTEGER PRIMARY KEY,
    calculation_rule_id  INTEGER NOT NULL REFERENCES calculation_rules(id) ON DELETE CASCADE,
    name                 TEXT NOT NULL,
    weight               REAL NOT NULL,
    sort_order           INTEGER NOT NULL DEFAULT 0
);
