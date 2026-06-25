use std::path::PathBuf;
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use schemas::compilation::{CompilationRequest, CompilationScope};
use schemas::search::{SearchQuery, RetrievalLevel};

use schemas::config::SamgrahaConfig;
use runtime::KnowledgeRuntime;
use crate::output::{OutputFormat, format_output};

#[derive(Parser)]
#[command(name = "samgraha", version = "0.1.0", about = "Knowledge Engineering Platform")]
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
    },

    #[command(about = "Search compiled knowledge")]
    Search {
        query: String,

        #[arg(long = "domain", help = "Filter by domain")]
        domain: Option<String>,

        #[arg(long = "level", help = "Retrieval level (metadata, summary, section, full)", default_value = "metadata")]
        level: String,

        #[arg(long = "max", help = "Maximum results", default_value = "20")]
        max: usize,
    },

    #[command(about = "Run audit checks")]
    Audit {
        #[arg(help = "Domain to audit (default: all)")]
        domain: Option<String>,

        #[arg(long = "provider", help = "Audit provider(s)", default_value = "deterministic")]
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
}

impl Cli {
    pub fn execute(&self) -> Result<ExitCode> {
        let format = if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Text
        };

        match &self.command {
            Commands::Compile { path, domain, force, watch } => {
                self.execute_compile(path.as_ref(), domain, *force, *watch, &format)
            }
            Commands::Search { query, domain, level, max } => {
                self.execute_search(query, domain.as_deref(), level, *max, &format)
            }
            Commands::Audit { domain, provider, all, gate } => {
                self.execute_audit(domain.as_deref(), provider, *all, *gate, &format)
            }
            Commands::Info { path } => {
                self.execute_info(path.as_ref(), &format)
            }
            Commands::Init { path, force } => {
                self.execute_init(path.as_ref(), *force, &format)
            }
        }
    }

    fn execute_compile(
        &self,
        path: Option<&PathBuf>,
        domains: &[String],
        force: bool,
        watch: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path.cloned().unwrap_or_else(|| std::env::current_dir().unwrap());
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let scope = if domains.is_empty() {
            CompilationScope::Repository
        } else {
            CompilationScope::Domains(domains.to_vec())
        };

        let request = CompilationRequest { scope, force, watch };
        let result = runtime.compile(&request)?;

        println!("{}", format_output(&result, format));

        if result.success {
            Ok(ExitCode::Success)
        } else {
            Ok(ExitCode::CompilationError)
        }
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
        println!("{}", format_output(&results, format));

        if results.results.is_empty() {
            eprintln!("No results found");
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
            audit::DeterministicAuditProvider::execute(docs, rules)
        });

        let audit_domain = if _all { None } else { domain };
        let provider_names: Vec<String> = if providers.is_empty() {
            vec!["deterministic".to_string()]
        } else {
            providers.to_vec()
        };

        let report = runtime.audit(audit_domain, &provider_names, None)?;
        println!("{}", format_output(&report, format));

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

    fn execute_info(
        &self,
        path: Option<&PathBuf>,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path.cloned().unwrap_or_else(|| {
            crate::config::discover_repository_root().unwrap_or_else(|_| std::env::current_dir().unwrap())
        });
        let config = crate::config::load_config(self.config.as_ref())?;
        let runtime = KnowledgeRuntime::new(&root, config)?;

        let info = runtime.info();
        println!("{}", format_output(&info, format));
        Ok(ExitCode::Success)
    }

    fn execute_init(
        &self,
        path: Option<&PathBuf>,
        force: bool,
        format: &OutputFormat,
    ) -> Result<ExitCode> {
        let root = path.cloned().unwrap_or_else(|| std::env::current_dir().unwrap());
        let config_path = root.join("samgraha.toml");

        if config_path.exists() && !force {
            anyhow::bail!(
                "Configuration already exists at {}. Use --force to overwrite.",
                config_path.display()
            );
        }

        let config = SamgrahaConfig::default();
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(&config_path, content)
            .context(format!("Failed to write config to {}", config_path.display()))?;

        println!("Initialized samgraha configuration at {}", config_path.display());
        println!("{}", format_output(&config, format));
        Ok(ExitCode::Success)
    }
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
