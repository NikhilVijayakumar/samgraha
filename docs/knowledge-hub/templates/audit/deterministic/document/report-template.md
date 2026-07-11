# Deterministic Whole-Document Audit Report — {{ domain }}

**Document:** {{ document_path }}
**Standard:** {{ standard_name }}
**Audit Date:** {{ created_at }}

---

## 1. Structural Compliance

Checks the document against `audit/deterministic/document/{{ domain }}.yaml` rules.

| Rule | Description | Status | Severity |
|------|-------------|--------|----------|
{% for rule in rules -%}
| {{ rule.id }} | {{ rule.description }} | {% if rule.passed %}PASS{% else %}FAIL{% endif %} | {{ rule.severity }} |
{% endfor %}

{% if failed_rules | length > 0 %}
### Failures

| Rule | Message | Evidence |
|------|---------|----------|
{% for r in failed_rules -%}
| {{ r.id }} | {{ r.message }} | {% if r.evidence %}{{ r.evidence }}{% else %}—{% endif %} |
{% endfor %}
{% endif %}

---

## 2. Required Sections Coverage

| Section | Required | Present | Status |
|---------|:--------:|:-------:|--------|
{% for s in section_coverage -%}
| {{ s.semantic_type }} | {% if s.required %}✓{% else %}—{% endif %} | {% if s.present %}✓{% else %}✗{% endif %} | {{ s.status }} |
{% endfor %}

---

## 3. Score

**Deterministic Whole Score:** {{ score }} / 100

| Component | Weight | Contribution |
|-----------|--------|-------------|
{% for c in score_components -%}
| {{ c.name }} | {{ c.weight }}% | {{ c.contribution }} |
{% endfor %}

---

## 4. Metadata

| Field | Value |
|-------|-------|
| Audit Type | Deterministic — Whole Document |
| Domain | {{ domain }} |
| Rule File | `audit/deterministic/document/{{ domain }}.yaml` |
| Audit Date | {{ created_at }} |
| Session | {{ session_id }} |
