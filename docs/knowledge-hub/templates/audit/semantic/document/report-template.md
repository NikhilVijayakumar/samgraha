# Semantic Whole-Document Audit Report — {{ domain }}

**Document:** {{ document_path }}
**Standard:** {{ standard_name }}
**Audit Date:** {{ created_at }}

---

## 1. LLM Judgment Results

Whole-document semantic checks — the LLM reads the full document and judges
cross-section consistency, terminology drift, contradictions, and collection coherence.

| Rule | Description | Score | Severity |
|------|-------------|------:|----------|
{% for rule in rules -%}
| {{ rule.id }} | {{ rule.description }} | {{ rule.score }} | {{ rule.severity }} |
{% endfor %}

---

## 2. Findings

{% if findings | length > 0 %}
| Severity | Rule | Message | Evidence |
|----------|------|---------|----------|
{% for f in findings -%}
| {{ f.severity }} | {{ f.rule_id }} | {{ f.message }} | {% if f.evidence %}"{{ f.evidence }}"{% else %}—{% endif %} |
{% endfor %}
{% else %}
_No findings._
{% endif %}

---

## 3. Cross-Section Consistency

{% if cross_section_issues | length > 0 %}
| Issue | Sections Involved | Description |
|-------|-------------------|-------------|
{% for i in cross_section_issues -%}
| {{ i.type }} | {{ i.sections }} | {{ i.description }} |
{% endfor %}
{% else %}
No cross-section consistency issues detected.
{% endif %}

---

## 4. Score

**Semantic Whole Score:** {{ score }} / 100

| Category | Weight | Score | Contribution |
|----------|--------|------:|-------------|
{% for c in score_components -%}
| {{ c.name }} | {{ c.weight }}% | {{ c.score }} | {{ c.contribution }} |
{% endfor %}

---

## 5. Metadata

| Field | Value |
|-------|-------|
| Audit Type | Semantic — Whole Document |
| Domain | {{ domain }} |
| Rule File | `audit/semantic/document/{{ domain }}-audit.md` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
| Model | {{ model }} |
