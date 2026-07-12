# Plan Scenarios — Generation Template

> **Domain:** build
> **Section:** plan_scenarios
> **Source:** `documentation-standards/14-build-standards.md` §Plan Scenarios
> **Relationships:** `audit/deterministic/document/14-build-relationships.yaml`

Generate the Plan Scenarios section for a Build Plan document.

## Relationships

| Relationship | Target | Constraint |
|---|---|---|
| `derives_from` | engineering / build_standards | Scenario selection must align with Engineering(07) CI/CD configuration |
| `derives_from` | external-context / constraints | Environment constraints affect scenario selection |

## Template

```markdown
## Plan Scenarios

### Applicable Scenario

> **scenario:** [Full Generation | New Build Rule | New Integration | Environment Change | Security Threat]
> **scope:** [entire pipeline | specific stage | specific tool | specific environment | security check]
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
> > **scenario:** New Build Rule
> > **scope:** Specific build stage
> > **inputs:** Security(03) updated threat model, QA(12) security test requirements
> > **outputs:** Build plan addition for the specific stage
>
> Use New Build Rule when adding a new security check category to the existing pipeline. The scope is limited to the new stage and its integration with existing gates.

**Incorrect:**
> **Applicable Scenario**
>
> > **scenario:** Full Generation
>
> We need a build plan.
> *Why wrong: Missing scope, inputs, and outputs; no description of when to use this scenario or what it produces.*

## Writing Guidance

- **Tone:** prescriptive
- **Voice:** imperative
- **Structure:** bullet lists
- **Audience:** engineer
- **Do:** Select the correct scenario for the build context; list all upstream documents consumed; state what the plan produces
- **Don't:** Select a scenario without justification; omit upstream document references; describe the scenario without stating scope and outputs

**Minimum content:** 1 scenario with full metadata block
**Length guidance:** concise
**Required diagrams:** none
**Required cross-references:** Engineering(07), Security(03), External Context(08)

## Audit Fix

<!-- Phase 5: populate with finding→generation handoff -->
