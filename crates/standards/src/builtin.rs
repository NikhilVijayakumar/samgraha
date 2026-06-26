use schemas::standard::StandardDefinition;
use schemas::standard::{AuditRuleDef, RequiredSection, StandardRelationship};

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

fn readme_standard() -> StandardDefinition {
    StandardDefinition {
        id: "readme".into(),
        name: "README Standard".into(),
        version: "1.0.0".into(),
        domain: "readme".into(),
        description: "Primary repository entry point and documentation navigation.".into(),
        required_sections: vec![
            section(
                "Title",
                "Repository name and brief description",
                true,
                false,
            ),
            section(
                "Getting Started",
                "Installation and basic usage",
                true,
                false,
            ),
            section("Documentation", "Links to key documentation", true, false),
        ],
        prohibited_content: vec!["Detailed API reference".into()],
        relationships: vec![relationship("readme", "vision", "references")],
        audit_rules: vec![
            rule(
                "readme-001",
                "README exists",
                "Repository must have a README.md",
                "error",
            ),
            rule(
                "readme-002",
                "Has title",
                "README must have a top-level title",
                "error",
            ),
            rule(
                "readme-003",
                "Has getting started",
                "README must have getting started section",
                "warning",
            ),
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
            section("Purpose", "Why the product exists", true, false),
            section("Target Audience", "Who the product serves", true, false),
            section("Core Values", "Guiding principles", true, false),
        ],
        prohibited_content: vec![
            "Implementation details".into(),
            "Architecture decisions".into(),
        ],
        relationships: vec![
            relationship("vision", "feature", "derives"),
            relationship("vision", "philosophy", "inspires"),
        ],
        audit_rules: vec![
            rule(
                "vision-001",
                "Has purpose",
                "Vision must include product purpose",
                "error",
            ),
            rule(
                "vision-002",
                "Has audience",
                "Vision must define target audience",
                "warning",
            ),
            rule(
                "vision-003",
                "No implementation",
                "Vision must not contain implementation details",
                "warning",
            ),
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
            section(
                "Principles",
                "Core design and engineering principles",
                true,
                false,
            ),
            section("Values", "What the product values", true, false),
            section("Trade-offs", "Deliberate trade-offs made", true, false),
        ],
        prohibited_content: vec!["Specific technology choices".into()],
        relationships: vec![
            relationship("philosophy", "architecture", "guides"),
            relationship("philosophy", "design", "guides"),
        ],
        audit_rules: vec![
            rule(
                "phil-001",
                "Has principles",
                "Philosophy must document principles",
                "error",
            ),
            rule(
                "phil-002",
                "Has values",
                "Philosophy must document values",
                "warning",
            ),
            rule(
                "phil-003",
                "Has trade-offs",
                "Philosophy should document trade-offs",
                "suggestion",
            ),
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
            section(
                "System Overview",
                "High-level system description",
                true,
                false,
            ),
            section(
                "Component Model",
                "Components and responsibilities",
                true,
                false,
            ),
            section("Communication", "How components communicate", true, false),
            section(
                "Data Flow",
                "How data moves through the system",
                true,
                false,
            ),
            section("Security", "Security architecture", true, false),
        ],
        prohibited_content: vec![
            "Implementation details".into(),
            "Technology frameworks".into(),
        ],
        relationships: vec![
            relationship("architecture", "feature-technical", "constrains"),
            relationship("architecture", "engineering", "guides"),
        ],
        audit_rules: vec![
            rule(
                "arch-001",
                "Has overview",
                "Architecture must include system overview",
                "error",
            ),
            rule(
                "arch-002",
                "Has component model",
                "Architecture must define component responsibilities",
                "error",
            ),
            rule(
                "arch-003",
                "No implementation details",
                "Architecture must avoid implementation specifics",
                "warning",
            ),
            rule(
                "arch-004",
                "Has security",
                "Architecture must address security",
                "warning",
            ),
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
            section("Purpose", "Feature objective and user value", true, false),
            section("Requirements", "Functional requirements", true, false),
            section(
                "Acceptance Criteria",
                "How to verify completion",
                true,
                false,
            ),
        ],
        prohibited_content: vec!["Implementation details".into(), "Architecture".into()],
        relationships: vec![
            relationship("feature", "feature-design", "designs"),
            relationship("feature", "feature-technical", "implements"),
            relationship("vision", "feature", "derives-from"),
        ],
        audit_rules: vec![
            rule(
                "feat-001",
                "Has purpose",
                "Feature must document its purpose",
                "error",
            ),
            rule(
                "feat-002",
                "Has requirements",
                "Feature must list functional requirements",
                "error",
            ),
            rule(
                "feat-003",
                "Has acceptance criteria",
                "Feature must define acceptance criteria",
                "error",
            ),
            rule(
                "feat-004",
                "Technology independent",
                "Feature must not specify technology",
                "warning",
            ),
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
            section(
                "User Experience",
                "How users interact with the feature",
                true,
                false,
            ),
            section("Workflow", "Step-by-step user workflow", true, false),
            section(
                "States",
                "Empty, loading, success, error states",
                true,
                false,
            ),
        ],
        prohibited_content: vec!["Implementation details".into(), "API design".into()],
        relationships: vec![
            relationship("feature-design", "feature", "implements"),
            relationship("design", "feature-design", "applies-to"),
        ],
        audit_rules: vec![
            rule(
                "fd-001",
                "Has UX description",
                "Feature Design must describe UX",
                "error",
            ),
            rule(
                "fd-002",
                "Has workflow",
                "Feature Design must document workflow",
                "error",
            ),
            rule(
                "fd-003",
                "Has states",
                "Feature Design must cover all UI states",
                "warning",
            ),
            rule(
                "fd-004",
                "No implementation",
                "Feature Design must not include implementation",
                "warning",
            ),
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
            section(
                "Participating Components",
                "Which components are involved",
                true,
                false,
            ),
            section(
                "Component Interactions",
                "How components interact",
                true,
                false,
            ),
            section("Data Ownership", "Who owns what data", true, false),
            section(
                "Security Considerations",
                "Security implications",
                true,
                false,
            ),
        ],
        prohibited_content: vec!["Source code".into(), "Algorithm details".into()],
        relationships: vec![
            relationship("feature-technical", "feature", "realizes"),
            relationship("architecture", "feature-technical", "constrains"),
        ],
        audit_rules: vec![
            rule(
                "ft-001",
                "Has components",
                "Feature Technical must list participating components",
                "error",
            ),
            rule(
                "ft-002",
                "Has interactions",
                "Feature Technical must describe component interactions",
                "error",
            ),
            rule(
                "ft-003",
                "Has data ownership",
                "Feature Technical must define data ownership",
                "error",
            ),
            rule(
                "ft-004",
                "Has security",
                "Feature Technical must address security",
                "warning",
            ),
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
            section("Design Principles", "Core design philosophy", true, false),
            section("UX Principles", "User experience guidelines", true, false),
            section("Accessibility", "Accessibility standards", true, false),
        ],
        prohibited_content: vec!["Feature-specific workflows".into()],
        relationships: vec![
            relationship("design", "feature-design", "applies-to"),
            relationship("philosophy", "design", "guides"),
        ],
        audit_rules: vec![
            rule(
                "dsg-001",
                "Has design principles",
                "Design must document principles",
                "error",
            ),
            rule(
                "dsg-002",
                "Has UX principles",
                "Design must document UX guidelines",
                "warning",
            ),
            rule(
                "dsg-003",
                "Has accessibility",
                "Design must address accessibility",
                "warning",
            ),
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
            section(
                "Engineering Principles",
                "Core engineering principles",
                true,
                false,
            ),
            section(
                "Technology Selection",
                "Why technologies were chosen",
                true,
                false,
            ),
            section("Build Standards", "How the project is built", true, false),
            section(
                "Testing Standards",
                "How the project is tested",
                true,
                false,
            ),
        ],
        prohibited_content: vec!["Feature-specific decisions".into()],
        relationships: vec![
            relationship("engineering", "architecture", "implements"),
            relationship("engineering", "feature-technical", "guides"),
        ],
        audit_rules: vec![
            rule(
                "eng-001",
                "Has principles",
                "Engineering must document principles",
                "error",
            ),
            rule(
                "eng-002",
                "Has technology rationale",
                "Engineering must explain technology choices",
                "error",
            ),
            rule(
                "eng-003",
                "Has build standards",
                "Engineering must define build standards",
                "warning",
            ),
            rule(
                "eng-004",
                "Has testing standards",
                "Engineering must define testing approach",
                "warning",
            ),
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
            section("Purpose", "Why this dependency exists", true, false),
            section("Integration Contract", "How integration works", true, false),
            section(
                "Constraints",
                "Constraints imposed by dependency",
                true,
                false,
            ),
        ],
        prohibited_content: vec!["Complete external documentation".into()],
        relationships: vec![
            relationship("external-context", "feature-technical", "informs"),
            relationship("external-context", "engineering", "informs"),
        ],
        audit_rules: vec![
            rule(
                "ec-001",
                "Has purpose",
                "External Context must explain why dependency exists",
                "error",
            ),
            rule(
                "ec-002",
                "Has constraints",
                "External Context must document constraints",
                "warning",
            ),
            rule(
                "ec-003",
                "References external docs",
                "External Context should reference authoritative docs",
                "suggestion",
            ),
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
            section("Scope", "What the prototype covers", true, false),
            section("Mock APIs", "API contracts being simulated", true, false),
            section("Data Model", "Data structures used", true, false),
        ],
        prohibited_content: vec!["Production implementation".into()],
        relationships: vec![
            relationship("prototype", "feature-design", "validates"),
            relationship("prototype", "feature-technical", "validates"),
        ],
        audit_rules: vec![
            rule(
                "proto-001",
                "Has scope",
                "Prototype must define scope",
                "error",
            ),
            rule(
                "proto-002",
                "Has mock APIs",
                "Prototype must document mock APIs",
                "warning",
            ),
            rule(
                "proto-003",
                "Disposable",
                "Prototype must be disposable (not production code)",
                "warning",
            ),
        ],
    }
}

fn section(name: &str, desc: &str, required: bool, allows_children: bool) -> RequiredSection {
    RequiredSection {
        name: name.into(),
        description: desc.into(),
        required,
        allows_children,
    }
}

fn rule(id: &str, name: &str, description: &str, severity: &str) -> AuditRuleDef {
    AuditRuleDef {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        severity: severity.into(),
        scope: "document".into(),
    }
}

fn relationship(from: &str, to: &str, rel: &str) -> StandardRelationship {
    StandardRelationship {
        from_domain: from.into(),
        to_domain: to.into(),
        relationship: rel.into(),
    }
}
