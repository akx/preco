use crate::file_matching::MatchingFiles;
use crate::helpers::{hash_additional_dependencies, prepend_to_path_envvar};
use crate::run_hook::{helpers, RunHookCtx, RunHookResult};
use anyhow::Result;

use tracing::{debug, instrument, trace_span};

const NODE_MODULES_DIR_NAME: &str = "node_modules_preco";

pub(crate) fn run_node_hook(rhc: &RunHookCtx) -> Result<RunHookResult> {
    let RunHookCtx {
        files: mf,
        hook,
        info: _,
        loaded_checkout,
        run_config: _,
    } = *rhc;
    let MatchingFiles { files, root_path } = mf;
    let checkout_path = &loaded_checkout.path;
    let node_modules_name = &get_node_modules_name(rhc);
    let node_modules_path = checkout_path.join(node_modules_name);
    if !node_modules_path.exists() {
        setup_node_env(rhc, node_modules_name)?;
    }
    // TODO: for some reason pnpm still installs scripts in `node_modules/.bin`, not in the module-path
    let node_modules_bin_path = loaded_checkout.path.join("node_modules").join(".bin");
    let path_with_node_modules_bin =
        prepend_to_path_envvar(&node_modules_bin_path.to_string_lossy())?;
    let mut command = helpers::get_command(hook)?;
    // TODO: will probably need to slice `command` in a `xargs` way to avoid coming up
    //       with a command that's too long
    if hook.pass_filenames {
        command = format!(
            "{} {}",
            command,
            shell_words::join(files.iter().map(|f| f.to_string_lossy()))
        );
    }

    let run_span = trace_span!("run command", command = command);
    let _enter = run_span.enter();
    let status = std::process::Command::new("sh") // TODO: windows
        .env("NODE_PATH", node_modules_path)
        .env("PATH", path_with_node_modules_bin)
        .arg("-c") // TODO: windows
        .arg(command)
        .current_dir(root_path)
        .status()?;
    Ok(if status.success() {
        RunHookResult::Success
    } else {
        RunHookResult::Failure
    })
}

fn get_node_modules_name(rhc: &RunHookCtx) -> String {
    // Ensure each set of additional dependencies gets its own
    // node_modules (see python.rs for more details).
    let addl_deps = &rhc.hook.additional_dependencies;
    if !addl_deps.is_empty() {
        format!(
            "{}-{}",
            NODE_MODULES_DIR_NAME,
            hash_additional_dependencies(addl_deps)
        )
    } else {
        NODE_MODULES_DIR_NAME.to_string()
    }
}

#[instrument(skip(rhc))]
fn setup_node_env(rhc: &RunHookCtx, node_modules_name: &str) -> Result<()> {
    let RunHookCtx {
        loaded_checkout,
        hook,
        ..
    } = *rhc;
    let checkout_path = &loaded_checkout.path;
    debug!("installing deps with `pnpm`");
    // TODO: doesn't support node version specification right now

    std::process::Command::new("pnpm")
        .env("NPM_UPDATE_NOTIFIER", "false")
        .arg("i")
        .arg("--modules-dir")
        .arg(node_modules_name)
        .current_dir(checkout_path)
        .status()?;
    if !hook.additional_dependencies.is_empty() {
        debug!("adding additional deps with `pnpm`");
        std::process::Command::new("pnpm")
            .env("NPM_UPDATE_NOTIFIER", "false")
            .arg("add")
            .arg("--modules-dir")
            .arg(node_modules_name)
            .args(&hook.additional_dependencies)
            .current_dir(checkout_path)
            .status()?;
    }
    Ok(())
}
