use std::{borrow::Cow, path::Path, process::Command};

use color_eyre::eyre::{Result, bail};
use tracing::instrument;

use crate::{config::Config, forge::Forge};

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    forge: Option<String>,
    path: String,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "kebab-case", default)]
pub struct GetConfig {
    pub clone_kind: CloneKind,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CloneKind {
    #[default]
    Ssh,
    Https,
}

#[instrument(skip_all)]
pub fn run(config: &Config, args: Args) -> Result<()> {
    let forge = args.forge.map(Forge::from_str);
    let forge = forge.as_ref().unwrap_or(&config.default_forge);
    let url: &str = &forge.get_info().url;

    let repo = get_repo(config, &args.path);
    let remote = get_remote(config, url, &repo);
    let mut target = config.base()?;
    target.push(forge.name());
    target.push(repo.as_ref());

    tracing::info!(?remote, ?target);

    if std::fs::exists(&target)? {
        bail!("path already exists");
    }

    run_clone(&remote, &target, config.colocate)
}

fn run_clone(remote: &str, target: &Path, colocate: bool) -> Result<()> {
    let mut command = Command::new("jj");
    command.args(["git", "clone", remote]);
    command.arg(target);
    if colocate {
        command.arg("--colocate");
    };

    let status = command.status()?;
    tracing::debug!(?command, %status);
    if !status.success() {
        return Err(color_eyre::eyre::eyre!("clone did not exit successfully"));
    }

    Ok(())
}

fn get_repo<'p>(config: &Config, path: &'p str) -> Cow<'p, str> {
    if path.contains('/') {
        path.into()
    } else {
        format!("{}/{}", config.user, path).into()
    }
}

fn get_remote(config: &Config, url: &str, path: &str) -> String {
    match config.get.clone_kind {
        CloneKind::Ssh => format!("git@{url}:{path}"),
        CloneKind::Https => format!("{url}/{path}"),
    }
}
