use std::{
    ffi::OsString,
    io::Write,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::BytesMut;
use color_eyre::eyre::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{DisableLineWrap, EnableLineWrap},
};
use futures::{FutureExt, StreamExt};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
};
use tracing::instrument;

use crate::{
    config::Config,
    repos::{self, Repo},
};

/// Run `jj git fetch` in all repositories.
///
/// If any of forge, user, or repo are passed, only matching repositories are
/// updated. Multiple filters of the same level will be ORed together, while
/// separate levels will be ANDed.
#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    forge: Vec<OsString>,
    #[arg(short, long)]
    user: Vec<OsString>,
    #[arg(short, long)]
    repo: Vec<OsString>,
}

#[instrument(skip_all)]
pub async fn run(config: &Config, args: Args) -> Result<()> {
    let base = config.base()?;
    let repos = repos::list(&base).into_iter().filter_map(|p| {
        let r = Repo::from_path(&base, &p)?;
        if !{
            contains_or_empty(&args.forge, &r.forge)
                && contains_or_empty(&args.user, &r.user)
                && contains_or_empty(&args.repo, &r.repo)
        } {
            return None;
        }
        let name = format!(
            "{}/{}/{}",
            r.forge.to_string_lossy(),
            r.user.to_string_lossy(),
            r.repo.to_string_lossy()
        );
        Some((p, name))
    });

    Preschool::from_repos(repos).run().await;

    Ok(())
}

fn contains_or_empty<T, U>(vals: &[T], v: &U) -> bool
where
    T: PartialEq<U>,
{
    vals.is_empty() || vals.iter().any(|i| i == v)
}

struct Preschool {
    children: Vec<(String, Child)>,
}

impl Preschool {
    fn from_repos(repos: impl Iterator<Item = (PathBuf, String)>) -> Self {
        let children = repos.flat_map(|(r, name)| {
            let mut command = Command::new("jj");
            command
                .arg("-R")
                .arg(&r)
                .args(["git", "fetch", "--color=always"]);
            tracing::debug!(?command);
            let child = command
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .inspect_err(|e| tracing::warn!(%e, "while spawning child for update:"))
                .ok()?;

            Some((name, child))
        });
        Self {
            children: children.collect(),
        }
    }

    async fn run(self) {
        let state = Arc::new(Mutex::new(State {
            should_print_newline: false,
            running_children: self.children.iter().map(|(r, ..)| r.clone()).collect(),
        }));

        let mut futures = futures::stream::FuturesUnordered::new();
        for (r, c) in self.children {
            futures.push(run_child(Arc::clone(&state), r, c).boxed());
        }
        futures.push(printer(Arc::clone(&state)).boxed());
        while let Some(()) = futures.next().await {}
    }
}

struct State {
    should_print_newline: bool,
    running_children: std::collections::BTreeSet<String>,
}

async fn printer(state: Arc<Mutex<State>>) {
    let mut chars = ['/', '-', '\\', '|'].iter().cycle();

    let _ = std::io::stdout().execute(DisableLineWrap);

    loop {
        {
            let mut state = state.lock().unwrap();
            if state.running_children.is_empty() {
                break;
            }

            state.should_print_newline = true;
            print!("\r{} updating: ", chars.next().unwrap());

            for (idx, ch) in state.running_children.iter().enumerate() {
                let last = idx == state.running_children.len() - 1;
                let end = if last { "" } else { ", " };
                print!("{ch}{end}");
            }

            std::io::stdout().flush().ok();
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    let _ = std::io::stdout().execute(EnableLineWrap);
}

async fn run_child(state: Arc<Mutex<State>>, repo: String, mut child: Child) {
    let state = || state.lock().unwrap();
    let newline = || {
        if std::mem::replace(&mut state().should_print_newline, false) {
            print!("\r\x1b[K");
        }
    };
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut buf_out = bytes::BytesMut::with_capacity(64);
    let mut buf_err = bytes::BytesMut::with_capacity(64);
    loop {
        tokio::select! {
            _ = stdout.read_buf(&mut buf_out) => {
                newline();
                show_lines(&repo, &mut buf_out);
            }
            _ = stderr.read_buf(&mut buf_err) => {
                newline();
                show_lines(&repo, &mut buf_err);
            }
            status = child.wait() => {
                state().running_children.remove(&repo);
                tracing::debug!(%repo, ?status);
                return;
            }
        }
    }
}

fn show_lines(repo: &str, data: &mut BytesMut) {
    if data.is_empty() {
        return;
    }

    let last_nl = data
        .iter()
        .rev()
        .position(|ch| *ch == b'\n')
        .unwrap_or(data.len());
    let last_nl_pos = data.len() - last_nl;
    let ending_nl = data[..last_nl_pos]
        .iter()
        .rev()
        .take_while(|ch| **ch == b'\n')
        .count();
    let last = last_nl_pos - ending_nl;
    if last != 0 {
        for line in data[..last].split(|ch| *ch == b'\n') {
            let line = String::from_utf8_lossy(line);
            println!("\x1b[2m{repo}>\x1b[0m {line}");
        }
    }

    let last = if ending_nl == 0 { last } else { last + 1 };
    data.copy_within(last.., 0);
    data.truncate(last_nl);
}
