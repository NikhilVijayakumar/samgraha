use std::collections::HashMap;
use schemas::registry::Relationship;
use schemas::standard::StandardDefinition;
use schemas::document::Document;

pub struct RelationshipResolver;

impl RelationshipResolver {
    pub fn resolve_all(
        documents: &[Document],
        standards: &[StandardDefinition],
    ) -> Vec<Relationship> {
        let mut rels = Vec::new();
        for std in standards {
            for rel_def in &std.relationships {
                let from_docs: Vec<&Document> = documents
                    .iter()
                    .filter(|d| d.standard == rel_def.from_domain)
                    .collect();
                let to_docs: Vec<&Document> = documents
                    .iter()
                    .filter(|d| d.standard == rel_def.to_domain)
                    .collect();

                for from in &from_docs {
                    for to in &to_docs {
                        if from.id != to.id {
                            rels.push(Relationship {
                                id: 0,
                                source_id: from.id,
                                target_id: to.id,
                                rel_type: rel_def.relationship.clone(),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }

        rels.dedup_by_key(|r| (r.source_id, r.target_id, r.rel_type.clone()));
        rels.sort_by_key(|r| r.source_id);
        let mut id_counter: i64 = 1;
        for rel in &mut rels {
            rel.id = id_counter;
            id_counter += 1;
        }

        rels
    }
}
