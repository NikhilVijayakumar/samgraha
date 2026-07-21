use anyhow::Result;
use clap::{Parser, Subcommand};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::package::PackageProfile;
use services::package::PackageFormat;
use schemas::search::{RetrievalLevel, SearchQuery, SectionQuery};
use std::path::PathBuf;

use crate::output::{format_output, render_audit, render_audit_report, render_compile, render_info, render_registry_list, render_search, render_sections, render_workspace_compile, OutputFormat};
use common::config::{resolve_configured_dir, SamgrahaConfig};
use services::{CompilationService, KnowledgeRuntime, WorkspaceService};

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
        #[arg(help = "Domain to audit (default: all) — Documentation Audit only")]
        domain: Option<String>,

        #[arg(
            long = "pipeline",
            help = "Audit pipeline to run: 'all' (RepositoryKind-appropriate bundle, see audit::PipelineFactory), doc (default), build, security, consistency, coverage, architecture, vision, design, readme, prototype, external-context, engineering, feature, feature-technical, feature-design, deterministic-runtime, external-context-ownership, implementation, dependency, help, documentation-structure, knowledge-system"
        )]
        pipeline: Option<String>,

        #[arg(
            long = "provider",
            help = "Audit provider(s) — Documentation Audit only",
            default_value = "deterministic"
        )]
        provider: Vec<String>,

        #[arg(long = "all", help = "Audit all domains — Documentation Audit only")]
        all: bool,

        #[arg(
            long = "gate",
            help = "Quality gate minimum score (default: 100.0). Pass --gate or --gate <SCORE>",
            default_missing_value = "100.0",
            num_args = 0..=1,
            value_name = "SCORE",
        )]
        gate: Option<f64>,

        #[arg(long = "report", help = "Save markdown report under [report].dir/{pipeline}/{latest,archive}/")]
        report: bool,

        #[arg(long = "inspect-artifact", help = "Enable artifact-level checks (Build Audit only)")]
        inspect_artifact: bool,

        #[arg(long = "runtime", help = "Enable runtime-level checks (Security Audit only)")]
        runtime: bool,

        #[arg(long = "execute", help = "Run the declared [pipelines.build] contract instead of verify-only (Build Audit only)")]
        execute: bool,

        #[arg(long = "dry-run", help = "Print the resolved build command without running it (Build Audit only)")]
        dry_run: bool,

        #[arg(long = "yes", help = "Skip the confirmation prompt for --execute")]
        yes: bool,

        #[arg(long = "list-pipelines", help = "Print the RepositoryKind-appropriate pipeline bundle (see audit::PipelineFactory) and exit, without running anything")]
        list_pipelines: bool,
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

        #[arg(long = "standard-system", help = "Document standard system name (e.g. 'samgraha-documentation')")]
        standard_system: Option<String>,

        #[arg(long = "auto-detect", help = "Auto-detect docs/src/tests/scripts directories")]
        auto_detect: bool,

        #[arg(long = "sync", help = "Sync Knowledge System from global store after init")]
        sync: bool,
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

    #[command(about = "Generate reports from stored pipeline results")]
    Report {
        #[arg(help = "Pipeline to report on (build, security, consistency, coverage, dependency)")]
        pipeline: Option<String>,

        #[arg(long = "session", help = "Session UUID to render (default: latest)")]
        session: Option<String>,

        #[arg(long = "template", help = "Template name (without .md extension, default: pipeline-default)")]
        template: Option<String>,

        #[arg(long = "stdout", help = "Print to stdout only, do not write to disk")]
        stdout: bool,

        #[arg(long = "list-sessions", help = "List available sessions")]
        list_sessions: bool,

        #[arg(long = "list-templates", help = "List available templates")]
        list_templates: bool,
    },

    #[command(about = "Display version information")]
    Version,

    #[command(about = "Run a documentation check by name")]
    Check {
        /// Check name (e.g. "build-succeeds", "specs-compile")
        name: String,

        #[arg(long = "repo-root", help = "Path to repository root (default: cwd)")]
        repo_root: Option<PathBuf>,
    },

    #[command(about = "Documentation standards management")]
    Standards {
        #[command(subcommand)]
        action: StandardsAction,
    },

    #[command(about = "Knowledge System management")]
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },
}

#[derive(Subcommand)]
pub enum StandardsAction {
    #[command(about = "List all registered standards")]
    List,

    #[command(about = "Show details of a specific standard")]
    Show {
        #[arg(help = "Domain name (e.g. 'architecture')")]
        domain: String,

        #[arg(short, long, help = "Version (default: 1.0.0)")]
        version: Option<String>,
    },

