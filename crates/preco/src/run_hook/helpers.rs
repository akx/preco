use crate::cfg::pre_commit_config::Hook;
use crate::cfg::pre_commit_hooks::PrecommitHook;
use crate::helpers::append_args;
use tracing::warn;

pub fn get_command(cfg_hook: &&Hook, hook_def: &&PrecommitHook) -> anyhow::Result<String> {
    let args = match &cfg_hook.args {
        Some(args) => {
            if !hook_def.args.is_empty() {
                warn!(
                    "hook def args = {:?}, user config args = {:?}; using user config args",
                    hook_def.args, cfg_hook.args,
                );
            }
            args
        }
        None => &hook_def.args,
    };
    let command = append_args(&hook_def.entry, args)?;
    Ok(command)
}
