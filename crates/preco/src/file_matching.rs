use crate::cfg::pre_commit_hooks::HookDefinition;
use crate::commands::run::RunConfig;
use crate::file_set::FileSet;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{instrument, warn};

#[derive(Debug)]
pub(crate) struct MatchingFiles {
    pub(crate) root_path: PathBuf,
    pub(crate) files: HashSet<Rc<PathBuf>>,
}

impl MatchingFiles {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

#[instrument(skip(run_config, fileset, hook))]
pub(crate) fn get_matching_files(
    run_config: &RunConfig,
    fileset: &FileSet,
    hook: &HookDefinition,
) -> anyhow::Result<MatchingFiles> {
    if run_config.files.is_some() {
        warn!(
            "not implemented: files in run configuration; not honoring {}",
            run_config.files.as_ref().unwrap()
        );
    }
    if run_config.exclude.is_some() {
        warn!(
            "not implemented: exclude in run configuration; not honoring {}",
            run_config.exclude.as_ref().unwrap()
        );
    }
    if hook.exclude.is_some() {
        warn!(
            "not implemented: exclude in hook configuration; not honoring {}",
            hook.exclude.as_ref().unwrap()
        );
    }
    if hook.files.is_some() {
        warn!(
            "not implemented: files in hook configuration; not honoring {}",
            hook.files.as_ref().unwrap()
        );
    }
    let mut matching_files = HashSet::new();
    for file in fileset.files.iter() {
        if let Some(types) = &hook.types {
            if !types.is_empty() && types.iter().all(|t| fileset.has_type(file, t)) {
                matching_files.insert(Rc::clone(file));
                // TODO: break here if inserted
            }
        }
        if let Some(types_or) = &hook.types_or {
            if !types_or.is_empty() && types_or.iter().any(|t| fileset.has_type(file, t)) {
                matching_files.insert(Rc::clone(file));
                // TODO: break here if inserted
            }
        }
    }
    Ok(MatchingFiles {
        files: matching_files,
        root_path: fileset.root_path.clone(),
    })
}
