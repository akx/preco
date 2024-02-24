use crate::git::files::{get_git_tracked_files, get_staged_files, get_unstaged_files};
use anyhow::bail;
use git2::Repository;
use identify::mappings::{map_extension, map_name};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, instrument};

#[derive(Debug)]
pub(crate) struct FileSet {
    pub(crate) root_path: PathBuf,
    pub(crate) files: Vec<Rc<PathBuf>>,
    pub(crate) files_by_type: HashMap<String, Vec<Rc<PathBuf>>>,
}

impl FileSet {
    pub(crate) fn has_type(&self, file: &Rc<PathBuf>, typename: &str) -> bool {
        // TODO: this is a probable bottleneck, should probably prebuild
        //       a mapping from file -> types too
        self.files_by_type
            .get(typename)
            .map(|files| files.contains(file))
            .unwrap_or(false)
    }
}

// TODO: tags such as 'directory', 'symlink', 'socket', 'file', 'executable', 'non_executable', 'text', 'binary'

#[instrument]
pub(crate) fn get_file_set(root_path: &PathBuf, all_files: bool) -> anyhow::Result<FileSet> {
    let repo = Repository::open(root_path)?;
    let files_raw = if all_files {
        get_git_tracked_files(&repo)?
    } else {
        let unstaged = get_unstaged_files(&repo)?;
        if !unstaged.is_empty() {
            bail!(
                "We can't deal with stashing unstaged files yet (and you have: {:?})",
                unstaged
            );
        }
        get_staged_files(&repo)?
    };
    debug!("files_raw: {:?}", files_raw);
    let files: Vec<Rc<PathBuf>> = files_raw.into_iter().map(Rc::new).collect();
    let mut files_by_type: HashMap<String, Vec<Rc<PathBuf>>> = HashMap::new();
    for file in &files {
        if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
            if let Some((n, types)) = map_extension(ext) {
                for typename in &types[0..(*n)] {
                    files_by_type
                        .entry(typename.to_string())
                        .or_default()
                        .push(Rc::clone(file));
                }
            }
        }
        if let Some(name) = file.file_name().and_then(|s| s.to_str()) {
            if let Some((n, types)) = map_name(name) {
                for typename in &types[0..(*n)] {
                    files_by_type
                        .entry(typename.to_string())
                        .or_default()
                        .push(Rc::clone(file));
                }
            }
        }
    }
    Ok(FileSet {
        root_path: root_path.clone(),
        files,
        files_by_type,
    })
}
