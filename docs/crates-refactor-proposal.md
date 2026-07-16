# Proposal: Structured JSON Output Alongside Markdown Reports

**Created:** 2026-07-17
**Scope:** All report generation in samgraha
**Status:** Proposal — pending review

---

## Problem

All samgraha reports are rendered as markdown via templates. This is excellent for human readability but poor for programmatic consumption:

- **Visualization tools** (dashboards, charts, trend analysis) need structured data
- **Custom logic** (threshold comparisons, trend detection, cross-domain aggregation) requires parsing markdown back into data
- **CI/CD integration** needs machine-readable pass/fail signals
- **External consumers** (Pitha, other tools) want JSON APIs, not markdown parsing

The scorecard system already writes `latest.json` alongside `latest.md` for domain audits, but all other report types (pipeline reports, per-audit reports, CLI reports) only produce markdown.

---

## Current State

| Report Path | JSON Output | Markdown Output |
|-------------|-------------|-----------------|
| `{domain}/scorecard/latest.{json,md}` | `AuditScorecard` (full) | Template-rendered |
| `{domain}/{doc_id}/{section}/latest.json` | Raw `SemanticReport` | None |
| `reports/{type}/latest/report.md` | **None** | Template-rendered |
| `reports/{type}/archive/{ts}.md` | **None** | Template-rendered |
| Pipeline in-memory (`render_report_from_pipeline`) | **None** | Template-rendered |

The gap: 15+ report types produce markdown only. The template context structs (`ArchitectureTeraContext`, `BuildTemplateContext`, etc.) are already `#[derive(Serialize)]` but the JSON is never written to disk.

---

## Proposed Solution

Add a `latest.json` alongside every `latest.md` (or standalone `latest.json`) in the report output directories. The JSON contains the same data that was embedded in the template, serialized from the already-serializable context structs.

### What Changes

1. **`render_report()`** — currently returns `String` (markdown). Add a companion JSON output.
2. **`render_report_from_pipeline()`** — currently returns `String` (markdown). Add JSON serialization.
3. **CLI `report` command** — writes `latest/report.md`. Add `latest/report.json`.
4. **`write_report()` / `write_report_file()`** — already supports arbitrary filenames. No change needed.

### What Doesn't Change

- Template files (`.md`) stay as-is
- SQLite storage stays as-is
- The `AuditScorecard` flow (already writes JSON) stays as-is
- MCP tool responses stay as-is (they return JSON already)

---

## Detailed Design

### Phase 1: Add JSON Serialization to `render_report()`

**File:** `crates/services/src/reporting.rs`

Currently `render_report()` builds a context struct, renders markdown, and returns the markdown string. The context struct is already `Serialize`. Change it to return both:

```rust
pub struct ReportOutput {
    pub markdown: String,
    pub json: String,
}

pub fn render_report(
    report_type: &str,
    templates_dir: &Path,
    store: &registry::RegistryStore,
) -> Result<ReportOutput> {
    // ... existing context building ...
    let markdown = render_*_template(&ctx, &template);
    let json = serde_json::to_string_pretty(&ctx)
        .context("Failed to serialize report context")?;
    Ok(ReportOutput { markdown, json })
}
```

**Backward compatibility:** Existing callers that do `let md = render_report(...)?;` will need to destructure: `let ReportOutput { markdown, json } = render_report(...)?;`. The markdown content is identical — no visual change.

**Affected functions (15):**
- `render_build_template` → `BuildTemplateContext`
- `render_security_template` → `SecurityTemplateContext`
- `render_consistency_template` → `ConsistencyTemplateContext`
- `render_coverage_template` → `CoverageTemplateContext`
- `render_help_template` → `HelpTemplateContext`
- `render_architecture_template` → `ArchitectureTeraContext`
- `render_vision_template` → `VisionTeraContext`
- `render_design_template` → `DesignTeraContext`
- `render_readme_template` → `ReadmeTeraContext`
- `render_prototype_template` → `PrototypeTeraContext`
- `render_external_context_template` → `ExternalContextTeraContext`
- `render_engineering_template` → `EngineeringTeraContext`
- `render_feature_template` → `FeatureTeraContext`
- `render_feature_technical_template` → `FeatureTechnicalTeraContext`
- `render_feature_design_template` → `FeatureDesignTeraContext`
- `render_deterministic_runtime_template` → `DeterministicRuntimeTeraContext`
- `render_external_context_ownership_template` → `ExternalContextOwnershipTeraContext`
- `render_documentation_structure_template` → `DocumentationStructureTeraContext`
- `render_implementation_template` → `ImplementationTeraContext`

**All context structs already have `#[derive(Serialize)]`** — verified in codebase exploration. No schema changes needed.

### Phase 2: Add JSON to `render_report_from_pipeline()`

**File:** `crates/services/src/reporting.rs` (line 3671)

Currently builds a context struct from `PipelineReport` and renders markdown. Add JSON output:

```rust
pub fn render_report_from_pipeline(
    report_type: &str,
    template: &str,
    report: &schemas::audit::PipelineReport,
) -> ReportOutput {
    let markdown = /* existing render logic */;
    let json = serde_json::to_string_pretty(report)
        .unwrap_or_else(|_| "{}".to_string());
    ReportOutput { markdown, json }
}
```

For pipeline reports, the JSON is the `PipelineReport` itself (already `Serialize`) — no need to serialize the template context, since the context is a lossy transformation of the report.

### Phase 3: CLI Report Command Writes JSON

**File:** `crates/cli/src/commands.rs` (line ~1002)

