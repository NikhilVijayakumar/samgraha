use schemas::standard::{AuditRuleDef, SectionDefinition, StandardDefinition, StandardRelationship};

pub fn all_builtin_standards() -> Vec<StandardDefinition> {
    vec![
        readme_standard(),
        vision_standard(),
        philosophy_standard(),
        architecture_standard(),
        feature_standard(),
        feature_design_standard(),
        feature_technical_standard(),
        design_standard(),
        engineering_standard(),
        external_context_standard(),
        prototype_standard(),
    ]
}

// ── Common semantic types shared across standards ──────────────────────────────

fn sec(
    canonical: &str,
    semantic_type: &str,
    aliases: &[&str],
    required: bool,
    desc: &str,
) -> SectionDefinition {
    SectionDefinition {
        canonical_name: canonical.into(),
        semantic_type: semantic_type.into(),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        required,
        description: desc.into(),
    }
}

// Reusable section defs present in multiple standards.
fn purpose(required: bool) -> SectionDefinition {
    sec("Purpose", "purpose", &["Purpose", "Overview", "Summary"], required, "Why this document exists")
}
fn constraints() -> SectionDefinition {
    sec("Constraints", "constraints", &["Constraints", "Limitations", "Non-Functional Requirements"], false, "Constraints and limitations")
}
fn dependencies() -> SectionDefinition {
    sec("Dependencies", "dependencies", &["Dependencies", "Dependency", "Depends On"], false, "Dependencies")
}
fn traceability() -> SectionDefinition {
    sec("Traceability", "traceability", &["Traceability", "Traces To", "Derived From"], false, "Traceability links")
}
fn non_goals() -> SectionDefinition {
    sec("Non-Goals", "non_goals", &["Non-Goals", "Non Goals", "Out of Scope", "Not In Scope"], false, "Explicit non-goals")
}
fn future_extensions() -> SectionDefinition {
    sec("Future Extensions", "future_extensions", &["Future Extensions", "Future", "Roadmap", "Future Work"], false, "Future capabilities")
}
fn success_criteria() -> SectionDefinition {
    sec("Success Criteria", "success_criteria", &["Success Criteria", "Acceptance Criteria", "Definition of Done"], false, "How success is measured")
}
fn security_considerations() -> SectionDefinition {
    sec("Security Considerations", "security_considerations", &["Security Considerations", "Security"], false, "Security implications")
}
fn performance_considerations() -> SectionDefinition {
    sec("Performance Considerations", "performance_considerations", &["Performance Considerations", "Performance"], false, "Performance requirements and constraints")
}
fn failure_handling() -> SectionDefinition {
    sec("Failure Handling", "failure_handling", &["Failure Handling", "Error Handling", "Failures", "Fault Handling"], false, "How failures are handled")
}
fn extension_points() -> SectionDefinition {
    sec("Extension Points", "extension_points", &["Extension Points", "Extensions", "Extension", "Extensibility"], false, "How the component can be extended")
}

// ── Standards ──────────────────────────────────────────────────────────────────

fn readme_standard() -> StandardDefinition {
    StandardDefinition {
        id: "readme".into(),
        name: "README Standard".into(),
        version: "1.0.0".into(),
        domain: "readme".into(),
        description: "Primary repository entry point and documentation navigation.".into(),
        required_sections: vec![
            sec("Title", "title", &["Title"], true, "Repository name and brief description"),
            sec("Getting Started", "getting_started", &["Getting Started", "Quick Start", "Installation"], true, "Installation and basic usage"),
            sec("Documentation", "documentation", &["Documentation", "Docs", "Further Reading"], true, "Links to key documentation"),
        ],
        prohibited_content: vec!["Detailed API reference".into()],
        relationships: vec![relationship("readme", "vision", "references")],
        audit_rules: vec![
            rule("readme-001", "README exists", "Repository must have a README.md", "error", "corpus_exists", ""),
            rule("readme-002", "Has title", "README must have a top-level title", "error", "has_title", ""),
            rule("readme-003", "Has getting started", "README must have getting started section", "warning", "has_section", "Getting Started"),
        ],
    }
}

