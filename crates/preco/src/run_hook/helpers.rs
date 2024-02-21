use crate::cfg::pre_commit_hooks::HookDefinition;
use crate::helpers::append_args;

pub fn get_command(hook: &HookDefinition) -> anyhow::Result<String> {
    append_args(&hook.entry, &hook.args)
}
