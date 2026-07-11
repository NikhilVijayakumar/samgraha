# Semantic Section Audit Report — {{ domain }}

**Document:** {{ document_path }}
**Standard:** {{ standard_name }}
**Audit Date:** {{ created_at }}

---

## 1. Section Scores

| Section | Score | Rating | Status |
|---------|------:|--------|--------|
{% for s in section_scores -%}
| {{ s.semantic_type }} | {{ s.score }} | {{ s.rating }} | {% if s.score >= 80 %}PASS{% elif s.score >= 50 %}WARN{% else %}FAIL{% endif %} |
{% endfor %}

---

## 2. Section Detail

{% for s in sections %}
### {{ s.semantic_type }}

**Score:** {{ s.score }} / 100 — **{{ s.rating }}**

**Engineering Intent:** {{ s.engineing_intent }}

| Criterion | Weight | Score | Description |
|-----------|--------|------:|-------------|
{% for c in s.criteria -%}
| {{ c.id }} | {{ c.weight }} | {{ c.score }} | {{ c.description }} |
{% endfor %}

{% if s.findings | length > 0 %}
**Findings:**
{% for f in s.findings -%}
- **{{ f.severity }}:** {{ f.message }}{% if f.evidence %} — "{{ f.evidence }}"{% endif %}
{% endfor %}
{% endif %}

{% endfor %}

---

## 3. Score

**Semantic Section Score:** {{ score }} / 100

Calculated as: average of all section scores, weighted by criterion count.

---

## 4. Metadata

| Field | Value |
|-------|-------|
| Audit Type | Semantic — Section |
| Domain | {{ domain }} |
| Rule Directory | `audit/semantic/section/{{ domain }}/` |
| Sections Evaluated | {{ sections | length }} |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
| Model | {{ model }} |
