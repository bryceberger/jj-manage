use std::{
    io::{Write, stdout},
    path::Path,
};

use color_eyre::{
    Section,
    eyre::{Result, eyre},
};

use crate::{config::Config, repos::RepoIter};

/// Print the path to a repo given its name
///
/// If there is exactly one match, print it. Otherwise, return an error.
///
/// If the name contains no path separators, it is interpreted as a repository,
/// then a username.
#[derive(clap::Parser)]
pub struct Args {
    /// Print absolute paths
    #[arg(short, long)]
    long: bool,
    target: String,
}

pub fn run(config: &Config, args: Args) -> Result<()> {
    let base = config.base()?;

    let target = match args.target.rsplit_once('/') {
        Some((b, a)) => Target::NameAndRepo(b, a),
        None => Target::Name(&args.target),
    };

    let repos: Vec<_> = RepoIter::new(&base)
        .filter_map(|r| {
            let short = r.strip_prefix(&base).ok()?;
            if matches(&target, short)? {
                Some(if args.long { r } else { short.to_owned() })
            } else {
                None
            }
        })
        .collect();

    match &repos[..] {
        [] => Err(eyre!("No repositories matched")),

        [single_match] => {
            let mut stdout = stdout().lock();
            stdout.write_all(single_match.as_os_str().as_encoded_bytes())?;
            stdout.write_all(b"\n")?;
            Ok(())
        }

        multiple => {
            let multiple = multiple
                .iter()
                .fold(String::from("matched:"), |mut acc, p| {
                    acc.push_str("\n   ");
                    acc.push_str(&p.as_os_str().to_string_lossy());
                    acc
                });
            Err(eyre!("Multiple repositories matched").with_note(|| multiple))
        }
    }
}

enum Target<'a> {
    Name(&'a str),
    NameAndRepo(&'a str, &'a str),
}

fn matches(target: &Target, repo: &Path) -> Option<bool> {
    let mut components = repo.components();

    let repo_name = components.next_back()?;

    Some(match *target {
        Target::Name(n) => n == repo_name.as_os_str() || components.any(|c| n == c.as_os_str()),
        Target::NameAndRepo(n, r) => {
            let p = components.as_path();
            r == repo_name.as_os_str()
                && (n == p.as_os_str() || components.any(|c| n == c.as_os_str()))
        }
    })
}
