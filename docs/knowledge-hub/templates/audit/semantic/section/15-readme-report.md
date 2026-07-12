# {{ document_title }} — README Semantic Section Audit Report

> **Domain:** readme
> **Scope:** section
> **Kind:** semantic
> **Date:** {{ audit_date }}
> **Auditor:** {{ auditor_name }}

---

## Section-Level Score

| Metric | Value |
|---|---|
| **Weight Sum** | {{ section_weight_sum }} |
| **Weighted Score** | {{ weighted_score }} |
| **Max Possible** | {{ section_weight_sum }} |
| **Percentage** | {{ score_percentage }} |
| **Verdict** | {{ verdict }} |

**Why this matters:** Semantic section audit evaluates the quality of each README section individually — whether content is substantive, internally consistent, and project-specific rather than generic. Each section contributes to the overall onboarding coherence.

---

## Section: project_name

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ name_c1_status }}
- **Confidence:** {{ name_c1_confidence }}
- **Evidence:** {{ name_c1_evidence }}
- **Why this matters:** A Project Name section without a specific project name is a placeholder that provides no identification value.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ name_c2_status }}
- **Confidence:** {{ name_c2_confidence }}
- **Evidence:** {{ name_c2_evidence }}
- **Why this matters:** Project name that contradicts the name used in Installation or Usage commands creates confusion about what the project is actually called.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ name_c3_status }}
- **Confidence:** {{ name_c3_confidence }}
- **Evidence:** {{ name_c3_evidence }}
- **Why this matters:** A generic project name section ("Project Name: [insert name]") provides no value.

---

## Section: short_description

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ short_desc_c1_status }}
- **Confidence:** {{ short_desc_c1_confidence }}
- **Evidence:** {{ short_desc_c1_evidence }}
- **Why this matters:** A Short Description section without project-specific content is a placeholder that provides no identification value.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ short_desc_c2_status }}
- **Confidence:** {{ short_desc_c2_confidence }}
- **Evidence:** {{ short_desc_c2_evidence }}
- **Why this matters:** Short Description that contradicts the Vision Statement or Overview creates confusion about what the project actually does.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ short_desc_c3_status }}
- **Confidence:** {{ short_desc_c3_confidence }}
- **Evidence:** {{ short_desc_c3_evidence }}
- **Why this matters:** A generic short description ("A software project") provides no value.

---

## Section: overview

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ overview_c1_status }}
- **Confidence:** {{ overview_c1_confidence }}
- **Evidence:** {{ overview_c1_evidence }}
- **Why this matters:** An Overview section without project-specific content is a placeholder that provides no architectural guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ overview_c2_status }}
- **Confidence:** {{ overview_c2_confidence }}
- **Evidence:** {{ overview_c2_evidence }}
- **Why this matters:** Overview that contradicts Architecture Documentation creates confusion about the system's actual structure.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ overview_c3_status }}
- **Confidence:** {{ overview_c3_confidence }}
- **Evidence:** {{ overview_c3_evidence }}
- **Why this matters:** Generic overview descriptions ("This project is a software system") provide no architectural insight.

---

## Section: purpose

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ purpose_c1_status }}
- **Confidence:** {{ purpose_c1_confidence }}
- **Evidence:** {{ purpose_c1_evidence }}
- **Why this matters:** A Purpose section without project-specific content is a placeholder that provides no identification value.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ purpose_c2_status }}
- **Confidence:** {{ purpose_c2_confidence }}
- **Evidence:** {{ purpose_c2_evidence }}
- **Why this matters:** Purpose that contradicts the Vision Statement creates confusion about what the project actually aspires to.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ purpose_c3_status }}
- **Confidence:** {{ purpose_c3_confidence }}
- **Evidence:** {{ purpose_c3_evidence }}
- **Why this matters:** Generic purpose statements ("This project exists to solve problems") provide no identification value.

---

## Section: key_capabilities

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ caps_c1_status }}
- **Confidence:** {{ caps_c1_confidence }}
- **Evidence:** {{ caps_c1_evidence }}
- **Why this matters:** A Key Capabilities section without project-specific content is a placeholder that provides no feature overview.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ caps_c2_status }}
- **Confidence:** {{ caps_c2_confidence }}
- **Evidence:** {{ caps_c2_evidence }}
- **Why this matters:** Key Capabilities that contradict Feature Documentation creates confusion about what the project actually delivers.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ caps_c3_status }}
- **Confidence:** {{ caps_c3_confidence }}
- **Evidence:** {{ caps_c3_evidence }}
- **Why this matters:** Generic capability descriptions ("It has many features") provide no overview value.

---

