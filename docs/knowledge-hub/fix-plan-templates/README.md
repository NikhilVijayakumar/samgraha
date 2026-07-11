# Fix-Plan Templates

Templates for generating remediation plans from audit findings. Each template corresponds to one of the four report types and provides guidance matched to the nature of the finding.

## Templates

| Template | Report Type | Fix Nature |
|----------|------------|------------|
| `deterministic-whole.md` | Deterministic — Whole Document | Structural: add missing property, fix constraint |
| `deterministic-section.md` | Deterministic — Section | Structural: add subsection, fix field presence |
| `semantic-whole.md` | Semantic — Whole Document | Content/reasoning: fix contradictions, coherence |
| `semantic-section.md` | Semantic — Section | Content/reasoning: rewrite for quality/consistency |

## Variable Reference

All templates use Jinja2 variables:

| Variable | Description |
|----------|-------------|
| `{{ domain }}` | Domain name (e.g. `architecture`) |
| `{{ section_type }}` | Section type slug (e.g. `component-model`) |
| `{{ document_path }}` | Path to the audited document |
| `{{ rule_id }}` | Failing rule ID |
| `{{ rule_description }}` | What the rule checks |
| `{{ message }}` | Audit message describing the failure |
| `{{ evidence }}` | Evidence from the document (may be empty) |
| `{{ severity }}` | `Critical`, `Warning`, or `Suggestion` |
| `{{ created_at }}` | ISO-8601 timestamp |
| `{{ session_id }}` | Audit session ID |
| `{{ model }}` | LLM model name (semantic templates only) |
| `{{ remediation_steps }}` | Pre-computed fix steps (may be empty) |
| `{{ related_findings }}` | List of related findings (may be empty) |

## Relationship to Report Templates

Report templates (`templates/audit/`) generate findings. Fix-plan templates (`fix-plan-templates/`) consume those findings and produce actionable remediation plans. The `report_type` field in findings maps directly to which fix-plan template to use.
