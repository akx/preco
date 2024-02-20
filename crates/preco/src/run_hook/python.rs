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
        fileset,
    } = *rhc;
    let matching_files = helpers::get_matching_files(fileset, cfg_hook, hook_def)?;
    if matching_files.is_empty() {
        debug!("no matching files for hook {}", hook_def.id);
        return Ok(RunHookResult::Skipped("no matching files".to_string()));
    }

    let checkout_path = &loaded_checkout.path;
    let venv_path = checkout_path.join(".preco-venv");
    if !venv_path.exists() {
        setup_venv(rhc, &venv_path)?;
    }
    let venv_bin_path = venv_path.join("bin"); // TODO: windows
    let path_with_venv = prepend_to_path_envvar(&venv_bin_path.to_string_lossy())?;
    let mut command = helpers::get_command(cfg_hook, hook_def)?;
    // TODO: will probably need to slice `command` in a `xargs` way to avoid coming up
    //       with a command that's too long
    if hook_def.pass_filenames {
        command = format!(
            "{} {}",
            command,
            shell_words::join(matching_files.iter().map(|f| f.to_string_lossy()))
        );
    }

    let run_span = trace_span!("run command", command = command);
    let _enter = run_span.enter();
    let status = std::process::Command::new("sh") // TODO: windows
        .env_remove("PYTHONHOME")
        .env("VIRTUAL_ENV", &venv_path)
        .env("PATH", path_with_venv)
        .arg("-c") // TODO: windows
        .arg(command)
        .current_dir(&fileset.root_path)
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
        .arg(venv_path)
        .status()?;
    debug!(
        "installing dependencies in {} with `uv`",
        venv_path.to_string_lossy()
    );
    let mut additional_deps = Vec::new();
    additional_deps.extend(hook_def.additional_dependencies.iter());
    if let Some(cfg_addl_deps) = &cfg_hook.additional_dependencies {
        additional_deps.extend(cfg_addl_deps.iter());
    }
    std::process::Command::new("uv")
        .env("VIRTUAL_ENV", venv_path)
        .arg("pip")
        .arg("install")
        .arg("-e") // TODO: see https://github.com/astral-sh/uv/issues/313
        .arg(checkout_path)
        .args(additional_deps)
        .status()?;
    Ok(())
}
