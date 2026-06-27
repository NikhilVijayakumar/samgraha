use schemas::document::DocumentSection;
use schemas::quality::ObjectStatistics;
use std::collections::HashMap;

pub struct QualityAnalyzer;

impl QualityAnalyzer {
    pub fn analyze(sections: &[DocumentSection], relationship_count: usize) -> ObjectStatistics {
        let mut stats = ObjectStatistics {
            total_relationship_count: relationship_count,
            ..Default::default()
        };

        let mut total = 0usize;
        let mut required = 0usize;
        let mut empty = 0usize;
        let mut per_type: HashMap<String, u32> = HashMap::new();

        Self::count_sections(sections, &mut total, &mut required, &mut empty, &mut per_type);

        stats.total_section_count = total;
        stats.required_section_count = required;
        stats.empty_section_count = empty;
        stats.per_type = per_type;
        stats.coverage = stats.coverage_ratio();

        stats
    }

    pub fn analyze_slice(
        sections: &[&DocumentSection],
        relationship_count: usize,
    ) -> ObjectStatistics {
        let mut stats = ObjectStatistics {
            total_relationship_count: relationship_count,
            ..Default::default()
        };

        let mut total = 0usize;
        let mut required = 0usize;
        let mut empty = 0usize;
        let mut per_type: HashMap<String, u32> = HashMap::new();

        for section in sections {
            Self::count_sections_inner(section, &mut total, &mut required, &mut empty, &mut per_type);
        }

        stats.total_section_count = total;
        stats.required_section_count = required;
        stats.empty_section_count = empty;
        stats.per_type = per_type;
        stats.coverage = stats.coverage_ratio();

        stats
    }

    fn count_sections(
        sections: &[DocumentSection],
        total: &mut usize,
        required: &mut usize,
        empty: &mut usize,
        per_type: &mut HashMap<String, u32>,
    ) {
        for section in sections {
            Self::count_sections_inner(section, total, required, empty, per_type);
        }
    }

    fn count_sections_inner(
        section: &DocumentSection,
        total: &mut usize,
        required: &mut usize,
        empty: &mut usize,
        per_type: &mut HashMap<String, u32>,
    ) {
        *total += 1;
        let _ = per_type
            .entry(section.semantic_type.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);

        if section.required {
            *required += 1;
        }
        if section.body.trim().is_empty() {
            *empty += 1;
        }

        Self::count_sections(&section.subsections, total, required, empty, per_type);
    }
}
