# MCP Issues Fix Proposal

**Created:** 2026-07-17
**Scope:** 7 MCP issues found during Pitha integration testing
**Status:** Proposal — pending review

---

## Issue Validation Summary

| ID | Issue | Verdict | Reason |
|----|-------|---------|--------|
| MCP-001 | Check-scope false positives | **ACCEPT** | Confirmed in `providers.rs:92-131` — `section_presence` iterates all docs, no collection-scope path |
| MCP-002 | No prerequisite checking in project plan | **REJECT** | Already implemented — `orchestrator.rs:113-127` checks `phase.dependencies` and bails if incomplete |
| MCP-003 | No score threshold configuration | **ACCEPT** | `PASS_THRESHOLD` is hardcoded `70.0` in `executors.rs:170`, no `samgraha.toml` override |
| MCP-004 | No graceful degradation | **REJECT** | Already implemented — `orchestrator.rs:164-179` marks phase Failed + dependent Blocked, continues pipeline iteration |
| MCP-005 | Literal keyword matching | **PARTIAL** | `providers.rs:152` uses `body_lower.contains()` (substring) but `pipeline.rs:242-260` already has word-boundary matching — inconsistency only in deterministic provider |
| MCP-006 | Philosophy missing from pipeline registry | **ACCEPT** | Confirmed — no `Philosophy` variant in `PipelineKind` enum (`schemas/src/audit.rs:6-29`), not in `DOC_DOMAINS` list |
| MCP-007 | Semantic review never executes | **ACCEPT** | Semantic provider exists (`semantic.rs`) and runs heuristic checks, but it produces findings — the issue is that `audit()` doesn't propagate them to `semantic_review.tasks` or the LLM-based rubric review is unimplemented |

---

## Rejection Details

### MCP-002: No Prerequisite Checking — REJECTED

The claim is that phases proceed without verifying upstream docs. The code shows otherwise:

```rust
// orchestrator.rs:113-127
for dep_id in &phase.dependencies {
    let dep = self.get_phase(dep_id)?;
    if dep.status != PhaseStatus::Completed {
        return Err(anyhow!("Dependency '{}' has not completed (status: {:?})", dep_id, dep.status));
    }
}
```

Each phase declares dependencies via `make_phase()`. `NewProjectPlanner` chains phases 1→2→3→...→8 sequentially. Phase 4 (Audit impl) depends on Phase 3 (Fix docs), which depends on Phase 2 (Audit docs), which depends on Phase 1 (Generate docs). The dependency chain is enforced at runtime.

**What's actually missing:** There's no check that the *content* of generated docs is sufficient — only that the previous phase completed. But that's a different issue (quality gate, not prerequisite checking). The prerequisite mechanism itself works.

### MCP-004: No Graceful Degradation — REJECTED

The claim is that workflow fails when domains can't reach target score. The code shows otherwise:

```rust
// executors.rs:213-219
Err(e) => {
    all_passed = false;
    results.push(serde_json::json!({
        "pipeline": pipeline_str,
        "error": e.to_string(),
        "passed": false,
    }));
}
```

```rust
// orchestrator.rs:164-179
PhaseStatus::Failed => {
    for dep in downstream_phases { dep.status = PhaseStatus::Blocked; }
}
```

When a pipeline fails, execution continues to the next pipeline. When a phase fails, downstream phases are blocked (not crashed). The system already degrades gracefully — it doesn't crash, it marks phases as Failed/Blocked and returns structured JSON with the error.

**What's actually missing:** There's no "document limitation and proceed with warning" mode — it's binary pass/fail. But the core graceful degradation (don't crash, continue execution, report status) is implemented.

---

## Accepted Issues — Phasewise Implementation Plan

### Phase 1: Collection-Scope Section Presence (MCP-001)

**Priority:** Critical — blocks accurate scores for Architecture, Engineering, Implementation, Vision
**Files:** `crates/audit/src/providers.rs`
**Effort:** Small

**Problem:** `section_presence` check at `providers.rs:92-131` creates a finding for every document missing a section. Should create one finding if the section is missing from the entire domain.

