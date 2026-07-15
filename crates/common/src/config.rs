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
    /// Repository-declared build/test/package/deploy contracts — see
    /// `PipelineContractConfig`. Absent means the repository hasn't declared
    /// any (Build/Security/Coverage pipeline checks fall back to reporting
    /// "no contract declared" rather than guessing a build system).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipelines: Option<PipelineContractConfig>,
}

/// Repository-declared operational contracts (`samgraha.toml [pipelines.*]`).
/// Deliberately not named `PipelineConfig` / anything with a bare `Pipeline`
/// prefix — `crates::audit::pipeline` already owns that name for the 5 audit
/// types (Build, Security, Consistency, Coverage, Dependency). This is a
/// different concept: user-declared, executable commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineContractConfig {
    #[serde(default = "default_pipelines_version")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<ContractSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<ContractSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<ContractSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy: Option<ContractSpec>,
}

fn default_pipelines_version() -> String {
    "1.0".to_string()
}

/// This build's supported `[pipelines] version` — see [`PipelineContractConfig::check_version`].
pub const SUPPORTED_PIPELINES_MAJOR: u32 = 1;
pub const SUPPORTED_PIPELINES_MINOR: u32 = 0;

impl PipelineContractConfig {
    /// Checks `version` (`"major.minor"`) against what this build supports.
    /// `Err` on a different major version (hard error — the contract shape
    /// may have changed incompatibly). `Ok(true)` on a higher minor than
    /// supported (forward-compatible field additions — proceed, caller should
    /// warn). `Ok(false)` otherwise.
    pub fn check_version(&self) -> Result<bool, String> {
        let (major, minor) = self
            .version
            .split_once('.')
            .and_then(|(maj, min)| Some((maj.parse::<u32>().ok()?, min.parse::<u32>().ok()?)))
            .ok_or_else(|| {
                format!(
                    "Invalid [pipelines] version '{}': expected 'major.minor'",
                    self.version
                )
            })?;
        if major != SUPPORTED_PIPELINES_MAJOR {
            return Err(format!(
                "samgraha.toml declares [pipelines] version {}, this build supports {}.x — \
                 upgrade Saṃgraha or downgrade the declared version",
                self.version, SUPPORTED_PIPELINES_MAJOR
            ));
        }
        Ok(minor > SUPPORTED_PIPELINES_MINOR)
    }
}

/// One build/test/package/deploy contract. Same shape for all 4 types —
/// `artifacts`/`working_directory` are always present so evidence-freshness
/// checking runs one algorithm over every contract type, not one per type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContractSpec {
    pub command: Vec<String>,
    #[serde(default = "default_working_directory")]
    pub working_directory: String,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub success_exit_code: Option<i32>,
    #[serde(default)]
    pub timeout: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub produces: Vec<String>,
    #[serde(default)]
    pub consumes: Vec<String>,
}

fn default_working_directory() -> String {
    "${PROJECT_ROOT}".to_string()
}

/// A `ContractSpec` with every `${VAR}`/`${VAR:-default}`/`${PROJECT_ROOT}`
/// placeholder substituted.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedContract {
    pub command: Vec<String>,
    pub working_directory: PathBuf,
    pub artifacts: Vec<PathBuf>,
    pub success_exit_code: i32,
}

impl ContractSpec {
    pub fn resolve(&self, project_root: &Path) -> Result<ResolvedContract, String> {
        let command = self
            .command
            .iter()
            .map(|c| interpolate(c, project_root))
            .collect::<Result<Vec<_>, _>>()?;
        let working_directory = PathBuf::from(interpolate(&self.working_directory, project_root)?);
        let artifacts = self
            .artifacts
            .iter()
            .map(|a| interpolate(a, project_root).map(PathBuf::from))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ResolvedContract {
            command,
            working_directory,
            artifacts,
            success_exit_code: self.success_exit_code.unwrap_or(0),
        })
    }
}

