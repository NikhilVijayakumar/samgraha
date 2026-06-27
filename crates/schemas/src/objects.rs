use serde::{Deserialize, Serialize};
use crate::document::SourceSpan;
use crate::urn::Urn;

pub trait KnowledgeObject {
    fn urn(&self) -> &Urn;
    fn parent(&self) -> Option<&Urn>;
    fn object_type(&self) -> &'static str;
    fn title(&self) -> &str;
    fn source_span(&self) -> Option<&SourceSpan>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeObjectEnum {
    FunctionalRequirement(FunctionalRequirement),
    BusinessRule(BusinessRule),
    Constraint(Constraint),
    Dependency(Dependency),
    AcceptanceCriterion(AcceptanceCriterion),
    Input(Input),
    Output(Output),
    NonGoal(NonGoal),
    FutureExtension(FutureExtension),
    TraceabilityLink(TraceabilityLink),
    Component(Component),
    Principle(Principle),
    SecurityConsideration(SecurityConsideration),
    PerformanceRequirement(PerformanceRequirement),
    FailureMode(FailureMode),
    ExtensionPoint(ExtensionPoint),
    CommunicationPath(CommunicationPath),
}

macro_rules! impl_knowledge_object {
    ($name:ident, $type:expr) => {
        impl KnowledgeObject for $name {
            fn urn(&self) -> &Urn { &self.urn }
            fn parent(&self) -> Option<&Urn> { self.parent.as_ref() }
            fn object_type(&self) -> &'static str { $type }
            fn title(&self) -> &str { &self.title }
            fn source_span(&self) -> Option<&SourceSpan> { self.source_span.as_ref() }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionalRequirement {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(FunctionalRequirement, "functional_requirement");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BusinessRule {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(BusinessRule, "business_rule");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Constraint {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Constraint, "constraint");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Dependency, "dependency");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcceptanceCriterion {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(AcceptanceCriterion, "acceptance_criterion");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Input {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Input, "input");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Output {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Output, "output");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NonGoal {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(NonGoal, "non_goal");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FutureExtension {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(FutureExtension, "future_extension");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceabilityLink {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(TraceabilityLink, "traceability_link");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Component {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Component, "component");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Principle {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(Principle, "principle");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityConsideration {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(SecurityConsideration, "security_consideration");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceRequirement {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(PerformanceRequirement, "performance_requirement");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureMode {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(FailureMode, "failure_mode");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionPoint {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(ExtensionPoint, "extension_point");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommunicationPath {
    pub urn: Urn,
    pub parent: Option<Urn>,
    pub title: String,
    pub description: String,
    pub source_span: Option<SourceSpan>,
}
impl_knowledge_object!(CommunicationPath, "communication_path");
