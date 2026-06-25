use std::path::Path;
use anyhow::{Result, Context};
use schemas::document::{Document, DocumentMetadata, DocumentSection, ContentHash};

pub struct DocumentProcessor;

impl DocumentProcessor {
    pub fn process<P: AsRef<Path>>(
        path: P,
        relative_path: P,
        standard: &str,
        id: i64,
    ) -> Result<Document> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read {}", path.display()))?;

        let hash = compute_hash(&content);
        let sections = parse_sections(&content);
        let title = extract_title(&content, path);
        let metadata = extract_metadata(&content, &title, standard);

        Ok(Document {
            id,
            path: schemas::document::DocumentPath(relative_path.as_ref().to_path_buf()),
            hash,
            standard: standard.to_string(),
            title,
            body: content,
            metadata,
            sections,
            created_at: chrono_now(),
            updated_at: chrono_now(),
        })
    }
}

fn compute_hash(content: &str) -> ContentHash {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_title(content: &str, path: &Path) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed.trim_start_matches("# ").to_string();
        }
        if trimmed.starts_with("#") && !trimmed.starts_with("##") {
            return trimmed.trim_start_matches('#').trim().to_string();
        }
    }

    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace('-', " ")
        .replace('_', " ")
}

fn parse_sections(content: &str) -> Vec<DocumentSection> {
    let mut sections = Vec::new();
    let mut current_section: Option<DocumentSection> = None;
    let mut current_body = String::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            if let Some(section) = current_section.take() {
                let mut sec = section;
                sec.body = current_body.trim().to_string();
                sections.push(sec);
                current_body = String::new();
            }
            current_section = Some(DocumentSection {
                heading: line.trim_start_matches("## ").to_string(),
                level: 2,
                body: String::new(),
                subsections: Vec::new(),
            });
        } else if line.starts_with("### ") {
            if current_section.is_none() {
                current_section = Some(DocumentSection {
                    heading: String::new(),
                    level: 2,
                    body: String::new(),
                    subsections: Vec::new(),
                });
            }
            if let Some(ref mut section) = current_section {
                section.subsections.push(DocumentSection {
                    heading: line.trim_start_matches("### ").to_string(),
                    level: 3,
                    body: String::new(),
                    subsections: Vec::new(),
                });
            }
        } else {
            if !current_body.is_empty() || !line.trim().is_empty() {
                if !current_body.is_empty() {
                    current_body.push('\n');
                }
                current_body.push_str(line);
            }
        }
    }

    if let Some(section) = current_section {
        let mut sec = section;
        sec.body = current_body.trim().to_string();
        sections.push(sec);
    }

    sections
}

fn extract_metadata(content: &str, title: &str, standard: &str) -> DocumentMetadata {
    let mut metadata = DocumentMetadata {
        title: title.to_string(),
        ..Default::default()
    };

    metadata.extra.insert("standard".to_string(), standard.to_string());

    for line in content.lines() {
        let lower = line.trim().to_lowercase();
        if lower.contains("purpose") && metadata.document_type.is_none() {
            metadata.document_type = Some("specification".to_string());
        }
        if lower.contains("status:") {
            metadata.status = Some(
                line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default(),
            );
        }
        if lower.contains("owner:") || lower.contains("ownership:") {
            metadata.ownership = Some(
                line.split(':').nth(1).map(|s| s.trim().to_string()).unwrap_or_default(),
            );
        }
    }

    metadata
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_sections() {
        let content = "# Title\n\n## Section One\n\nBody text\n\n## Section Two\n\nMore text";
        let sections = parse_sections(content);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading, "Section One");
        assert_eq!(sections[1].heading, "Section Two");
    }

    #[test]
    fn test_extract_title_from_h1() {
        let content = "# My Document Title\n\nSome content";
        let path = Path::new("test.md");
        assert_eq!(extract_title(content, path), "My Document Title");
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let content = "Hello, world!";
        let h1 = compute_hash(content);
        let h2 = compute_hash(content);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}
