use schemas::document::{DocumentSection, SourceSpan};
use schemas::objects::{
    AcceptanceCriterion, BusinessRule, CommunicationPath, Component, Constraint, Dependency,
    ExtensionPoint, FailureMode, FunctionalRequirement, FutureExtension, Input, NonGoal, Output,
    PerformanceRequirement, Principle, SecurityConsideration, TraceabilityLink,
};
use schemas::urn::Urn;

struct BulletItem {
    text: String,
    span: Option<SourceSpan>,
}

fn extract_bullets_from_body(body: &str) -> Vec<BulletItem> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
                let text = trimmed.trim_start_matches("* ")
                    .trim_start_matches("- ")
                    .trim()
                    .to_string();
                if text.is_empty() { None } else { Some(BulletItem { text, span: None }) }
            } else {
                None
            }
        })
        .collect()
}

pub fn parse_fr_headings(sections: &[DocumentSection], doc_urn: &Urn) -> Vec<FunctionalRequirement> {
    let re = regex::Regex::new(r"^FR\s*(\d+)[\.\:\s]\s*(.*)$").unwrap();
    let mut frs = Vec::new();
    for section in sections {
        if let Some(caps) = re.captures(section.heading.trim()) {
            let num = caps.get(1).unwrap().as_str();
            let title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("").to_string();
            let fragment = format!("FR{}", num);
            let urn = doc_urn.for_item(&fragment);
            frs.push(FunctionalRequirement {
                urn,
                parent: Some(doc_urn.clone()),
                title,
                description: section.body.clone(),
                source_span: section.source_span.clone(),
            });
            continue;
        }
        // Also check subsections
        for sub in &section.subsections {
            if let Some(caps) = re.captures(sub.heading.trim()) {
                let num = caps.get(1).unwrap().as_str();
                let title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("").to_string();
                let fragment = format!("FR{}", num);
                let urn = doc_urn.for_item(&fragment);
                frs.push(FunctionalRequirement {
                    urn,
                    parent: Some(doc_urn.clone()),
                    title,
                    description: sub.body.clone(),
                    source_span: sub.source_span.clone(),
                });
            }
        }
    }
    frs
}

