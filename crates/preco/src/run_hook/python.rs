use crate::helpers::prepend_to_path_envvar;
use crate::run_hook::{helpers, RunHookCtx, RunHookResult};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, instrument, trace_span, warn};

pub(crate) fn run_python_hook(rhc: &RunHookCtx) -> Result<RunHookResult> {
    let RunHookCtx {
        cfg_hook,
        hook_def,
        loaded_checkout,
        fileset: _,
    } = *rhc;
    let checkout_path = &loaded_checkout.path;
    let venv_path = checkout_path.join(".preco-venv");
    if !venv_path.exists() {
        setup_venv(rhc, &venv_path)?;
    }
    if hook_def.pass_filenames {
        warn!("not implemented: pass_filenames=true; WILL WORK AS IF IT WERE FALSE!");
    }
    let venv_bin_path = venv_path.join("bin"); // TODO: windows
    let path_with_venv = prepend_to_path_envvar(&venv_bin_path.to_string_lossy())?;
    let command = helpers::get_command(&cfg_hook, &hook_def)?;

    let run_span = trace_span!("run command", command = command);
    let _enter = run_span.enter();
    let status = std::process::Command::new("sh") // TODO: windows
        .env_remove("PYTHONHOME")
        .env("VIRTUAL_ENV", &venv_path)
        .env("PATH", path_with_venv)
        .arg("-c") // TODO: windows
        .arg(command)
        .status()?;
    Ok(if status.success() {
        RunHookResult::Success
    } else {
        RunHookResult::Failure
    })
}

#[instrument(skip(rhc))]
fn setup_venv(rhc: &RunHookCtx, venv_path: &PathBuf) -> Result<()> {
    let RunHookCtx {
        loaded_checkout,
        hook_def,
        cfg_hook,
        ..
    } = *rhc;
    let checkout_path = &loaded_checkout.path;
    debug!("creating venv in {} with `uv`", venv_path.to_string_lossy());
    // TODO: doesn't support python version specification right now
    std::process::Command::new("uv")
        .arg("venv")
        .arg(&venv_path)
        .status()?;
    debug!(
        "installing dependencies in {} with `uv`",
        venv_path.to_string_lossy()
    );
    if !hook_def.additional_dependencies.is_empty() {
        warn!(
            "not implemented: additional_dependencies in hook definition: {:?}",
            hook_def.additional_dependencies
        );
    }
    if let Some(additional_dependencies) = &cfg_hook.additional_dependencies {
        warn!(
            "not implemented: additional_dependencies in user configuration: {:?}",
            additional_dependencies
        );
    }
    std::process::Command::new("uv")
        .env("VIRTUAL_ENV", &venv_path)
        .arg("pip")
        .arg("install")
        .arg("-e")
        .arg(checkout_path)
        .status()?;
    Ok(())
}