    #[command(about = "Show the human-readable documentation-standards spec content for a domain")]
    ShowDoc {
        #[arg(help = "Domain name (e.g. 'architecture')")]
        domain: String,
    },

    #[command(about = "Switch which registered system is the default (used when a repo's samgraha.toml doesn't specify [repository.documentation] standard_system)")]
    SetDefault {
        #[arg(help = "System name to make default")]
        system: String,
    },

    #[command(about = "Remove a standard from the local registry")]
    Remove {
        #[arg(help = "Standard domain (e.g. 'architecture')")]
        domain: String,
    },
}

#[derive(Subcommand)]
pub enum KnowledgeAction {
    #[command(about = "Publish Knowledge System(s) (writes locally first, then pushes to global). \
                        With no --path, discovers every system under [knowledge].root via \
                        KnowledgeSystemLoader and publishes all of them; --path publishes one \
                        knowledge-hub directory manually instead.")]
    Publish {
        #[arg(long = "path", help = "Path to a single knowledge-hub directory to publish manually, bypassing discovery under [knowledge].root")]
        path: Option<PathBuf>,

        #[arg(long, help = "With --path: system name to publish (default: samgraha-documentation). Without --path: limit discovery to this one system id.")]
        system: Option<String>,

        #[arg(long, help = "Path to a JSON file overriding directory-name keys for a differently-laid-out knowledge-hub directory")]
        layout: Option<PathBuf>,

        #[arg(long, help = "Publish locally only, do not push to global standards.db")]
        no_push: bool,

        #[arg(long = "dry-run", help = "Parse and validate the knowledge-hub directory without writing to the DB")]
        dry_run: bool,
    },

    #[command(about = "Pull Knowledge System + help content from the global store to this repo's local DBs")]
    Pull {
        #[arg(long = "force", help = "Force re-sync even if local copy appears current")]
        force: bool,
    },

    #[command(about = "Show sync status of the local Knowledge System vs global store")]
    Status,
}

#[derive(Subcommand)]
pub enum WorkspaceAction {
    #[command(about = "Initialize a workspace configuration")]
    Init {
        #[arg(help = "Workspace name")]
        name: String,

        #[arg(help = "Repository paths to include")]
        repositories: Vec<String>,

        #[arg(long, help = "Path for workspace root (default: current directory)")]
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
                pipeline,
                provider,
                all,
                gate,
                report,
                inspect_artifact,
                runtime,
                execute,
                dry_run,
                yes,
                list_pipelines,
            } => self.execute_audit(
                domain.as_deref(),
                pipeline.as_deref(),
                provider,
                *all,
                *gate,
                *report,
                *inspect_artifact,
                *runtime,
                *execute,
                *dry_run,
                *yes,
                *list_pipelines,
                &format,
            ),
            Commands::Info { path } => self.execute_info(path.as_ref(), &format),
            Commands::Init { path, force, standard_system, auto_detect, sync } => {
                self.execute_init(path.as_ref(), *force, standard_system.as_deref(), *auto_detect, *sync, &format)
            }
            Commands::Package { output, profile, json } => {
                self.execute_package(output.as_ref(), profile.as_deref(), *json, &format)
            }
            Commands::Registry { action } => self.execute_registry(action, &format),
            Commands::Env { path } => self.execute_env(path.as_ref(), &format),
            Commands::Report { pipeline, session, template, stdout, list_sessions, list_templates } => {
                self.execute_report(pipeline.as_deref(), session.as_deref(), template.as_deref(), *stdout, *list_sessions, *list_templates, &format)
            }
            Commands::Workspace { action } => self.execute_workspace(action, &format),
            Commands::Check { name, repo_root } => self.execute_check(name, repo_root.as_ref()),
            Commands::Standards { action } => self.execute_standards(action, &format),
            Commands::Knowledge { action } => self.execute_knowledge(action, &format),
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
        CompilationService::validate_config(&runtime.context.config, &runtime.standard_registry, domains)?;

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

