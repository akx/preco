use crate::cfg::pre_commit_config::PrecommitConfig;
use crate::checkout::LoadedCheckout;
use crate::file_set::get_file_set;
use crate::run_hook::configured_hook::{configure_hook, ConfiguredHook};
use crate::run_hook::RunHookCtx;
use crate::{checkout, run_hook};

use crate::file_matching::get_matching_files;
use crate::regex_cache::get_regex_with_warning;
use anyhow::{bail, Context, Result};
use checkout::get_checkout;
use clap::Args;
use regex::Regex;
use run_hook::RunHookResult;
use rustc_hash::FxHashSet;
use serde_yaml::from_reader;
use std::collections::HashMap;
use std::fs;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, error, info, instrument, warn};

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    #[arg(long)]
    all_files: bool,

    /// Do everything except actually run the hook.
    #[arg(long)]
    dry_run: bool,

    /// Hook ID(s) or alias(es) to run. If unset, everything is run.
    hooks: Option<Vec<String>>,

    #[clap(long, hide = true, conflicts_with = "hooks")]
    git_hook: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub fail_fast: bool,
    pub files_re: Option<Regex>,
    pub exclude_re: Option<Regex>,
}

#[instrument(skip(args))]
pub(crate) fn run(args: &RunArgs) -> Result<ExitCode> {
    let root_path = canonicalize(PathBuf::from("."))?;
    let pre_commit_config_path = root_path.join(".pre-commit-config.yaml");
    let rdr = fs::File::open(&pre_commit_config_path)
        .with_context(|| format!("unable to open {}", pre_commit_config_path.display()))?;
    let precommit_config: PrecommitConfig = from_reader(rdr)
        .with_context(|| format!("could not parse {}", pre_commit_config_path.display()))?;

    let mut selected_hooks = FxHashSet::default();
    if let Some(git_hook) = &args.git_hook {
        selected_hooks.insert(git_hook);
    } else if let Some(hooks) = &args.hooks {
        for hook in hooks {
            selected_hooks.insert(hook);
        }
    }

    let fileset = get_file_set(&root_path, args.all_files)?;
    info!("Running on {} files", fileset.files.len());
    // TODO: should probably apply global exclude + files here!

    let run_config: RunConfig = RunConfig {
        fail_fast: precommit_config.fail_fast,
        exclude_re: get_regex_with_warning(
            precommit_config.exclude.as_deref(),
            "unable to compile regex for `exclude`",
        ),
        files_re: get_regex_with_warning(
            precommit_config.files.as_deref(),
            "unable to compile regex for `files`",
        ),
    };
    let mut checkouts: HashMap<PathBuf, LoadedCheckout> = HashMap::new();
    for repo in &precommit_config.repos {
        let ru = repo.url.to_string();
        let span = tracing::debug_span!("repo", url = ru);
        let _ = span.enter();
        for hook_cfg in &repo.hooks {
            if !selected_hooks.is_empty() {
                let alias = hook_cfg.info.alias.as_ref();
                if !selected_hooks.contains(&hook_cfg.id)
                    && alias.map(|a| selected_hooks.contains(a)).is_none()
                {
                    debug!(
                        "skipping hook {} due to command line configuration",
                        hook_cfg.id
                    );
                    continue;
                }
            }
            let co = get_checkout(repo, hook_cfg)?;
            let loaded_checkout = checkouts.entry(co.path.clone()).or_insert_with(|| {
                debug!("loading checkout {}", co.path.display());
                co.ensure_checkout_cloned().unwrap();
                co.load().unwrap()
            });
            let ConfiguredHook { hook } = configure_hook(loaded_checkout, hook_cfg)?;
            let info = &hook_cfg.info;

            if info.verbose {
                warn!("verbose hooks not implemented");
            }
            if info.log_file.is_some() {
                warn!(
                    "log_file not implemented, not honoring {}",
                    info.log_file.as_ref().unwrap()
                );
            }
            if info.language_version.is_some() {
                warn!(
                    "language_version not implemented, not honoring {}",
                    info.language_version.as_ref().unwrap()
                );
            }
            if hook.always_run {
                warn!("always_run not implemented");
            }
            let matching_files = get_matching_files(&run_config, &fileset, &hook)?;

            if matching_files.is_empty() {
                warn!("hook {} skipped: no matching files", hook_cfg.id);
                continue;
            }

            let rhc = RunHookCtx {
                run_config: &run_config,
                loaded_checkout,
                hook: &hook,
                info,
                files: &matching_files,
                dry_run: args.dry_run,
            };

            // TODO: track changes to files before/after runs
            match run_hook::run_hook(&rhc)? {
                RunHookResult::Success => {}
                RunHookResult::Failure => {
                    error!("hook {} failed", hook_cfg.id);
                    if run_config.fail_fast {
                        bail!("fail-fast enabled, stopping");
                    }
                }
                RunHookResult::Skipped(reason) => {
                    warn!("hook {} skipped: {}", hook_cfg.id, reason);
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS) // TODO: this should return a proper exit code if there were failures or changes
}
