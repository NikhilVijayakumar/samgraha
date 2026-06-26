use schemas::audit::QualityGate;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePolicy {
    pub repository_isolation: bool,
    pub audit_enforcement: AuditEnforcement,
    pub quality_gates: HashMap<String, QualityGate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AuditEnforcement {
    None,
    Warn,
    Block,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self {
            repository_isolation: true,
            audit_enforcement: AuditEnforcement::Warn,
            quality_gates: HashMap::new(),
        }
    }
}
