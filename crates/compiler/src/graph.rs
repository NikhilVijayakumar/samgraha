use schemas::document::{Document, DocumentBody};
use schemas::graph::{EdgeType, GraphEdge, GraphNode, KnowledgeGraph};
use schemas::urn::Urn;
use std::collections::HashMap;

pub struct RelationshipBuilder;

impl RelationshipBuilder {
    pub fn build_graph(documents: &[Document]) -> KnowledgeGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Phase 1: create a node for every document
        for doc in documents {
            let domain = doc.standard.to_lowercase().replace(' ', "-");
            let slug = doc.title.to_lowercase().replace(' ', "-").replace('/', "-");
            let doc_urn = Urn::for_document(&domain, &slug);

            nodes.push(GraphNode {
                urn: doc_urn.clone(),
                node_type: "document".to_string(),
                document_id: Some(doc.id),
                title: doc.title.clone(),
            });

            // Phase 2: create nodes and parent edges for every sub-item
            match &doc.body {
                DocumentBody::Feature(ref fb) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &fb.functional_requirements, "functional_requirement");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &fb.business_rules, "business_rule");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &fb.constraints, "constraint");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &fb.dependencies, "dependency");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &fb.acceptance_criteria, "acceptance_criterion");
                }
                DocumentBody::Architecture(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.components, "component");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.communication_paths, "communication_path");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.constraints, "constraint");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                DocumentBody::FeatureTechnical(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.components, "component");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                DocumentBody::Engineering(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.principles, "principle");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.constraints, "constraint");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                DocumentBody::Vision(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.principles, "principle");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                DocumentBody::Design(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.principles, "principle");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.constraints, "constraint");
                }
                DocumentBody::Philosophy(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.principles, "principle");
                }
                DocumentBody::ExternalContext(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.constraints, "constraint");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.dependencies, "dependency");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                DocumentBody::Prototype(ref b) => {
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.constraints, "constraint");
                    Self::add_item_nodes(&mut nodes, &mut edges, &doc_urn, &b.traceability, "traceability_link");
                }
                _ => {}
            }
        }

        // Phase 3: resolve cross-document edges from dependency sections
        for doc in documents {
            let domain = doc.standard.to_lowercase().replace(' ', "-");
            let slug = doc.title.to_lowercase().replace(' ', "-").replace('/', "-");
            let doc_urn = Urn::for_document(&domain, &slug);

            if let DocumentBody::Feature(ref fb) = doc.body {
                for dep in &fb.dependencies {
                    // Try to match dependency title to an existing document node
                    let dep_lower = dep.title.to_lowercase();
                    for other in documents {
                        if other.id == doc.id {
                            continue;
                        }
                        if other.title.to_lowercase() == dep_lower {
                            let other_domain = other.standard.to_lowercase().replace(' ', "-");
                            let other_slug = other.title.to_lowercase().replace(' ', "-").replace('/', "-");
                            let target_urn = Urn::for_document(&other_domain, &other_slug);
                            edges.push(GraphEdge {
                                source_urn: doc_urn.clone(),
                                target_urn,
                                edge_type: EdgeType::DependsOn,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        // Phase 4: resolve traceability edges
        for doc in documents {
            if let DocumentBody::Feature(ref fb) = doc.body {
                let domain = doc.standard.to_lowercase().replace(' ', "-");
                let slug = doc.title.to_lowercase().replace(' ', "-").replace('/', "-");
                let doc_urn = Urn::for_document(&domain, &slug);

                for tl in &fb.traceability {
                    for other in documents {
                        if other.id == doc.id {
                            continue;
                        }
                        if tl.title.to_lowercase().contains(&other.title.to_lowercase()) {
                            let other_domain = other.standard.to_lowercase().replace(' ', "-");
                            let other_slug = other.title.to_lowercase().replace(' ', "-").replace('/', "-");
                            let target_urn = Urn::for_document(&other_domain, &other_slug);
                            edges.push(GraphEdge {
                                source_urn: doc_urn.clone(),
                                target_urn,
                                edge_type: EdgeType::DerivesFrom,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate edges
        edges.dedup_by_key(|e| (e.source_urn.clone(), e.target_urn.clone(), e.edge_type.as_str().to_string()));

        KnowledgeGraph { nodes, edges }
    }

    fn add_item_nodes<T: schemas::objects::KnowledgeObject>(
        nodes: &mut Vec<GraphNode>,
        edges: &mut Vec<GraphEdge>,
        parent_urn: &Urn,
        items: &[T],
        node_type: &str,
    ) {
        for item in items {
            Self::add_node_and_parent_edge(nodes, edges, parent_urn, item.urn(), node_type, item.title());
        }
    }

    fn add_node_and_parent_edge(
        nodes: &mut Vec<GraphNode>,
        edges: &mut Vec<GraphEdge>,
        parent_urn: &Urn,
        item_urn: &Urn,
        node_type: &str,
        title: &str,
    ) {
        nodes.push(GraphNode {
            urn: item_urn.clone(),
            node_type: node_type.to_string(),
            document_id: None,
            title: title.to_string(),
        });
        edges.push(GraphEdge {
            source_urn: item_urn.clone(),
            target_urn: parent_urn.clone(),
            edge_type: EdgeType::ParentOf,
            metadata: HashMap::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemas::document::{DocumentBody, DocumentMetadata, DocumentPath};
    use schemas::objects::FunctionalRequirement;
    use schemas::quality::ObjectStatistics;
    use std::path::PathBuf;

    fn make_feature_doc(id: i64, title: &str, body: DocumentBody) -> Document {
        Document {
            id,
            path: DocumentPath(PathBuf::from(format!("docs/feature/{}.md", title))),
            hash: "abc".into(),
            standard: "feature".into(),
            title: title.into(),
            body,
            metadata: DocumentMetadata::default(),
            provenance: None,
            quality: ObjectStatistics::default(),
            created_at: "0".into(),
            updated_at: "0".into(),
        }
    }

    fn empty_feature_body() -> DocumentBody {
        let raw = String::new();
        DocumentBody::Feature(schemas::document::FeatureBody {
            raw,
            functional_requirements: vec![],
            business_rules: vec![],
            constraints: vec![],
            dependencies: vec![],
            acceptance_criteria: vec![],
            inputs: vec![],
            outputs: vec![],
            non_goals: vec![],
            future_extensions: vec![],
            traceability: vec![],
        })
    }

    #[test]
    fn test_document_node_created() {
        let doc = make_feature_doc(1, "Knowledge Compilation", empty_feature_body());
        let graph = RelationshipBuilder::build_graph(&[doc]);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].urn.as_str(), "feature/knowledge-compilation");
        assert_eq!(graph.nodes[0].node_type, "document");
    }

    #[test]
    fn test_fr_node_and_parent_edge() {
        let urn_doc = Urn::for_document("feature", "test-feature");
        let fr = FunctionalRequirement {
            urn: urn_doc.for_item("FR1"),
            parent: Some(urn_doc.clone()),
            title: "Discovery".into(),
            description: "Desc".into(),
            source_span: None,
        };
        let body = DocumentBody::Feature(schemas::document::FeatureBody {
            raw: String::new(),
            functional_requirements: vec![fr],
            business_rules: vec![],
            constraints: vec![],
            dependencies: vec![],
            acceptance_criteria: vec![],
            inputs: vec![],
            outputs: vec![],
            non_goals: vec![],
            future_extensions: vec![],
            traceability: vec![],
        });
        let doc = make_feature_doc(1, "Test Feature", body);
        let graph = RelationshipBuilder::build_graph(&[doc]);
        // 1 doc node + 1 fr node
        assert_eq!(graph.nodes.len(), 2);
        // 1 parent edge
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].edge_type, EdgeType::ParentOf);
    }

    #[test]
    fn test_dependency_edge_resolved() {
        let doc_a = make_feature_doc(1, "Feature A", empty_feature_body());

        let urn_b = Urn::for_document("feature", "feature-b");
        let dep_body = DocumentBody::Feature(schemas::document::FeatureBody {
            raw: String::new(),
            functional_requirements: vec![],
            business_rules: vec![],
            constraints: vec![],
            dependencies: vec![
                schemas::objects::Dependency {
                    urn: urn_b.for_item("D-1"),
                    parent: Some(urn_b.clone()),
                    title: "Feature A".into(),
                    description: "Feature A".into(),
                    source_span: None,
                },
            ],
            acceptance_criteria: vec![],
            inputs: vec![],
            outputs: vec![],
            non_goals: vec![],
            future_extensions: vec![],
            traceability: vec![],
        });
        let doc_b = make_feature_doc(2, "Feature B", dep_body);

        let graph = RelationshipBuilder::build_graph(&[doc_a, doc_b]);
        let depends_edges: Vec<&GraphEdge> = graph.edges.iter().filter(|e| e.edge_type == EdgeType::DependsOn).collect();
        assert_eq!(depends_edges.len(), 1);
    }

    #[test]
    fn test_empty_docs_yield_empty_graph() {
        let graph = RelationshipBuilder::build_graph(&[]);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_edge_types_as_str() {
        assert_eq!(EdgeType::References.as_str(), "references");
        assert_eq!(EdgeType::DependsOn.as_str(), "depends_on");
        assert_eq!(EdgeType::DerivesFrom.as_str(), "derives_from");
        assert_eq!(EdgeType::Implements.as_str(), "implements");
        assert_eq!(EdgeType::ParentOf.as_str(), "parent_of");
        assert_eq!(EdgeType::from_str("depends_on"), EdgeType::DependsOn);
        assert_eq!(EdgeType::from_str("derives"), EdgeType::DerivesFrom);
        assert_eq!(EdgeType::from_str("derives-from"), EdgeType::DerivesFrom);
        assert_eq!(EdgeType::from_str("applies-to"), EdgeType::AppliesTo);
        assert_eq!(EdgeType::from_str("unknown"), EdgeType::References);
    }
}
