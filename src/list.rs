use std::{
    io::{Write, stdout},
    path::Path,
};

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::{config::Config, repos::RepoIter};

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    short: bool,
}

#[instrument(skip_all)]
pub fn run(config: &Config, args: Args) -> Result<()> {
    let base = config.base()?;

    let repos: Vec<_> = RepoIter::new(&base).collect();

    let mut stdout = stdout().lock();
    for r in &repos {
        let mut r: &Path = r;
        if args.short {
            r = r.strip_prefix(&base).unwrap_or(r)
        };
        stdout.write_all(r.as_os_str().as_encoded_bytes())?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}
