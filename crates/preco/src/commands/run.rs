use crate::cfg::pre_commit_config::PrecommitConfig;
use crate::checkout::LoadedCheckout;
use crate::run_hook::RunHookCtx;
use crate::{checkout, run_hook};
use anyhow::Result;
use checkout::get_checkout;
use clap::Args;
use run_hook::RunHookResult;
use serde_yaml::from_reader;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{debug, error, instrument, warn};

#[derive(Args, Debug, Clone)]
pub struct RunArgs {
    #[arg(long)]
    all_files: bool,
}

#[instrument]
pub(crate) fn run(args: &RunArgs) -> Result<ExitCode> {
    if !args.all_files {
        anyhow::bail!("not implemented: not --all-files");
    }
    let rdr = fs::File::open(".pre-commit-config.yaml")?;
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
            match loaded_checkout.hooks.iter().find(|h| h.id == cfg_hook.id) {
                Some(hook_def) => {
                    let rhc = RunHookCtx {
                        loaded_checkout,
                        hook_def,
                        cfg_hook,
                    };
                    match run_hook::run_hook(&rhc)? {
                        RunHookResult::Success => {}
                        RunHookResult::Failure => {
                            error!("hook {} failed", cfg_hook.id);
                        }
                        RunHookResult::Skipped => {
                            warn!("hook {} skipped", cfg_hook.id);
                        }
                    }
                }
                None => {
                    error!("hook {} not found", cfg_hook.id);
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
