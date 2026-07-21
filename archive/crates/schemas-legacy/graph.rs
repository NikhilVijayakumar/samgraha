use serde::{Deserialize, Serialize};
use crate::urn::Urn;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    DerivesFrom,
    References,
    DependsOn,
    ProvidesFor,
    Designs,
    Implements,
    Constrains,
    Guides,
    Validates,
    Informs,
    Inspires,
    AppliesTo,
    Realizes,
    ParentOf,
    ChildOf,
    CompiledFrom,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DerivesFrom => "derives_from",
            Self::References => "references",
            Self::DependsOn => "depends_on",
            Self::ProvidesFor => "provides_for",
            Self::Designs => "designs",
            Self::Implements => "implements",
            Self::Constrains => "constrains",
            Self::Guides => "guides",
            Self::Validates => "validates",
            Self::Informs => "informs",
            Self::Inspires => "inspires",
            Self::AppliesTo => "applies_to",
            Self::Realizes => "realizes",
            Self::ParentOf => "parent_of",
            Self::ChildOf => "child_of",
            Self::CompiledFrom => "compiled_from",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "references" => Self::References,
            "depends_on" | "depends-on" | "depends" => Self::DependsOn,
            "derives_from" | "derives-from" | "derives" => Self::DerivesFrom,
            "provides_for" | "provides-for" | "provides" => Self::ProvidesFor,
            "designs" => Self::Designs,
            "implements" => Self::Implements,
            "constrains" => Self::Constrains,
            "guides" => Self::Guides,
            "validates" => Self::Validates,
            "informs" => Self::Informs,
            "inspires" => Self::Inspires,
            "applies_to" | "applies-to" | "applies" => Self::AppliesTo,
            "realizes" => Self::Realizes,
            "parent_of" | "parent-of" => Self::ParentOf,
            "child_of" | "child-of" => Self::ChildOf,
            "compiled_from" | "compiled-from" => Self::CompiledFrom,
            _ => Self::References,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphNode {
    pub urn: Urn,
    pub node_type: String,
    pub document_id: Option<i64>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub source_urn: Urn,
    pub target_urn: Urn,
    pub edge_type: EdgeType,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
