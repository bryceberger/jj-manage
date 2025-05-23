use std::{
    io::{Write, stdout},
    path::Path,
};

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::{config::Config, repos};

/// List paths to managed repositories
#[derive(clap::Parser)]
pub struct Args {
    /// Print absolute paths
    #[arg(short, long)]
    long: bool,
}

#[instrument(skip_all)]
pub fn run(config: &Config, args: Args) -> Result<()> {
    let base = config.base()?;

    let repos = repos::list(&base);

    let mut stdout = stdout().lock();
    for r in &repos {
        let mut r: &Path = r;
        if !args.long {
            r = r.strip_prefix(&base).unwrap_or(r)
        };
        stdout.write_all(r.as_os_str().as_encoded_bytes())?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}