**Fix:**
```rust
"section_presence" => {
    let section_key = rule.scope.to_lowercase().replace(' ', "_").replace('-', "_");
    let has_anywhere = documents.iter().any(|doc| {
        let count = doc.quality.per_type.get(&section_key).copied().unwrap_or(0);
        if count > 0 { return true; }
        let title_key = doc.title.to_lowercase().replace(' ', "_").replace('-', "_");
        title_key == section_key
    });
    if !has_anywhere {
        vec![AuditFinding {
            check_id: rule.id.clone(),
            severity: Severity::from_str(&rule.severity),
            message: format!("{}: section '{}' missing from domain", rule.description, rule.scope),
            location: None,
            document_id: None,
            provider: "deterministic".into(),
            stage: None, section_id: None, confidence: None,
            evidence: None, status: None, strategy: None,
        }]
    } else {
        vec![]
    }
}
```

**Verification:** Run `audit(domain="architecture", providers=["deterministic"])` — score should jump from ~88 to ~95+. Check that `findings` count drops from 106+ to <10.

---

### Phase 2: Philosophy Pipeline Registration (MCP-006)

**Priority:** Medium — philosophy domain can't get pipeline-level scoring
**Files:** `crates/schemas/src/audit.rs`, `crates/audit/src/pipelines/mod.rs`, `crates/services/src/runtime/runtime.rs`, `crates/services/src/project_planner/planners.rs`
**Effort:** Medium

**Problem:** No `Philosophy` variant in `PipelineKind`, no `PhilosophyPipeline` struct, not in `DOC_DOMAINS`.

**Fix:**
1. Add `Philosophy` variant to `PipelineKind` enum in `schemas/src/audit.rs`
2. Add `"philosophy" => Some(Self::Philosophy)` to `from_str()`
3. Add `"Philosophy" => "philosophy"` to `as_str()`
4. Create `crates/audit/src/pipelines/philosophy.rs` implementing `Pipeline` trait
5. Register in `pipelines/mod.rs` and `runtime.rs` dispatch match
6. Add `"philosophy"` to `DOC_DOMAINS` in `planners.rs`

**Pipeline checks** (analogous to Vision pipeline):
- P1: Philosophy document exists
- P2: Guiding Principles section present
- P3: Values section present
- P4: Trade-offs section present
- P5: No implementation-specific technology references

**Verification:** `audit(pipeline="philosophy")` should return a valid `PipelineReport` with score.

---

### Phase 3: Word-Boundary Keyword Matching in Deterministic Provider (MCP-005)

**Priority:** Low — affects edge cases ("aspires" vs "aspiration", "pipelines" vs "pip")
**Files:** `crates/audit/src/providers.rs`
**Effort:** Small

**Problem:** `keyword_absence` at line 152 uses `body_lower.contains(kw.as_str())` — substring match. `content_check` at lines 187, 197 also uses `body_lower.contains()`. The pipeline layer (`pipeline.rs:242-260`) already has `has_word_boundaries()` — inconsistent.

**Fix:** Reuse the existing word-boundary logic from `pipeline.rs`:

```rust
// In providers.rs, add a helper:
fn contains_word(text: &str, keyword: &str) -> bool {
    text.split_whitespace().any(|w| w == keyword)
    // Or for substring-within-word safety:
    // text.split(|c: char| !c.is_alphanumeric()).any(|w| w == keyword)
}

// Replace all body_lower.contains(kw.as_str()) calls with:
contains_word(&body_lower, kw.as_str())
```

**Verification:** `audit(domain="vision")` — "aspires" should not trigger "aspiration" absence. "pipelines" should not trigger "pip" absence.

---

### Phase 4: Configurable Score Thresholds (MCP-003)

**Priority:** Low — current 70.0 threshold is reasonable default
**Files:** `crates/services/src/project_planner/executors.rs`, `crates/common/src/config.rs`, `samgraha.toml`
**Effort:** Small

**Problem:** `PASS_THRESHOLD` is hardcoded `70.0`. Users can't set per-domain or per-project thresholds.

**Fix:**
1. Add `[audit.gates]` section to `samgraha.toml` schema:
   ```toml
   [audit.gates]
   default = 70
   architecture = 85
   engineering = 70
   ```
