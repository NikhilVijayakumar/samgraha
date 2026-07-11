# Fix Plan — Deterministic Whole-Document Finding

**Finding Source:** Deterministic — Whole Document
**Report Type:** `deterministic_whole`
**Domain:** {{ domain }}
**Document:** {{ document_path }}
**Rule:** {{ rule_id }}
**Severity:** {{ severity }}
**Date:** {{ created_at }}

---

## 1. Finding Summary

| Field | Value |
|-------|-------|
| Rule ID | {{ rule_id }} |
| Description | {{ rule_description }} |
| Message | {{ message }} |
| Evidence | {% if evidence %}{{ evidence }}{% else %}—{% endif %} |

---

## 2. Fix Classification

**Type:** Structural — whole-document level
**Nature:** The document as a collection is missing a required property, has an extra property, or violates a document-level constraint.

**Common causes:**
- Missing required section (e.g. no `system_overview` in an architecture document)
- Empty required section (heading exists but no content)
- Implementation technology references in an architecture-level document
- Missing required cross-references to upstream/downstream standards

---

## 3. Remediation Steps

{% for step in remediation_steps %}
{{ loop.index }}. {{ step }}
{% endfor %}

{% if remediation_steps | length == 0 %}
1. Open `{{ document_path }}`
2. Address the failing rule: {{ rule_description }}
3. Verify the fix by re-running `samgraha audit --domain {{ domain }}`
{% endif %}

---

## 4. Verification

After applying the fix:

```bash
samgraha audit --domain {{ domain }} --document {{ document_path }}
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
| Fix Plan Type | Deterministic — Whole Document |
| Domain | {{ domain }} |
| Rule File | `audit/deterministic/document/{{ domain }}.yaml` |
| Generated | {{ created_at }} |
| Session | {{ session_id }} |
