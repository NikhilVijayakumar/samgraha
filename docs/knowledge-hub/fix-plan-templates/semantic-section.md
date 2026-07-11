# Fix Plan — Semantic Section Finding

**Finding Source:** Semantic — Section
**Report Type:** `semantic_section`
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
| Evidence | {% if evidence %}"{{ evidence }}"{% else %}—{% endif %} |

---

## 2. Fix Classification

**Type:** Content/reasoning — section level
**Nature:** The LLM judged that a specific section's content quality, clarity, or consistency with related sections needs improvement.

**Common causes:**
- Section content is vague or generic (not specific to this project)
- Section contradicts a related section in another document
- Missing detail where the rubric expects it
- Tone/voice inconsistency (e.g. imperative in a section that should be third-person)
- Section describes things outside its scope

---

## 3. Remediation Steps

Semantic section findings require **rewriting content**, not just structural fixes.
Focus on what the section says and how it says it.

{% for step in remediation_steps %}
{{ loop.index }}. {{ step }}
{% endfor %}

{% if remediation_steps | length == 0 %}
1. Read the `{{ section_type }}` section in `{{ document_path }}`
2. Review the scoring criteria in `audit/semantic/section/{{ domain }}/{{ section_type }}.md`
3. Identify why the section scored low: {{ message }}
4. Rewrite the section to address the specific criterion that failed
5. Verify the fix by re-running `samgraha audit --domain {{ domain }} --section {{ section_type }}`
{% endif %}

---

## 4. Verification

After applying the fix:

```bash
samgraha audit --domain {{ domain }} --document {{ document_path }} --section {{ section_type }} --provider semantic
```

Expected: section score improves, finding resolves.

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
| Fix Plan Type | Semantic — Section |
| Domain | {{ domain }} |
| Section | {{ section_type }} |
| Rule File | `audit/semantic/section/{{ domain }}/{{ section_type }}.md` |
| Generated | {{ created_at }} |
| Session | {{ session_id }} |
| Model | {{ model }} |