fn vision_standard() -> StandardDefinition {
    StandardDefinition {
        id: "vision".into(),
        name: "Vision Standard".into(),
        version: "1.0.0".into(),
        domain: "vision".into(),
        description: "Product vision and long-term direction.".into(),
        required_sections: vec![
            purpose(true),
            sec("Vision", "vision_statement", &["Vision", "Long-Term Vision", "The Vision"], true, "The long-term vision statement"),
            sec("Problem", "problem", &["Problem", "Problem Statement", "The Problem"], true, "The problem being solved"),
            sec("Solution", "solution", &["Solution", "The Solution", "Our Solution"], true, "How the problem is solved"),
            sec("Platform Pillars", "pillars", &["Platform Pillars", "Pillars", "Foundations", "Core Pillars"], false, "Foundational capabilities"),
            sec("Philosophy", "philosophy", &["Philosophy", "Product Philosophy", "Design Philosophy"], false, "Design and product philosophy"),
            sec("Guiding Principles", "guiding_principles", &["Guiding Principles", "Principles", "Core Principles"], false, "Guiding principles"),
            sec("Target Audience", "target_audience", &["Target Audience", "Audience", "Who Is This For"], true, "Who the product serves"),
            success_criteria(),
            traceability(),
        ],
        prohibited_content: vec!["Implementation details".into(), "Architecture decisions".into()],
        relationships: vec![
            relationship("vision", "feature", "derives"),
            relationship("vision", "philosophy", "inspires"),
        ],
        audit_rules: vec![
            rule("vision-001", "Has purpose", "Vision must include product purpose", "error", "has_section", "Purpose"),
            rule("vision-002", "Has audience", "Vision must define target audience", "warning", "has_section", "Target Audience"),
            rule("vision-003", "No implementation", "Vision must not contain implementation details", "warning", "no_implementation", ""),
        ],
    }
}

fn philosophy_standard() -> StandardDefinition {
    StandardDefinition {
        id: "philosophy".into(),
        name: "Philosophy Standard".into(),
        version: "1.0.0".into(),
        domain: "philosophy".into(),
        description: "Product philosophy and design principles.".into(),
        required_sections: vec![
            purpose(false),
            sec("Principles", "guiding_principles", &["Principles", "Core Principles", "Design Principles"], true, "Core design and engineering principles"),
            sec("Values", "values", &["Values", "Core Values", "What We Value"], true, "What the product values"),
            sec("Trade-offs", "tradeoffs", &["Trade-offs", "Trade offs", "Tradeoffs", "Deliberate Trade-offs"], true, "Deliberate trade-offs made"),
        ],
        prohibited_content: vec!["Specific technology choices".into()],
        relationships: vec![
            relationship("philosophy", "architecture", "guides"),
            relationship("philosophy", "design", "guides"),
        ],
        audit_rules: vec![
            rule("phil-001", "Has principles", "Philosophy must document principles", "error", "has_section", "Principles"),
            rule("phil-002", "Has values", "Philosophy must document values", "warning", "has_section", "Values"),
            rule("phil-003", "Has trade-offs", "Philosophy should document trade-offs", "suggestion", "has_section", "Trade-offs"),
        ],
    }
}

fn architecture_standard() -> StandardDefinition {
    StandardDefinition {
        id: "architecture".into(),
        name: "Architecture Standard".into(),
        version: "1.0.0".into(),
        domain: "architecture".into(),
        description: "Structural organization of the system.".into(),
        required_sections: vec![
            purpose(false),
            sec("System Overview", "system_overview", &["System Overview", "Overview", "Architecture Overview"], true, "High-level system description"),
            sec("Component Model", "component_model", &["Component Model", "Components", "Component Architecture"], true, "Components and responsibilities"),
            sec("Communication", "communication_paths", &["Communication", "Communication Paths", "Component Communication"], true, "How components communicate"),
            sec("Data Flow", "data_flow", &["Data Flow", "Data Movement", "Information Flow"], true, "How data moves through the system"),
            sec("Security", "security_considerations", &["Security", "Security Architecture", "Security Model"], true, "Security architecture"),
            sec("Rationale", "rationale", &["Rationale", "Decision Rationale", "Architectural Decisions", "Why"], false, "Why this architecture was chosen"),
            constraints(),
            traceability(),
        ],
        prohibited_content: vec!["Implementation details".into(), "Technology frameworks".into()],
        relationships: vec![
            relationship("architecture", "feature-technical", "constrains"),
            relationship("architecture", "engineering", "guides"),
        ],
        audit_rules: vec![
            rule("arch-001", "Has overview", "Architecture must include system overview", "error", "has_section", "System Overview"),
            rule("arch-002", "Has component model", "Architecture must define component responsibilities", "error", "has_section", "Component Model"),
            rule("arch-003", "No implementation details", "Architecture must avoid implementation specifics", "warning", "no_implementation", ""),
            rule("arch-004", "Has security", "Architecture must address security", "warning", "has_section", "Security"),
        ],
    }
}

