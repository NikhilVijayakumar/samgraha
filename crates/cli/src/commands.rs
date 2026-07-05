use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::package::PackageProfile;
use services::package::PackageFormat;
use schemas::search::{RetrievalLevel, SearchQuery, SectionQuery};
use std::path::PathBuf;

use crate::output::{format_output, render_audit, render_audit_report, render_compile, render_info, render_registry_list, render_search, render_sections, render_workspace_compile, OutputFormat};
use common::config::{resolve_configured_dir, SamgrahaConfig};
use services::{KnowledgeRuntime, WorkspaceService};

#[derive(Parser)]
#[command(
    name = "samgraha",
    version = env!("CARGO_PKG_VERSION"),
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

        #[arg(
            long = "gate",
            help = "Quality gate minimum score (default: 100.0). Pass --gate or --gate <SCORE>",
            default_missing_value = "100.0",
            num_args = 0..=1,
            value_name = "SCORE",
        )]
        gate: Option<f64>,

        #[arg(long = "report", help = "Save markdown report under [report].dir/audit/{latest,archive}/")]
        report: bool,
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

        #[arg(long = "json", help = "Output as single JSON file (legacy) instead of directory")]
        json: bool,
    },

    #[command(about = "Repository Registry operations")]
    Registry {
        #[command(subcommand)]
        action: RegistryAction,
    },

    #[command(about = "Generate .env.example with all env keys samgraha reads")]
    Env {
        #[arg(help = "Path to repository")]
        path: Option<PathBuf>,
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

#[derive(Subcommand)]
pub enum RegistryAction {
    #[command(about = "Register this repository in local registry")]
    Register,
    #[command(about = "Unregister a repository by UUID")]
    Unregister {
        #[arg(help = "Repository UUID")]
        uuid: String,
    },
    #[command(about = "Sync dependency metadata from their manifests")]
    Sync,
    #[command(about = "Refresh all cached dependency metadata")]
    Refresh,
    #[command(about = "Show registry status")]
    Status,
    #[command(about = "List registered repositories")]
    List,
    #[command(about = "Resolve dependencies for runtime")]
    Resolve {
        #[arg(help = "Resolution mode (runtime)")]
        mode: String,
    },
}

impl Cli {
    pub fn execute(&self) -> Result<ExitCode> {
        let format = if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        };

        // Skip repo guard for commands that don't need an initialized repo.
        let skip_guard = matches!(&self.command,
            Commands::Init { .. } | Commands::Version { .. }
        );
        if !skip_guard {
            ensure_samgraha_repo()?;
        }

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
                report,
            } => self.execute_audit(domain.as_deref(), provider, *all, *gate, *report, &format),
            Commands::Info { path } => self.execute_info(path.as_ref(), &format),
            Commands::Init { path, force } => self.execute_init(path.as_ref(), *force, &format),
            Commands::Package { output, profile, json } => {
                self.execute_package(output.as_ref(), profile.as_deref(), *json, &format)
            }
            Commands::Registry { action } => self.execute_registry(action, &format),
            Commands::Env { path } => self.execute_env(path.as_ref(), &format),
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

        if result.success {
            sync_registry_best_effort(&root, &runtime.context.config);
        }

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
        ensure_compiled(&root, &config)?;
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
        ensure_compiled(&root, &config)?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let query = SectionQuery {
            semantic_type: semantic_type.to_string(),
            domain: domain.map(|d| d.to_string()),
            max_results: max,
            document_id: None,
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
        report: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        ensure_compiled(&root, &config)?;
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

        let audit_report = runtime.audit(audit_domain, &provider_names, None)?;
        println!("{}", render_audit(&audit_report, format));

        if report {
            let reports_base = resolve_configured_dir(
                &runtime.context.config.report.dir,
                &root,
                "docs/raw/reports",
            );
            let latest_dir = reports_base.join("audit").join("latest");
            let archive_dir = reports_base.join("audit").join("archive");
            std::fs::create_dir_all(&latest_dir)?;
            std::fs::create_dir_all(&archive_dir)?;

            let md = render_audit_report(&audit_report);
            let now = chrono::Local::now();
            let archive_path = archive_dir.join(format!("{}.md", now.format("%Y%m%d-%H%M%S")));
            std::fs::write(&archive_path, &md)?;
            let latest_path = latest_dir.join("report.md");
            std::fs::write(&latest_path, &md)?;

            println!("Report saved: {}", latest_path.display());
            println!("Archived:     {}", archive_path.display());
        }

        if let Some(min_score) = gate {
            if audit_report.score.overall < min_score {
                eprintln!(
                    "Quality gate failed: score {:.1}% < minimum {:.1}%",
                    audit_report.score.overall, min_score
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
        let samgraha_dir = root.join(".samgraha");

        std::fs::create_dir_all(&samgraha_dir).context(format!(
            "Failed to create {}",
            samgraha_dir.display()
        ))?;

        // Full-schema template: what a fresh samgraha.toml would contain.
        // Used as-is on first init, or as the source of missing keys/sections
        // to backfill into an existing samgraha.toml (never overwrites a key
        // that's already there — see `merge_missing_keys`).
        let mut template = SamgrahaConfig::default();
        let dir_name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repository")
            .to_string();
        template.repository.id = Some(dir_name.clone());
        template.repository.name = Some(dir_name);
        template.repository.uuid = Some(uuid::Uuid::new_v4());
        // Declare every builtin standard by default; repos that don't use one
        // (e.g. no `prototype` docs) add it to `domain_exclusion` instead of
        // deleting it here, so the full catalog stays visible in the toml.
        template.repository.documentation.domain = standards::all_builtin_standards()
            .into_iter()
            .map(|s| s.domain)
            .collect();

        let (config, status) = if config_path.exists() && !force {
            let existing = std::fs::read_to_string(&config_path)
                .context(format!("Failed to read {}", config_path.display()))?;
            let mut doc: toml_edit::DocumentMut = existing
                .parse()
                .context(format!("Failed to parse {}", config_path.display()))?;
            let template_content = toml::to_string_pretty(&template)?;
            let template_doc: toml_edit::DocumentMut = template_content
                .parse()
                .context("Failed to parse generated template config")?;

            let added = merge_missing_keys(doc.as_table_mut(), template_doc.as_table());
            if added > 0 {
                std::fs::write(&config_path, doc.to_string()).context(format!(
                    "Failed to write config to {}",
                    config_path.display()
                ))?;
            }

            let merged: SamgrahaConfig = toml::from_str(&doc.to_string())
                .context("Merged samgraha.toml failed to parse back as valid config")?;
            let status = if added > 0 {
                format!(
                    "Updated {} — added {added} missing key(s), left the rest untouched",
                    config_path.display()
                )
            } else {
                format!("{} already covers every known key — nothing to add", config_path.display())
            };
            (merged, status)
        } else {
            let content = toml::to_string_pretty(&template)?;
            std::fs::write(&config_path, content).context(format!(
                "Failed to write config to {}",
                config_path.display()
            ))?;
            (template, format!("Initialized samgraha repository at {}", root.display()))
        };

        let env_path = write_env_example(&root)?;

        println!("{status}");
        println!("Generated {}", env_path.display());
        println!("{}", format_output(&config, format));
        Ok(ExitCode::Success)
    }

    fn execute_env(&self, path: Option<&PathBuf>, _format: &OutputFormat) -> Result<ExitCode> {
        let root = path
            .cloned()
            .unwrap_or_else(|| crate::config::discover_repository_root().unwrap());
        let env_path = write_env_example(&root)?;
        println!("Generated {}", env_path.display());
        Ok(ExitCode::Success)
    }

    fn execute_package(
        &self,
        output: Option<&PathBuf>,
        profile: Option<&str>,
        json_mode: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        ensure_compiled(&root, &config)?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let pkg_profile = match profile.unwrap_or("full") {
            "minimal" => PackageProfile::Minimal,
            "development" => PackageProfile::Development,
            "documentation" => PackageProfile::Documentation,
            "engineering" => PackageProfile::Engineering,
            "ai-assistant" => PackageProfile::AiAssistant,
            _ => PackageProfile::Full,
        };

        let pkg_format = if json_mode {
            PackageFormat::Json
        } else {
            PackageFormat::Directory
        };

        let output_path = output.cloned().unwrap_or_else(|| {
            if json_mode {
                root.join("knowledge-package.json")
            } else {
                root.join("knowledge-package")
            }
        });

        let result = runtime.package(output_path.clone(), pkg_profile, pkg_format)?;

        let format_label = if json_mode { "json" } else { "directory" };
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
                    "format": format_label,
                }),
                format,
            )
        );

        Ok(ExitCode::Success)
    }

    fn execute_registry(&self, action: &RegistryAction, format: &OutputFormat) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;

        use services::registry_client::FileRegistryClient;
        use services::registry_client::RegistryClient;

        let client = FileRegistryClient::with_config(&root, &config.resolver);

        match action {
            RegistryAction::Register => {
                let manifest_path = root.join(".samgraha").join("manifest.json");
                if !manifest_path.exists() {
                    anyhow::bail!(
                        "No manifest found at {}. Compile the repository first.",
                        manifest_path.display()
                    );
                }
                let content = std::fs::read_to_string(&manifest_path)?;
                let manifest: schemas::manifest::RepositoryManifest =
                    serde_json::from_str(&content)?;
                client.register(&manifest)?;
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "success": true,
                            "action": "register",
                            "repository": manifest.repository.id,
                            "uuid": manifest.repository.uuid.to_string(),
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
            RegistryAction::Unregister { uuid } => {
                let parsed_uuid = uuid::Uuid::parse_str(uuid)?;
                client.unregister(&parsed_uuid)?;
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "success": true,
                            "action": "unregister",
                            "uuid": uuid,
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
            RegistryAction::Sync => {
                client.sync(&config)?;
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "success": true,
                            "action": "sync",
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
            RegistryAction::Refresh => {
                client.sync(&config)?;
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "success": true,
                            "action": "refresh",
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
            RegistryAction::Status => {
                let entries = client.list()?;
                let now = std::time::SystemTime::now();
                let repos: Vec<_> = entries
                    .iter()
                    .map(|e| serde_json::json!({
                        "id": e.repository.id,
                        "uuid": e.repository.uuid.to_string(),
                        "status": format!("{:?}", e.status(now)),
                        "revision": e.revision,
                        "audit": e.audit,
                        "expires": e.expires,
                    }))
                    .collect();
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "registered": entries.len(),
                            "dependencies": config.repository.dependencies.len(),
                            "repositories": repos,
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
            RegistryAction::List => {
                let entries = client.list()?;
                println!("{}", render_registry_list(&entries, format));
                Ok(ExitCode::Success)
            }
            RegistryAction::Resolve { mode } => {
                if mode != "runtime" {
                    anyhow::bail!("Unsupported resolve mode: {}. Use 'runtime'.", mode);
                }
                ensure_compiled(&root, &config)?;
                let runtime = KnowledgeRuntime::new(&root, config)?;
                let output_path = root.join(".samgraha").join("resolved");
                let result = runtime.resolve(
                    schemas::package::PackageProfile::Full,
                    output_path,
                    PackageFormat::Directory,
                    schemas::package::PackageLayout::Virtual,
                )?;
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({
                            "success": true,
                            "action": "resolve",
                            "mode": mode,
                            "repositories": result.context.dependencies.len() + 1,
                            "output": result.output_path.display().to_string(),
                        }),
                        format,
                    )
                );
                Ok(ExitCode::Success)
            }
        }
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

/// Automatically compile if the knowledge database is empty or missing.
fn ensure_compiled(root: &PathBuf, config: &SamgrahaConfig) -> Result<()> {
    let runtime = KnowledgeRuntime::new(root, config.clone())?;
    let info = runtime.info();
    if info.document_count == 0 {
        let request = CompilationRequest {
            scope: schemas::compilation::CompilationScope::Repository,
            force: false,
            watch: false,
        };
        let result = runtime.compile(&request)?;
        if !result.success {
            tracing::warn!("Auto-compile produced errors: {} failures", result.documents_failed);
        }
    }
    Ok(())
}

/// Auto-register/refresh declared dependencies and interests in the local
/// registry after a successful compile — the "automatic" half of the hybrid
/// registration model. `registry sync`/`registry register` remain available
/// as an explicit, manual path at any time; this just means a repo's
/// dependencies don't go stale (or missing) just because nobody remembered
/// to run `sync` by hand. Failures are logged, not fatal — a registry hiccup
/// shouldn't fail an otherwise-successful compile.
fn sync_registry_best_effort(root: &std::path::Path, config: &SamgrahaConfig) {
    if !config.resolver.auto_refresh {
        return;
    }
    use services::registry_client::{FileRegistryClient, RegistryClient};
    let client = FileRegistryClient::with_config(root, &config.resolver);
    if let Err(e) = client.sync(config) {
        tracing::warn!("Registry sync after compile failed: {e}");
    }
}

/// Recursively insert keys present in `defaults` but absent from `existing`,
/// at any table depth. Never touches a key `existing` already has, even if
/// its value differs from the default (a scalar/array already present wins
/// outright; a table already present is recursed into for its own missing
/// sub-keys, never replaced wholesale). Returns the number of keys added.
///
/// This is what lets `samgraha init` be re-run safely on an existing
/// samgraha.toml: schema growth (a new section/field in a later samgraha
/// version) gets backfilled, while every existing customization — including
/// comments and formatting, since this operates on a `toml_edit` document —
/// survives untouched.
fn merge_missing_keys(existing: &mut toml_edit::Table, defaults: &toml_edit::Table) -> usize {
    let mut added = 0;
    for (key, default_item) in defaults.iter() {
        if !existing.contains_key(key) {
            existing.insert(key, default_item.clone());
            added += 1;
        } else if let Some(default_tbl) = default_item.as_table() {
            if let Some(existing_tbl) = existing.get_mut(key).and_then(|i| i.as_table_mut()) {
                added += merge_missing_keys(existing_tbl, default_tbl);
            }
        }
    }
    added
}

/// Ensure `.env.example` documents every env key samgraha reads for `${VAR}`
/// placeholders in samgraha.toml (see `resolve_configured_dir`), values left
/// blank/commented for the user to fill in per machine.
///
/// Additive, not overwriting: a repo may already have an `.env.example` with
/// unrelated keys (e.g. this repo's own release-build settings). Only keys
/// not already present get appended, so regenerating never clobbers existing
/// content.
fn write_env_example(root: &std::path::Path) -> Result<PathBuf> {
    const KEYS: &[(&str, &str)] = &[
        (
            "SAMGRAHA_REPORT_DIR",
            "# Absolute path for generated reports (e.g. `samgraha audit --report`).\n\
             # Unset falls back to <repo>/docs/raw/reports.\n\
             # SAMGRAHA_REPORT_DIR=\n",
        ),
        (
            "SAMGRAHA_DOCS_DIR",
            "# Absolute path to this repository's documentation root.\n\
             # Unset falls back to <repo>/docs.\n\
             # SAMGRAHA_DOCS_DIR=\n",
        ),
        (
            "SAMGRAHA_IMPLEMENTATION_DIR",
            "# Absolute path to this repository's implementation/source directory.\n\
             # Reserved for future traceability checks; unset falls back to <repo>/src.\n\
             # SAMGRAHA_IMPLEMENTATION_DIR=\n",
        ),
        (
            "SAMGRAHA_SCRIPTS_DIR",
            "# Absolute path to this repository's external scripts directory.\n\
             # Only relevant if [repository.scripts] is set in samgraha.toml.\n\
             # SAMGRAHA_SCRIPTS_DIR=\n",
        ),
        (
            "SAMGRAHA_TESTS_DIR",
            "# Absolute path to this repository's test directory, if kept outside\n\
             # implementation.dir. Only relevant if [repository.tests] is set.\n\
             # SAMGRAHA_TESTS_DIR=\n",
        ),
    ];

    let path = root.join(".env.example");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();

    let mut appended = String::new();
    for (key, block) in KEYS {
        if !existing.contains(key) {
            if !appended.is_empty() || !existing.is_empty() {
                appended.push('\n');
            }
            appended.push_str(block);
        }
    }

    if appended.is_empty() {
        return Ok(path);
    }

    let mut content = existing;
    content.push_str(&appended);
    if !content.contains("cp .env.example .env") {
        content.push_str("\n# Copy this file to .env and uncomment the values to configure:\n#   cp .env.example .env\n");
    }
    std::fs::write(&path, content).context(format!("Failed to write {}", path.display()))?;
    Ok(path)
}

/// Check that we are inside a samgraha repository (has `.samgraha/` dir or `samgraha.toml`).
fn ensure_samgraha_repo() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let mut current = Some(cwd.as_path());
    while let Some(dir) = current {
        if dir.join(".samgraha").is_dir() || dir.join("samgraha.toml").exists() {
            return Ok(());
        }
        current = dir.parent();
    }
    anyhow::bail!(
        "fatal: not a samgraha repository (or any of the parent directories). \
         Run 'samgraha init' first to initialize."
    );
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