/// Substitute every `${VAR}` / `${VAR:-default}` occurrence anywhere inside
/// `input` — unlike [`resolve_env_string`] (whole-value-only, used for
/// simple config path fields), this substitutes mid-string, needed for
/// Pipeline Contract fields like `${PROJECT_ROOT}/target/release/samgraha.exe`.
/// `PROJECT_ROOT` resolves to `project_root`; everything else reads the
/// process environment. Errors on an unresolved bare `${VAR}` — contract
/// fields have no repo-relative fallback the way `resolve_configured_dir`
/// does, so an unset var is a clear misconfiguration, not a default case.
pub fn interpolate(input: &str, project_root: &Path) -> Result<String, String> {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            return Err(format!("Unterminated '${{' in '{}'", input));
        };
        let inner = &after[..end];
        let (var_name, default) = match inner.split_once(":-") {
            Some((n, d)) => (n, Some(d)),
            None => (inner, None),
        };
        let value = if var_name == "PROJECT_ROOT" {
            Some(project_root.to_string_lossy().to_string())
        } else {
            std::env::var(var_name).ok()
        };
        match value.or_else(|| default.map(|d| d.to_string())) {
            Some(v) => out.push_str(&v),
            None => {
                return Err(format!(
                    "Unresolved environment variable '{}' in '{}'",
                    var_name, input
                ))
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
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

/// Resolve `${VAR}` or `${VAR:-default}` from the process environment.
///
/// - `"${VAR}"`: read `VAR` from the environment. Returns `None` if unset —
///   callers that have their own fallback (e.g. [`resolve_configured_dir`])
///   use that; callers with no fallback should treat `None` as an error.
/// - `"${VAR:-default}"`: read `VAR`; if unset, use the literal `default`
///   instead (always `Some`, never propagates to a caller's fallback).
/// - Anything else is returned unchanged (a literal value).
pub fn resolve_env_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let Some(inner) = trimmed.strip_prefix("${").and_then(|s| s.strip_suffix('}')) else {
        return Some(trimmed.to_string());
    };
    if let Some((var_name, default)) = inner.split_once(":-") {
        Some(std::env::var(var_name).unwrap_or_else(|_| default.to_string()))
    } else {
        std::env::var(inner).ok()
    }
}

/// Resolve a configured path that may be a `${VAR}` or `${VAR:-default}` placeholder.
///
/// - `"${VAR}"` (the whole string, nothing else): read `VAR` from the process
///   environment. If set, use its value (joined to `root` if relative). If
///   unset, fall back to `root.join(fallback_rel)` — so the tool works with
///   zero env configuration on a single-machine setup.
/// - `"${VAR:-default}"`: read `VAR`; if unset, use `default` (joined to
///   `root` if relative) instead of `fallback_rel`.
/// - Anything else is a literal path, used as-is (joined to `root` if relative).
pub fn resolve_configured_dir(raw: &str, root: &Path, fallback_rel: &str) -> PathBuf {
    let literal = resolve_env_string(raw);

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeConfig {
    /// The root directory containing Knowledge Systems (used when kind = "knowledge").
    #[serde(default = "default_knowledge_root")]
    pub root: String,
    /// Always loaded, high priority (required dependencies).
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Always loaded, lower priority (adjacent repos to consult).
    #[serde(default)]
    pub interests: Vec<String>,
}

fn default_knowledge_root() -> String {
    "system".to_string()
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            root: default_knowledge_root(),
            dependencies: Vec::new(),
            interests: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryKind {
    Repository,
    Knowledge,
}

impl Default for RepositoryKind {
    fn default() -> Self {
        Self::Repository
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryConfig {
    #[serde(default)]
    pub kind: RepositoryKind,
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
    /// Paths under `implementation.dir` to skip when pipelines scan for
    /// source (e.g. `["vendor", "target"]`). Relative to `implementation.dir`.
    #[serde(default)]
    pub source_exclude: Vec<String>,
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
            kind: RepositoryKind::default(),
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
            source_exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentationConfig {
    /// Documentation root directory, relative to the repository root (or an
    /// absolute path resolved from env — see [`resolve_configured_dir`]).
    #[serde(default = "default_docs_root_dir")]
    pub root_dir: String,
    /// Which documentation standard system to use (fallback to default if None).
    #[serde(default)]
    pub standard_system: Option<String>,
    /// Script check overrides, mapping rule ID to custom script paths.
    #[serde(default)]
    pub script_overrides: std::collections::HashMap<String, String>,
    /// Check overrides, mapping check name to custom script path.
    /// Higher priority than script_overrides — the audit engine resolves
    /// check_overrides[check_name] before script_overrides[rule_id].
    /// Paths are relative to repo root (e.g. "scripts/build-succeeds.sh").
    #[serde(default)]
    pub check_overrides: std::collections::HashMap<String, String>,
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
            standard_system: None,
            script_overrides: std::collections::HashMap::new(),
            check_overrides: std::collections::HashMap::new(),
            domain: Vec::new(),
            domain_exclusion: Vec::new(),
        }
    }
}

/// Options controlling `init_repository()` behavior.
/// All fields have safe defaults — zeroing `InitOptions` preserves
/// the exact current behavior of `init_repository(root, false)`.
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    /// Overwrite existing `samgraha.toml` (vs. backfill missing keys).
    pub force: bool,
    /// Document standard system name to set in
    /// `[repository.documentation] standard_system`.
    /// `None` = leave unset (current default behavior).
    pub standard_system: Option<String>,
    /// Script check overrides: rule_id -> script path.
    pub script_overrides: HashMap<String, String>,
    /// Check overrides: check_name -> script path.
    /// Higher priority than `script_overrides` in the audit resolution chain.
    pub check_overrides: HashMap<String, String>,
    /// Probe repo root for `docs/`, `src|crates/`, `tests/`, `scripts/` and set
    /// literal paths in the TOML if found. Skip missing dirs.
    pub auto_detect_dirs: bool,
    /// Sync the declared Knowledge System from global store into
    /// the local `.samgraha/` after writing `samgraha.toml`.
    pub sync_knowledge_system: bool,
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
            pipelines: None,
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

#[cfg(test)]
mod resolve_env_string_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_var_name() -> String {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("SAMGRAHA_TEST_VAR_{}_{}", std::process::id(), id)
    }

    #[test]
    fn literal_string_passes_through() {
        assert_eq!(resolve_env_string("literal/path"), Some("literal/path".to_string()));
    }

    #[test]
    fn bare_var_resolves_when_set() {
        let name = unique_var_name();
        std::env::set_var(&name, "resolved-value");
        assert_eq!(resolve_env_string(&format!("${{{}}}", name)), Some("resolved-value".to_string()));
        std::env::remove_var(&name);
    }

    #[test]
    fn bare_var_is_none_when_unset() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        assert_eq!(resolve_env_string(&format!("${{{}}}", name)), None);
    }

    #[test]
    fn var_with_default_falls_back_when_unset() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        assert_eq!(
            resolve_env_string(&format!("${{{}:-fallback}}", name)),
            Some("fallback".to_string())
        );
    }

    #[test]
    fn var_with_default_prefers_env_when_set() {
        let name = unique_var_name();
        std::env::set_var(&name, "from-env");
        assert_eq!(
            resolve_env_string(&format!("${{{}:-fallback}}", name)),
            Some("from-env".to_string())
        );
        std::env::remove_var(&name);
    }

    #[test]
    fn resolve_configured_dir_uses_fallback_rel_when_bare_var_unset() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        let root = std::path::Path::new("/repo");
        let resolved = resolve_configured_dir(&format!("${{{}}}", name), root, "docs");
        assert_eq!(resolved, root.join("docs"));
    }

    #[test]
    fn resolve_configured_dir_uses_inline_default_over_fallback_rel() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        let root = std::path::Path::new("/repo");
        let resolved = resolve_configured_dir(&format!("${{{}:-custom}}", name), root, "docs");
        assert_eq!(resolved, root.join("custom"));
    }

    #[test]
    fn interpolate_substitutes_project_root_mid_string() {
        let root = std::path::Path::new("/repo");
        let result = interpolate("${PROJECT_ROOT}/target/release/x.exe", root).unwrap();
        assert_eq!(result, "/repo/target/release/x.exe");
    }

    #[test]
    fn interpolate_errors_on_unresolved_bare_var() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        let root = std::path::Path::new("/repo");
        assert!(interpolate(&format!("${{{}}}/x", name), root).is_err());
    }

    #[test]
    fn interpolate_uses_inline_default_mid_string() {
        let name = unique_var_name();
        std::env::remove_var(&name);
        let root = std::path::Path::new("/repo");
        let result = interpolate(&format!("${{{}:-cargo}}", name), root).unwrap();
        assert_eq!(result, "cargo");
    }

    #[test]
    fn contract_spec_resolve_expands_all_fields() {
        let spec = ContractSpec {
            command: vec!["${PROJECT_ROOT}/build.sh".to_string()],
            working_directory: "${PROJECT_ROOT}".to_string(),
            artifacts: vec!["${PROJECT_ROOT}/out.bin".to_string()],
            success_exit_code: None,
            timeout: None,
            description: None,
            produces: vec![],
            consumes: vec![],
        };
        let root = std::path::Path::new("/repo");
        let resolved = spec.resolve(root).unwrap();
        assert_eq!(resolved.working_directory, root);
        assert_eq!(resolved.artifacts, vec![root.join("out.bin")]);
        assert_eq!(resolved.success_exit_code, 0);
    }

    #[test]
    fn check_version_ok_on_matching_major() {
        let cfg = PipelineContractConfig {
            version: "1.0".to_string(),
            build: None,
            test: None,
            package: None,
            deploy: None,
        };
        assert_eq!(cfg.check_version(), Ok(false));
    }

    #[test]
    fn check_version_warns_on_higher_minor() {
        let cfg = PipelineContractConfig {
            version: "1.7".to_string(),
            build: None,
            test: None,
            package: None,
            deploy: None,
        };
        assert_eq!(cfg.check_version(), Ok(true));
    }

    #[test]
    fn check_version_errors_on_different_major() {
        let cfg = PipelineContractConfig {
            version: "2.0".to_string(),
            build: None,
            test: None,
            package: None,
            deploy: None,
        };
        assert!(cfg.check_version().is_err());
    }
}
