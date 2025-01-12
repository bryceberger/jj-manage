use clap::Parser;
use color_eyre::eyre::Result;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, prelude::*};

mod config;
mod forge;
mod get;
mod list;

#[derive(clap::Parser)]
enum Commands {
    Get(get::Args),
    List,
    Config,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish()
        .with(ErrorLayer::default())
        .init();

    let config = config::Config::realize(config::Config::default_layers()?)?;

    match Commands::parse() {
        Commands::Get(args) => get::run(&config, args),
        Commands::List => list::run(&config),
        Commands::Config => {
            println!("{}", toml::to_string_pretty(&config).unwrap());
            Ok(())
        }
    }
}
