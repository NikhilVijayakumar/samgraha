use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::package::PackageProfile;
use schemas::search::{RetrievalLevel, SearchQuery, SectionQuery};
use std::path::PathBuf;

use crate::output::{format_output, render_audit, render_compile, render_info, render_search, render_sections, render_workspace_compile, OutputFormat};
use common::config::SamgrahaConfig;
use services::{KnowledgeRuntime, WorkspaceService};

#[derive(Parser)]
#[command(
    name = "samgraha",
    version = "0.1.0",
    about = "Knowledge Engineering Platform"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, long = "config")]
    pub config: Option<PathBuf>,

    #[arg(global = true, long = "json", help = "Output in JSON format")]
    pub json: bool,

    #[arg(global = true, long = "no-color", help = "Disable color output")]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Discover and compile documentation")]
    Compile {
        #[arg(help = "Path to compile (default: current directory)")]
        path: Option<PathBuf>,

        #[arg(long = "domain", help = "Compile only specific domains")]
        domain: Vec<String>,

        #[arg(long = "force", help = "Force full recompilation")]
        force: bool,

        #[arg(long = "watch", help = "Watch for changes and recompile")]
        watch: bool,

        #[arg(long = "workspace", help = "Compile all repositories in the workspace")]
        workspace: bool,
    },

    #[command(about = "Search compiled knowledge")]
    Search {
        query: String,

        #[arg(long = "domain", help = "Filter by domain")]
        domain: Option<String>,

        #[arg(
            long = "level",
            help = "Retrieval level (metadata, summary, section, full)",
            default_value = "metadata"
        )]
        level: String,

        #[arg(long = "max", help = "Maximum results", default_value = "20")]
        max: usize,
    },

    #[command(about = "Query sections by semantic type")]
    Sections {
        #[arg(help = "Semantic type to query (e.g. functional_requirements, business_rules)")]
        semantic_type: String,

        #[arg(long = "domain", help = "Filter by domain (standard)")]
        domain: Option<String>,

        #[arg(long = "max", help = "Maximum results", default_value = "50")]
        max: usize,
    },

    #[command(about = "Run audit checks")]
    Audit {
        #[arg(help = "Domain to audit (default: all)")]
        domain: Option<String>,

        #[arg(
            long = "provider",
            help = "Audit provider(s)",
            default_value = "deterministic"
        )]
        provider: Vec<String>,

        #[arg(long = "all", help = "Audit all domains")]
        all: bool,

        #[arg(long = "gate", help = "Minimum score for quality gate")]
        gate: Option<f64>,
    },

    #[command(about = "Display repository information")]
    Info {
        #[arg(help = "Path to repository")]
        path: Option<PathBuf>,
    },

    #[command(about = "Initialize a new repository configuration")]
    Init {
        #[arg(help = "Path to initialize")]
        path: Option<PathBuf>,

        #[arg(long = "force", help = "Overwrite existing configuration")]
        force: bool,
    },

    #[command(about = "Package compiled knowledge for distribution")]
    Package {
        #[arg(help = "Output path for the package")]
        output: Option<PathBuf>,

        #[arg(
            long = "profile",
            help = "Package profile (minimal, development, documentation, engineering, ai-assistant, full)"
        )]
        profile: Option<String>,
    },

    #[command(about = "Workspace multi-repository operations")]
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },

    #[command(about = "Display version information")]
    Version,
}

#[derive(Subcommand)]
pub enum WorkspaceAction {
    #[command(about = "Initialize a workspace configuration")]
    Init {
        #[arg(help = "Workspace name")]
        name: String,

        #[arg(help = "Repository paths to include")]
        repositories: Vec<String>,

        #[arg(help = "Path for workspace root (default: current directory)")]
        path: Option<PathBuf>,
    },

    #[command(about = "Compile all workspace repositories")]
    Compile {
        #[arg(long = "force", help = "Force full recompilation")]
        force: bool,
    },

    #[command(about = "Search across all workspace repositories")]
    Search {
        query: String,

        #[arg(long = "max", default_value = "20")]
        max: usize,
    },
}

impl Cli {
    pub fn execute(&self) -> Result<ExitCode> {
        let format = if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        };