pub fn parse_business_rules(body: &str, doc_urn: &Urn) -> Vec<BusinessRule> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        let fragment = format!("BR-{}", i + 1);
        BusinessRule {
            urn: doc_urn.for_item(&fragment),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_constraints(body: &str, doc_urn: &Urn) -> Vec<Constraint> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        Constraint {
            urn: doc_urn.for_item(&format!("C-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_inputs(body: &str, doc_urn: &Urn) -> Vec<Input> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        Input {
            urn: doc_urn.for_item(&format!("I-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_outputs(body: &str, doc_urn: &Urn) -> Vec<Output> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        Output {
            urn: doc_urn.for_item(&format!("O-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_non_goals(body: &str, doc_urn: &Urn) -> Vec<NonGoal> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        NonGoal {
            urn: doc_urn.for_item(&format!("NG-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_future_extensions(body: &str, doc_urn: &Urn) -> Vec<FutureExtension> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        FutureExtension {
            urn: doc_urn.for_item(&format!("FE-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_acceptance_criteria(body: &str, doc_urn: &Urn) -> Vec<AcceptanceCriterion> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        AcceptanceCriterion {
            urn: doc_urn.for_item(&format!("AC-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_dependencies(body: &str, doc_urn: &Urn) -> Vec<Dependency> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        Dependency {
            urn: doc_urn.for_item(&format!("D-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_traceability(body: &str, doc_urn: &Urn) -> Vec<TraceabilityLink> {
    extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
        TraceabilityLink {
            urn: doc_urn.for_item(&format!("TL-{}", i + 1)),
            parent: Some(doc_urn.clone()),
            title: item.text.clone(),
            description: item.text,
            source_span: item.span,
        }
    }).collect()
}

pub fn parse_components(sections: &[DocumentSection], doc_urn: &Urn) -> Vec<Component> {
    let re = regex::Regex::new(r"^##\s+(.+)$").unwrap();
    let mut components = Vec::new();
    for (i, section) in sections.iter().enumerate() {
        if let Some(caps) = re.captures(section.heading.trim()) {
            let title = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("").to_string();
            if title.is_empty() { continue; }
            let fragment = format!("CMP-{}", i + 1);
            components.push(Component {
                urn: doc_urn.for_item(&fragment),
                parent: Some(doc_urn.clone()),
                title,
                description: section.body.clone(),
                source_span: section.source_span.clone(),
            });
        }
    }
    components
}

macro_rules! bullet_parser {
    ($name:ident, $type:ident, $prefix:expr) => {
        pub fn $name(body: &str, doc_urn: &Urn) -> Vec<$type> {
            extract_bullets_from_body(body).into_iter().enumerate().map(|(i, item)| {
                $type {
                    urn: doc_urn.for_item(&format!("{}-{}", $prefix, i + 1)),
                    parent: Some(doc_urn.clone()),
                    title: item.text.clone(),
                    description: item.text,
                    source_span: item.span,
                }
            }).collect()
        }
    };
}

bullet_parser!(parse_principles, Principle, "PR");
bullet_parser!(parse_communication_paths, CommunicationPath, "CP");
bullet_parser!(parse_security_considerations, SecurityConsideration, "SEC");
bullet_parser!(parse_performance_requirements, PerformanceRequirement, "PERF");
bullet_parser!(parse_failure_modes, FailureMode, "FM");
bullet_parser!(parse_extension_points, ExtensionPoint, "EP");

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::objects::KnowledgeObject;

    fn doc_urn() -> Urn {
        Urn::for_document("feature", "knowledge-compilation")
    }

    #[test]
    fn test_parse_fr_from_section_heading() {
        let sections = vec![
            DocumentSection {
                heading: "FR1. Documentation Discovery".into(),
                semantic_type: "generic".into(),
                level: 2,
                body: "The compiler shall discover documentation sources.".into(),
                required: false,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
        ];
        let frs = parse_fr_headings(&sections, &doc_urn());
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].title, "Documentation Discovery");
        assert_eq!(frs[0].urn().as_str(), "feature/knowledge-compilation/FR1");
    }

    #[test]
    fn test_parse_fr_with_colon() {
        let sections = vec![
            DocumentSection {
                heading: "FR1: Documentation Discovery".into(),
                semantic_type: "generic".into(),
                level: 2,
                body: "Body text".into(),
                required: false,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
        ];
        let frs = parse_fr_headings(&sections, &doc_urn());
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].title, "Documentation Discovery");
    }

    #[test]
    fn test_parse_fr_from_subsection() {
        let sections = vec![
            DocumentSection {
                heading: "Functional Requirements".into(),
                semantic_type: "functional_requirements".into(),
                level: 2,
                body: String::new(),
                required: true,
                source_span: None,
                hash: String::new(),
                subsections: vec![
                    DocumentSection {
                        heading: "FR1. Documentation Discovery".into(),
                        semantic_type: "generic".into(),
                        level: 3,
                        body: "The compiler shall discover docs.".into(),
                        required: false,
                        source_span: None,
                        hash: String::new(),
                        subsections: vec![],
                    },
                ],
            },
        ];
        let frs = parse_fr_headings(&sections, &doc_urn());
        assert_eq!(frs.len(), 1);
        assert_eq!(frs[0].urn().as_str(), "feature/knowledge-compilation/FR1");
    }

    #[test]
    fn test_parse_bullet_business_rules() {
        let body = "* Documentation is the authoritative source.\n* Compilation never modifies documentation.\n* Generated artifacts are disposable.";
        let rules = parse_business_rules(body, &doc_urn());
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].urn().as_str(), "feature/knowledge-compilation/BR-1");
        assert_eq!(rules[1].urn().as_str(), "feature/knowledge-compilation/BR-2");
    }

    #[test]
    fn test_parse_empty_section() {
        let frs = parse_fr_headings(&[], &doc_urn());
        assert!(frs.is_empty());
        let rules = parse_business_rules("", &doc_urn());
        assert!(rules.is_empty());
    }

    #[test]
    fn test_parse_acceptance_criteria_bullets() {
        let body = "* documentation is transformed into deterministic knowledge\n* generated knowledge remains reproducible\n* repository relationships are preserved";
        let criteria = parse_acceptance_criteria(body, &doc_urn());
        assert_eq!(criteria.len(), 3);
        assert_eq!(criteria[0].urn().as_str(), "feature/knowledge-compilation/AC-1");
    }

    #[test]
    fn test_parse_constraints_bullets() {
        let body = "* support large repositories\n* support workspaces\n* operate deterministically";
        let constraints = parse_constraints(body, &doc_urn());
        assert_eq!(constraints.len(), 3);
        assert_eq!(constraints[2].urn().as_str(), "feature/knowledge-compilation/C-3");
    }

    #[test]
    fn test_parse_inputs_outputs() {
        let in_body = "* repository documentation\n* repository configuration";
        let inputs = parse_inputs(in_body, &doc_urn());
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].title, "repository documentation");

        let out_body = "* Knowledge Registry\n* compiled knowledge";
        let outputs = parse_outputs(out_body, &doc_urn());
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[1].title, "compiled knowledge");
    }

    #[test]
    fn test_parse_non_goals_and_future() {
        let ng_body = "* execute audits\n* generate enrichment\n* resolve knowledge packages";
        let non_goals = parse_non_goals(ng_body, &doc_urn());
        assert_eq!(non_goals.len(), 3);

        let fe_body = "* additional documentation formats\n* custom document processors";
        let future = parse_future_extensions(fe_body, &doc_urn());
        assert_eq!(future.len(), 2);
    }

    #[test]
    fn test_parse_dependencies_and_traceability() {
        let dep_body = "* Repository Configuration\n* Workspace Management";
        let deps = parse_dependencies(dep_body, &doc_urn());
        assert_eq!(deps.len(), 2);

        let trace_body = "* **Documentation is the source of truth.**\n* **Knowledge is compiled before delivery.**";
        let traces = parse_traceability(trace_body, &doc_urn());
        assert_eq!(traces.len(), 2);
    }

    #[test]
    fn test_urn_format() {
        let urn = Urn::for_document("feature", "knowledge-compilation");
        assert_eq!(urn.as_str(), "feature/knowledge-compilation");
        let item = urn.for_item("FR1");
        assert_eq!(item.as_str(), "feature/knowledge-compilation/FR1");
    }

    #[test]
    fn test_urn_slugify() {
        let urn = Urn::for_document("feature", "My Feature Document");
        assert_eq!(urn.as_str(), "feature/my-feature-document");
    }

    #[test]
    fn test_parse_principles() {
        let body = "* Principle of Least Astonishment\n* Fail Fast\n* Convention over Configuration";
        let items = parse_principles(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/PR-1");
        assert_eq!(items[1].title, "Fail Fast");
    }

    #[test]
    fn test_parse_communication_paths() {
        let body = "* Compiler → Registry (gRPC)\n* Registry → MCP (SQLite reads)\n* CLI → Compiler (stdin pipes)";
        let items = parse_communication_paths(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/CP-1");
        assert_eq!(items[2].title, "CLI → Compiler (stdin pipes)");
    }

    #[test]
    fn test_parse_security_considerations() {
        let body = "* Input validation on all URN queries\n* RBAC for registry mutations\n* Audit log for all writes";
        let items = parse_security_considerations(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/SEC-1");
        assert_eq!(items[1].title, "RBAC for registry mutations");
    }

    #[test]
    fn test_parse_performance_requirements() {
        let body = "* Query latency under 100ms\n* Concurrent read capacity > 1000 ops/s\n* Registry startup under 1s";
        let items = parse_performance_requirements(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/PERF-1");
    }

    #[test]
    fn test_parse_failure_modes() {
        let body = "* Registry corruption on disk full\n* Stale graph edges after incremental compile\n* Duplicate URN on import";
        let items = parse_failure_modes(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/FM-1");
        assert_eq!(items[2].title, "Duplicate URN on import");
    }

    #[test]
    fn test_parse_extension_points() {
        let body = "* Custom document parsers (plugin trait)\n* Custom relationship inferrers\n* Custom quality metrics";
        let items = parse_extension_points(body, &doc_urn());
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].urn().as_str(), "feature/knowledge-compilation/EP-1");
    }

    #[test]
    fn test_parse_components_from_headings() {
        let sections = vec![
            DocumentSection {
                heading: "## Compiler".into(),
                semantic_type: "generic".into(),
                level: 2,
                body: "Parses documentation into typed objects.".into(),
                required: false,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
            DocumentSection {
                heading: "## Registry".into(),
                semantic_type: "generic".into(),
                level: 2,
                body: "Persists compiled knowledge.".into(),
                required: false,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
        ];
        let components = parse_components(&sections, &doc_urn());
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].urn().as_str(), "feature/knowledge-compilation/CMP-1");
        assert_eq!(components[0].title, "Compiler");
        assert_eq!(components[1].title, "Registry");
    }

    #[test]
    fn test_parse_components_skips_unknown_headings() {
        let sections = vec![
            DocumentSection {
                heading: "## Purpose".into(),
                semantic_type: "purpose".into(),
                level: 2,
                body: "Describe the system.".into(),
                required: true,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
            DocumentSection {
                heading: "## Components".into(),
                semantic_type: "components".into(),
                level: 2,
                body: "Listing.".into(),
                required: false,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
        ];
        // Both headings match ## pattern, so both get parsed
        let components = parse_components(&sections, &doc_urn());
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn test_new_parsers_empty_body() {
        assert!(parse_principles("", &doc_urn()).is_empty());
        assert!(parse_communication_paths("", &doc_urn()).is_empty());
        assert!(parse_security_considerations("", &doc_urn()).is_empty());
        assert!(parse_performance_requirements("", &doc_urn()).is_empty());
        assert!(parse_failure_modes("", &doc_urn()).is_empty());
        assert!(parse_extension_points("", &doc_urn()).is_empty());
        assert!(parse_components(&[], &doc_urn()).is_empty());
    }

    #[test]
    fn test_no_fr_match_for_regular_heading() {
        let sections = vec![
            DocumentSection {
                heading: "Purpose".into(),
                semantic_type: "purpose".into(),
                level: 2,
                body: "Purpose text".into(),
                required: true,
                source_span: None,
                hash: String::new(),
                subsections: vec![],
            },
        ];
        let frs = parse_fr_headings(&sections, &doc_urn());
        assert!(frs.is_empty());
    }
}
