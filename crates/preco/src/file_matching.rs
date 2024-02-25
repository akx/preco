use crate::cfg::pre_commit_hooks::HookDefinition;
use crate::commands::run::RunConfig;
use crate::file_set::FileSet;
use crate::regex_cache::get_regex_with_warning;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, instrument, warn};

type PathBufSet = BTreeSet<Rc<PathBuf>>;

#[derive(Debug)]
pub(crate) struct MatchingFiles {
    pub(crate) root_path: PathBuf,
    pub(crate) files: PathBufSet,
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
    let run_config_files_re = &run_config.files_re;
    let run_config_exclude_re = &run_config.exclude_re;
    let hook_exclude_re = get_regex_with_warning(
        hook.exclude.as_deref(),
        "unable to compile regex for `exclude`",
    );
    let hook_files_re =
        get_regex_with_warning(hook.files.as_deref(), "unable to compile regex for `files`");
    let mut matching_files = PathBufSet::new();
    for file in fileset.files.iter() {
        if run_config_files_re.is_some()
            || run_config_exclude_re.is_some()
            || hook_exclude_re.is_some()
            || hook_files_re.is_some()
        {
            let file_str = file.to_string_lossy();
            if let Some(run_config_files_re) = run_config_files_re {
                if !run_config_files_re.is_match(&file_str) {
                    debug!(
                        "skipping file: {} does not match run_config.files",
                        file_str
                    );
                    continue;
                }
            }
            if let Some(run_config_exclude_re) = run_config_exclude_re {
                if run_config_exclude_re.is_match(&file_str) {
                    debug!("skipping file: {} matches run_config.exclude", file_str);
                    continue;
                }
            }
            if let Some(hook_exclude_re) = &hook_exclude_re {
                if hook_exclude_re.is_match(&file_str) {
                    debug!("skipping file: {} matches hook.exclude", file_str);
                    continue;
                }
            }
            if let Some(hook_files_re) = &hook_files_re {
                if !hook_files_re.is_match(&file_str) {
                    debug!("skipping file: {} does not match hook.files", file_str);
                    continue;
                }
            }
        }
        let mut matched = false;
        if let Some(types) = &hook.types {
            if !types.is_empty() && types.iter().all(|t| fileset.has_type(file, t)) {
                matching_files.insert(Rc::clone(file));
                matched = true;
            }
        }
        if !matched {
            if let Some(types_or) = &hook.types_or {
                if !types_or.is_empty() && types_or.iter().any(|t| fileset.has_type(file, t)) {
                    matching_files.insert(Rc::clone(file));
                }
            }
        }
    }
    Ok(MatchingFiles {
        files: matching_files,
        root_path: fileset.root_path.clone(),
    })
}
