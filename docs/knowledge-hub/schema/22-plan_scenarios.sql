-- Plan scenarios: the repo-state x doc-state x tier x step matrix
-- (generation/audit/fix instructions for each combination), full content
-- per row rather than a path into plan/usecase/.

CREATE TABLE plan_scenarios (
    id           INTEGER PRIMARY KEY,
    standard_id  INTEGER NOT NULL REFERENCES standards(id),
    repo_state   TEXT NOT NULL CHECK (repo_state IN ('existing','new')),
    doc_state    TEXT NOT NULL CHECK (doc_state IN ('no_documentation','has_documentation')),
    tier         INTEGER NOT NULL,
    step         TEXT NOT NULL CHECK (step IN ('generation','audit','fix')),
    content      TEXT NOT NULL,
    UNIQUE(standard_id, repo_state, doc_state, tier, step)
);
