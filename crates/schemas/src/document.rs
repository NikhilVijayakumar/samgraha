use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::quality::ObjectStatistics;
use crate::objects::{AcceptanceCriterion, BusinessRule, CommunicationPath, Component,
    Constraint, Dependency, FunctionalRequirement, FutureExtension,
    Input, NonGoal, Output, Principle, TraceabilityLink};

pub type DocumentId = i64;
pub type ContentHash = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub id: DocumentId,
    pub path: DocumentPath,
    pub hash: ContentHash,
    pub standard: String,
    pub title: String,
    pub body: DocumentBody,
    pub metadata: DocumentMetadata,
    pub provenance: Option<CompiledMetadata>,
    pub quality: ObjectStatistics,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentPath(pub PathBuf);

impl DocumentPath {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap_or("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentMetadata {
    pub title: String,
    pub purpose: String,
    pub document_type: Option<String>,
    pub status: Option<String>,
    pub ownership: Option<String>,
    pub tags: Vec<String>,
    pub extra: std::collections::HashMap<String, String>,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            purpose: String::new(),
            document_type: None,
            status: None,
            ownership: None,
            tags: Vec::new(),
            extra: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: String,
    pub line_start: u32,
    pub line_end: u32,
}

macro_rules! body_with_raw {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct $name {
            pub raw: String,
            pub sections: Vec<DocumentSection>,
            $(pub $field: $ty),*
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentSection {
    pub heading: String,
    pub semantic_type: String,
    pub level: u32,
    pub body: String,
    pub required: bool,
    pub source_span: Option<SourceSpan>,
    pub subsections: Vec<DocumentSection>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentBody {
    Generic {
        raw: String,
        sections: Vec<DocumentSection>,
    },
    Feature(FeatureBody),
    FeatureTechnical(FeatureTechnicalBody),
    Architecture(ArchitectureBody),
    Vision(VisionBody),
    Design(DesignBody),
    Engineering(EngineeringBody),
    ExternalContext(ExternalContextBody),
    Prototype(PrototypeBody),
    Philosophy(PhilosophyBody),
    Readme(ReadmeBody),
}

impl DocumentBody {
    pub fn raw(&self) -> &str {
        match self {
            Self::Generic { raw, .. } => raw,
            Self::Feature(b) => &b.raw,
            Self::FeatureTechnical(b) => &b.raw,
            Self::Architecture(b) => &b.raw,
            Self::Vision(b) => &b.raw,
            Self::Design(b) => &b.raw,
            Self::Engineering(b) => &b.raw,
            Self::ExternalContext(b) => &b.raw,
            Self::Prototype(b) => &b.raw,
            Self::Philosophy(b) => &b.raw,
            Self::Readme(b) => &b.raw,
        }
    }

    pub fn sections(&self) -> Vec<&DocumentSection> {
        match self {
            Self::Generic { sections, .. } => sections.iter().collect(),
            Self::Feature(b) => b.sections.iter().collect(),
            Self::FeatureTechnical(b) => b.sections.iter().collect(),
            Self::Architecture(b) => b.sections.iter().collect(),
            Self::Vision(b) => b.sections.iter().collect(),
            Self::Design(b) => b.sections.iter().collect(),
            Self::Engineering(b) => b.sections.iter().collect(),
            Self::ExternalContext(b) => b.sections.iter().collect(),
            Self::Prototype(b) => b.sections.iter().collect(),
            Self::Philosophy(b) => b.sections.iter().collect(),
            Self::Readme(b) => b.sections.iter().collect(),
        }
    }
}

body_with_raw!(FeatureBody {
    functional_requirements: Vec<FunctionalRequirement>,
    business_rules: Vec<BusinessRule>,
    constraints: Vec<Constraint>,
    dependencies: Vec<Dependency>,
    acceptance_criteria: Vec<AcceptanceCriterion>,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    non_goals: Vec<NonGoal>,
    future_extensions: Vec<FutureExtension>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(FeatureTechnicalBody {
    components: Vec<Component>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(ArchitectureBody {
    components: Vec<Component>,
    communication_paths: Vec<CommunicationPath>,
    constraints: Vec<Constraint>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(VisionBody {
    vision_statement: String,
    principles: Vec<Principle>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(DesignBody {
    principles: Vec<Principle>,
    constraints: Vec<Constraint>,
});

body_with_raw!(EngineeringBody {
    principles: Vec<Principle>,
    constraints: Vec<Constraint>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(ExternalContextBody {
    constraints: Vec<Constraint>,
    dependencies: Vec<Dependency>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(PrototypeBody {
    constraints: Vec<Constraint>,
    traceability: Vec<TraceabilityLink>,
});

body_with_raw!(PhilosophyBody {
    principles: Vec<Principle>,
});

body_with_raw!(ReadmeBody {});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledMetadata {
    pub compiler_version: String,
    pub compiled_at: String,
    pub standard_version: String,
    pub repository: String,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentStatus {
    Draft,
    Review,
    Approved,
    Deprecated,
    Superseded,
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Review => write!(f, "review"),
            Self::Approved => write!(f, "approved"),
            Self::Deprecated => write!(f, "deprecated"),
            Self::Superseded => write!(f, "superseded"),
        }
    }
}