## Section: repository_structure

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ repo_c1_status }}
- **Confidence:** {{ repo_c1_confidence }}
- **Evidence:** {{ repo_c1_evidence }}
- **Why this matters:** A Repository Structure section without project-specific content is a placeholder that provides no layout guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ repo_c2_status }}
- **Confidence:** {{ repo_c2_confidence }}
- **Evidence:** {{ repo_c2_evidence }}
- **Why this matters:** Repository Structure that contradicts Engineering Documentation creates confusion about the project's actual layout.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ repo_c3_status }}
- **Confidence:** {{ repo_c3_confidence }}
- **Evidence:** {{ repo_c3_evidence }}
- **Why this matters:** Generic layout descriptions ("The code is organized in folders") provide no navigation value.

---

## Section: documentation_structure

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ doc_struct_c1_status }}
- **Confidence:** {{ doc_struct_c1_confidence }}
- **Evidence:** {{ doc_struct_c1_evidence }}
- **Why this matters:** A Documentation Structure section without project-specific content is a placeholder that provides no doc navigation guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ doc_struct_c2_status }}
- **Confidence:** {{ doc_struct_c2_confidence }}
- **Evidence:** {{ doc_struct_c2_evidence }}
- **Why this matters:** Documentation Structure that contradicts the actual doc layout creates confusion about where to find information.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ doc_struct_c3_status }}
- **Confidence:** {{ doc_struct_c3_confidence }}
- **Evidence:** {{ doc_struct_c3_evidence }}
- **Why this matters:** Generic documentation structure descriptions ("Docs are in the docs folder") provide no navigation value.

---

## Section: getting_started

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ gs_c1_status }}
- **Confidence:** {{ gs_c1_confidence }}
- **Evidence:** {{ gs_c1_evidence }}
- **Why this matters:** A Getting Started section without a minimal path to first use is a placeholder that provides no onboarding value.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ gs_c2_status }}
- **Confidence:** {{ gs_c2_confidence }}
- **Evidence:** {{ gs_c2_evidence }}
- **Why this matters:** Getting Started steps that contradict Installation or Build instructions create confusion about the actual setup process.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ gs_c3_status }}
- **Confidence:** {{ gs_c3_confidence }}
- **Evidence:** {{ gs_c3_evidence }}
- **Why this matters:** Generic getting started descriptions ("Follow the instructions below") provide no quick-start value.

---

## Section: installation

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ install_c1_status }}
- **Confidence:** {{ install_c1_confidence }}
- **Evidence:** {{ install_c1_evidence }}
- **Why this matters:** An Installation section without step-by-step instructions is a placeholder that provides no installation guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ install_c2_status }}
- **Confidence:** {{ install_c2_confidence }}
- **Evidence:** {{ install_c2_evidence }}
- **Why this matters:** Installation steps that contradict Build instructions create confusion about the actual installation process.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ install_c3_status }}
- **Confidence:** {{ install_c3_confidence }}
- **Evidence:** {{ install_c3_evidence }}
- **Why this matters:** Generic installation descriptions ("Install the package") provide no actionable guidance.

---

## Section: build

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ build_c1_status }}
- **Confidence:** {{ build_c1_confidence }}
- **Evidence:** {{ build_c1_evidence }}
- **Why this matters:** A Build section without build instructions is a placeholder that provides no build guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ build_c2_status }}
- **Confidence:** {{ build_c2_confidence }}
- **Evidence:** {{ build_c2_evidence }}
- **Why this matters:** Build steps that contradict Build Documentation create confusion about the actual build process.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ build_c3_status }}
- **Confidence:** {{ build_c3_confidence }}
- **Evidence:** {{ build_c3_evidence }}
- **Why this matters:** Generic build descriptions ("Build the project") provide no actionable guidance.

---

## Section: usage

### Criteria

#### C1 — Working command/code examples for primary functions
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ usage_c1_status }}
- **Confidence:** {{ usage_c1_confidence }}
- **Evidence:** {{ usage_c1_evidence }}
- **Why this matters:** "Check --help for more information" instead of actual examples means the README has no usage guidance — it defers to the very thing the user is trying to avoid reading.

#### C2 — Expected output shown
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ usage_c2_status }}
- **Confidence:** {{ usage_c2_confidence }}
- **Evidence:** {{ usage_c2_evidence }}
- **Why this matters:** Examples without expected output leave readers unable to verify correctness — they run the command and don't know if the output is right.

#### C3 — Common workflows demonstrated beyond a single command
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ usage_c3_status }}
- **Confidence:** {{ usage_c3_confidence }}
- **Evidence:** {{ usage_c3_evidence }}
- **Why this matters:** Usage that only covers trivial/toy cases, not the primary function the project exists for, fails to demonstrate real value.

---

## Section: development

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ dev_c1_status }}
- **Confidence:** {{ dev_c1_confidence }}
- **Evidence:** {{ dev_c1_evidence }}
- **Why this matters:** A Development section without local setup instructions is a placeholder that provides no development guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ dev_c2_status }}
- **Confidence:** {{ dev_c2_confidence }}
- **Evidence:** {{ dev_c2_evidence }}
- **Why this matters:** Development setup that contradicts Contributing guidelines creates confusion about the actual development workflow.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ dev_c3_status }}
- **Confidence:** {{ dev_c3_confidence }}
- **Evidence:** {{ dev_c3_evidence }}
- **Why this matters:** Generic development descriptions ("Set up your environment") provide no actionable guidance.

