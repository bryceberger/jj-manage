use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

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
        let repo = it.last()?;
        Some(Self { forge, user, repo })
    }
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
