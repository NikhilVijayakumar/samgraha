# Knowledge Compilation

## Purpose

Transform engineering documentation into structured knowledge.

## Requirements

- Must be deterministic
- Must work offline
- Must produce SQLite index

## Acceptance Criteria

- Compilation completes without errors given valid input documents
- Output registry contains all compiled documents and is queryable
- Incremental compilation reuses unchanged documents and only recompiles changed ones
