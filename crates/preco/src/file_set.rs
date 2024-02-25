use crate::git::files::{get_git_tracked_files, get_staged_files, get_unstaged_files};
use anyhow::bail;
use git2::Repository;
use identify::mappings::{map_extension, map_name};
use rustc_hash::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tracing::{instrument};

#[derive(Debug)]
pub(crate) struct FileSet {
    pub(crate) root_path: PathBuf,
    pub(crate) files: Vec<Rc<PathBuf>>,
    #[allow(dead_code)]
    pub(crate) files_by_type: FxHashMap<String, Vec<Rc<PathBuf>>>,
    pub(crate) types_by_file: FxHashMap<Rc<PathBuf>, FxHashSet<String>>,
}

impl FileSet {
    pub(crate) fn has_type(&self, file: &Rc<PathBuf>, typename: &str) -> bool {
        self.types_by_file
            .get(file)
            .map(|types| types.contains(typename))
            .unwrap_or(false)
    }

    pub(crate) fn from_raw_files(root_path: &Path, files: Vec<PathBuf>) -> anyhow::Result<FileSet> {
        let files: Vec<Rc<PathBuf>> = files.into_iter().map(Rc::new).collect();
        let mut files_by_type: FxHashMap<String, Vec<Rc<PathBuf>>> = FxHashMap::default();
        files_by_type.reserve(files.len());
        let mut types_by_file: FxHashMap<Rc<PathBuf>, FxHashSet<String>> = FxHashMap::default();
        for file in &files {
            if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
                if let Some((n, types)) = map_extension(ext) {
                    for typename in &types[0..(*n)] {
                        files_by_type
                            .entry(typename.to_string())
                            .or_default()
                            .push(Rc::clone(file));
                        types_by_file
                            .entry(Rc::clone(file))
                            .or_default()
                            .insert(typename.to_string());
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
                        types_by_file
                            .entry(Rc::clone(file))
                            .or_default()
                            .insert(typename.to_string());
                    }
                }
            }
        }
        Ok(FileSet {
            root_path: root_path.to_path_buf(),
            files,
            files_by_type,
            types_by_file,
        })
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
    FileSet::from_raw_files(root_path, files_raw)
}