fn feature_standard() -> StandardDefinition {
    StandardDefinition {
        id: "feature".into(),
        name: "Feature Standard".into(),
        version: "1.0.0".into(),
        domain: "feature".into(),
        description: "Atomic functional capability specifications.".into(),
        required_sections: vec![
            purpose(true),
            sec("Functional Requirements", "functional_requirements", &[
                "Functional Requirements", "Requirements", "FRs",
                "Functional Reqs", "Feature Requirements",
            ], true, "Functional requirements"),
            sec("Business Rules", "business_rules", &["Business Rules", "Rules", "Business Logic"], false, "Business rules governing this feature"),
            sec("Inputs", "inputs", &["Inputs", "Input", "Input Data"], false, "Inputs the feature consumes"),
            sec("Outputs", "outputs", &["Outputs", "Output", "Output Data"], false, "Outputs the feature produces"),
            constraints(),
            dependencies(),
            sec("Acceptance Criteria", "acceptance_criteria", &[
                "Acceptance Criteria", "Success Criteria", "Definition of Done", "Criteria",
            ], true, "How to verify the feature is complete"),
            non_goals(),
            future_extensions(),
            traceability(),
        ],
        prohibited_content: vec!["Implementation details".into(), "Architecture".into()],
        relationships: vec![
            relationship("feature", "feature-design", "designs"),
            relationship("feature", "feature-technical", "implements"),
            relationship("vision", "feature", "derives-from"),
        ],
        audit_rules: vec![
            rule("feat-001", "Has purpose", "Feature must document its purpose", "error", "has_section", "Purpose"),
            rule("feat-002", "Has requirements", "Feature must list functional requirements", "error", "has_section", "Functional Requirements"),
            rule("feat-003", "Has acceptance criteria", "Feature must define acceptance criteria", "error", "has_section", "Acceptance Criteria"),
            rule("feat-004", "Technology independent", "Feature must not specify technology", "warning", "no_implementation", ""),
        ],
    }
}

fn feature_design_standard() -> StandardDefinition {
    StandardDefinition {
        id: "feature-design".into(),
        name: "Feature Design Standard".into(),
        version: "1.0.0".into(),
        domain: "feature-design".into(),
        description: "User-centered design for a single feature.".into(),
        required_sections: vec![
            purpose(false),
            sec("User Experience", "user_experience", &["User Experience", "UX", "User Flow"], true, "How users interact with the feature"),
            sec("Workflow", "workflow", &["Workflow", "User Workflow", "Flow"], true, "Step-by-step user workflow"),
            sec("States", "states", &["States", "UI States", "Application States", "State Transitions"], true, "Empty, loading, success, error states"),
            non_goals(),
            constraints(),
            traceability(),
        ],
        prohibited_content: vec!["Implementation details".into(), "API design".into()],
        relationships: vec![
            relationship("feature-design", "feature", "implements"),
            relationship("design", "feature-design", "applies-to"),
        ],
        audit_rules: vec![
            rule("fd-001", "Has UX description", "Feature Design must describe UX", "error", "has_section", "User Experience"),
            rule("fd-002", "Has workflow", "Feature Design must document workflow", "error", "has_section", "Workflow"),
            rule("fd-003", "Has states", "Feature Design must cover all UI states", "warning", "has_section", "States"),
            rule("fd-004", "No implementation", "Feature Design must not include implementation", "warning", "no_implementation", ""),
        ],
    }
}

