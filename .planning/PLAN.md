# Phase 8 — Per-Audit Detailed Report Schemas + Per-Audit Templates

## Motivation

Phase 7 added a single generic `pipeline_reports` + `pipeline_findings` with a
one-size-fits-all template. Each audit type (Build, Security, Consistency,
Coverage) has fundamentally different domain concerns — Build tracks artifact
freshness and commands, Security tracks threats and secrets, Consistency tracks
naming and cross-references, Coverage tracks doc-to-source ratios. A single
schema discards domain context.

## Design Principle

> Each audit type owns its schema. Shared concerns share tables.

Domain-specific columns live in per-audit report tables. Cross-cutting concerns
(findings, evidence, summaries, improvements) live in shared polymorphic tables.

## SQLite Schema — V14: new tables in knowledge.db

### Per-Audit Report Tables

```sql
CREATE TABLE build_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'build',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    -- build-specific
    contract_name TEXT,
    declared_produces TEXT,       -- JSON array of declared artifact patterns
    actual_artifacts TEXT,        -- JSON array of artifact paths found on disk
    artifact_freshness TEXT,      -- JSON: {path: "fresh"|"stale"|"missing", ...}
    execution_success INTEGER,    -- 0/1 — whether --execute ran successfully
    execution_output TEXT,        -- truncated stdout/stderr from --execute
    UNIQUE(session_id, pipeline)
);

CREATE TABLE security_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'security',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    -- security-specific
    secrets_scanned INTEGER DEFAULT 0,
    secrets_found INTEGER DEFAULT 0,
    runtime_checks INTEGER DEFAULT 0,
    runtime_issues INTEGER DEFAULT 0,
    high_risk_findings INTEGER DEFAULT 0,
    threat_summary TEXT,          -- human-readable summary of threat posture
    UNIQUE(session_id, pipeline)
);

CREATE TABLE consistency_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'consistency',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    -- consistency-specific
    vision_exists INTEGER DEFAULT 0,
    architecture_exists INTEGER DEFAULT 0,
    structure_score REAL,         -- 0-100, from cross-reference density
    naming_issues TEXT,           -- JSON array of naming convention violations
    cross_references INTEGER DEFAULT 0,
    UNIQUE(session_id, pipeline)
);

CREATE TABLE coverage_reports (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    pipeline TEXT NOT NULL DEFAULT 'coverage',
    score REAL NOT NULL,
    git_revision TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    -- coverage-specific
    features_count INTEGER DEFAULT 0,
    src_files_count INTEGER DEFAULT 0,
    feature_coverage_pct REAL,    -- % of features that have source implementations
    uncovered_features TEXT,      -- JSON array of feature names with no src/
    doc_types_covered TEXT,       -- JSON: {standard: true, architecture: true, ...}
    UNIQUE(session_id, pipeline)
);
```

### Shared Polymorphic Tables

```sql
CREATE TABLE report_findings (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,       -- 'build' | 'security' | 'consistency' | 'coverage'
    report_id INTEGER NOT NULL,      -- FK to corresponding per-audit table
    check_id TEXT NOT NULL,
    severity TEXT NOT NULL,          -- error | warning | suggestion
    message TEXT NOT NULL,
    location TEXT,
    status TEXT NOT NULL DEFAULT 'open',  -- FindingStatus: open|fixed|accepted|ignored|false_positive
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE report_evidence (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    finding_id INTEGER REFERENCES report_findings(id),
    key TEXT NOT NULL,
    value TEXT,
    source TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE report_summaries (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    summary_text TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE report_improvements (
    id INTEGER PRIMARY KEY,
    report_type TEXT NOT NULL,
    report_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    suggestion TEXT NOT NULL,
    priority TEXT DEFAULT 'medium',   -- high | medium | low
    created_at TEXT DEFAULT (datetime('now'))
);
```

**No tables dropped.** Old `pipeline_reports`, `pipeline_findings`, `report_comments`
remain untouched.

