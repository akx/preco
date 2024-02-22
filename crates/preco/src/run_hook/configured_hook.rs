use crate::cfg::pre_commit_config::{HookConfiguration, HookDefinitionOverrides};
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

    Ok(ConfiguredHook {
        hook: merged_hook_def,
    })
}
