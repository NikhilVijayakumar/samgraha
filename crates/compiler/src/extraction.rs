use std::collections::HashMap;
use schemas::document::{Document, DocumentMetadata};
use schemas::registry::Relationship;

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract_standard_metadata(doc: &Document) -> DocumentMetadata {
        let mut metadata = doc.metadata.clone();

        for line in doc.body.lines() {
            let lower = line.trim().to_lowercase();
            if lower.starts_with("## purpose") || lower.starts_with("## overview") {
                metadata.extra.insert("has_purpose".to_string(), "true".to_string());
            }
            if lower.starts_with("## ") {
                let section = line.trim_start_matches("## ").to_string();
                metadata.extra.insert(
                    format!("section_{}", section.to_lowercase().replace(' ', "_")),
                    "present".to_string(),
                );
            }
            if let Some(pos) = line.find(":**") {
                let key = line[..pos].trim().to_lowercase().replace(' ', "_");
                let value = line[pos + 3..].trim().to_string();
                metadata.extra.insert(key, value);
            }
        }

        metadata
    }
}

pub struct RelationshipExtractor;

impl RelationshipExtractor {
    pub fn extract_refs(doc: &Document, all_docs: &[Document]) -> Vec<Relationship> {
        let mut rels = Vec::new();
        for other in all_docs {
            if other.id == doc.id {
                continue;
            }
            if doc.body.contains(&other.title) || doc.body.contains(other.path.as_str()) {
                rels.push(Relationship {
                    id: 0,
                    source_id: doc.id,
                    target_id: other.id,
                    rel_type: "references".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }
        rels
    }
}
