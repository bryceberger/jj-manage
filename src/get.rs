use std::{borrow::Cow, path::Path, process::Command};

use color_eyre::eyre::{Result, bail, eyre};
use tracing::instrument;

use crate::{config::Config, forge::Forge};

/// Clone a repository
#[derive(clap::Parser)]
pub struct Args {
    /// Force cloning with http. Overrides the config value `get.clone-kind`.
    #[arg(long, conflicts_with = "ssh")]
    https: bool,
    /// Force cloning with ssh. Overrides the config value `get.clone-kind`.
    #[arg(long, conflicts_with = "https")]
    ssh: bool,

    #[arg(short, long)]
    forge: Option<String>,
    repo: String,
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
    let forge = args.forge.as_deref().unwrap_or(&config.default_forge);
    let forge = Forge::named(config, forge).ok_or_else(|| eyre!("unknown forge: {forge}"))?;

    let repo = get_repo(config, &args.repo);
    let remote = get_remote(config, &args, &forge.info.url, &repo);
    let mut target = config.base()?;
    target.push(forge.name);
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
    tracing::debug!(%status, ?command);
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

fn get_remote(config: &Config, args: &Args, url: &str, path: &str) -> String {
    let ssh = || format!("git@{url}:{path}");
    let https = || format!("{url}/{path}");
    match (args.https, args.ssh, &config.get.clone_kind) {
        (true, _, _) => https(),
        (_, true, _) => ssh(),
        (_, _, CloneKind::Https) => https(),
        (_, _, CloneKind::Ssh) => ssh(),
    }
}