---

## Section: contributing

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ contrib_c1_status }}
- **Confidence:** {{ contrib_c1_confidence }}
- **Evidence:** {{ contrib_c1_evidence }}
- **Why this matters:** A Contributing section without contribution guidelines is a placeholder that provides no contribution guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ contrib_c2_status }}
- **Confidence:** {{ contrib_c2_confidence }}
- **Evidence:** {{ contrib_c2_evidence }}
- **Why this matters:** Contributing guidelines that contradict Development setup create confusion about the actual contribution workflow.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ contrib_c3_status }}
- **Confidence:** {{ contrib_c3_confidence }}
- **Evidence:** {{ contrib_c3_evidence }}
- **Why this matters:** Generic contributing descriptions ("We welcome contributions") provide no actionable guidance.

---

## Section: configuration

### Criteria

#### C1 — Configuration options listed with defaults (table form)
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ config_c1_status }}
- **Confidence:** {{ config_c1_confidence }}
- **Evidence:** {{ config_c1_evidence }}
- **Why this matters:** "The config file is in YAML, you can set env vars too" with no actual options listed means the README has no configuration guidance.

#### C2 — Valid values stated per option
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ config_c2_status }}
- **Confidence:** {{ config_c2_confidence }}
- **Evidence:** {{ config_c2_evidence }}
- **Why this matters:** Options listed without default values leave readers guessing what the valid configuration space looks like.

#### C3 — Working example provided
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ config_c3_status }}
- **Confidence:** {{ config_c3_confidence }}
- **Evidence:** {{ config_c3_evidence }}
- **Why this matters:** No working example means readers can't tell what a valid config actually looks like.

---

## Section: license

### Criteria

#### C1 — Section exists with substantive content specific to this project
- **Weight:** mandatory
- **Score if passed:** 40
- **Status:** {{ license_c1_status }}
- **Confidence:** {{ license_c1_confidence }}
- **Evidence:** {{ license_c1_evidence }}
- **Why this matters:** A License section without a specific license type is a placeholder that provides no legal guidance.

#### C2 — Content is internally consistent and does not contradict other sections
- **Weight:** mandatory
- **Score if passed:** 30
- **Status:** {{ license_c2_status }}
- **Confidence:** {{ license_c2_confidence }}
- **Evidence:** {{ license_c2_evidence }}
- **Why this matters:** License that contradicts LICENSE file or other project metadata creates legal confusion.

#### C3 — Content includes concrete examples, evidence, or project-specific detail
- **Weight:** recommended
- **Score if passed:** 30
- **Status:** {{ license_c3_status }}
- **Confidence:** {{ license_c3_confidence }}
- **Evidence:** {{ license_c3_evidence }}
- **Why this matters:** Generic license descriptions ("This project is licensed") without specifying the license provide no legal clarity.

---

## Score History

| Date | Auditor | Score | Verdict | Revision |
|---|---|---|---|---|
| {{ audit_date }} | {{ auditor_name }} | {{ weighted_score }} | {{ verdict }} | 1 |

---

## Trend

{{ trend_indicator }} ({{ trend_description }})

---

## Failures

| Criterion | Severity | Section | Evidence | Regression? |
|---|---|---|---|---|
{{ failures_table }}

---

## Summary

{{ summary_text }}

### Section-Level Breakdown

| Section | Weight | Score | Status |
|---|---|---|---|
| project_name | {{ name_weight }} | {{ name_score }} | {{ name_status }} |
| short_description | {{ short_desc_weight }} | {{ short_desc_score }} | {{ short_desc_status }} |
| overview | {{ overview_weight }} | {{ overview_score }} | {{ overview_status }} |
| purpose | {{ purpose_weight }} | {{ purpose_score }} | {{ purpose_status }} |
| key_capabilities | {{ caps_weight }} | {{ caps_score }} | {{ caps_status }} |
| repository_structure | {{ repo_weight }} | {{ repo_score }} | {{ repo_status }} |
| documentation_structure | {{ doc_struct_weight }} | {{ doc_struct_score }} | {{ doc_struct_status }} |
| getting_started | {{ gs_weight }} | {{ gs_score }} | {{ gs_status }} |
| installation | {{ install_weight }} | {{ install_score }} | {{ install_status }} |
| build | {{ build_weight }} | {{ build_score }} | {{ build_status }} |
| usage | {{ usage_weight }} | {{ usage_score }} | {{ usage_status }} |
| development | {{ dev_weight }} | {{ dev_score }} | {{ dev_status }} |
| contributing | {{ contrib_weight }} | {{ contrib_score }} | {{ contrib_status }} |
| configuration | {{ config_weight }} | {{ config_score }} | {{ config_status }} |
| license | {{ license_weight }} | {{ license_score }} | {{ license_status }} |
