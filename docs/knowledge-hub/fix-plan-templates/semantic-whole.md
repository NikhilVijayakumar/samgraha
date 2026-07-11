# Fix Plan — Semantic Whole-Document Finding

**Finding Source:** Semantic — Whole Document
**Report Type:** `semantic_whole`
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
| Evidence | {% if evidence %}"{{ evidence }}"{% else %}—{% endif %} |

---

## 2. Fix Classification

**Type:** Content/reasoning — whole-document level
**Nature:** The LLM judged that the document as a whole has consistency issues, terminology drift, contradictions between sections, or collection-level coherence problems.

**Common causes:**
- Contradictions between sections (e.g. `Purpose` says X, `Constraints` says not-X)
- Terminology drift (same concept called different names across sections)
- Cross-section consistency failure (sections don't agree on scope/boundaries)
- Collection coherence issue (documents in the domain don't form a coherent whole)

---

## 3. Remediation Steps

Semantic findings require **rewriting or restructuring content**, not just adding
missing pieces. The fix is about what the section says, not whether it exists.

{% for step in remediation_steps %}
{{ loop.index }}. {{ step }}
{% endfor %}

{% if remediation_steps | length == 0 %}
1. Read the full document at `{{ document_path }}`
2. Identify the contradiction or inconsistency described in: {{ message }}
3. Rewrite the affected sections to resolve the conflict
4. Ensure terminology is consistent across all sections
5. Verify the fix by re-running `samgraha audit --domain {{ domain }}`
{% endif %}

---

## 4. Verification

After applying the fix:

```bash
samgraha audit --domain {{ domain }} --document {{ document_path }} --provider semantic
```

Expected: rule `{{ rule_id }}` score improves, finding resolves.

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
| Fix Plan Type | Semantic — Whole Document |
| Domain | {{ domain }} |
| Rule File | `audit/semantic/document/{{ domain }}-audit.md` |
| Generated | {{ created_at }} |
| Session | {{ session_id }} |
| Model | {{ model }} |
