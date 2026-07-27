use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::discover_repository_root;
use crate::output::{format_output, OutputFormat};
use common::config::{InitOptions, SamgrahaConfig};

#[derive(Parser)]
#[command(
    name = "samgraha",
    version = env!("CARGO_PKG_VERSION"),
    about = "MCP execution substrate for knowledge standards"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, long = "json", help = "Output in JSON format")]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize samgraha.toml + .samgraha/ in the current repository")]
    Init {
        #[arg(help = "Path to initialize (default: current directory)")]
        path: Option<PathBuf>,

        #[arg(long = "force", help = "Overwrite existing samgraha.toml")]
        force: bool,

        #[arg(long = "standard", help = "Standard system name to declare")]
        standard: Option<String>,

        #[arg(long = "auto-detect", help = "Probe for docs/src/tests/scripts directories")]
        auto_detect: bool,
    },

    #[command(about = "Register a knowledge standard's standard.yaml into knowledge.db")]
    RegisterStandard {
        #[arg(help = "Path to the standard's source root (containing standard.yaml)")]
        path: PathBuf,

        #[arg(long = "knowledge-db", help = "Target knowledge.db path (default: ./.samgraha/knowledge.db)")]
        knowledge_db: Option<PathBuf>,
    },
}

impl Cli {
    pub fn execute(&self) -> Result<i32> {
        let format = if self.json { OutputFormat::Json } else { OutputFormat::Text };

        match &self.command {
            Commands::Init { path, force, standard, auto_detect } => {
                let root = path.clone().unwrap_or(std::env::current_dir()?);
                let options = InitOptions {
                    force: *force,
                    standard_system: standard.clone(),
                    auto_detect_dirs: *auto_detect,
                    ..Default::default()
                };
                let result = services::init_repository(&root, &options)?;
                println!("{}", result.status);
                println!("{}", format_output(&result.config, &format));
                Ok(0)
            }
            Commands::RegisterStandard { path, knowledge_db } => {
                let db_path = match knowledge_db {
                    Some(p) => p.clone(),
                    None => {
                        let root = discover_repository_root()?;
                        let config = load_config_or_default(&root);
                        config.repository.resolve_samgraha_dir(&root).join("knowledge.db")
                    }
                };
                let result = services::register_standard::register_standard(path, &db_path, None)?;
                println!("{}", format_output(&result, &format));
                Ok(0)
            }
        }
    }
}

/// Load `samgraha.toml` from `root`, or return the default config. Used by
/// CLI commands that need to resolve `samgraha_dir` without failing on a
/// missing config.
fn load_config_or_default(root: &std::path::Path) -> SamgrahaConfig {
    let config_path = root.join("samgraha.toml");
    std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default()
}
