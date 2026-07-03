# Semantic Audit Orchestration

This section details the Semantic Audit Orchestration.

## Pipeline Order

1. Deterministic Audit
2. Section Audit
3. Document Audit
4. Cross-Domain Audit

## Gate Conditions

Each stage runs only after the previous stage gate passes.

| Stage | Gate Check | Pass Condition |
|---|---|---|
| Section | `check_gate(stage="deterministic", document_id=N)` | Zero ERROR-severity deterministic findings |
| Document | `check_gate(stage="section", document_id=N)` | Zero ERROR-severity section findings |
| Cross-Domain | `check_gate(stage="document")` | Zero ERROR-severity document findings across all domains |

## Incremental Skip Logic

1. Call `get_section_changed(section_id)` before auditing each section
2. If `{changed: false, previous_report_id: N}`, reuse previous report, skip LLM call
3. If `{changed: true}`, execute section audit

## Output Schemas

This section details the Output Schemas.

### Section Report

```json
{
  "report_id": "uuid",
  "stage": "section",
  "domain": "feature",
  "document_id": 42,
  "section_id": 17,
  "strategy": "completeness",
  "score": 85,
  "findings": [
    {
      "criterion_id": "C1",
      "passed": true,
      "severity": "error",
      "confidence": 0.95,
      "evidence": {
        "section_id": 17,
        "paragraph_index": 3,
        "sentence": null,
        "excerpt": "FR1. System shall accept user registration with email and password."
      },
      "message": "All 12 functional requirements enumerated without gaps.",
      "status": "open"
    }
  ]
}
```

### Document Report

Same schema with `stage: "document"`, `section_id: null`. Findings reference relationships between sections.

### Cross-Domain Report

Same schema with `stage: "cross_domain"`, `section_id: null`, `document_id: null`. Findings reference document pairs.

## Error Handling

- If `store_section_report` returns `{error: "schema_violation", field: "...", expected: "..."}`, fix the output and retry
- If `check_gate` returns `{blocked: true}`, do not proceed; report the blocking findings to the user
- If a section does not exist (404), skip it and log a warning
