use crate::items;
use crate::quality::QualityAnalyzer;
use anyhow::{Context, Result};
use pulldown_cmark::{Event, Parser, Tag, TagEnd, HeadingLevel, Options};
use schemas::document::{
    CompiledMetadata, ContentHash, Document, DocumentBody, DocumentMetadata, DocumentSection, SourceSpan,
};
use schemas::standard::StandardDefinition;
use schemas::urn::Urn;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const COMPILER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct DocumentProcessor;

impl DocumentProcessor {
    pub fn process<P: AsRef<Path>>(
        path: P,
        relative_path: P,
        standard: &str,
        id: i64,
        standard_def: Option<&StandardDefinition>,
    ) -> Result<Document> {
        let path = path.as_ref();
        let content =
            std::fs::read_to_string(path).context(format!("Failed to read {}", path.display()))?;

        let hash = compute_hash(&content);
        let file_path = relative_path.as_ref().to_string_lossy().to_string();
        let sections = parse_sections(&content, &file_path, standard_def);
        let title = extract_title(&content, path);
        let purpose = extract_purpose(&sections);
        let metadata = extract_metadata(&content, &title, &purpose, standard);

        let domain = standard.to_lowercase().replace(' ', "-");
        let slug = title.to_lowercase().replace(' ', "-").replace('/', "-");
        let doc_urn = Urn::for_document(&domain, &slug);

        // Compute quality from sections before they are consumed into typed body.
        // relationship_count is 0 here; pipeline updates it after graph is built.
        let quality = QualityAnalyzer::analyze(&sections, 0);

        let body = build_body(standard, &content, &sections, &doc_urn);

        Ok(Document {
            id,
            path: schemas::document::DocumentPath(relative_path.as_ref().to_path_buf()),
            hash,
            standard: standard.to_string(),
            title,
            body,
            metadata,
            provenance: Some(provenance_for(standard)),
            quality,
            created_at: chrono_now(),
            updated_at: chrono_now(),
        })
    }
}

