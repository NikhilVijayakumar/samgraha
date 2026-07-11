# Deterministic Section Audit Report — {{ domain }}

**Document:** {{ document_path }}
**Standard:** {{ standard_name }}
**Audit Date:** {{ created_at }}

---

## 1. Section Scores

| Section | Rules Passed | Rules Failed | Score | Status |
|---------|:------------:|:------------:|------:|--------|
{% for s in section_scores -%}
| {{ s.semantic_type }} | {{ s.passed }} | {{ s.failed }} | {{ s.score }} | {% if s.score >= 80 %}PASS{% elif s.score >= 50 %}WARN{% else %}FAIL{% endif %} |
{% endfor %}

---

## 2. Section Detail

{% for s in sections %}
### {{ s.semantic_type }}

**Score:** {{ s.score }} / 100

| Rule | Description | Status | Severity |
|------|-------------|--------|----------|
{% for r in s.rules -%}
| {{ r.id }} | {{ r.description }} | {% if r.passed %}PASS{% else %}FAIL{% endif %} | {{ r.severity }} |
{% endfor %}

{% if s.failures | length > 0 %}
**Failures:**
{% for f in s.failures -%}
- **{{ f.id }}:** {{ f.message }}{% if f.evidence %} — {{ f.evidence }}{% endif %}
{% endfor %}
{% endif %}

{% endfor %}

---

## 3. Score

**Deterministic Section Score:** {{ score }} / 100

Calculated as: average of all section scores, weighted by rule count.

---

## 4. Metadata

| Field | Value |
|-------|-------|
| Audit Type | Deterministic — Section |
| Domain | {{ domain }} |
| Rule Directory | `audit/deterministic/section/{{ domain }}/` |
| Sections Evaluated | {{ sections | length }} |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
