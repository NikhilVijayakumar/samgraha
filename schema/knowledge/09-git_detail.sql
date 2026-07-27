-- knowledge.db — git provenance: records the commit SHA, branch, and dirty
-- state of a repository at the time a step executed. One row per unique
-- (repo_root, commit_sha, dirty) combination; a step execution that
-- happens to run at the same commit as a prior run reuses the existing row
-- rather than creating a duplicate (UNIQUE constraint). Linked from the
-- `execution` table via `git_detail_id`.

CREATE TABLE IF NOT EXISTS git_detail (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_root   TEXT    NOT NULL,
    commit_sha  TEXT    NOT NULL,
    branch      TEXT    NOT NULL DEFAULT '',
    dirty       INTEGER NOT NULL DEFAULT 0,
    captured_at TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(repo_root, commit_sha, dirty)
);