fn build_body(
    standard: &str,
    raw: &str,
    sections: &[DocumentSection],
    doc_urn: &Urn,
) -> DocumentBody {
    match standard.to_lowercase().as_str() {
        "feature" => {
            DocumentBody::Feature(schemas::document::FeatureBody {
                raw: raw.to_string(),
                functional_requirements: items::parse_fr_headings(sections, doc_urn),
                business_rules: find_and_parse(sections, "business_rules", doc_urn, items::parse_business_rules),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
                dependencies: find_and_parse(sections, "dependencies", doc_urn, items::parse_dependencies),
                acceptance_criteria: find_and_parse(sections, "acceptance_criteria", doc_urn, items::parse_acceptance_criteria),
                inputs: find_and_parse(sections, "inputs", doc_urn, items::parse_inputs),
                outputs: find_and_parse(sections, "outputs", doc_urn, items::parse_outputs),
                non_goals: find_and_parse(sections, "non_goals", doc_urn, items::parse_non_goals),
                future_extensions: find_and_parse(sections, "future_extensions", doc_urn, items::parse_future_extensions),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "architecture" => {
            DocumentBody::Architecture(schemas::document::ArchitectureBody {
                raw: raw.to_string(),
                components: items::parse_components(sections, doc_urn),
                communication_paths: find_and_parse(sections, "communication_paths", doc_urn, items::parse_communication_paths),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "feature-technical" => {
            DocumentBody::FeatureTechnical(schemas::document::FeatureTechnicalBody {
                raw: raw.to_string(),
                components: items::parse_components(sections, doc_urn),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "engineering" => {
            DocumentBody::Engineering(schemas::document::EngineeringBody {
                raw: raw.to_string(),
                principles: find_and_parse(sections, "guiding_principles", doc_urn, items::parse_principles),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "vision" => {
            let vision_text = sections.iter()
                .find(|s| s.semantic_type == "vision_statement")
                .map(|s| s.body.clone())
                .unwrap_or_default();
            DocumentBody::Vision(schemas::document::VisionBody {
                raw: raw.to_string(),
                vision_statement: vision_text,
                principles: find_and_parse(sections, "guiding_principles", doc_urn, items::parse_principles),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "philosophy" => {
            DocumentBody::Philosophy(schemas::document::PhilosophyBody {
                raw: raw.to_string(),
                principles: find_and_parse(sections, "guiding_principles", doc_urn, items::parse_principles),
            })
        }
        "design" => {
            DocumentBody::Design(schemas::document::DesignBody {
                raw: raw.to_string(),
                principles: find_and_parse(sections, "design_principles", doc_urn, items::parse_principles),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
            })
        }
        "external-context" => {
            DocumentBody::ExternalContext(schemas::document::ExternalContextBody {
                raw: raw.to_string(),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
                dependencies: find_and_parse(sections, "dependencies", doc_urn, items::parse_dependencies),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "prototype" => {
            DocumentBody::Prototype(schemas::document::PrototypeBody {
                raw: raw.to_string(),
                constraints: find_and_parse(sections, "constraints", doc_urn, items::parse_constraints),
                traceability: find_and_parse(sections, "traceability", doc_urn, items::parse_traceability),
            })
        }
        "readme" | "feature-design" | _ => DocumentBody::Generic {
            raw: raw.to_string(),
            sections: sections.to_vec(),
        },
    }
}

fn find_and_parse<T>(
    sections: &[DocumentSection],
    semantic_type: &str,
    doc_urn: &Urn,
    parser: fn(&str, &Urn) -> Vec<T>,
) -> Vec<T> {
    sections
        .iter()
        .find(|s| s.semantic_type == semantic_type)
        .map(|s| parser(&s.body, doc_urn))
        .unwrap_or_default()
}

pub fn compute_content_hash(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_hash(content: &str) -> ContentHash {
    compute_content_hash(content)
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

fn resolve_section_info(heading: &str, standard_def: Option<&StandardDefinition>) -> (String, bool) {
    match standard_def.and_then(|s| s.find_section_type(heading)) {
        Some(def) => (def.semantic_type.clone(), def.required),
        None => ("generic".to_string(), false),
    }
}

fn extract_headings(content: &str) -> Vec<ParsedHeading> {
    let opts = Options::empty();
    let parser = Parser::new_ext(content, opts);

    let mut headings = Vec::new();
    let mut in_heading = false;
    let mut heading_text = String::new();
    let mut heading_level = 0u32;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = match level {
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    _ => 0,
                };
                in_heading = heading_level >= 2;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                if in_heading {
                    headings.push(ParsedHeading {
                        level: heading_level,
                        text: heading_text.clone(),
                    });
                }
                in_heading = false;
            }
            Event::Text(t) | Event::Code(t) => {
                if in_heading {
                    heading_text.push_str(&t);
                }
            }
            _ => {}
        }
    }

    headings
}

struct ParsedHeading {
    level: u32,
    text: String,
}

fn byte_to_line_number(line_starts: &[usize], byte: usize) -> u32 {
    match line_starts.binary_search(&byte) {
        Ok(i) => (i + 1) as u32,
        Err(i) => i as u32,
    }
}

fn parse_sections(
    content: &str,
    file_path: &str,
    standard_def: Option<&StandardDefinition>,
) -> Vec<DocumentSection> {
    let headings = extract_headings(content);

    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    // Find byte positions of each heading by locating their text in order
    let mut heading_positions: Vec<(usize, &ParsedHeading)> = Vec::new();
    let mut search_from = 0;

    for h in &headings {
        let needle = h.text.trim();
        if needle.is_empty() {
            continue;
        }
        if let Some(rel_pos) = content[search_from..].find(needle) {
            let abs_pos = search_from + rel_pos;
            // Walk back to find start of the heading marker line (## or ###)
            let line_start = content[..abs_pos]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            heading_positions.push((line_start, h));
            search_from = abs_pos + needle.len();
        }
    }

    let h2_positions: Vec<(usize, &ParsedHeading)> = heading_positions
        .iter()
        .filter(|(_, h)| h.level == 2)
        .map(|(p, h)| (*p, *h))
        .collect();

    if h2_positions.is_empty() {
        return Vec::new();
    }

    let mut sections = Vec::new();

    for i in 0..h2_positions.len() {
        let (start_byte, h2) = h2_positions[i];
        let end_byte = if i + 1 < h2_positions.len() {
            h2_positions[i + 1].0
        } else {
            content.len()
        };

        let section_content = &content[start_byte..end_byte];
        let body_start = section_content.find('\n').map(|i| i + 1).unwrap_or(0);
        let body = section_content[body_start..].trim().to_string();

        let (semantic_type, required) = resolve_section_info(&h2.text, standard_def);

        // Find H3 subsections within this H2 section
        let h3_in_section: Vec<(usize, &ParsedHeading)> = heading_positions
            .iter()
            .filter(|(p, h)| h.level == 3 && *p >= start_byte && *p < end_byte)
            .map(|(p, h)| (*p, *h))
            .collect();

        let mut subsections = Vec::new();
        for j in 0..h3_in_section.len() {
            let (h3_start, h3) = h3_in_section[j];
            let h3_end = if j + 1 < h3_in_section.len() {
                h3_in_section[j + 1].0
            } else {
                end_byte
            };

            let h3_section = &content[h3_start..h3_end];
            let h3_body_start = h3_section.find('\n').map(|i| i + 1).unwrap_or(0);
            let h3_body = h3_section[h3_body_start..].trim().to_string();

            let (st, req) = resolve_section_info(&h3.text, standard_def);
            let h3_line_end = if h3_end > 0 { h3_end - 1 } else { 0 };
            subsections.push(DocumentSection {
                heading: h3.text.clone(),
                semantic_type: st,
                level: 3,
                body: h3_body,
                required: req,
                source_span: Some(SourceSpan {
                    file: file_path.to_string(),
                    line_start: byte_to_line_number(&line_starts, h3_start),
                    line_end: byte_to_line_number(&line_starts, h3_line_end),
                }),
                subsections: Vec::new(),
            });
        }

        let h2_line_end = if end_byte > 0 { end_byte - 1 } else { 0 };
        sections.push(DocumentSection {
            heading: h2.text.clone(),
            semantic_type,
            level: 2,
            body,
            required,
            source_span: Some(SourceSpan {
                file: file_path.to_string(),
                line_start: byte_to_line_number(&line_starts, start_byte),
                line_end: byte_to_line_number(&line_starts, h2_line_end),
            }),
            subsections,
        });
    }

    sections
}

fn extract_purpose(sections: &[DocumentSection]) -> String {
    for section in sections {
        if section.semantic_type == "purpose" {
            let body = section.body.trim();
            if !body.is_empty() {
                return body.lines().next().unwrap_or("").trim().to_string();
            }
        }
    }
    String::new()
}

fn provenance_for(_standard: &str) -> CompiledMetadata {
    CompiledMetadata {
        compiler_version: COMPILER_VERSION.to_string(),
        compiled_at: chrono_now(),
        standard_version: "1.0.0".to_string(),
        repository: String::new(),
        workspace: None,
    }
}

fn extract_metadata(content: &str, title: &str, purpose: &str, standard: &str) -> DocumentMetadata {
    let mut metadata = DocumentMetadata {
        title: title.to_string(),
        purpose: purpose.to_string(),
        ..Default::default()
    };

    metadata
        .extra
        .insert("standard".to_string(), standard.to_string());

    for line in content.lines() {
        let lower = line.trim().to_lowercase();
        if lower.contains("purpose") && metadata.document_type.is_none() {
            metadata.document_type = Some("specification".to_string());
        }
        if lower.contains("status:") {
            metadata.status = Some(
                line.split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
            );
        }
        if lower.contains("owner:") || lower.contains("ownership:") {
            metadata.ownership = Some(
                line.split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
            );
        }
    }

    metadata
}

fn chrono_now() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sections_basic() {
        let content = "# Title\n\n## Section One\n\nBody text\n\n## Section Two\n\nMore text";
        let sections = parse_sections(content, "test.md", None);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].heading, "Section One");
        assert_eq!(sections[1].heading, "Section Two");
        assert!(sections[0].source_span.is_some());
        let span = sections[0].source_span.as_ref().unwrap();
        assert_eq!(span.file, "test.md");
        assert_eq!(span.line_start, 3);
    }

    #[test]
    fn test_parse_sections_with_code_block() {
        let content = "# Title\n\n## Section One\n\n```\n## This is not a heading\n```\n\n## Section Two\n\nMore text";
        let sections = parse_sections(content, "test.md", None);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[1].heading, "Section Two");
    }

    #[test]
    fn test_parse_sections_with_subheadings() {
        let content = "# Title\n\n## Section\n\nText\n\n### Subsection\n\nSub text\n\n### Subsection Two\n\nMore sub\n\n## Next Section\n\nNext text";
        let sections = parse_sections(content, "test.md", None);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].subsections.len(), 2);
        assert_eq!(sections[0].subsections[0].heading, "Subsection");
        assert_eq!(sections[0].subsections[0].level, 3);
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

    #[test]
    fn test_empty_content() {
        let sections = parse_sections("", "test.md", None);
        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn test_content_without_headings() {
        let content = "Just some text\n\nNo headings here\n";
        let sections = parse_sections(content, "test.md", None);
        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn test_source_span_line_numbers() {
        let content = "\n\n\n## Late Heading\n\nBody";
        let sections = parse_sections(content, "spans.md", None);
        assert_eq!(sections.len(), 1);
        let span = sections[0].source_span.as_ref().unwrap();
        assert_eq!(span.file, "spans.md");
        assert_eq!(span.line_start, 4);
    }
}
