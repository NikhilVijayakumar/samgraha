-- Rule evidence params: a rule's evidence-extractor configuration as rows
-- instead of a JSON blob. Shape varies per evidence_type (e.g.
-- keyword_absence needs "categories", section_presence needs
-- "required_semantic_types") — multi-value params get one row per value,
-- sharing param_key, ordered by sort_order.

CREATE TABLE rule_evidence_params (
    id          INTEGER PRIMARY KEY,
    rule_id     INTEGER NOT NULL REFERENCES rules(id),
    param_key   TEXT NOT NULL,
    param_value TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);
