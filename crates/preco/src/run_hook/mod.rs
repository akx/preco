use crate::cfg::pre_commit_config::HookConfigurationInfo;
use tracing::warn;

use crate::cfg::pre_commit_hooks::{HookDefinition, Language, LanguageOrUnknown};
use crate::checkout::LoadedCheckout;
use crate::commands::run::RunConfig;
use crate::file_set::FileSet;

pub(crate) mod configured_hook;
mod helpers;
mod python;

#[derive(Debug)]
pub struct RunHookCtx<'a> {
    pub run_config: &'a RunConfig,
    pub loaded_checkout: &'a LoadedCheckout,
    pub hook: &'a HookDefinition,
    pub info: &'a HookConfigurationInfo,
    pub fileset: &'a FileSet,
}

pub enum RunHookResult {
    Success,
    Failure,
    Skipped(String),
}

pub fn run_hook(rhc: &RunHookCtx) -> anyhow::Result<RunHookResult> {
    match &rhc.hook.language {
        LanguageOrUnknown::Language(lang) => match lang {
            Language::Python => python::run_python_hook(rhc),
        },
        LanguageOrUnknown::Unknown(lang) => {
            warn!("Unsupported language: {:?}", lang);
            Ok(RunHookResult::Skipped(format!(
                "Unsupported language: {:?}",
                lang
            )))
        }
    }
}
