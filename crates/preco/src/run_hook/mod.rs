use crate::cfg::pre_commit_config::HookConfigurationInfo;
use tracing::{instrument, warn};

use crate::cfg::pre_commit_hooks::{HookDefinition, Language, LanguageOrUnknown};
use crate::checkout::LoadedCheckout;
use crate::commands::run::RunConfig;
use crate::file_matching::MatchingFiles;

pub(crate) mod configured_hook;
mod helpers;
mod node;
mod python;

#[derive(Debug)]
pub struct RunHookCtx<'a> {
    pub run_config: &'a RunConfig,
    pub loaded_checkout: &'a LoadedCheckout,
    pub hook: &'a HookDefinition,
    pub info: &'a HookConfigurationInfo,
    pub files: &'a MatchingFiles,
}

pub enum RunHookResult {
    Success,
    Failure,
    Skipped(String),
}

#[instrument(skip(rhc), fields(hook_id=rhc.hook.id))]
pub fn run_hook(rhc: &RunHookCtx) -> anyhow::Result<RunHookResult> {
    match &rhc.hook.language {
        LanguageOrUnknown::Language(lang) => match lang {
            Language::Node => node::run_node_hook(rhc),
            Language::Python => python::run_python_hook(rhc),
        },
        LanguageOrUnknown::Unknown(lang) => Ok(RunHookResult::Skipped(format!(
            "Unsupported language: {:?}",
            lang
        ))),
    }
}
