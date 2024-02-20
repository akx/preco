use git2::Repository;
use std::path::{Path, PathBuf};
use tracing::{instrument, warn};

#[instrument]
pub(crate) fn get_git_tracked_files(working_copy_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let repo = Repository::open(working_copy_path)?;
    let mut tracked_files = Vec::new();
    for entry in repo.index()?.iter() {
        tracked_files.push(PathBuf::from(String::from_utf8(entry.path.to_vec())?));
    }
    Ok(tracked_files)
}

#[instrument]
pub(crate) fn get_changed_files(working_copy_path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let repo = Repository::open(working_copy_path)?;
    let mut changed_files = Vec::new();
    for status in repo.statuses(None)?.iter() {
        if status.status() != git2::Status::CURRENT {
            if let Some(path) = status.path() {
                changed_files.push(PathBuf::from(path));
            } else {
                warn!("unable to decode path {:?}", status.path_bytes());
            }
        }
    }
    Ok(changed_files)
}