## Template Files

Located in `docs/raw/report-templates/`:

| File | Purpose |
|------|---------|
| `build-report.md` | Deep Build report: contract name, artifact list with freshness table, execution output, findings by severity |
| `security-report.md` | Deep Security report: secrets scanned/found table, runtime check summary, threat level, top risk findings |
| `consistency-report.md` | Deep Consistency report: vision/architecture presence, structure score, naming violations list, cross-reference count |
| `coverage-report.md` | Deep Coverage report: feature count vs source count, coverage % bar, uncovered features list, doc types matrix |

Each template uses the existing `{{variable}}` / `{{variable|filter}}` /
`{{#section}}...{{/section}}` syntax — no grammar changes to the engine.

Template context for each type includes:
- Common: `{{score}}`, `{{score_bar}}`, `{{session_id}}`, `{{date}}`, `{{git_revision}}`
- Build: `{{contract_name}}`, `{{artifact_table}}`, `{{execution_success}}`, `{{execution_output}}`
- Security: `{{secrets_scanned}}`, `{{secrets_found}}`, `{{threat_summary}}`, `{{high_risk_count}}`
- Consistency: `{{vision_exists}}`, `{{structure_score}}`, `{{naming_issues_table}}`, `{{cross_references}}`
- Coverage: `{{features_count}}`, `{{src_files_count}}`, `{{coverage_bar}}`, `{{uncovered_features_list}}`, `{{doc_types_table}}`
- Shared: `{{errors_table}}`, `{{warnings_table}}`, `{{suggestions_table}}`, `{{improvements}}`

## Store Layer — `store.rs`

### Per-Audit Functions

```rust
// Build
fn insert_build_report(score, session_id, git_revision, contract_name,
    declared_produces, actual_artifacts, artifact_freshness,
    execution_success, execution_output, findings) -> Result<i64>
fn query_build_sessions(limit) -> Result<Vec<BuildSessionInfo>>
fn get_build_report_with_findings(report_id) -> Result<Option<BuildReportWithFindings>>

// Security
fn insert_security_report(score, session_id, git_revision,
    secrets_scanned, secrets_found, runtime_checks, runtime_issues,
    high_risk_findings, threat_summary, findings) -> Result<i64>
fn query_security_sessions(limit) -> Result<Vec<SecuritySessionInfo>>
fn get_security_report_with_findings(report_id) -> Result<Option<SecurityReportWithFindings>>

// Consistency
fn insert_consistency_report(score, session_id, git_revision,
    vision_exists, architecture_exists, structure_score,
    naming_issues, cross_references, findings) -> Result<i64>
fn query_consistency_sessions(limit) -> Result<Vec<ConsistencySessionInfo>>
fn get_consistency_report_with_findings(report_id) -> Result<Option<ConsistencyReportWithFindings>>

// Coverage
fn insert_coverage_report(score, session_id, git_revision,
    features_count, src_files_count, feature_coverage_pct,
    uncovered_features, doc_types_covered, findings) -> Result<i64>
fn query_coverage_sessions(limit) -> Result<Vec<CoverageSessionInfo>>
fn get_coverage_report_with_findings(report_id) -> Result<Option<CoverageReportWithFindings>>
```

### Shared Functions

```rust
fn insert_report_findings(report_type, report_id, &[AuditFinding]) -> Result<()>
fn query_findings(report_type, report_id) -> Result<Vec<StoredFinding>>
fn update_finding_status(finding_id, status) -> Result<()>
fn insert_report_evidence(report_type, report_id, finding_id, key, value, source) -> Result<()>
fn insert_report_summary(report_type, report_id, summary_text) -> Result<()>
fn insert_report_improvement(report_type, report_id, category, suggestion, priority) -> Result<()>
```

### New Types (in store.rs or schemas/pipeline.rs)