        match &self.command {
            Commands::Compile {
                path,
                domain,
                force,
                watch,
                workspace,
            } => self.execute_compile(path.as_ref(), domain, *force, *watch, *workspace, &format),
            Commands::Search {
                query,
                domain,
                level,
                max,
            } => self.execute_search(query, domain.as_deref(), level, *max, &format),
            Commands::Sections {
                semantic_type,
                domain,
                max,
            } => self.execute_sections(semantic_type, domain.as_deref(), *max, &format),
            Commands::Audit {
                domain,
                provider,
                all,
                gate,
            } => self.execute_audit(domain.as_deref(), provider, *all, *gate, &format),
            Commands::Info { path } => self.execute_info(path.as_ref(), &format),
            Commands::Init { path, force } => self.execute_init(path.as_ref(), *force, &format),
            Commands::Package { output, profile } => {
                self.execute_package(output.as_ref(), profile.as_deref(), &format)
            }
            Commands::Workspace { action } => self.execute_workspace(action, &format),
            Commands::Version => self.execute_version(&format),
        }
    }

    fn execute_compile(
        &self,
        path: Option<&PathBuf>,
        domains: &[String],
        force: bool,
        watch: bool,
        workspace: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap());

        // Auto-detect workspace if --workspace flag or samgraha-workspace.toml found.
        let is_workspace = workspace || root.join("samgraha-workspace.toml").exists();

        if is_workspace {
            if let Some((ws_root, ws_config)) = WorkspaceService::discover(&root) {
                let request = CompilationRequest {
                    scope: CompilationScope::Workspace,
                    force,
                    watch: false,
                };
                let result = services::WorkspaceService::compile(&ws_root, &ws_config, &request)?;
                println!("{}", render_workspace_compile(&result, format));
                return Ok(if result.total_errors == 0 {
                    ExitCode::Success
                } else {
                    ExitCode::CompilationError
                });
            }
        }

        let config = crate::config::load_config(self.config.as_ref())?;
        let debounce_ms = config.compilation.debounce_ms;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let scope = if domains.is_empty() {
            CompilationScope::Repository
        } else {
            CompilationScope::Domains(domains.to_vec())
        };

        let initial_request = CompilationRequest {
            scope: scope.clone(),
            force,
            watch: false,
        };
        let result = runtime.compile(&initial_request)?;
        println!("{}", render_compile(&result, format));

        if watch {
            self.watch_compile(&runtime, &root, &scope, debounce_ms, format)?;
        }

        if result.success {
            Ok(ExitCode::Success)
        } else {
            Ok(ExitCode::CompilationError)
        }
    }

    fn watch_compile(
        &self,
        runtime: &KnowledgeRuntime,
        root: &PathBuf,
        scope: &CompilationScope,
        debounce_ms: u64,
        format: &OutputFormat,
    ) -> Result<()> {
        use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(root.as_ref(), RecursiveMode::Recursive)?;

        println!("Watching {:?} for changes (Ctrl+C to stop)...", root);

        loop {
            // Block until first event.
            let first = match rx.recv() {
                Ok(ev) => ev,
                Err(_) => break,
            };

            if !is_relevant_event(&first) {
                continue;
            }

            // Drain any additional events within the debounce window.
            let deadline = std::time::Instant::now()
                + Duration::from_millis(debounce_ms.max(50));
            loop {
                let remaining = deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    break;
                }
                match rx.recv_timeout(remaining) {
                    Ok(_) => {} // drain
                    Err(_) => break,
                }
            }

            let request = CompilationRequest {
                scope: scope.clone(),
                force: false,
                watch: false,
            };
            match runtime.compile(&request) {
                Ok(result) => println!("{}", render_compile(&result, format)),
                Err(e) => eprintln!("Compilation error: {}", e),
            }
        }

        Ok(())
    }

    fn execute_search(
        &self,
        query: &str,
        domain: Option<&str>,
        level: &str,
        max: usize,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let level = match level {
            "summary" => RetrievalLevel::Summary,
            "section" => RetrievalLevel::Section,
            "full" => RetrievalLevel::Full,
            _ => RetrievalLevel::Metadata,
        };

        let search_query = SearchQuery {
            query: query.to_string(),
            domain: domain.map(|d| d.to_string()),
            level,
            max_results: max,
            ..Default::default()
        };

        let results = runtime.search(&search_query)?;
        println!("{}", render_search(&results, format));

        if results.results.is_empty() {
            return Ok(ExitCode::InputError);
        }

        Ok(ExitCode::Success)
    }

    fn execute_sections(
        &self,
        semantic_type: &str,
        domain: Option<&str>,
        max: usize,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let query = SectionQuery {
            semantic_type: semantic_type.to_string(),
            domain: domain.map(|d| d.to_string()),
            max_results: max,
        };

        let response = runtime.get_sections(&query)?;
        println!("{}", render_sections(&response, format));

        if response.sections.is_empty() {
            return Ok(ExitCode::InputError);
        }

        Ok(ExitCode::Success)
    }

    fn execute_audit(
        &self,
        domain: Option<&str>,
        providers: &[String],
        _all: bool,
        gate: Option<f64>,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let mut runtime = KnowledgeRuntime::new(&root, config)?;

        runtime.register_audit_provider("deterministic", |docs, rules| {
            services::DeterministicAuditProvider::execute(docs, rules)
        });
        runtime.register_audit_provider("semantic", |docs, rules| {
            providers::SemanticAuditProvider::execute(docs, rules)
        });

        let audit_domain = if _all { None } else { domain };
        let provider_names: Vec<String> = if providers.is_empty() {
            vec!["deterministic".to_string()]
        } else {
            providers.to_vec()
        };

        let report = runtime.audit(audit_domain, &provider_names, None)?;
        println!("{}", render_audit(&report, format));

        if let Some(min_score) = gate {
            if report.score.overall < min_score {
                eprintln!(
                    "Quality gate failed: score {:.1}% < minimum {:.1}%",
                    report.score.overall, min_score
                );
                return Ok(ExitCode::AuditFailure);
            }
        }

        Ok(ExitCode::Success)
    }

    fn execute_info(&self, path: Option<&PathBuf>, format: &OutputFormat) -> Result<ExitCode> {
        let root = path.cloned().unwrap_or_else(|| {
            crate::config::discover_repository_root()
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
        });
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let info = runtime.info();
        println!("{}", render_info(&info, format));
        Ok(ExitCode::Success)
    }

    fn execute_init(
        &self,
        path: Option<&PathBuf>,
        force: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let config_path = root.join("samgraha.toml");

        if config_path.exists() && !force {
            anyhow::bail!(
                "Configuration already exists at {}. Use --force to overwrite.",
                config_path.display()
            );
        }

        let config = SamgrahaConfig::default();
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, content).context(format!(
            "Failed to write config to {}",
            config_path.display()
        ))?;

        println!(
            "Initialized samgraha configuration at {}",
            config_path.display()
        );
        println!("{}", format_output(&config, format));
        Ok(ExitCode::Success)
    }

    fn execute_package(
        &self,
        output: Option<&PathBuf>,
        profile: Option<&str>,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let pkg_profile = match profile.unwrap_or("full") {
            "minimal" => PackageProfile::Minimal,
            "development" => PackageProfile::Development,
            "documentation" => PackageProfile::Documentation,
            "engineering" => PackageProfile::Engineering,
            "ai-assistant" => PackageProfile::AiAssistant,
            _ => PackageProfile::Full,
        };

        let output_path = output
            .cloned()
            .unwrap_or_else(|| root.join("knowledge-package.json"));

        let result = runtime.package(output_path.clone(), pkg_profile)?;

        println!(
            "{}",
            format_output(
                &serde_json::json!({
                    "success": true,
                    "output": result.output_path.display().to_string(),
                    "profile": result.package.manifest.profile,
                    "documents_packaged": result.documents_packaged,
                    "domains": result.package.manifest.included_domains,
                    "package_hash": result.package.integrity.package_hash,
                }),
                format,
            )
        );

        Ok(ExitCode::Success)
    }

    fn execute_workspace(&self, action: &WorkspaceAction, format: &OutputFormat) -> Result<ExitCode> {
        let cwd = std::env::current_dir().unwrap();
        match action {
            WorkspaceAction::Init { name, repositories, path } => {
                let root = path.clone().unwrap_or(cwd);
                let ws_path = WorkspaceService::init(&root, name, repositories.clone())?;
                println!("Initialized workspace at {}", ws_path.display());
                Ok(ExitCode::Success)
            }
            WorkspaceAction::Compile { force } => {
                let (ws_root, ws_config) = WorkspaceService::discover(&cwd)
                    .ok_or_else(|| anyhow::anyhow!("No samgraha-workspace.toml found"))?;
                let request = CompilationRequest {
                    scope: CompilationScope::Workspace,
                    force: *force,
                    watch: false,
                };
                let result = WorkspaceService::compile(&ws_root, &ws_config, &request)?;
                println!("{}", render_workspace_compile(&result, format));
                Ok(if result.total_errors == 0 { ExitCode::Success } else { ExitCode::CompilationError })
            }
            WorkspaceAction::Search { query, max } => {
                let (ws_root, ws_config) = WorkspaceService::discover(&cwd)
                    .ok_or_else(|| anyhow::anyhow!("No samgraha-workspace.toml found"))?;
                let search_query = SearchQuery {
                    query: query.clone(),
                    max_results: *max,
                    ..Default::default()
                };
                let results = WorkspaceService::search(&ws_root, &ws_config, &search_query)?;
                println!("{}", render_search(&results, format));
                Ok(ExitCode::Success)
            }
        }
    }

    fn execute_version(&self, format: &OutputFormat) -> Result<ExitCode> {
        let version = env!("CARGO_PKG_VERSION");
        let name = env!("CARGO_PKG_NAME");

        println!(
            "{}",
            format_output(
                &serde_json::json!({
                    "name": name,
                    "version": version,
                    "description": "Knowledge Engineering Platform",
                }),
                format,
            )
        );

        Ok(ExitCode::Success)
    }
}

fn is_relevant_event(ev: &notify::Result<notify::Event>) -> bool {
    let Ok(ev) = ev else { return false };
    matches!(
        ev.kind,
        notify::EventKind::Create(_)
            | notify::EventKind::Modify(_)
            | notify::EventKind::Remove(_)
    ) && ev.paths.iter().any(|p| {
        p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "md" || e == "toml")
            .unwrap_or(false)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    CompilationError = 1,
    AuditFailure = 2,
    ConfigurationError = 3,
    InputError = 4,
    InternalError = 5,
}

impl ExitCode {
    pub fn code(&self) -> i32 {
        *self as i32
    }

    pub fn to_process_exit_code(&self) -> std::process::ExitCode {
        match self {
            ExitCode::Success => std::process::ExitCode::SUCCESS,
            _ => std::process::ExitCode::FAILURE,
        }
    }
}