fn feature_technical_standard() -> StandardDefinition {
    StandardDefinition {
        id: "feature-technical".into(),
        name: "Feature Technical Standard".into(),
        version: "1.0.0".into(),
        domain: "feature-technical".into(),
        description: "Architectural realization of a single feature.".into(),
        required_sections: vec![
            purpose(true),
            sec("Feature Specification", "feature_specification", &["Feature Specification", "Feature Spec", "Specification"], false, "Link to feature spec"),
            sec("Participating Components", "participating_components", &["Participating Components", "Components", "Involved Components"], true, "Which components are involved"),
            sec("Component Responsibilities", "component_responsibilities", &["Component Responsibilities", "Responsibilities", "Component Roles"], false, "What each component is responsible for"),
            sec("Component Interactions", "component_interactions", &["Component Interactions", "Interactions", "Communication Flows"], true, "How components interact"),
            sec("Runtime Behavior", "runtime_behavior", &["Runtime Behavior", "Behavior", "Execution Model"], false, "Runtime and execution behavior"),
            sec("Communication Paths", "communication_paths", &["Communication Paths", "Communication", "Message Flows"], false, "How components communicate"),
            sec("Data Ownership", "data_ownership", &["Data Ownership", "Ownership", "Data Responsibilities"], true, "Who owns what data"),
            sec("Integration Points", "integration_points", &["Integration Points", "Integration", "External Integration"], false, "How this integrates with other components"),
            sec("External Dependency Integration", "external_dependencies", &["External Dependency Integration", "External Dependencies", "External Systems"], false, "External dependency details"),
            sec("Runtime Constraints", "runtime_constraints", &["Runtime Constraints", "Constraints", "Operational Constraints"], false, "Constraints on runtime behavior"),
            sec("Architectural Constraints", "architectural_constraints", &["Architectural Constraints", "Architecture Constraints"], false, "Architectural constraints"),
            security_considerations(),
            performance_considerations(),
            failure_handling(),
            extension_points(),
            traceability(),
        ],
        prohibited_content: vec!["Source code".into(), "Algorithm details".into()],
        relationships: vec![
            relationship("feature-technical", "feature", "realizes"),
            relationship("architecture", "feature-technical", "constrains"),
        ],
        audit_rules: vec![
            rule("ft-001", "Has components", "Feature Technical must list participating components", "error", "has_section", "Participating Components"),
            rule("ft-002", "Has interactions", "Feature Technical must describe component interactions", "error", "has_section", "Component Interactions"),
            rule("ft-003", "Has data ownership", "Feature Technical must define data ownership", "error", "has_section", "Data Ownership"),
            rule("ft-004", "Has security", "Feature Technical must address security", "warning", "has_section", "Security Considerations"),
        ],
    }
}

fn design_standard() -> StandardDefinition {
    StandardDefinition {
        id: "design".into(),
        name: "Design Standard".into(),
        version: "1.0.0".into(),
        domain: "design".into(),
        description: "Reusable design language, principles, and UX standards.".into(),
        required_sections: vec![
            purpose(false),
            sec("Design Principles", "design_principles", &["Design Principles", "Principles", "Core Design"], true, "Core design philosophy"),
            sec("UX Principles", "ux_principles", &["UX Principles", "User Experience Principles", "UX Guidelines"], true, "User experience guidelines"),
            sec("Accessibility", "accessibility", &["Accessibility", "A11y", "Accessibility Standards"], true, "Accessibility standards"),
            constraints(),
            traceability(),
        ],
        prohibited_content: vec!["Feature-specific workflows".into()],
        relationships: vec![
            relationship("design", "feature-design", "applies-to"),
            relationship("philosophy", "design", "guides"),
        ],
        audit_rules: vec![
            rule("dsg-001", "Has design principles", "Design must document principles", "error", "has_section", "Design Principles"),
            rule("dsg-002", "Has UX principles", "Design must document UX guidelines", "warning", "has_section", "UX Principles"),
            rule("dsg-003", "Has accessibility", "Design must address accessibility", "warning", "has_section", "Accessibility"),
        ],
    }
}

