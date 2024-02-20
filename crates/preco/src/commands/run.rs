use crate::cfg::pre_commit_config::PrecommitConfig;
use crate::checkout::LoadedCheckout;
use crate::file_set::get_file_set;
use crate::run_hook::RunHookCtx;
use crate::{checkout, run_hook};
use anyhow::{bail, Result};
use checkout::get_checkout;
use clap::Args;
use run_hook::RunHookResult;
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
}

#[instrument]
pub(crate) fn run(args: &RunArgs) -> Result<ExitCode> {
    let root_path = canonicalize(PathBuf::from("."))?;
    let rdr = fs::File::open(root_path.join(".pre-commit-config.yaml")).or_else(|_| {
        bail!(
            "no .pre-commit-config.yaml found in {}",
            root_path.display()
        )
    })?;

    let fileset = get_file_set(&root_path, args.all_files)?;
    info!("fileset: {} files", fileset.files.len());

    let cfg: PrecommitConfig = from_reader(rdr)?;
    let mut checkouts: HashMap<PathBuf, LoadedCheckout> = HashMap::new();
    for repo in &cfg.repos {
        let ru = repo.url.to_string();
        let span = tracing::debug_span!("repo", url = ru);
        let _ = span.enter();
        for cfg_hook in &repo.hooks {
            let co = get_checkout(repo, cfg_hook)?;
            let loaded_checkout = checkouts.entry(co.path.clone()).or_insert_with(|| {
                debug!("loading checkout {}", co.path.display());
                co.ensure_checkout_cloned().unwrap();
                co.load().unwrap()
            });
            let maybe_hook_def = loaded_checkout.hooks.iter().find(|h| h.id == cfg_hook.id);
            if let Some(hook_def) = maybe_hook_def {
                let rhc = RunHookCtx {
                    loaded_checkout,
                    hook_def,
                    cfg_hook,
                    fileset: &fileset,
                };
                match run_hook::run_hook(&rhc)? {
                    RunHookResult::Success => {}
                    RunHookResult::Failure => {
                        error!("hook {} failed", cfg_hook.id);
                    }
                    RunHookResult::Skipped(reason) => {
                        warn!("hook {} skipped: {}", cfg_hook.id, reason);
                    }
                }
            } else {
                error!("hook {} not found", cfg_hook.id);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
