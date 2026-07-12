# {{ document_title }} — Product Guide Deterministic Section Audit Report

> **Domain:** product-guide
> **Scope:** section
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

**Why this matters:** Section-level deterministic audit checks that each Product Guide section exists, has substantive content, and follows structural conventions. A section that fails here cannot contribute meaningfully to the document.

---

## Section Results

### title
- **Rules checked:** {{ title_rules }}
- **Passed:** {{ title_passed }}
- **Failed:** {{ title_failed }}

#### pg-sec-title-001 — Title section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ title_001_status }}
- **Evidence:** {{ title_001_evidence }}
- **Why this matters:** A Product Guide without a title cannot be identified or referenced — readers cannot tell what topic this guide covers.

#### pg-sec-title-002 — Title is descriptive
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ title_002_status }}
- **Evidence:** {{ title_002_evidence }}
- **Why this matters:** A generic title like "Guide" gives readers no signal about the specific topic — the title must convey what the guide covers.

#### pg-sec-title-003 — Title matches document focus
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ title_003_status }}
- **Evidence:** {{ title_003_evidence }}
- **Why this matters:** A title that doesn't match the document's actual focus misleads readers and undermines discoverability.

---

### body
- **Rules checked:** {{ body_rules }}
- **Passed:** {{ body_passed }}
- **Failed:** {{ body_failed }}

#### pg-sec-body-001 — Body section exists
- **Severity:** error
- **Weight:** 1.5
- **Status:** {{ body_001_status }}
- **Evidence:** {{ body_001_evidence }}
- **Why this matters:** A Product Guide without a body has no content to deliver — the entire purpose of the guide is unfulfilled.

#### pg-sec-body-002 — Body has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Status:** {{ body_002_status }}
- **Evidence:** {{ body_002_evidence }}
- **Why this matters:** A body under 50 words cannot provide meaningful guidance — readers arrive for answers and find a stub.

#### pg-sec-body-003 — Body has clear structure
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ body_003_status }}
- **Evidence:** {{ body_003_evidence }}
- **Why this matters:** Unstructured body content forces readers to scan wall-of-text prose instead of jumping to the section they need.

#### pg-sec-body-004 — Body contains actionable content
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ body_004_status }}
- **Evidence:** {{ body_004_evidence }}
- **Why this matters:** A Product Guide body that describes without instructing leaves readers informed but unable to act.

---

### purpose
- **Rules checked:** {{ purpose_rules }}
- **Passed:** {{ purpose_passed }}
- **Failed:** {{ purpose_failed }}

#### pg-sec-purpose-001 — Purpose section exists
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}
- **Why this matters:** Optional section — but when present, it anchors why the guide exists and who it's for.

#### pg-sec-purpose-002 — Purpose states guide intent
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}
- **Why this matters:** A purpose that doesn't state why the guide exists leaves readers guessing whether this guide addresses their problem.

#### pg-sec-purpose-003 — Purpose defines audience
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}
- **Why this matters:** A guide without a defined audience cannot calibrate its depth — it may be too basic for experts or too advanced for beginners.

---

### product_context
- **Rules checked:** {{ context_rules }}
- **Passed:** {{ context_passed }}
- **Failed:** {{ context_failed }}

#### pg-sec-context-001 — Product context section exists
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ context_001_status }}
- **Evidence:** {{ context_001_evidence }}
- **Why this matters:** Optional section — but when present, it grounds the guide in the product's actual context and positioning.

#### pg-sec-context-002 — Product context provides background
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ context_002_status }}
- **Evidence:** {{ context_002_evidence }}
- **Why this matters:** Product context without background leaves readers unable to understand why this guide exists within the larger product.

#### pg-sec-context-003 — Product context references vision
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ context_003_status }}
- **Evidence:** {{ context_003_evidence }}
- **Why this matters:** Product context disconnected from Vision Documentation means the guide's positioning has no upstream source of truth.

---

### public_contract
- **Rules checked:** {{ contract_rules }}
- **Passed:** {{ contract_passed }}
- **Failed:** {{ contract_failed }}

#### pg-sec-contract-001 — Public contract section exists
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ contract_001_status }}
- **Evidence:** {{ contract_001_evidence }}
- **Why this matters:** Optional section — but when present, it defines the API or interface the guide exposes to users.

#### pg-sec-contract-002 — Public contract defines interface
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ contract_002_status }}
- **Evidence:** {{ contract_002_evidence }}
- **Why this matters:** A public contract without a defined interface leaves users guessing what inputs, outputs, and flags the guide's subject exposes.

#### pg-sec-contract-003 — Public contract defines stability
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ contract_003_status }}
- **Evidence:** {{ contract_003_evidence }}
- **Why this matters:** A contract without stability guarantees means users cannot trust that their integration won't break without notice.

---

### related
- **Rules checked:** {{ related_rules }}
- **Passed:** {{ related_passed }}
- **Failed:** {{ related_failed }}

#### pg-sec-related-001 — Related section exists
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ related_001_status }}
- **Evidence:** {{ related_001_evidence }}
- **Why this matters:** Optional section — but when present, it connects this guide to related documentation for discoverability.

#### pg-sec-related-002 — Related lists related guides
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ related_002_status }}
- **Evidence:** {{ related_002_evidence }}
- **Why this matters:** A related section with no links is a dead end — readers looking for connected topics find nothing.

#### pg-sec-related-003 — Related has valid links
- **Severity:** warning
- **Weight:** 0.5
- **Status:** {{ related_003_status }}
- **Evidence:** {{ related_003_evidence }}
- **Why this matters:** Broken links in the related section erode trust — readers who follow a link to "related guidance" and hit a 404 stop trusting the rest of the guide.

---

## Failures

| Rule | Section | Severity | Weight | Evidence |
|---|---|---|---|---|
{{ failures_table }}

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})
