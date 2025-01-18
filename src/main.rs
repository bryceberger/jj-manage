use std::io::Write;

use clap::Parser;
use color_eyre::eyre::Result;
use config::Config;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

mod config;
mod forge;
mod get;
mod list;
mod repos;
mod resolve;
mod update;

#[derive(clap::Parser)]
enum Commands {
    Get(get::Args),
    List(list::Args),
    Resolve(resolve::Args),
    Update(update::Args),
    /// Print the base path
    Base,
    /// Print the resolved config
    Config,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish()
        .with(ErrorLayer::default())
        .init();

    let config = Config::realize(Config::default_layers()?)?;

    run(config)
}

#[tokio::main(flavor = "current_thread")]
async fn run(config: Config) -> Result<()> {
    match Commands::parse() {
        Commands::Get(args) => get::run(&config, args),
        Commands::List(args) => list::run(&config, args),
        Commands::Resolve(args) => resolve::run(&config, args),
        Commands::Update(args) => update::run(&config, args).await,
        Commands::Base => {
            std::io::stdout().write_all(config.base()?.as_os_str().as_encoded_bytes())?;
            println!();
            Ok(())
        }
        Commands::Config => {
            println!("{}", toml::to_string_pretty(&config).unwrap());
            Ok(())
        }
    }
}
