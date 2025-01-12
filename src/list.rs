use std::{
    io::{Write, stdout},
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use tracing::instrument;

use crate::config::Config;

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

pub struct RepoIter {
    it: walkdir::IntoIter,
}

impl RepoIter {
    pub fn new(base: impl AsRef<Path>) -> Self {
        Self {
            it: walkdir::WalkDir::new(base).into_iter(),
        }
    }
}

impl Iterator for RepoIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ent = self.it.next()?;
            let Ok(ent) = ent else {
                continue;
            };

            if ent.file_type().is_dir() && ent.file_name() == ".jj" {
                self.it.skip_current_dir();
                let mut path = ent.into_path();
                if path.pop() {
                    return Some(path);
                }
            }
        }
    }
}
