use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamgrahaConfig {
    #[serde(default)]
    pub repository: RepositoryConfig,
    #[serde(default)]
    pub compilation: CompilationConfigSection,
    #[serde(default)]
    pub resolver: ResolverConfig,
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
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub uuid: Option<Uuid>,
}

/// Parse duration strings like "24h", "7d", "3600" into seconds.
pub fn parse_ttl_duration(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num, unit) = s
        .find(|c: char| !c.is_ascii_digit())
        .map_or((s, ""), |i| s.split_at(i));
    let n: i64 = num.parse().ok()?;
    match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => Some(n),
        "m" | "min" | "mins" | "minute" | "minutes" => Some(n * 60),
        "" | "h" | "hr" | "hrs" | "hour" | "hours" => Some(n * 3600),
        "d" | "day" | "days" => Some(n * 86400),
        _ => None,
    }
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            root: None,
            documentation: DocumentationConfig::default(),
            ignore: IgnoreConfig::default(),
            workspace: None,
            dependencies: Vec::new(),
            id: None,
            name: None,
            uuid: None,
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

/// Resolver configuration — controls how the Knowledge Resolver
/// locates dependencies and interacts with the Repository Registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolverConfig {
    #[serde(default = "default_metadata_cache")]
    pub metadata_cache: bool,
    #[serde(default = "default_metadata_ttl")]
    pub metadata_ttl: String,
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,
    #[serde(default)]
    pub registry_type: RegistryType,
    #[serde(default)]
    pub registry_url: Option<String>,
}

fn default_metadata_cache() -> bool {
    true
}

fn default_metadata_ttl() -> String {
    "24h".to_string()
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            metadata_cache: true,
            metadata_ttl: "24h".to_string(),
            auto_refresh: true,
            registry_type: RegistryType::File,
            registry_url: None,
        }
    }
}

impl ResolverConfig {
    pub fn with_remote(url: &str) -> Self {
        Self {
            registry_type: RegistryType::Http,
            registry_url: Some(url.to_string()),
            ..Self::default()
        }
    }
}

/// Registry storage backend type.
///
/// Default is `File` (local SQLite). `Http` is reserved for future
/// remote registry support (Phase 7+).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegistryType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "http")]
    Http,
}

impl Default for RegistryType {
    fn default() -> Self {
        Self::File
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
            resolver: ResolverConfig::default(),
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
