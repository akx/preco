use crate::cfg::pre_commit_config::Hook;
use crate::cfg::pre_commit_hooks::PrecommitHook;
use crate::commands::run::RunConfig;
use crate::file_set::FileSet;
use crate::helpers::append_args;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{instrument, warn};

pub fn get_command(cfg_hook: &Hook, hook_def: &PrecommitHook) -> anyhow::Result<String> {
    let args = match &cfg_hook.args {
        Some(args) => {
            if !hook_def.args.is_empty() {
                warn!(
                    "hook def args = {:?}, user config args = {:?}; using user config args",
                    hook_def.args, cfg_hook.args,
                );
            }
            args
        }
        None => &hook_def.args,
    };
    let command = append_args(&hook_def.entry, args)?;
    Ok(command)
}

#[instrument(skip(run_config, fileset, cfg_hook, hook_def))]
pub fn get_matching_files(
    run_config: &RunConfig,
    fileset: &FileSet,
    cfg_hook: &Hook,
    hook_def: &PrecommitHook,
) -> anyhow::Result<HashSet<Rc<PathBuf>>> {
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
    if cfg_hook.exclude.is_some() {
        warn!(
            "not implemented: exclude in user configuration; not honoring {}",
            cfg_hook.exclude.as_ref().unwrap()
        );
    }
    if hook_def.files.is_some() {
        warn!(
            "not implemented: files in hook configuration; not honoring {}",
            hook_def.files.as_ref().unwrap()
        );
    }
    let mut matching_files = HashSet::new();
    for file in fileset.files.iter() {
        if let Some(types) = &hook_def.types {
            if !types.is_empty() && types.iter().all(|t| fileset.has_type(file, t)) {
                matching_files.insert(Rc::clone(file));
                // TODO: break here if inserted
            }
        }
        if let Some(types_or) = &hook_def.types_or {
            if !types_or.is_empty() && types_or.iter().any(|t| fileset.has_type(file, t)) {
                matching_files.insert(Rc::clone(file));
                // TODO: break here if inserted
            }
        }
    }
    Ok(matching_files)
}
