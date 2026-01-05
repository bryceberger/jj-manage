use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use ignore::{DirEntry, WalkState};

#[derive(Debug)]
pub struct Repo<'a> {
    pub forge: &'a OsStr,
    pub user: &'a OsStr,
    pub repo: &'a OsStr,
}

impl<'a> Repo<'a> {
    pub fn from_path(base: &Path, full_path: &'a Path) -> Option<Self> {
        let mut it = full_path.strip_prefix(base).ok()?.iter();
        let forge = it.next()?;
        let user = it.next()?;
        let repo = it.next_back()?;
        Some(Self { forge, user, repo })
    }
}

pub fn list(base: &Path) -> Vec<PathBuf> {
    let (tx, rx) = std::sync::mpsc::channel();

    ignore::WalkBuilder::new(base)
        .hidden(true)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |ent| {
                let (jj_base, ret) = has_jj_dir(ent);
                if let Some(jj_base) = jj_base
                    && tx.send(jj_base).is_err()
                {
                    return WalkState::Quit;
                }
                ret
            })
        });
    drop(tx);

    rx.into_iter().collect()
}

fn has_jj_dir(ent: Result<DirEntry, ignore::Error>) -> (Option<PathBuf>, WalkState) {
    let Ok(ent) = ent else {
        return (None, WalkState::Skip);
    };

    if !ent.file_type().is_some_and(|t| t.is_dir()) {
        return (None, WalkState::Skip);
    }
    let path = ent.path();

    let Ok(mut ents) = std::fs::read_dir(path) else {
        return (None, WalkState::Skip);
    };
    if ents.any(|e| {
        e.as_ref()
            .is_ok_and(|e| e.file_name() == ".jj" && e.file_type().is_ok_and(|t| t.is_dir()))
    }) {
        (Some(ent.into_path()), WalkState::Skip)
    } else {
        (None, WalkState::Continue)
    }
}
