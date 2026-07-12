# Data Formats — Generation Template

> **Domain:** external-context
> **Section:** data_formats (subsection of integration_contract)
> **Source:** `documentation-standards/08-external-context-standards.md` §Integration Contract §Data Formats
> **Relationships:** `audit/deterministic/document/08-external-context-relationships.yaml`

Generate the Data Formats subsection within Integration Contract for an External Context document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `informs` | feature_technical / communication_paths | Data formats define how feature technical designs structure communication |

## Template

```markdown
### Data Formats

[1 paragraph: how the external system exchanges data — encoding, content types, serialization format]

| Direction | Format | Content Type | Encoding | Constraints |
|-----------|--------|-------------|----------|-------------|
| Request | [format] | [MIME type] | [encoding] | [size limits, structure requirements] |
| Response | [format] | [MIME type] | [encoding] | [pagination, field availability] |

[1 paragraph per non-trivial format: field naming conventions, required vs optional fields, date/time formats]
```

## Examples

**Correct:**
> | Direction | Format | Content Type | Encoding | Constraints |
> |-----------|--------|-------------|----------|-------------|
> | Request | JSON | application/json | UTF-8 | Max 1MB payload |
> | Response | JSON | application/json | UTF-8 | Paginated, max 100 items per page |
>
> All timestamps use ISO 8601 format. Field names are camelCase. Error responses follow a standard structure with `code` and `message` fields.

**Incorrect:**
> The API uses JSON.
> *Why wrong: Missing encoding, constraints, direction-specific details, and content type specificity.*

## Writing Guidance

- **Tone:** technical
- **Voice:** third person
- **Structure:** tables
- **Audience:** engineer
- **Do:** Specify format, content type, encoding, and constraints for both request and response directions. Document field naming conventions and date/time formats.
- **Don't:** Include serialization code. Omit encoding details. Leave format ambiguous.

**Required subsections:** none (this is a subsection of Integration Contract)
**Optional subsections:** none
**Required diagrams:** none
**Required cross-references:** authoritative external documentation

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