fn engineering_standard() -> StandardDefinition {
    StandardDefinition {
        id: "engineering".into(),
        name: "Engineering Standard".into(),
        version: "1.0.0".into(),
        domain: "engineering".into(),
        description: "Repository-wide engineering decisions and standards.".into(),
        required_sections: vec![
            purpose(false),
            sec("Engineering Principles", "guiding_principles", &["Engineering Principles", "Principles", "Core Principles"], true, "Core engineering principles"),
            sec("Technology Selection", "rationale", &["Technology Selection", "Technology Choices", "Technology Rationale", "Why"], true, "Why technologies were chosen"),
            sec("Build Standards", "build_standards", &["Build Standards", "Build", "Build Process", "CI/CD"], true, "How the project is built"),
            sec("Testing Standards", "testing_standards", &["Testing Standards", "Testing", "Test Strategy"], true, "How the project is tested"),
            sec("Code Standards", "code_standards", &["Code Standards", "Coding Standards", "Code Style"], false, "Code quality and style standards"),
            constraints(),
            traceability(),
        ],
        prohibited_content: vec!["Feature-specific decisions".into()],
        relationships: vec![
            relationship("engineering", "architecture", "implements"),
            relationship("engineering", "feature-technical", "guides"),
        ],
        audit_rules: vec![
            rule("eng-001", "Has principles", "Engineering must document principles", "error", "has_section", "Engineering Principles"),
            rule("eng-002", "Has technology rationale", "Engineering must explain technology choices", "error", "has_section", "Technology Selection"),
            rule("eng-003", "Has build standards", "Engineering must define build standards", "warning", "has_section", "Build Standards"),
            rule("eng-004", "Has testing standards", "Engineering must define testing approach", "warning", "has_section", "Testing Standards"),
        ],
    }
}

fn external_context_standard() -> StandardDefinition {
    StandardDefinition {
        id: "external-context".into(),
        name: "External Context Standard".into(),
        version: "1.0.0".into(),
        domain: "external-context".into(),
        description: "Knowledge dependencies on external systems.".into(),
        required_sections: vec![
            purpose(true),
            sec("Integration Contract", "integration_contract", &["Integration Contract", "Contract", "API Contract", "Interface"], true, "How integration works"),
            constraints(),
            dependencies(),
            traceability(),
        ],
        prohibited_content: vec!["Complete external documentation".into()],
        relationships: vec![
            relationship("external-context", "feature-technical", "informs"),
            relationship("external-context", "engineering", "informs"),
        ],
        audit_rules: vec![
            rule("ec-001", "Has purpose", "External Context must explain why dependency exists", "error", "has_section", "Purpose"),
            rule("ec-002", "Has constraints", "External Context must document constraints", "warning", "has_section", "Constraints"),
            rule("ec-003", "References external docs", "External Context should reference authoritative docs", "suggestion", "has_section", "References"),
        ],
    }
}

fn prototype_standard() -> StandardDefinition {
    StandardDefinition {
        id: "prototype".into(),
        name: "Prototype Standard".into(),
        version: "1.0.0".into(),
        domain: "prototype".into(),
        description: "Executable simulation of the application.".into(),
        required_sections: vec![
            purpose(false),
            sec("Scope", "scope", &["Scope", "Coverage", "What Is Covered"], true, "What the prototype covers"),
            sec("Mock APIs", "mock_apis", &["Mock APIs", "Mocked APIs", "API Contracts", "Simulated APIs"], true, "API contracts being simulated"),
            sec("Data Model", "data_model", &["Data Model", "Data Structures", "Schema"], true, "Data structures used"),
            constraints(),
            traceability(),
        ],
        prohibited_content: vec!["Production implementation".into()],
        relationships: vec![
            relationship("prototype", "feature-design", "validates"),
            relationship("prototype", "feature-technical", "validates"),
        ],
        audit_rules: vec![
            rule("proto-001", "Has scope", "Prototype must define scope", "error", "has_section", "Scope"),
            rule("proto-002", "Has mock APIs", "Prototype must document mock APIs", "warning", "has_section", "Mock APIs"),
            rule("proto-003", "Disposable", "Prototype must be disposable (not production code)", "warning", "no_implementation", ""),
        ],
    }
}

fn rule(id: &str, name: &str, description: &str, severity: &str, check_type: &str, scope: &str) -> AuditRuleDef {
    AuditRuleDef {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        severity: severity.into(),
        check_type: check_type.into(),
        scope: scope.into(),
    }
}

fn relationship(from: &str, to: &str, rel: &str) -> StandardRelationship {
    StandardRelationship {
        from_domain: from.into(),
        to_domain: to.into(),
        relationship: rel.into(),
    }
}
