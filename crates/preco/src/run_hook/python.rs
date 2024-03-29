use crate::commando::run_command_on_files;
use crate::file_matching::MatchingFiles;
use crate::helpers::hash_additional_dependencies;
use crate::helpers::prepend_to_path_envvar;
use crate::run_hook::{helpers, RunHookCtx, RunHookResult};
use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, instrument};

pub(crate) fn run_python_hook(rhc: &RunHookCtx) -> Result<RunHookResult> {
    let RunHookCtx {
        dry_run,
        files: mf,
        hook,
        info,
        loaded_checkout: _,
        run_config: _,
    } = *rhc;
    let MatchingFiles { files, root_path } = mf;
    let venv_path = ensure_venv(rhc)?;
    let venv_bin_path = venv_path.join("bin"); // TODO: windows
    let path_with_venv = prepend_to_path_envvar(&venv_bin_path.to_string_lossy())?;
    let base_command = helpers::get_command(hook)?;
    if dry_run {
        return Ok(RunHookResult::Skipped("dry-run".to_string()));
    }

    let res = run_command_on_files(
        &base_command,
        root_path,
        &[
            ("VIRTUAL_ENV", venv_path),
            ("PATH", PathBuf::from(path_with_venv)),
        ],
        &["PYTHONHOME"],
        info.verbose,
        if hook.pass_filenames {
            Some(files)
        } else {
            None
        },
        hook.require_serial,
    );
    res.print_output_if_failed();
    Ok(res.to_run_hook_result())
}

const VENV_DIR_NAME: &str = ".preco-venv";

fn ensure_venv(rhc: &RunHookCtx) -> Result<PathBuf> {
    let checkout_path = &rhc.loaded_checkout.path;
    let venv_path = checkout_path.join(get_venv_name(rhc));
    if !venv_path.exists() {
        setup_venv(rhc, &venv_path)?;
    }
    Ok(venv_path)
}

fn get_venv_name(rhc: &RunHookCtx) -> String {
    // Ensure each set of additional dependencies gets its own
    // virtualenv too; we could have built the checkout directory
    // name based on add'l deps just from the checkout, but we
    // couldn't have taken into account the add'l deps from the
    // hook configuration at that point.

    // TODO: add support for python version here too
    let addl_deps = &rhc.hook.additional_dependencies;
    if !addl_deps.is_empty() {
        format!(
            "{}-{}",
            VENV_DIR_NAME,
            hash_additional_dependencies(addl_deps)
        )
    } else {
        VENV_DIR_NAME.to_string()
    }
}

#[instrument(skip(rhc))]
fn setup_venv(rhc: &RunHookCtx, venv_path: &PathBuf) -> Result<()> {
    let RunHookCtx {
        loaded_checkout,
        hook,
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
    std::process::Command::new("uv")
        .env("VIRTUAL_ENV", venv_path)
        .arg("pip")
        .arg("install")
        .arg("-e") // TODO: see https://github.com/astral-sh/uv/issues/313
        .arg(checkout_path)
        .args(hook.additional_dependencies.iter())
        .status()?;
    Ok(())
}
