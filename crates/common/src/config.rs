use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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
    pub knowledge: KnowledgeConfig,
    #[serde(default)]
    pub ai: AiConfigSection,
    #[serde(default)]
    pub audit: AuditConfigSection,
    #[serde(default)]
    pub output: OutputConfigSection,
    #[serde(default)]
    pub report: ReportConfig,
}

/// Where generated reports (e.g. audit `--report` output) are written.
///
/// `dir` may be a literal path or a `${VAR}` placeholder resolved from the
/// process environment at load time via [`resolve_configured_dir`] — see
/// docs/raw/feature-technical/cli-interface.md "Environment-Resolved Paths".
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportConfig {
    #[serde(default = "default_report_dir")]
    pub dir: String,
}

fn default_report_dir() -> String {
    "${SAMGRAHA_REPORT_DIR}".to_string()
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            dir: default_report_dir(),
        }
    }
}

/// Resolve a configured path that may be a `${VAR}` placeholder.
///
/// - `"${VAR}"` (the whole string, nothing else): read `VAR` from the process
///   environment. If set, use its value (joined to `root` if relative). If
///   unset, fall back to `root.join(fallback_rel)` — so the tool works with
///   zero env configuration on a single-machine setup.
/// - Anything else is a literal path, used as-is (joined to `root` if relative).
pub fn resolve_configured_dir(raw: &str, root: &Path, fallback_rel: &str) -> PathBuf {
    let trimmed = raw.trim();
    let literal = if let Some(var_name) = trimmed.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        std::env::var(var_name).ok()
    } else {
        Some(trimmed.to_string())
    };

    match literal {
        Some(value) if !value.is_empty() => {
            let p = PathBuf::from(&value);
            if p.is_absolute() {
                p
            } else {
                root.join(p)
            }
        }
        _ => root.join(fallback_rel),
    }
}

/// Which repositories to load into the Knowledge Package for this repo.
/// Planner reads this alongside .meta files to produce a deterministic Knowledge Plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KnowledgeConfig {
    /// Always loaded, high priority (required dependencies).
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Always loaded, lower priority (adjacent repos to consult).
    #[serde(default)]
    pub interests: Vec<String>,
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
    #[serde(default)]
    pub implementation: ImplementationConfig,
    /// External scripts directory, if this repo keeps one separate from
    /// `implementation.dir` (e.g. a top-level `scripts/`). Optional — most
    /// repos don't declare this; absent means "not applicable".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<ScriptsConfig>,
    /// Test directory, if this repo keeps tests outside `implementation.dir`
    /// (e.g. a top-level `tests/` alongside a `crates/`/`src/` implementation
    /// dir). Optional — absent means tests live inside `implementation.dir`
    /// or aren't tracked separately.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tests: Option<TestsConfig>,
}

/// Where this repository's source/implementation lives, relative to the
/// repository root (or an absolute path resolved from env — see
/// [`resolve_configured_dir`]). Reserved for future traceability/audit checks
/// that cross-reference documentation against actual source; not yet read by
/// any consumer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImplementationConfig {
    #[serde(default = "default_implementation_dir")]
    pub dir: String,
}

fn default_implementation_dir() -> String {
    "${SAMGRAHA_IMPLEMENTATION_DIR}".to_string()
}

impl Default for ImplementationConfig {
    fn default() -> Self {
        Self {
            dir: default_implementation_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptsConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestsConfig {
    pub dir: String,
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
            implementation: ImplementationConfig::default(),
            scripts: None,
            tests: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentationConfig {
    /// Documentation root directory, relative to the repository root (or an
    /// absolute path resolved from env — see [`resolve_configured_dir`]).
    #[serde(default = "default_docs_root_dir")]
    pub root_dir: String,
    /// Domain names this repo declares (e.g. "architecture", "feature").
    /// Empty means "all builtin standards" (back-compat default).
    #[serde(default)]
    pub domain: Vec<String>,
    /// Domain names to ignore even though they're listed in `domain` — for
    /// repos that don't use every builtin standard (e.g. no `prototype` docs).
    /// Effective domains = `domain` minus `domain_exclusion`.
    #[serde(default)]
    pub domain_exclusion: Vec<String>,
}

fn default_docs_root_dir() -> String {
    "${SAMGRAHA_DOCS_DIR}".to_string()
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            root_dir: default_docs_root_dir(),
            domain: Vec::new(),
            domain_exclusion: Vec::new(),
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
                "**/audit-standards/**".to_string(),
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
    /// How long .meta files are valid before sync is needed (~1 day).
    #[serde(default = "default_metadata_ttl")]
    pub metadata_ttl: String,
    /// How long a Knowledge Session (assembled Knowledge Package) is valid (~30 days).
    #[serde(default = "default_knowledge_ttl")]
    pub knowledge_ttl: String,
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

fn default_knowledge_ttl() -> String {
    "720h".to_string()
}

fn default_auto_refresh() -> bool {
    true
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            metadata_cache: true,
            metadata_ttl: "24h".to_string(),
            knowledge_ttl: "720h".to_string(),
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
            knowledge: KnowledgeConfig::default(),
            ai: AiConfigSection::default(),
            audit: AuditConfigSection::default(),
            output: OutputConfigSection::default(),
            report: ReportConfig::default(),
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
