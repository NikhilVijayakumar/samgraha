# Plan Scenarios — Generation Template

> **Domain:** implementation
> **Section:** plan_scenarios
> **Source:** `documentation-standards/13-implementation-standards.md` §Plan Scenarios
> **Relationships:** `audit/deterministic/document/13-implementation-relationships.yaml`

Generate the Plan Scenarios section for an Implementation Plan document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | feature / purpose | Scenario selection must align with what Feature(04) defines to build |
| `derives_from` | architecture / system_overview | Scope selection must respect Architecture(05) system boundaries |

## Template

```markdown
## Plan Scenarios

### Applicable Scenario

> **scenario:** [Full Generation | Per Feature | Enhancement | Refactor | Change Request]
> **scope:** [entire project | per feature | per module | per section]
> **inputs:** [list of upstream documents consumed]
> **outputs:** [what this plan produces]

[2-3 sentences describing when to use this scenario and what it produces.]

### Scope Options

| Scope | When to Use | Required Inputs |
|-------|-------------|-----------------|
| [scope option] | [trigger condition] | [upstream docs] |
```

## Examples

**Correct:**
> **Applicable Scenario**
>
> > **scenario:** Per Feature
> > **scope:** Per feature
> > **inputs:** Feature(04), Feature Design(09), Architecture(05), Security(03)
> > **outputs:** Implementation plan for the specific feature
>
> Use Per Feature when adding a new feature to an existing project. The feature has unique implementation requirements and feature-specific deviation tracking is needed. Upstream verification is limited to the feature's scope.

**Incorrect:**
> **Applicable Scenario**
>
> > **scenario:** Per Feature
>
> We are implementing a feature.
> *Why wrong: Missing scope, inputs, and outputs; no description of when to use this scenario or what it produces.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Select the correct scenario for the implementation context; list all upstream documents consumed; state what the plan produces
- **Don't:** Select a scenario without justification; omit upstream document references; describe the scenario without stating scope and outputs

**Minimum content:** 1 scenario with full metadata block
**Length guidance:** concise
**Required diagrams:** none
**Required cross-references:** Feature(04), Architecture(05), Security(03)

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
