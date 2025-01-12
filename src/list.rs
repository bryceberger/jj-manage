use std::io::{Write, stdout};

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::{config::Config, repos::RepoIter};

#[instrument(skip_all)]
pub fn run(config: &Config) -> Result<()> {
    let base = config.base()?;

    let repos: Vec<_> = RepoIter::new(base).collect();

    let mut stdout = stdout().lock();
    for r in &repos {
        stdout.write_all(r.as_os_str().as_encoded_bytes())?;
        stdout.write_all(b"\n")?;
    }

    Ok(())
}