```rust
struct BuildReportWithFindings { id, session_id, score, git_revision, created_at,
    contract_name, declared_produces, actual_artifacts, artifact_freshness,
    execution_success, execution_output, findings: Vec<StoredFinding> }
struct BuildSessionInfo { id, session_id, score, created_at, finding_counts: FindingCounts }

// Same pattern for Security, Consistency, Coverage
struct StoredFinding { id, check_id, severity, message, location, status }

struct FindingCounts { errors: usize, warnings: usize, suggestions: usize }
```

## Template Engine — `reporting.rs`

New entry point replacing `render_from_template()` for per-audit reports:

```rust
fn render_report(
    report_type: &str,           // "build" | "security" | "consistency" | "coverage"
    store: &RegistryStore,
    session_id: Option<&str>,
    template_path: Option<&str>,
) -> Result<String>
```

Logic:
1. `session_id` specified? → use it. None? → query latest session for `report_type`.
2. `template_path` specified? → read that file. None? → resolve `{report_type}-report.md`.
3. Resolve template: try filesystem `docs/raw/report-templates/{name}`, fallback to copy embedded in binary.
4. Load per-audit data from the correct table + `query_findings(report_type, report_id)`.
5. Build per-audit `TemplateContext` (typed struct for each audit type).
6. Run existing placeholder substitution + conditional block logic.
7. Return rendered markdown.

Embedded default templates (new constants in `reporting.rs`):
- `DEFAULT_BUILD_TEMPLATE`
- `DEFAULT_SECURITY_TEMPLATE`
- `DEFAULT_CONSISTENCY_TEMPLATE`
- `DEFAULT_COVERAGE_TEMPLATE`

## Pipeline Refactor — `runtime.rs` / `contract.rs`

`run_pipeline()` currently calls the generic `store_pipeline_report()`. Change to:

```
match pipeline_type {
    "build"        => store_build_report(params...),
    "security"     => store_security_report(params...),
    "consistency"  => store_consistency_report(params...),
    "coverage"     => store_coverage_report(params...),
}
```

Each variant:
1. Runs its checks (unchanged — the check logic stays as-is).
2. Builds domain-specific params from the check results.
3. Calls the matching `insert_*_report()` + `insert_report_findings()`.
4. Returns the report ID.

The `--report` flag in `audit` command calls `render_report(type, store, session_id)`.

## CLI — `commands.rs`

Replace the current `Commands::Report { pipeline, session, template, stdout, list_sessions, list_templates }` with:

```text
samgraha report <audit-type> [options]

Arguments:
  audit-type    build | security | consistency | coverage

Options:
  --session <uuid>          Specific session
  --template <path>          Custom template path
  --stdout                   Write to stdout only (no file)
  --list-sessions            List past sessions for this audit type
  --list-templates           List available templates for this audit type
```

Remove `--pipeline` (now positional argument).

## MCP — `adapter.rs`

| Method | Old | New |
|--------|-----|-----|
| `report_templates` | Lists all templates | Same (filters by type optional) |
| `report_generate` | `{pipeline, session?, template?}` | `{type: "build"\|"security"\|"consistency"\|"coverage", session?, template?}` |
| `report_sessions` | `{pipeline?, limit?}` | `{type: "build"\|..., limit?}` — required `type` param |

Remove old `pipeline` param; replace with `type`.

## Migration

**V14 — additive only.** No DROP, no ALTER. Old data stays readable via old
code paths; new audits write to new tables.

## Impact on Existing Code

