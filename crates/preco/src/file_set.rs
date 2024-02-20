use crate::git::files::get_git_tracked_files;
use anyhow::bail;
use identify::mappings::{map_extension, map_name};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::instrument;

#[derive(Debug)]
pub(crate) struct FileSet {
    pub(crate) root_path: PathBuf,
    pub(crate) files: Vec<Rc<PathBuf>>,
    pub(crate) files_by_type: HashMap<String, Vec<Rc<PathBuf>>>,
}

// TODO: tags such as 'directory', 'symlink', 'socket', 'file', 'executable', 'non_executable', 'text', 'binary'

#[instrument]
pub(crate) fn get_file_set(root_path: &PathBuf, all_files: bool) -> anyhow::Result<FileSet> {
    let files_raw = if all_files {
        get_git_tracked_files(&root_path)?
    } else {
        bail!("not implemented: not --all-files");
    };
    let files: Vec<Rc<PathBuf>> = files_raw.into_iter().map(Rc::new).collect();
    let mut files_by_type: HashMap<String, Vec<Rc<PathBuf>>> = HashMap::new();
    for file in &files {
        if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
            if let Some((n, types)) = map_extension(ext) {
                for typename in &types[0..(*n)] {
                    files_by_type
                        .entry(typename.to_string())
                        .or_insert_with(Vec::new)
                        .push(Rc::clone(file));
                }
            }
        }
        if let Some(name) = file.file_name().and_then(|s| s.to_str()) {
            if let Some((n, types)) = map_name(name) {
                for typename in &types[0..(*n)] {
                    files_by_type
                        .entry(typename.to_string())
                        .or_insert_with(Vec::new)
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