Currently:
```rust
let rendered = services::reporting::render_report(audit_type, &templates_dir, &runtime.registry)?;
std::fs::write(&latest_path.join("report.md"), &rendered)?;
std::fs::write(&archive_path.join(format!("{}.md", ts)), &rendered)?;
```

Change to:
```rust
let output = services::reporting::render_report(audit_type, &templates_dir, &runtime.registry)?;
std::fs::write(latest_dir.join("report.md"), &output.markdown)?;
std::fs::write(latest_dir.join("report.json"), &output.json)?;
std::fs::write(archive_dir.join(format!("{}.md", ts)), &output.markdown)?;
std::fs::write(archive_dir.join(format!("{}.json", ts)), &output.json)?;
```

### Phase 4: Add `json_output` Config Toggle

**File:** `samgraha.toml`

```toml
[report]
dir = "${SAMGRAHA_REPORT_DIR}"
json = true  # Write JSON alongside markdown for all reports
```

**File:** `crates/common/src/config.rs`

Add to `ReportConfigSection`:
```rust
#[serde(default = "default_true")]
pub json: bool,
```

This lets users disable JSON output if they only want markdown (saves disk I/O).

### Phase 5: Update MCP Report Tools

**File:** `crates/mcp/src/adapter.rs`

The MCP `render_report` tool currently returns markdown. Add an optional `format` parameter:

```rust
// Existing: returns markdown
// New: format = "json" returns the structured JSON directly
```

This lets MCP consumers (Pitha, other tools) request structured data without parsing markdown.

---

## Output Structure

### Domain Scorecard (already exists)

```
docs/raw/reports/{domain}/scorecard/
  latest.json    # AuditScorecard { report, semantic_results }
  latest.md      # Template-rendered markdown
```

No change — this already works.

### Pipeline Reports (new JSON)

```
docs/raw/reports/{domain}/{doc_id}/{section}/
  latest.json    # SemanticReport (already exists for some)
  latest.md      # (new — template-rendered, currently not written here)
```

### CLI Reports (new JSON)

```
reports/{type}/
  latest/
    report.md    # Existing
    report.json  # NEW — structured context data
  archive/
    {ts}.md      # Existing
    {ts}.json    # NEW — structured context data
```

---

## JSON Schema Examples

### Architecture Report JSON

```json
{
  "session_id": "abc-123",
  "score": 92.5,
  "rating": "Very Good",
  "rating_description": "...",
  "previous_score": 88.0,
  "score_change_display": "+4.5 (improvement)",
  "git_revision": "abc123",
  "created_at": "2026-07-17T10:00:00Z",
  "engineering_readiness": "READY",
  "collection_integrity_score": 95.0,
  "collection_integrity_rating": "Excellent",
  "structural_integrity_score": 90.0,
  "structural_integrity_rating": "Very Good",
  "consistency_score": 88.0,
  "consistency_rating": "Very Good",
  "cross_repo_score": 94.0,
  "cross_repo_rating": "Excellent",
  "doc_scores": [
    { "name": "overview.md", "score": 95.0, "rating": "Excellent" }
  ],
  "critical_findings": [],
  "major_findings": [
    { "check_id": "A4", "message": "...", "location": "..." }
  ],
  "minor_findings": [],
  "recommendations": [
    { "category": "Architecture", "priority": "medium", "description": "..." }
  ]
}
```

### Build Report JSON

```json
{
  "session_id": "def-456",
  "score": 100.0,
  "date": "2026-07-17T10:00:00Z",
  "git_revision": "def456",
  "contract_name": "build",
  "declared_produces": "target/debug/myapp",
  "execution_success": "true",
  "errors": [],
  "warnings": [],
  "suggestions": []
}
```

---

## Phasewise Implementation Plan

| Phase | Change | Files | Risk | Effort |
|-------|--------|-------|------|--------|
| 1 | `ReportOutput` struct + `render_report()` returns both | `reporting.rs` | Low — additive | Small |
| 2 | `render_report_from_pipeline()` returns both | `reporting.rs` | Low — additive | Small |
| 3 | CLI writes JSON alongside markdown | `commands.rs` | Low — new files only | Small |
| 4 | `[report].json` config toggle | `config.rs`, `samgraha.toml` | Low — default true | Small |
| 5 | MCP `format` parameter | `adapter.rs` | Low — optional param | Small |

**Phases 1-3 are independent.** Phase 4 is independent. Phase 5 depends on Phase 1.

---

## Backward Compatibility

- **Markdown output is unchanged.** Templates render identically. No visual regression.
- **JSON files are additive.** New `report.json` files appear alongside existing `report.md`. No existing files are modified or removed.
- **`render_report()` return type changes** from `String` to `ReportOutput`. This is a breaking change for callers. All callers are internal (CLI, MCP, tests) — no external API break.
- **Config default is `true`.** Existing projects get JSON output automatically. Set `json = false` to disable.

---

## Testing Strategy

1. **Unit test:** Serialize each context struct to JSON, verify it's valid JSON and contains expected keys
2. **Integration test:** Run `render_report("architecture", ...)`, verify both `markdown` and `json` are non-empty, verify JSON deserializes back to the same context struct
3. **CLI test:** Run `samgraha report architecture`, verify `report.json` exists alongside `report.md`
4. **Regression:** Verify existing tests still pass (markdown output unchanged)

---

## Out of Scope

- **SQLite schema changes** — reports are already stored in SQLite; JSON files are a parallel output, not a replacement
- **Template changes** — markdown templates stay as-is
- **Real-time streaming** — JSON files are written atomically, not streamed
- **Report compression** — JSON files can be large for domains with many findings; compression is a future optimization
