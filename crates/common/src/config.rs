use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamgrahaConfig {
    #[serde(default)]
    pub repository: RepositoryConfig,
    #[serde(default)]
    pub compilation: CompilationConfigSection,
    #[serde(default)]
    pub ai: AiConfigSection,
    #[serde(default)]
    pub audit: AuditConfigSection,
    #[serde(default)]
    pub output: OutputConfigSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryConfig {
    #[serde(default)]
    pub root: Option<PathBuf>,
    #[serde(default)]
    pub documentation: DocumentationConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
    #[serde(default)]
    pub workspace: Option<WorkspaceMembershipConfig>,
    #[serde(default)]
    pub dependencies: Vec<DependencyConfig>,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            root: None,
            documentation: DocumentationConfig::default(),
            ignore: IgnoreConfig::default(),
            workspace: None,
            dependencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentationConfig {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub standards: Vec<String>,
    #[serde(default)]
    pub exclusions: Vec<String>,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            paths: vec!["docs".to_string()],
            standards: Vec::new(),
            exclusions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceMembershipConfig {
    pub workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyConfig {
    pub name: String,
    pub path: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompilationConfigSection {
    #[serde(default)]
    pub watch: bool,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub documentation: DocumentationCompilationConfig,
}

fn default_debounce_ms() -> u64 {
    100
}

fn default_batch_size() -> usize {
    100
}

impl Default for CompilationConfigSection {
    fn default() -> Self {
        Self {
            watch: false,
            debounce_ms: 100,
            batch_size: 100,
            documentation: DocumentationCompilationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentationCompilationConfig {
    #[serde(default)]
    pub standards: Vec<String>,
}

impl Default for DocumentationCompilationConfig {
    fn default() -> Self {
        Self {
            standards: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AiConfigSection {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub lms: Option<AiProviderEndpoint>,
    #[serde(default)]
    pub ollama: Option<AiProviderEndpoint>,
    #[serde(default)]
    pub openai: Option<AiProviderEndpoint>,
}

impl Default for AiConfigSection {
    fn default() -> Self {
        Self {
            provider: None,
            lms: None,
            ollama: None,
            openai: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProviderEndpoint {
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditConfigSection {
    #[serde(default = "default_severity")]
    pub default_severity: String,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub gates: HashMap<String, QualityGateConfig>,
}

fn default_severity() -> String {
    "suggestion".to_string()
}

impl Default for AuditConfigSection {
    fn default() -> Self {
        Self {
            default_severity: "suggestion".to_string(),
            providers: vec!["deterministic".to_string()],
            gates: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualityGateConfig {
    #[serde(default)]
    pub enabled: bool,
    pub min_score: Option<f64>,
    pub min_readiness: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputConfigSection {
    #[serde(default = "default_format")]
    pub format: String,
    #[serde(default)]
    pub color: bool,
}

fn default_format() -> String {
    "text".to_string()
}

impl Default for OutputConfigSection {
    fn default() -> Self {
        Self {
            format: "text".to_string(),
            color: true,
        }
    }
}

impl Default for SamgrahaConfig {
    fn default() -> Self {
        Self {
            repository: RepositoryConfig::default(),
            compilation: CompilationConfigSection::default(),
            ai: AiConfigSection::default(),
            audit: AuditConfigSection::default(),
            output: OutputConfigSection::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceConfig {
    pub name: String,
    #[serde(default)]
    pub repositories: Vec<String>,
    #[serde(default)]
    pub shared: SharedWorkspaceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedWorkspaceConfig {
    #[serde(default)]
    pub standards: Vec<String>,
    #[serde(default)]
    pub audit_policies: HashMap<String, QualityGateConfig>,
    pub registry_path: Option<String>,
}

impl Default for SharedWorkspaceConfig {
    fn default() -> Self {
        Self {
            standards: Vec::new(),
            audit_policies: HashMap::new(),
            registry_path: None,
        }
    }
}
