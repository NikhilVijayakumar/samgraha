use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectStatistics {
    pub total_section_count: usize,
    pub required_section_count: usize,
    pub missing_section_count: usize,
    pub empty_section_count: usize,
    pub total_relationship_count: usize,
    pub total_knowledge_object_count: usize,
    pub coverage: f64,
    pub per_type: HashMap<String, u32>,
}

impl Default for ObjectStatistics {
    fn default() -> Self {
        Self {
            total_section_count: 0,
            required_section_count: 0,
            missing_section_count: 0,
            empty_section_count: 0,
            total_relationship_count: 0,
            total_knowledge_object_count: 0,
            coverage: 0.0,
            per_type: HashMap::new(),
        }
    }
}

impl ObjectStatistics {
    pub fn coverage_ratio(&self) -> f64 {
        if self.total_section_count == 0 {
            return 1.0;
        }
        (self.total_section_count - self.missing_section_count) as f64 / self.total_section_count as f64
    }
}
