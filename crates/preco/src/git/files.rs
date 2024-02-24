use git2::{Deltas, Repository};
use std::path::PathBuf;
use tracing::{instrument, warn};

#[instrument(skip(repo))]
pub(crate) fn get_git_tracked_files(repo: &Repository) -> anyhow::Result<Vec<PathBuf>> {
    let mut tracked_files = Vec::new();
    for entry in repo.index()?.iter() {
        tracked_files.push(PathBuf::from(String::from_utf8(entry.path.to_vec())?));
    }
    Ok(tracked_files)
}

#[instrument(skip(repo))]
pub(crate) fn get_changed_files(repo: &Repository) -> anyhow::Result<Vec<PathBuf>> {
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

#[instrument(skip(repo))]
pub(crate) fn get_unstaged_files(repo: &Repository) -> anyhow::Result<Vec<PathBuf>> {
    let diff = repo.diff_index_to_workdir(None, None)?;
    Ok(get_delta_paths(diff.deltas()))
}

#[instrument(skip(repo))]
pub(crate) fn get_staged_files(repo: &Repository) -> anyhow::Result<Vec<PathBuf>> {
    let head = repo.head().unwrap().peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), None, None)?;
    Ok(get_delta_paths(diff.deltas()))
}

fn get_delta_paths(deltas: Deltas) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for diff in deltas {
        if let Some(path) = diff.new_file().path() {
            paths.push(PathBuf::from(path));
        } else {
            warn!("unable to decode path {:?}", diff.new_file().path_bytes());
        }
    }
    paths
}