| Component | Change |
|-----------|--------|
| `crates/registry/src/migration.rs` | V14: 8 new tables (`build_reports`, `security_reports`, `consistency_reports`, `coverage_reports`, `report_findings`, `report_evidence`, `report_summaries`, `report_improvements`) |
| `crates/registry/src/store.rs` | 4 × `insert_*_report()` + 4 × `query_*_sessions()` + 4 × `get_*_report_with_findings()` + 5 shared functions. Remove old `insert_pipeline_report()`, `query_pipeline_sessions()`, `get_pipeline_report_with_findings()`, `pipeline_update_finding_status()` |
| `crates/registry/src/store.rs` types | Remove `PipelineSessionInfo`, `PipelineReportWithFindings`, `StoredFinding` (for old pipeline). Add per-audit equivalents + shared `StoredFinding` in `report_findings` location |
| `crates/services/src/reporting.rs` | `render_from_template()` stays as low-level engine. New `render_report(type, store, session, template)` as high-level caller. Add 4 default template constants. Add 4 typed context structs |
| `crates/services/src/runtime.rs` | `run_pipeline()` → match on pipeline type, call correct per-audit store function. Remove generic `store_pipeline_report()`. Keep `query_pipeline_sessions()` → replace with per-audit queries |
| `crates/cli/src/commands.rs` | `Commands::Report { pipeline: String }` → `Commands::Report { audit_type: String }`. Pipeline is now positional arg |
| `crates/mcp/src/adapter.rs` | `report_generate`/`report_sessions` → `pipeline` param replaced with `type` param |
| `docs/raw/report-templates/build-report.md` | New — deep Build report template |
| `docs/raw/report-templates/security-report.md` | New — deep Security report template |
| `docs/raw/report-templates/consistency-report.md` | New — deep Consistency report template |
| `docs/raw/report-templates/coverage-report.md` | New — deep Coverage report template |
| `docs/raw/report-templates/pipeline-default.md` | Remove (superseded) |

## Proposed Execution Order

| Step | Task | Detail |
|------|------|--------|
| 1 | Documentation: 4 template files | Create `build-report.md`, `security-report.md`, `consistency-report.md`, `coverage-report.md` in `docs/raw/report-templates/` |
| 2 | Migration V14 | 8 new tables in `knowledge.db`, no DROP |
| 3 | Store: shared functions | `insert_report_findings()`, `query_findings()`, `update_finding_status()`, `insert_report_evidence()`, `insert_report_summary()`, `insert_report_improvement()` |
| 4 | Store: per-audit functions | `insert_build_report()`, `query_build_sessions()`, `get_build_report_with_findings()` — same for security/consistency/coverage |
| 5 | Template engine: `render_report()` | New high-level routing function + 4 typed context structs + 4 embedded default template constants |
| 6 | Pipeline refactor | `run_pipeline()` match on type → call per-audit store function |
| 7 | CLI: `report` subcommand update | `report <audit-type>` positional arg, route to correct table + template |
| 8 | MCP: param rename | `pipeline` → `type` in `report_generate`, `report_sessions` |
| 9 | Tests: store | 3 per-audit store tests (insert+query, empty query, finding status update) = 12 total + 3 shared table tests = 15 new |
| 10 | Tests: template | 4 render tests (one per audit type, using mock data) |
| 11 | Cleanup | Remove old `pipeline_default.md`, old generic store functions, old `PipelineSessionInfo` types |
| 12 | `cargo test --workspace` | All tests pass |
| 13 | `cargo check --workspace` | Zero warnings |

## Feasibility

| Aspect | Complexity | Detail |
|--------|-----------|--------|
| Migration | Low | 8 CREATE TABLE, no ALTER/DROP |
| Store per-audit functions | Low-Medium | 12 typed functions, repetitive but straightforward. Each follows same pattern with different column names |
| Shared store functions | Low | 6 generic functions, polymorphic via `report_type` |
| Template routing | Low | `render_report()` is a switch + load + render |
| Template context structs | Low | 4 structs, each mirrors its table columns |
| CLI arg change | Low | `pipeline: String` → `audit_type: String`, positional parsing |
| MCP param rename | Low | `pipeline` → `type` in two methods |
| Template content | Medium | Each template needs domain-specific formatting logic (artifact tables, coverage bars, naming issue lists). Not hard, but substantial content to write |