    #[allow(clippy::too_many_arguments)] // ponytail: mirrors the Audit CLI's own flag surface 1:1, one flag per param
    fn execute_audit(
        &self,
        domain: Option<&str>,
        pipeline: Option<&str>,
        providers: &[String],
        _all: bool,
        gate: Option<f64>,
        report: bool,
        inspect_artifact: bool,
        runtime: bool,
        execute: bool,
        dry_run: bool,
        yes: bool,
        list_pipelines: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;

        // Improvement 10: show which pipelines `--pipeline all` would run for
        // this repo's kind, without running anything.
        if list_pipelines {
            let kind = &config.repository.kind;
            let selected = audit::PipelineFactory::for_kind(kind);
            println!(
                "Repository kind: {:?} — `samgraha audit --pipeline all` would run {} pipeline(s):",
                kind, selected.len()
            );
            for pk in selected {
                println!("  - {}", pk.as_str());
            }
            return Ok(ExitCode::Success);
        }

        // "all" is not a real PipelineKind — it means "run the RepositoryKind-
        // appropriate bundle" (Gap 10: audit::PipelineFactory). Handled as its
        // own path since it dispatches multiple pipelines through different
        // runtime methods (audit() for Doc, run_pipeline() for the rest).
        if pipeline.map(|p| p == "all").unwrap_or(false) {
            if report {
                anyhow::bail!(
                    "--report is not yet supported with --pipeline all — run individual pipelines with --report instead."
                );
            }
            return self.execute_audit_all(&root, config, gate, format);
        }

        // Determine pipeline kind
        let pipeline_kind = match pipeline {
            Some(name) => match schemas::audit::PipelineKind::from_str(name) {
                Some(k) => k,
                None => anyhow::bail!(
                    "Unknown pipeline '{}'. Valid values: all, doc, build, security, consistency, coverage, architecture, vision, design, readme, prototype, external-context, engineering, feature, feature-technical, feature-design, deterministic-runtime, external-context-ownership, implementation, dependency, help, documentation-structure, knowledge-system",
                    name
                ),
            },
            None => schemas::audit::PipelineKind::Doc,
        };

        if pipeline_kind != schemas::audit::PipelineKind::Doc {
            let non_default_provider = !providers.is_empty() && providers != ["deterministic".to_string()];
            if domain.is_some() || _all || non_default_provider {
                anyhow::bail!(
                    "--provider, --all, and the domain argument only apply to the doc pipeline (default). \
                     '{}' does not accept them.",
                    pipeline_kind.as_str()
                );
            }
        }

        if (execute || dry_run) && pipeline_kind != schemas::audit::PipelineKind::Build {
            anyhow::bail!(
                "--execute and --dry-run only apply to the build pipeline ('--pipeline build'). \
                 '{}' does not accept them.",
                pipeline_kind.as_str()
            );
        }
        if execute && dry_run {
            anyhow::bail!("--execute and --dry-run are mutually exclusive");
        }

        if execute && !dry_run && !yes {
            let contract = config.pipelines.as_ref().and_then(|p| p.build.as_ref());
            if let Some(contract) = contract {
                match contract.resolve(&root) {
                    Ok(resolved) => {
                        eprintln!("About to execute: {}", resolved.command.join(" "));
                        eprintln!("Working directory: {}", resolved.working_directory.display());
                    }
                    Err(e) => anyhow::bail!("Cannot resolve [pipelines.build] contract: {}", e),
                }
            }
            eprint!("Run this command now? [y/N] ");
            use std::io::Write;
            std::io::stderr().flush().ok();
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
                eprintln!("Aborted.");
                return Ok(ExitCode::Success);
            }
        }

