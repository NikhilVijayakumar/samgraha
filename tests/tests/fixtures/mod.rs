use schemas::document::{Document, DocumentMetadata, DocumentPath};
use std::path::PathBuf;

pub fn sample_document(id: i64, standard: &str, title: &str, body: &str) -> Document {
    Document {
        id,
        path: DocumentPath(PathBuf::from(format!("docs/{}/{}.md", standard, title))),
        hash: compute_hash(body),
        standard: standard.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        metadata: DocumentMetadata::default(),
        sections: Vec::new(),
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    }
}

fn compute_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn sample_documents() -> Vec<Document> {
    vec![
        sample_document(
            1,
            "architecture",
            "System Overview",
            "# System Overview\n\n## Purpose\n\nThis document describes the system.\n\n## Components\n\n- Compiler\n- Registry\n- Runtime",
        ),
        sample_document(
            2,
            "architecture",
            "Component Model",
            "# Component Model\n\n## Purpose\n\nDefines components.\n\n## Responsibilities\n\nEach component has one responsibility.",
        ),
        sample_document(
            3,
            "feature",
            "Knowledge Compilation",
            "# Knowledge Compilation\n\n## Purpose\n\nTransform documentation into knowledge.\n\n## Requirements\n\n- Must be deterministic\n- Must work offline",
        ),
        sample_document(
            4,
            "feature",
            "Knowledge Search",
            "# Knowledge Search\n\n## Purpose\n\nSearch compiled knowledge.\n\n## Requirements\n\n- Full-text search\n- Progressive retrieval",
        ),
    ]
}
