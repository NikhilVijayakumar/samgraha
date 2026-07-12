# {{ document_title }} — Product Guide Document Audit Report

> **Domain:** product-guide
> **Scope:** document
> **Standard:** documentation-standards
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 4.5 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 4.5 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Product Guide documentation is a flat content domain — no derivation relationships. Document-level checks ensure the guide covers one topic, all required sections are present and populated, and body content is substantive rather than placeholder.

---

## Rule Results

### pg-doc-001 — Required sections present
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document contains all required sections per documentation-standards Product Guide requirements
- **Status:** {{ rule_001_status }}
- **Evidence:** {{ rule_001_evidence }}
- **Why this matters:** Product Guide without required sections (title, body) gives readers no structural anchor — the document cannot be identified or consumed.

### pg-doc-002 — No empty required sections
- **Severity:** error
- **Weight:** 1.0
- **Condition:** every required section has non-empty content (not just a heading)
- **Status:** {{ rule_002_status }}
- **Evidence:** {{ rule_002_evidence }}
- **Why this matters:** Empty sections create false completeness — a heading promises content that never arrives, wasting reader time.

### pg-doc-003 — Document covers one product topic
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** document has a single primary focus — does not mix unrelated product guides
- **Status:** {{ rule_003_status }}
- **Evidence:** {{ rule_003_evidence }}
- **Why this matters:** Bundling unrelated product guides into one document makes maintenance impossible — each guide drifts independently, and readers cannot find the specific topic they need.

### pg-doc-004 — Body section is substantive
- **Severity:** error
- **Weight:** 1.0
- **Condition:** body section contains substantive content (not just links or references)
- **Status:** {{ rule_004_status }}
- **Evidence:** {{ rule_004_evidence }}
- **Why this matters:** A Product Guide with an empty or link-only body gives readers no actual guidance — they arrive for answers and find signposts.

### pg-doc-005 — No duplicate content within document
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** no section repeats the same information as another section
- **Status:** {{ rule_005_status }}
- **Evidence:** {{ rule_005_evidence }}
- **Why this matters:** Duplicate content creates maintenance drift — one copy gets updated while the other silently goes stale, producing contradictions within the same document.

---

## Cross-Section Relationships

### pg-section-consistency (section_consistency)
- **Owner:** document
- **Description:** Sections within product guide docs are mutually consistent — title matches body content, purpose aligns with context

### pg-collection-coherence (collection_coherence)
- **Owner:** document
- **Description:** All product guide documents in the domain cohere as one system — no orphaned or contradictory guides

### pg-terminology-drift (terminology_drift)
- **Owner:** document
- **Description:** Terminology is consistent across all product guide sections — same concept, same name

---

## Section-Level Results

| Section | Rules | Passed | Failed | Errors | Warnings |
|---|---|---|---|---|---|
| title | {{ title_rules }} | {{ title_passed }} | {{ title_failed }} | {{ title_errors }} | {{ title_warnings }} |
| body | {{ body_rules }} | {{ body_passed }} | {{ body_failed }} | {{ body_errors }} | {{ body_warnings }} |
| purpose | {{ purpose_rules }} | {{ purpose_passed }} | {{ purpose_failed }} | {{ purpose_errors }} | {{ purpose_warnings }} |
| product_context | {{ context_rules }} | {{ context_passed }} | {{ context_failed }} | {{ context_errors }} | {{ context_warnings }} |
| public_contract | {{ contract_rules }} | {{ contract_passed }} | {{ contract_failed }} | {{ contract_errors }} | {{ contract_warnings }} |
| related | {{ related_rules }} | {{ related_passed }} | {{ related_failed }} | {{ related_errors }} | {{ related_warnings }} |

---

## Failures

| Rule | Severity | Weight | Evidence |
|---|---|---|---|
{{ failures_table }}

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