2. Load gates config in `VerifyPhaseExecutor::execute()` via `runtime.config()`
3. Fall back to `PASS_THRESHOLD` when no gate is configured for a pipeline

```rust
// executors.rs
let threshold = runtime.config()
    .and_then(|c| c.audit.gates.get(pipeline_str))
    .copied()
    .unwrap_or(Self::PASS_THRESHOLD);
let passed = report.score >= threshold;
```

**Verification:** Set `architecture = 85` in `samgraha.toml`, run verify phase, confirm architecture uses 85 threshold.

---

### Phase 5: Semantic Review Task Population (MCP-007)

**Priority:** Critical — audit scores only reflect structure, not content quality
**Files:** `crates/audit/src/framework.rs`, `crates/providers/src/semantic.rs`
**Effort:** Large

**Problem:** `semantic_review.tasks` is always empty. The semantic provider (`semantic.rs`) runs heuristic checks and returns findings, but `framework.rs` doesn't populate the `semantic_review` bundle from those findings.

**Root cause analysis:**
- `SemanticAuditProvider::execute()` returns `Vec<AuditFinding>` — these go into the combined findings array
- `AuditReport::semantic_review` is a separate `PipelineSemanticReviewBundle` struct
- Nothing bridges provider findings into the review bundle
- The intended flow (audit emits rubric tasks → agent judges → agent stores report) requires LLM integration that doesn't exist yet

**Fix (incremental):**
1. In `framework.rs`, after running semantic provider, populate `semantic_review.tasks` from the provider's findings:
   ```rust
   // After semantic provider execution
   let semantic_findings = semantic_provider.execute(docs, rules, standard);
   report.semantic_review.tasks = semantic_findings.iter().map(|f| SemanticReviewTask {
       document_id: f.document_id,
       section_id: f.section_id,
       check_id: f.check_id.clone(),
       message: f.message.clone(),
       severity: f.severity.clone(),
       score: None, // to be filled by LLM review
   }).collect();
   ```
2. Update `get_summary_report` to use `semantic_review.tasks` length/non-empty-ness for `standard_score`
3. Future: Add LLM-based rubric evaluation (separate effort)

**Verification:** `audit(domain="vision", providers=["deterministic", "semantic"])` should return non-empty `semantic_review.tasks`. `get_summary_report` should have non-null `standard_score`.

---

## Implementation Order

| Phase | Issue | Depends On | Risk |
|-------|-------|------------|------|
| 1 | MCP-001 (collection-scope) | None | Low — isolated change in one match arm |
| 2 | MCP-006 (philosophy pipeline) | None | Low — new code, no existing behavior changed |
| 3 | MCP-005 (word-boundary) | None | Low — helper function, existing pattern in pipeline.rs |
| 4 | MCP-003 (score thresholds) | None | Low — config loading, backward-compatible default |
| 5 | MCP-007 (semantic review) | MCP-001 | Medium — requires framework.rs changes, affects scoring |

Phases 1-4 are independent and can be done in parallel. Phase 5 depends on Phase 1 being stable (collection-scope fixes the false positive noise that would otherwise drown out semantic findings).

---

## Testing Strategy

Each phase should include:
1. **Unit test** in the modified file's `#[cfg(test)]` module
2. **Integration test** in `tests/tests/` — run audit against test fixtures
3. **Regression check** — confirm existing passing tests still pass (`cargo test`)

Key test scenarios:
- Phase 1: Section present in one doc → no finding. Missing from all docs → one finding.
- Phase 2: `audit(pipeline="philosophy")` returns valid report.
- Phase 3: "aspires" does not trigger "aspiration" check. "pipelines" does not trigger "pip".
- Phase 4: Custom threshold from config overrides default.
- Phase 5: `semantic_review.tasks` is non-empty after semantic provider runs.

---

## Out of Scope

- **MCP-002 (rejected):** Prerequisite checking already works. If quality-gate-before-phase is desired, that's a new feature request, not a bug fix.
- **MCP-004 (rejected):** Graceful degradation already works. If "document limitation and warn" mode is desired, that's a new feature request.
- **LLM-based rubric evaluation:** MCP-007 Phase 5 bridges heuristic findings into the review bundle. Full LLM rubric evaluation is a separate, larger effort.
