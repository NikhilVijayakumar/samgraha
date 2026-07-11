# Fix Plan — Deterministic Section Finding

**Finding Source:** Deterministic — Section
**Report Type:** `deterministic_section`
**Domain:** {{ domain }}
**Section:** {{ section_type }}
**Document:** {{ document_path }}
**Rule:** {{ rule_id }}
**Severity:** {{ severity }}
**Date:** {{ created_at }}

---

## 1. Finding Summary

| Field | Value |
|-------|-------|
| Rule ID | {{ rule_id }} |
| Section | {{ section_type }} |
| Description | {{ rule_description }} |
| Message | {{ message }} |
| Evidence | {% if evidence %}{{ evidence }}{% else %}—{% endif %} |

---

## 2. Fix Classification

**Type:** Structural — section level
**Nature:** A specific section is missing required content, has wrong structure, or violates section-level constraints.

**Common causes:**
- Missing required subsection (e.g. `Component Model` lacks `Components` subsection)
- Missing required field in section entries (e.g. no `Responsibility` on a component)
- Section references technologies it shouldn't (implementation leakage)
- Missing diagram where one is required

---

## 3. Remediation Steps

{% for step in remediation_steps %}
{{ loop.index }}. {{ step }}
{% endfor %}

{% if remediation_steps | length == 0 %}
1. Open `{{ document_path }}`
2. Navigate to the `{{ section_type }}` section
3. Address the failing rule: {{ rule_description }}
4. Verify the fix by re-running `samgraha audit --domain {{ domain }}`
{% endif %}

---

## 4. Verification

After applying the fix:

```bash
samgraha audit --domain {{ domain }} --document {{ document_path }} --section {{ section_type }}
```

Expected: rule `{{ rule_id }}` transitions from FAIL to PASS.

---

## 5. Related Findings

{% if related_findings | length > 0 %}
| Rule | Message | Severity |
|------|---------|----------|
{% for f in related_findings -%}
| {{ f.rule_id }} | {{ f.message }} | {{ f.severity }} |
{% endfor %}
{% else %}
No related findings.
{% endif %}

---

## 6. Metadata

| Field | Value |
|-------|-------|
| Fix Plan Type | Deterministic — Section |
| Domain | {{ domain }} |
| Section | {{ section_type }} |
| Rule File | `audit/deterministic/section/{{ domain }}/{{ section_type }}.yaml` |
| Generated | {{ created_at }} |
| Session | {{ session_id }} |