        if pipeline_kind == schemas::audit::PipelineKind::Doc {
            // Standard Documentation Audit path
            ensure_compiled(&root, &config)?;
            let runtime = KnowledgeRuntime::new(&root, config)?;

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
        } else {
            // Pipeline audit path (Build, Security, Consistency, Coverage, Dependency)
            let rt = KnowledgeRuntime::new(&root, config)?;
            let pipeline_report = rt.run_pipeline(&pipeline_kind, inspect_artifact, runtime, execute, dry_run)?;

            // Render pipeline report (always print summary to stdout)
            println!("Pipeline: {}", pipeline_kind.as_str());
            println!("Score: {:.1}%", pipeline_report.score);
            if !pipeline_report.categories.is_empty() {
                println!("Categories:");
                for (name, score) in &pipeline_report.categories {
                    println!("  {}: {:.1}%", name, score);
                }
            }
            println!();
            for f in &pipeline_report.findings {
                let sev = match f.severity {
                    schemas::audit::Severity::Error => "ERROR",
                    schemas::audit::Severity::Warning => "WARN ",
                    schemas::audit::Severity::Suggestion => "SUGG ",
                };
                let loc = f.location.as_deref().unwrap_or("");
                println!("  [{}] {} {} — {}", sev, f.check_id, loc, f.message);
            }

            // --report flag: render using per-audit template
            if report {
                let audit_type = pipeline_kind.as_str();
                let rendered = services::reporting::render_report_from_pipeline(
                    audit_type,
                    services::reporting::get_default_template(),
                    &pipeline_report,
                )?;
                let reports_base = resolve_configured_dir(
                    &rt.context.config.report.dir,
                    &root,
                    "docs/raw/reports",
                );
                let latest_dir = reports_base.join(audit_type).join("latest");
                let archive_dir = reports_base.join(audit_type).join("archive");
                std::fs::create_dir_all(&latest_dir)?;
                std::fs::create_dir_all(&archive_dir)?;

                let now = chrono::Local::now();
                let archive_path = archive_dir.join(format!("{}.md", now.format("%Y%m%d-%H%M%S")));
                std::fs::write(&archive_path, &rendered.markdown)?;
                let latest_path = latest_dir.join("report.md");
                std::fs::write(&latest_path, &rendered.markdown)?;

                if rt.context.config.report.json {
                    let json_path = latest_dir.join("report.json");
                    let json_archive = archive_dir.join(format!("{}.json", now.format("%Y%m%d-%H%M%S")));
                    std::fs::write(&json_path, &rendered.json)?;
                    std::fs::write(&json_archive, &rendered.json)?;
                    println!("Report saved: {}", json_path.display());
                }

                println!("Report saved: {}", latest_path.display());
                println!("Archived:     {}", archive_path.display());
            }

            if let Some(min_score) = gate {
                if pipeline_report.score < min_score {
                    eprintln!(
                        "Quality gate failed: score {:.1}% < minimum {:.1}%",
                        pipeline_report.score, min_score
                    );
                    return Ok(ExitCode::AuditFailure);
                }
            }

            Ok(ExitCode::Success)
        }
    }

    /// `--pipeline all`: run the `RepositoryKind`-appropriate pipeline bundle
    /// (Gap 10 fix, docs/errors-list/02-gaps.md) — `audit::PipelineFactory`
    /// selects which pipelines apply, this dispatches each one through the
    /// runtime method it actually needs (`audit()` for Doc, `run_pipeline()`
    /// for the rest) and combines them into one gate/exit-code decision.
    fn execute_audit_all(
        &self,
        root: &PathBuf,
        config: common::config::SamgrahaConfig,
        gate: Option<f64>,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        ensure_compiled(root, &config)?;
        let rt = KnowledgeRuntime::new(root, config)?;
        let kind = rt.context.config.repository.kind.clone();
        let selected = audit::PipelineFactory::for_kind(&kind);

        println!(
            "Repository kind: {:?} — running {} pipeline(s): {}",
            kind,
            selected.len(),
            selected.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", ")
        );
        println!();

        let mut worst_score: f64 = 100.0;
        let mut any_error = false;

        for pk in selected {
            if *pk == schemas::audit::PipelineKind::Doc {
                let audit_report = rt.audit(None, &["deterministic".to_string()], None)?;
                println!("{}", render_audit(&audit_report, format));
                worst_score = worst_score.min(audit_report.score.overall);
                any_error = any_error
                    || audit_report
                        .findings
                        .iter()
                        .any(|f| matches!(f.severity, schemas::audit::Severity::Error));
            } else {
                let pipeline_report = rt.run_pipeline(pk, false, false, false, false)?;
                println!("Pipeline: {}", pk.as_str());
                println!("Score: {:.1}%", pipeline_report.score);
                if !pipeline_report.categories.is_empty() {
                    println!("Categories:");
                    for (name, score) in &pipeline_report.categories {
                        println!("  {}: {:.1}%", name, score);
                    }
                }
                for f in &pipeline_report.findings {
                    let sev = match f.severity {
                        schemas::audit::Severity::Error => "ERROR",
                        schemas::audit::Severity::Warning => "WARN ",
                        schemas::audit::Severity::Suggestion => "SUGG ",
                    };
                    let loc = f.location.as_deref().unwrap_or("");
                    println!("  [{}] {} {} — {}", sev, f.check_id, loc, f.message);
                }
                println!();
                worst_score = worst_score.min(pipeline_report.score);
                any_error = any_error
                    || pipeline_report
                        .findings
                        .iter()
                        .any(|f| matches!(f.severity, schemas::audit::Severity::Error));
            }
        }

        if let Some(min_score) = gate {
            if worst_score < min_score {
                eprintln!(
                    "Quality gate failed: worst pipeline score {:.1}% < minimum {:.1}%",
                    worst_score, min_score
                );
                return Ok(ExitCode::AuditFailure);
            }
        }

        Ok(if any_error { ExitCode::AuditFailure } else { ExitCode::Success })
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

    fn execute_report(
        &self,
        pipeline: Option<&str>,
        _session: Option<&str>,
        _template_name: Option<&str>,
        stdout_only: bool,
        list_sessions: bool,
        list_templates: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;

        // List templates (doesn't need a compiled repo)
        if list_templates {
            let templates_dir = root.join("docs/raw/report-templates");
            let names = services::reporting::list_templates(&templates_dir)?;
            if names.is_empty() {
                println!("No templates found in {:?}", templates_dir);
                return Ok(ExitCode::Success);
            }
            println!("Available templates:");
            for name in &names {
                println!("  {}.md", name);
            }
            return Ok(ExitCode::Success);
        }

        let runtime = KnowledgeRuntime::new(&root, config)?;
        let audit_type = pipeline.unwrap_or("build");

        // List sessions
        if list_sessions {
            let sessions = runtime.query_sessions_by_type(audit_type, 50)?;
            if sessions.is_empty() {
                println!("No '{}' audit sessions found", audit_type);
                return Ok(ExitCode::Success);
            }
            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&sessions)?);
            } else {
                println!("{:<5} {:<12} {:<8} {:<6} {:<6} {:<6}  {}",
                         "ID", "Session", "Score", "Err", "Warn", "Sug", "Timestamp");
                for s in &sessions {
                    let id = s["id"].as_i64().unwrap_or(0);
                    let sid = s["session_id"].as_str().unwrap_or("");
                    let score = s["score"].as_f64().unwrap_or(0.0);
                    let fc = &s["finding_counts"];
                    let err = fc["errors"].as_i64().unwrap_or(0);
                    let warn = fc["warnings"].as_i64().unwrap_or(0);
                    let sug = fc["suggestions"].as_i64().unwrap_or(0);
                    let ts = s["created_at"].as_str().unwrap_or("");
                    println!("{:<5} {:<12} {:<8.1} {:<6} {:<6} {:<6}  {}",
                             id, sid, score, err, warn, sug, ts);
                }
            }
            return Ok(ExitCode::Success);
        }

        // Render the latest report using per-audit template
        let templates_dir = root.join("docs/raw/report-templates");
        let rendered = services::reporting::render_report(audit_type, &templates_dir, &runtime.registry)?;

        if stdout_only {
            println!("{}", rendered.markdown);
            return Ok(ExitCode::Success);
        }

        // Write to disk
        let reports_base = root.join("reports");
        let audit_dir = reports_base.join(audit_type);
        let latest_dir = audit_dir.join("latest");
        let archive_dir = audit_dir.join("archive");
        std::fs::create_dir_all(&latest_dir)?;
        std::fs::create_dir_all(&archive_dir)?;

        let now = chrono::Local::now();
        let archive_path = archive_dir.join(format!("{}.md", now.format("%Y%m%d-%H%M%S")));
        std::fs::write(&archive_path, &rendered.markdown)?;
        let latest_path = latest_dir.join("report.md");
        std::fs::write(&latest_path, &rendered.markdown)?;

        if runtime.context.config.report.json {
            let json_path = latest_dir.join("report.json");
            let json_archive = archive_dir.join(format!("{}.json", now.format("%Y%m%d-%H%M%S")));
            std::fs::write(&json_path, &rendered.json)?;
            std::fs::write(&json_archive, &rendered.json)?;
            println!("Report saved: {}", json_path.display());
        }

        println!("Report saved: {}", latest_path.display());
        println!("Archived:     {}", archive_path.display());

        Ok(ExitCode::Success)
    }

    fn execute_init(
        &self,
        path: Option<&PathBuf>,
        force: bool,
        standard_system: Option<&str>,
        auto_detect: bool,
        sync: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path
            .cloned()
            .unwrap_or_else(|| std::env::current_dir().unwrap());
        let options = common::config::InitOptions {
            force,
            standard_system: standard_system.map(|s| s.to_string()),
            auto_detect_dirs: auto_detect,
            sync_knowledge_system: sync,
            ..Default::default()
        };
        let result = services::init::init_repository(&root, &options)?;

        println!("{}", result.status);

        if let Some(ref sync) = result.sync_result {
            println!(
                "Standards synced: {}",
                if sync.standards_synced { "yes" } else { "no (source not found)" }
            );
            println!("Help documents synced: {}", sync.help_documents_synced);
            println!("Scripts synced: {}", sync.scripts_synced);
        }

        println!("Generated {}", result.env_path.display());
        println!("{}", format_output(&result.config, format));
        Ok(ExitCode::Success)
    }

    fn execute_env(&self, path: Option<&PathBuf>, _format: &OutputFormat) -> Result<ExitCode> {
        let root = path
            .cloned()
            .unwrap_or_else(|| crate::config::discover_repository_root().unwrap());
        let env_path = services::init::write_env_example(&root)?;
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

    fn execute_standards(&self, action: &StandardsAction, format: &OutputFormat) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        match action {
            StandardsAction::List => {
                let standards = runtime.standard_registry.all();
                let items: Vec<serde_json::Value> = standards
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "name": s.name,
                            "version": s.version,
                            "domain": s.domain,
                            "description": s.description,
                            "rules_count": s.audit_rules.len(),
                            "sections_count": s.required_sections.len(),
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    format_output(
                        &serde_json::json!({ "standards": items, "total": items.len() }),
                        format,
                    )
                );
            }
            StandardsAction::Show { domain, version } => {
                let ver = version.as_deref().unwrap_or("1.0.0");
                let std = runtime.standard_registry
                    .get(domain, ver)
                    .ok_or_else(|| anyhow::anyhow!("Standard '{}/{}' not found", domain, ver))?;
                println!(
                    "{}",
                    format_output(&serde_json::to_value(std)?, format)
                );
            }
            StandardsAction::ShowDoc { domain } => {
                let doc = runtime.standard_registry
                    .get_standard_doc(domain)
                    .ok_or_else(|| anyhow::anyhow!("No standard doc for domain '{}'", domain))?;
                println!("{}", format_output(&serde_json::to_value(doc)?, format));
            }
            StandardsAction::SetDefault { system } => {
                let db_path = common::env::mcp_dir().join("standards.db");
                if !db_path.exists() {
                    anyhow::bail!("No standards.db at {} — register a system first", db_path.display());
                }
                let conn = rusqlite::Connection::open(&db_path)?;
                let exists: bool = conn.query_row(
                    "SELECT 1 FROM systems WHERE name = ?",
                    [system],
                    |_| Ok(true),
                ).unwrap_or(false);
                if !exists {
                    anyhow::bail!("System '{}' not found in {}", system, db_path.display());
                }
                conn.execute("UPDATE systems SET is_default = 0 WHERE is_default = 1", [])?;
                conn.execute("UPDATE systems SET is_default = 1 WHERE name = ?", [system])?;
                println!("'{}' is now the default system.", system);
            }
            StandardsAction::Remove { domain } => {
                let db_path = root.join(".samgraha").join("standards.db");
                if db_path.exists() {
                    let conn = rusqlite::Connection::open(&db_path)?;
                    conn.execute("PRAGMA foreign_keys = ON", [])?;
                    let rows = conn.execute("DELETE FROM domains WHERE key = ?", [domain.clone()])?;
                    if rows > 0 {
                        println!("Removed standard domain '{}'", domain);
                    } else {
                        println!("Standard domain '{}' not found", domain);
                    }
                } else {
                    println!("No standards.db found");
                }
            }
                    }

        Ok(ExitCode::Success)
    }

    
    fn execute_knowledge(&self, action: &KnowledgeAction, _format: &OutputFormat) -> Result<ExitCode> {
        let root = crate::config::discover_repository_root()?;
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        match action {
            KnowledgeAction::Publish { path, system, layout, no_push, dry_run } => {
                // Repository Matrix (Gap 14): Publish Knowledge is ❌ for a
                // plain Repository — only a Knowledge Repository produces
                // Knowledge Systems. `knowledge pull` (Sync Knowledge) stays
                // unrestricted, it's ✅ for both kinds.
                if runtime.context.config.repository.kind != common::config::RepositoryKind::Knowledge {
                    anyhow::bail!(
                        "'knowledge publish' is not available for a plain Repository — only a \
                         Knowledge Repository (`[repository] kind = \"knowledge\"` in samgraha.toml) \
                         can publish Knowledge Systems."
                    );
                }
                let local_db = root.join(".samgraha").join("standards.db");
                if let Some(parent) = local_db.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let loader = services::knowledge_publish::resolve_knowledge_hub_loader()?;

                match path {
                    // Explicit --path: manual single-directory publish, unchanged
                    // from before — the escape hatch for a knowledge-hub
                    // directory that isn't (yet) under this repo's [knowledge].root.
                    Some(path) => {
                        if !path.exists() {
                            anyhow::bail!("Path does not exist: {}", path.display());
                        }
                        println!("Publishing Knowledge System from {} into {}...", path.display(), local_db.display());
                        let out = services::knowledge_publish::run_knowledge_hub_loader(&loader, &local_db, path, system.as_deref(), layout.as_deref(), *dry_run)?;
                        print!("{}", out);
                    }
                    // No --path: discover every system under [knowledge].root
                    // (the same KnowledgeSystemLoader `compile_knowledge` uses)
                    // and publish each of them into the same local standards.db.
                    None => {
                        let systems_dir = root.join(&runtime.context.config.knowledge.root);
                        let discovered = compiler::KnowledgeSystemLoader::load_systems(&systems_dir)?;
                        if discovered.is_empty() {
                            anyhow::bail!(
                                "No Knowledge Systems found under '{}'. Each system must be a subdirectory \
                                 containing a system.toml file, or pass --path to publish one directory manually.",
                                systems_dir.display()
                            );
                        }
                        let selected: Vec<_> = match system {
                            Some(id) => {
                                let matched: Vec<_> = discovered.iter().filter(|s| &s.identity.id == id).collect();
                                if matched.is_empty() {
                                    anyhow::bail!(
                                        "No discovered Knowledge System matches --system '{}'. Discovered: {}",
                                        id,
                                        discovered.iter().map(|s| s.identity.id.as_str()).collect::<Vec<_>>().join(", ")
                                    );
                                }
                                matched
                            }
                            None => discovered.iter().collect(),
                        };
                        println!(
                            "Publishing {} discovered Knowledge System(s) from {} into {}...",
                            selected.len(), systems_dir.display(), local_db.display()
                        );
                        for sys in &selected {
                            for w in &sys.warnings {
                                println!("  [WARN] {}", w.message);
                            }
                            println!("  → {} (v{}) from {}", sys.identity.id, sys.identity.version, sys.path.display());
                            let out = services::knowledge_publish::run_knowledge_hub_loader(&loader, &local_db, &sys.path, Some(&sys.identity.id), layout.as_deref(), *dry_run)?;
                            print!("{}", out);
                        }
                    }
                }

                if *dry_run {
                    println!("Dry run complete — nothing written.");
                } else {
                    println!("Knowledge System(s) published locally.");
                    if !*no_push {
                        let global_db = common::env::mcp_dir().join("standards.db");
                        if let Some(parent) = global_db.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        {
                            let check_conn = rusqlite::Connection::open(&local_db)?;
                            let ok: String = check_conn.query_row(
                                "PRAGMA integrity_check", [], |row| row.get(0)
                            )?;
                            if ok != "ok" {
                                anyhow::bail!("Local standards DB failed integrity check: {}", ok);
                            }
                            standards::check_schema_version(&check_conn)?;
                        }
                        std::fs::copy(&local_db, &global_db)?;
                        println!("Pushed to {}", global_db.display());
                    }
                }
            }
            KnowledgeAction::Pull { force } => {
                handle_knowledge_action(&root, KnowledgeAction::Pull { force: *force })?
            }
            KnowledgeAction::Status => {
                handle_knowledge_action(&root, KnowledgeAction::Status)?
            }
        }
        Ok(ExitCode::Success)
    }

    fn execute_check(&self, name: &str, repo_root: Option<&PathBuf>) -> Result<ExitCode> {
        let cwd = std::env::current_dir()?;
        let root = repo_root.unwrap_or(&cwd);

        let config = crate::config::load_config(Some(&root.join("samgraha.toml")))?;

        match audit::check_runner::resolve_check(name, root, Some(&config)) {
            Some(source) => {
                let result = audit::check_runner::execute_check(&source, root, name);
                println!("{}", serde_json::to_string_pretty(&result)?);
                match result.status {
                    audit::check_runner::CheckStatus::Pass => Ok(ExitCode::Success),
                    _ => Ok(ExitCode::AuditFailure),
                }
            }
            None => {
                eprintln!("No implementation found for check '{}'", name);
                Ok(ExitCode::InputError)
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

/// Handle a Knowledge pull or status action. Extracted from
/// `execute_knowledge` so tests can exercise the logic without
/// constructing the full `Commands` enum.
fn handle_knowledge_action(
    root: &std::path::Path,
    action: KnowledgeAction,
) -> Result<()> {
    match action {
        KnowledgeAction::Pull { force } => {
            match services::init::sync_if_stale(root, force)? {
                None => return Ok(()),
                Some(result) => {
                    println!(
                        "Standards synced: {}",
                        if result.standards_synced { "yes" } else { "no (source standards.db not found in global store)" }
                    );
                    println!("Help documents synced: {}", result.help_documents_synced);
                    println!("Scripts synced: {}", result.scripts_synced);

                    let meta_path = root.join(".samgraha").join("sync-meta.json");
                    if let Ok(meta) = std::fs::read_to_string(&meta_path)
                        .and_then(|s| serde_json::from_str::<services::init::SyncMeta>(&s).map_err(|e| e.into()))
                    {
                        println!("System: {} v{}", meta.system, meta.version);
                    }
                    println!("Knowledge System pulled successfully.");
                }
            }
        }
        KnowledgeAction::Status => {
            let meta_path = root.join(".samgraha").join("sync-meta.json");
            if !meta_path.exists() {
                println!("Never synced — run `samgraha knowledge pull` to sync from global.");
                return Ok(());
            }
            let meta: services::init::SyncMeta = serde_json::from_str(
                &std::fs::read_to_string(&meta_path)?,
            )?;
            println!("Knowledge System: {}", meta.system);
            println!("Version:          {}", meta.version);
            println!("Last sync:        {}", meta.synced_at);

            match services::init::check_knowledge_staleness(root)? {
                services::init::StalenessStatus::UpToDate { .. } => {
                    println!("Status:           Up to date")
                }
                services::init::StalenessStatus::Stale { local_version, global_version } => {
                    println!(
                        "Status:           STALE (local v{}, global v{})",
                        local_version, global_version
                    );
                    println!("                  Run `samgraha knowledge pull` to update.");
                }
                services::init::StalenessStatus::SourceMissing => {
                    println!("Status:           Source DB missing")
                }
                _ => println!("Status:           Unknown"),
            }
        }
        _ => unreachable!("only Pull and Status dispatched here"),
    }
    Ok(())
}

#[cfg(test)]
mod knowledge_action_tests {
    use super::*;

    struct TempEnvGuard {
        key: String,
        old_val: Option<String>,
    }
    impl TempEnvGuard {
        fn new(key: &str, val: &std::path::Path) -> Self {
            let old_val = std::env::var(key).ok();
            std::env::set_var(key, val);
            Self { key: key.to_string(), old_val }
        }
    }
    impl Drop for TempEnvGuard {
        fn drop(&mut self) {
            match &self.old_val {
                Some(v) => std::env::set_var(&self.key, v),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    fn setup_mock_global_store(dir: &std::path::Path, system_name: &str, version: &str) {
        let conn = rusqlite::Connection::open(dir.join("standards.db")).unwrap();
        conn.execute_batch(&format!(
            "PRAGMA user_version = 3;
             CREATE TABLE IF NOT EXISTS systems (
                 id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE,
                 description TEXT, is_default INTEGER NOT NULL DEFAULT 1
             );
             CREATE TABLE IF NOT EXISTS standards (
                 id INTEGER PRIMARY KEY, system_id INTEGER NOT NULL,
                 name TEXT NOT NULL, version TEXT NOT NULL, description TEXT,
                 generation_granularity TEXT NOT NULL DEFAULT 'section',
                 UNIQUE(system_id, name, version)
             );
             DELETE FROM systems;
             DELETE FROM standards;
             INSERT INTO systems (id, name, is_default) VALUES (1, '{system_name}', 1);
             INSERT INTO standards (id, system_id, name, version)
                 VALUES (1, 1, '{system_name}-std', '{version}');",
        ))
        .unwrap();
    }

    fn setup_repo(root: &std::path::Path) {
        std::fs::create_dir_all(root.join(".samgraha")).unwrap();
        std::fs::write(root.join("samgraha.toml"), "[repository]\nid = \"test\"\n").unwrap();
    }

    #[test]
    fn cli_knowledge_lifecycle() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("repo");
        setup_repo(&root);

        let global = tempfile::tempdir().unwrap();
        setup_mock_global_store(global.path(), "my-sys", "1.0.0");

        let _guard = TempEnvGuard::new("SAMGRAHA_MCP_DIR", global.path());

        // 1. First pull
        handle_knowledge_action(&root, KnowledgeAction::Pull { force: false }).unwrap();

        // 2. Verify sync-meta.json
        let meta: services::init::SyncMeta = serde_json::from_str(
            &std::fs::read_to_string(root.join(".samgraha").join("sync-meta.json")).unwrap(),
        ).unwrap();
        assert_eq!(meta.system, "my-sys");
        assert_eq!(meta.version, "1.0.0");

        // 3. Second pull — should skip (up to date)
        handle_knowledge_action(&root, KnowledgeAction::Pull { force: false }).unwrap();

        // 4. Status shows correct info
        handle_knowledge_action(&root, KnowledgeAction::Status).unwrap();

        // 5. Upgrade global
        setup_mock_global_store(global.path(), "my-sys", "2.0.0");

        // 6. Force pull — should re-sync
        handle_knowledge_action(&root, KnowledgeAction::Pull { force: true }).unwrap();

        let meta: services::init::SyncMeta = serde_json::from_str(
            &std::fs::read_to_string(root.join(".samgraha").join("sync-meta.json")).unwrap(),
        ).unwrap();
        assert_eq!(meta.version, "2.0.0");
    }
}
