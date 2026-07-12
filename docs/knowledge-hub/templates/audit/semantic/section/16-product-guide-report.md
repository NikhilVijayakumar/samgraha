# {{ document_title }} — Product Guide Semantic Section Audit Report

> **Domain:** product-guide
> **Scope:** section
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Document-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | 100 |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | 100 |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Section-level semantic audit checks that each Product Guide section delivers real content — not generic boilerplate, not placeholders, not marketing copy. Each section must be specific to this project, internally consistent, and backed by evidence.

---

## Section Results

### title
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ title_c1_status }}
- **Evidence:** {{ title_c1_evidence }}
- **Why this matters:** A title that names the specific product guide topic signals to readers exactly what they'll learn.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ title_c2_status }}
- **Evidence:** {{ title_c2_evidence }}
- **Why this matters:** A title that contradicts the body or purpose creates confusion before readers even start reading.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ title_c3_status }}
- **Evidence:** {{ title_c3_evidence }}
- **Why this matters:** A title with project-specific detail is findable and unambiguous — "Samgraha Backup Guide" beats "Backup Guide" every time.

---

### body
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ body_c1_status }}
- **Evidence:** {{ body_c1_evidence }}
- **Why this matters:** The body is the core of the Product Guide — without substantive, project-specific content, the guide has no reason to exist.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ body_c2_status }}
- **Evidence:** {{ body_c2_evidence }}
- **Why this matters:** Body content that contradicts the Purpose or Product Context means the guide is arguing with itself.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ body_c3_status }}
- **Evidence:** {{ body_c3_evidence }}
- **Why this matters:** Generic body content ("this feature is useful") without examples gives readers no concrete path forward.

---

### purpose
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ purpose_c1_status }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** Purpose anchors why the guide exists — without it, readers cannot determine if this guide addresses their need.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ purpose_c2_status }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** A purpose that claims one goal while the body solves a different problem makes the guide misleading.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ purpose_c3_status }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** A purpose statement grounded in specific user needs is more useful than a vague "this guide helps users."

---

### product_context
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ context_c1_status }}
- **Evidence:** {{ context_c1_evidence }}
- **Why this matters:** Product context without substance leaves readers unable to understand where this guide fits in the product landscape.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ context_c2_status }}
- **Evidence:** {{ context_c2_evidence }}
- **Why this matters:** Product context that contradicts the Body means the guide's positioning and its content are out of sync.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ context_c3_status }}
- **Evidence:** {{ context_c3_evidence }}
- **Why this matters:** Generic product context ("this is a useful product") without specifics gives readers no grounding.

---

### public_contract
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ contract_c1_status }}
- **Evidence:** {{ contract_c1_evidence }}
- **Why this matters:** Public contract without substance means the guide's interface is undocumented — users have no ground truth.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ contract_c2_status }}
- **Evidence:** {{ contract_c2_evidence }}
- **Why this matters:** A contract that lists flags the Body never uses, or omits flags the Body relies on, is a broken reference.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ contract_c3_status }}
- **Evidence:** {{ contract_c3_evidence }}
- **Why this matters:** A contract with types, defaults, and required/optional status is useful; one without them is just a list of names.

---

### related
#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory (40)
- **Status:** {{ related_c1_status }}
- **Evidence:** {{ related_c1_evidence }}
- **Why this matters:** Related section without links is a dead end — readers looking for connected topics find nothing.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory (30)
- **Status:** {{ related_c2_status }}
- **Evidence:** {{ related_c2_evidence }}
- **Why this matters:** Related links that point to documents contradicting this guide's content create contradictory guidance.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended (30)
- **Status:** {{ related_c3_status }}
- **Evidence:** {{ related_c3_evidence }}
- **Why this matters:** Related links with brief descriptions of why each is relevant help readers decide which to follow.

---

## Failures

| Section | Criterion | Severity | Weight | Evidence |
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
