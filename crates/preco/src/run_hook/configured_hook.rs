use crate::cfg::pre_commit_config::HookConfiguration;
use crate::cfg::pre_commit_hooks::HookDefinition;
use crate::checkout::LoadedCheckout;
use anyhow::{anyhow, Result};
use tracing::warn;

#[derive(Debug)]
pub(crate) struct ConfiguredHook {
    pub hook: HookDefinition, // Merged from the checkout and the configuration.
}

pub(crate) fn configure_hook(
    co: &LoadedCheckout,
    hc: &HookConfiguration,
) -> Result<ConfiguredHook> {
    let d = co
        .hooks
        .iter()
        .find(|h| h.id == hc.id)
        .ok_or_else(|| anyhow!("hook {} not found in checkout {}", hc.id, co.path.display()))?;
    let HookConfiguration {
        id,
        info: _,
        overrides: o,
    } = &hc;

    if o.exclude_types.is_some() {
        warn!(
            "not implemented: exclude_types; not honoring {:?}",
            o.exclude_types
        );
    }

    let merged_hook_def = HookDefinition {
        id: id.clone(),
        name: o.name.clone().unwrap_or_else(|| d.name.clone()),
        description: o
            .description
            .clone()
            .unwrap_or_else(|| d.description.clone()),
        entry: d.entry.clone(),
        args: o.args.clone().unwrap_or_else(|| d.args.clone()),
        language: d.language.clone(),
        stages: o.stages.clone().or_else(|| d.stages.clone()),
        types: o.types.clone().or_else(|| d.types.clone()),
        types_or: o.types_or.clone().or_else(|| d.types_or.clone()),
        files: o.files.clone().or_else(|| d.files.clone()),
        exclude: o.exclude.clone().or_else(|| d.exclude.clone()),
        pass_filenames: d.pass_filenames,
        always_run: o.always_run.unwrap_or(d.always_run),
        require_serial: d.require_serial,
        additional_dependencies: o
            .additional_dependencies
            .clone()
            .unwrap_or_else(|| d.additional_dependencies.clone()),
        minimum_pre_commit_version: d.minimum_pre_commit_version.clone(),
    };

    Ok(ConfiguredHook {
        hook: merged_hook_def,
    })
}
