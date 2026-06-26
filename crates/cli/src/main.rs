use anyhow::Result;
use clap::Parser;
use cli::commands::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("samgraha=info".parse().unwrap()),
        )
        .with_target(false)
        .try_init();

    let exit_code = cli.execute()?;
    std::process::exit(exit_code.code());
}
