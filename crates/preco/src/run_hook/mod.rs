use crate::cfg::pre_commit_config::Hook;
use crate::cfg::pre_commit_hooks::{Language, LanguageOrUnknown, PrecommitHook};
use crate::checkout::LoadedCheckout;
use crate::commands::run::RunConfig;
use crate::file_set::FileSet;
use tracing::warn;

mod helpers;
mod python;

#[derive(Debug)]
pub struct RunHookCtx<'a> {
    pub run_config: &'a RunConfig,
    pub loaded_checkout: &'a LoadedCheckout,
    pub hook_def: &'a PrecommitHook,
    pub cfg_hook: &'a Hook,
    pub fileset: &'a FileSet,
}

pub enum RunHookResult {
    Success,
    Failure,
    Skipped(String),
}

pub fn run_hook(rhc: &RunHookCtx) -> anyhow::Result<RunHookResult> {
    match &rhc.hook_def.language {
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
