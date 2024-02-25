use std::collections::BTreeSet;
use std::fs;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, bail, Context, Result};
use clap::Args;
use regex::Regex;
use serde_yaml::from_reader;
use tracing::{debug, error, info, instrument, warn};

use crate::cfg::pre_commit_config::PrecommitConfig;
use crate::cfg::pre_commit_config::{HookConfiguration, HookDefinitionOverrides};
use crate::cfg::pre_commit_hooks::HookDefinition;
use crate::checkout::get_checkout;
use crate::checkout::LoadedCheckout;
use crate::file_matching::{get_matching_files, MatchingFiles};
use crate::file_set::get_file_set;
use crate::regex_cache::get_regex_with_warning;
use crate::run_hook::{run_hook, RunHookCtx, RunHookResult};

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

struct ConfiguredHook<'a> {
    hook_cfg: &'a HookConfiguration,
    loaded_checkout: LoadedCheckout,
    hook: HookDefinition,
    matching_files: MatchingFiles,
}

#[instrument(skip(args))]
pub(crate) fn run(args: &RunArgs) -> Result<ExitCode> {
    let root_path = canonicalize(PathBuf::from("."))?;
    let pre_commit_config_path = root_path.join(".pre-commit-config.yaml");
    let rdr = fs::File::open(&pre_commit_config_path)
        .with_context(|| format!("unable to open {}", pre_commit_config_path.display()))?;
    let precommit_config: PrecommitConfig = from_reader(rdr)
        .with_context(|| format!("could not parse {}", pre_commit_config_path.display()))?;

    let mut selected_hooks = BTreeSet::default();
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
    let mut configured_hooks = Vec::new();
    // TODO: parallelize the below loop
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
            co.ensure_checkout_cloned()?;
            let loaded_checkout = co.load()?;
            let hook = merge_hook_definition(&loaded_checkout, hook_cfg)?;

            if hook.always_run {
                warn!("always_run not implemented");
            }
            let matching_files = get_matching_files(&run_config, &fileset, &hook)?;

            if matching_files.is_empty() {
                warn!("hook {} skipped: no matching files", hook_cfg.id);
                continue;
            }

            configured_hooks.push(ConfiguredHook {
                hook_cfg,
                loaded_checkout,
                hook,
                matching_files,
            });
        }
    }

    if configured_hooks.is_empty() {
        info!("Nothing to run!");
        return Ok(ExitCode::SUCCESS);
    }

    for ConfiguredHook {
        hook,
        hook_cfg,
        loaded_checkout,
        matching_files,
    } in configured_hooks
    {
        let info = &hook_cfg.info;

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

        let rhc = RunHookCtx {
            run_config: &run_config,
            loaded_checkout: &loaded_checkout,
            hook: &hook,
            info,
            files: &matching_files,
            dry_run: args.dry_run,
        };

        info!(
            "Running {} on {} files...",
            hook.name,
            matching_files.files.len()
        );

        // TODO: track changes to files before/after runs
        match run_hook(&rhc)? {
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
    Ok(ExitCode::SUCCESS) // TODO: this should return a proper exit code if there were failures or changes
}

pub(crate) fn merge_hook_definition(
    co: &LoadedCheckout,
    hc: &HookConfiguration,
) -> Result<HookDefinition> {
    let d = co
        .hooks
        .iter()
        .find(|h| h.id == hc.id)
        .ok_or_else(|| anyhow!("hook {} not found in checkout {}", hc.id, co.path.display()))?;
    let HookConfiguration {
        id,
        info: _,
        overrides,
    } = &hc;

    let HookDefinitionOverrides {
        name,
        description,
        files,
        exclude,
        types,
        types_or,
        exclude_types,
        additional_dependencies,
        args,
        stages,
        always_run,
    } = &overrides;

    if exclude_types.is_some() {
        warn!(
            "not implemented: exclude_types; not honoring {:?}",
            exclude_types
        );
    }

    let merged_hook_def = HookDefinition {
        id: id.clone(),
        name: name.clone().unwrap_or_else(|| d.name.clone()),
        description: description.clone().unwrap_or_else(|| d.description.clone()),
        entry: d.entry.clone(),
        args: args.clone().unwrap_or_else(|| d.args.clone()),
        language: d.language.clone(),
        stages: stages.clone().or_else(|| d.stages.clone()),
        types: types.clone().or_else(|| d.types.clone()),
        types_or: types_or.clone().or_else(|| d.types_or.clone()),
        files: files.clone().or_else(|| d.files.clone()),
        exclude: exclude.clone().or_else(|| d.exclude.clone()),
        pass_filenames: d.pass_filenames,
        always_run: always_run.unwrap_or(d.always_run),
        require_serial: d.require_serial,
        additional_dependencies: additional_dependencies
            .clone()
            .unwrap_or_else(|| d.additional_dependencies.clone()),
        minimum_pre_commit_version: d.minimum_pre_commit_version.clone(),
    };

    Ok(merged_hook_def)
}
