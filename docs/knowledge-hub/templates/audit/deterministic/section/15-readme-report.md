# {{ document_title }} — README Section Audit Report

> **Domain:** readme
> **Scope:** section
> **Standard:** documentation-standards
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

**Why this matters:** README sections define specific onboarding components. Section-level audits verify each concern is internally consistent and substantiated — the building blocks of a coherent entry point.

---

## Section: project_name

### Rules

#### readme-sec-name-001 — Project name section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'project_name'
- **Status:** {{ name_001_status }}
- **Evidence:** {{ name_001_evidence }}

#### readme-sec-name-002 — Project name is present
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a non-empty project name
- **Status:** {{ name_002_status }}
- **Evidence:** {{ name_002_evidence }}

#### readme-sec-name-003 — Project name matches repository
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** project name matches the repository or package name
- **Status:** {{ name_003_status }}
- **Evidence:** {{ name_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-name-derives-build | derives_from | build:versioning_naming | incoming | {{ rel_name_build }} |

---

## Section: short_description

### Rules

#### readme-sec-short-desc-001 — Short description section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'short_description'
- **Status:** {{ short_desc_001_status }}
- **Evidence:** {{ short_desc_001_evidence }}

#### readme-sec-short-desc-002 — Short description is concise
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section is no longer than 100 words
- **Status:** {{ short_desc_002_status }}
- **Evidence:** {{ short_desc_002_evidence }}

#### readme-sec-short-desc-003 — Short description references vision
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section aligns with Vision Statement
- **Status:** {{ short_desc_003_status }}
- **Evidence:** {{ short_desc_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-short-desc-derives-vision | derives_from | vision:vision_statement | incoming | {{ rel_short_desc_vision }} |

---

## Section: overview

### Rules

#### readme-sec-overview-001 — Overview section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'overview'
- **Status:** {{ overview_001_status }}
- **Evidence:** {{ overview_001_evidence }}

#### readme-sec-overview-002 — Overview describes system at high level
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section describes the system's architecture or purpose at a high level
- **Status:** {{ overview_002_status }}
- **Evidence:** {{ overview_002_evidence }}

#### readme-sec-overview-003 — Overview references architecture
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Architecture Documentation
- **Status:** {{ overview_003_status }}
- **Evidence:** {{ overview_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-overview-derives-architecture | derives_from | architecture:system_overview | incoming | {{ rel_overview_architecture }} |

---

## Section: purpose

### Rules

#### readme-sec-purpose-001 — Purpose section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'purpose'
- **Status:** {{ purpose_001_status }}
- **Evidence:** {{ purpose_001_evidence }}

#### readme-sec-purpose-002 — Purpose states project intent
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains a statement of why this project exists
- **Status:** {{ purpose_002_status }}
- **Evidence:** {{ purpose_002_evidence }}

#### readme-sec-purpose-003 — Purpose references vision
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Vision Documentation
- **Status:** {{ purpose_003_status }}
- **Evidence:** {{ purpose_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-purpose-derives-vision | derives_from | vision:purpose | incoming | {{ rel_purpose_vision }} |

---

## Section: key_capabilities

### Rules

#### readme-sec-caps-001 — Key capabilities section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'key_capabilities'
- **Status:** {{ caps_001_status }}
- **Evidence:** {{ caps_001_evidence }}

#### readme-sec-caps-002 — Key capabilities lists features
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section lists at least three key capabilities
- **Status:** {{ caps_002_status }}
- **Evidence:** {{ caps_002_evidence }}

#### readme-sec-caps-003 — Key capabilities references feature docs
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Feature Documentation
- **Status:** {{ caps_003_status }}
- **Evidence:** {{ caps_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-capabilities-derives-feature | derives_from | feature:purpose | incoming | {{ rel_caps_feature }} |

---

## Section: repository_structure

### Rules

#### readme-sec-repo-001 — Repository structure section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'repository_structure'
- **Status:** {{ repo_001_status }}
- **Evidence:** {{ repo_001_evidence }}

#### readme-sec-repo-002 — Repository structure describes layout
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section describes the directory or file layout
- **Status:** {{ repo_002_status }}
- **Evidence:** {{ repo_002_evidence }}

#### readme-sec-repo-003 — Repository structure references engineering
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Engineering Documentation
- **Status:** {{ repo_003_status }}
- **Evidence:** {{ repo_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-repo-struct-derives-engineering | derives_from | engineering:project_layout | incoming | {{ rel_repo_engineering }} |

---

## Section: documentation_structure

### Rules

#### readme-sec-doc-struct-001 — Documentation structure section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'documentation_structure'
- **Status:** {{ doc_struct_001_status }}
- **Evidence:** {{ doc_struct_001_evidence }}

#### readme-sec-doc-struct-002 — Documentation structure describes doc layout
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section describes where documentation lives and how it's organized
- **Status:** {{ doc_struct_002_status }}
- **Evidence:** {{ doc_struct_002_evidence }}

#### readme-sec-doc-struct-003 — Documentation structure references architecture
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Architecture Documentation
- **Status:** {{ doc_struct_003_status }}
- **Evidence:** {{ doc_struct_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-doc-struct-derives-architecture | derives_from | architecture:system_overview | incoming | {{ rel_doc_struct_architecture }} |

---

## Section: getting_started

### Rules

#### readme-sec-gs-001 — Getting started section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'getting_started'
- **Status:** {{ gs_001_status }}
- **Evidence:** {{ gs_001_evidence }}

#### readme-sec-gs-002 — Getting started provides quick start
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section provides a minimal path to first use
- **Status:** {{ gs_002_status }}
- **Evidence:** {{ gs_002_evidence }}

#### readme-sec-gs-003 — Getting started references build
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Build Documentation
- **Status:** {{ gs_003_status }}
- **Evidence:** {{ gs_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-getting-started-derives-build | derives_from | build:documentation_quality | incoming | {{ rel_gs_build }} |

---

## Section: installation

### Rules

#### readme-sec-install-001 — Installation section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'installation'
- **Status:** {{ install_001_status }}
- **Evidence:** {{ install_001_evidence }}

#### readme-sec-install-002 — Installation provides steps
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section provides step-by-step installation instructions
- **Status:** {{ install_002_status }}
- **Evidence:** {{ install_002_evidence }}

#### readme-sec-install-003 — Installation references build
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Build Documentation
- **Status:** {{ install_003_status }}
- **Evidence:** {{ install_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-install-derives-build | derives_from | build:versioning_naming | incoming | {{ rel_install_build }} |

---

## Section: build

### Rules

#### readme-sec-build-001 — Build section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'build'
- **Status:** {{ build_001_status }}
- **Evidence:** {{ build_001_evidence }}

#### readme-sec-build-002 — Build provides instructions
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section provides build instructions
- **Status:** {{ build_002_status }}
- **Evidence:** {{ build_002_evidence }}

#### readme-sec-build-003 — Build references build docs
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Build Documentation
- **Status:** {{ build_003_status }}
- **Evidence:** {{ build_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-build-derives-build | derives_from | build:documentation_quality | incoming | {{ rel_build_build }} |

---

## Section: usage

### Rules

#### readme-sec-usage-001 — Usage section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'usage'
- **Status:** {{ usage_001_status }}
- **Evidence:** {{ usage_001_evidence }}

#### readme-sec-usage-002 — Usage provides examples
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one usage example or code snippet
- **Status:** {{ usage_002_status }}
- **Evidence:** {{ usage_002_evidence }}

#### readme-sec-usage-003 — Usage references feature docs
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Feature Documentation for detailed usage
- **Status:** {{ usage_003_status }}
- **Evidence:** {{ usage_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-usage-derives-feature | derives_from | feature:purpose | incoming | {{ rel_usage_feature }} |

---

## Section: development

### Rules

#### readme-sec-dev-001 — Development section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'development'
- **Status:** {{ dev_001_status }}
- **Evidence:** {{ dev_001_evidence }}

#### readme-sec-dev-002 — Development provides setup instructions
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section provides local development setup instructions
- **Status:** {{ dev_002_status }}
- **Evidence:** {{ dev_002_evidence }}

#### readme-sec-dev-003 — Development references engineering
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Engineering Documentation
- **Status:** {{ dev_003_status }}
- **Evidence:** {{ dev_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-dev-derives-engineering | derives_from | engineering:code_standards | incoming | {{ rel_dev_engineering }} |

---

## Section: contributing

### Rules

#### readme-sec-contrib-001 — Contributing section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'contributing'
- **Status:** {{ contrib_001_status }}
- **Evidence:** {{ contrib_001_evidence }}

#### readme-sec-contrib-002 — Contributing provides guidelines
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section provides contribution guidelines
- **Status:** {{ contrib_002_status }}
- **Evidence:** {{ contrib_002_evidence }}

#### readme-sec-contrib-003 — Contributing references engineering
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section references Engineering Documentation
- **Status:** {{ contrib_003_status }}
- **Evidence:** {{ contrib_003_evidence }}

### Relationships

| ID | Type | Target | Direction | Status |
|---|---|---|---|---|
| readme-contrib-derives-engineering | derives_from | engineering:code_standards | incoming | {{ rel_contrib_engineering }} |

---

## Section: configuration

### Rules

#### readme-sec-config-001 — Configuration section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'configuration'
- **Status:** {{ config_001_status }}
- **Evidence:** {{ config_001_evidence }}

#### readme-sec-config-002 — Configuration has substantive content
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section contains at least one paragraph of project-specific content
- **Status:** {{ config_002_status }}
- **Evidence:** {{ config_002_evidence }}

#### readme-sec-config-003 — Configuration is specific to this project
- **Severity:** warning
- **Weight:** 0.5
- **Condition:** section contains project-specific details, not generic boilerplate
- **Status:** {{ config_003_status }}
- **Evidence:** {{ config_003_evidence }}

---

## Section: license

### Rules

#### readme-sec-license-001 — License section exists
- **Severity:** error
- **Weight:** 1.5
- **Condition:** document has a section with semantic_type = 'license'
- **Status:** {{ license_001_status }}
- **Evidence:** {{ license_001_evidence }}

#### readme-sec-license-002 — License specifies license type
- **Severity:** error
- **Weight:** 1.0
- **Condition:** section specifies the project's license
- **Status:** {{ license_002_status }}
- **Evidence:** {{ license_002_evidence }}

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

| Rule | Severity | Section | Evidence | Regression? |
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
