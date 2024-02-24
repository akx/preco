use anyhow::{bail, Context};
use clap::Args;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tracing::{debug, info, instrument, warn};

const PRECO_HOOK_MARKER: &str = "preco-piispis-1";
const HOOKS_TO_INSTALL: &[&str] = &["pre-commit"];

#[derive(Args, Debug, Clone)]
pub struct InstallArgs {
    #[arg(long, short)]
    force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct UninstallArgs {}

#[instrument(skip(args))]
pub(crate) fn install(args: &InstallArgs) -> anyhow::Result<ExitCode> {
    if cfg!(windows) {
        bail!("installing hooks is not supported on Windows (yet)");
    }
    let git_hooks_dir = get_git_hooks_dir()?;
    for hook in HOOKS_TO_INSTALL {
        install_hook(&git_hooks_dir, hook, args.force).with_context(|| {
            format!(
                "failed to install hook {} in {}",
                hook,
                git_hooks_dir.display()
            )
        })?;
    }
    Ok(ExitCode::SUCCESS)
}

#[instrument(skip(_args))]
pub(crate) fn uninstall(_args: &UninstallArgs) -> anyhow::Result<ExitCode> {
    let git_hooks_dir = get_git_hooks_dir()?;
    for hook in HOOKS_TO_INSTALL {
        remove_hook(&git_hooks_dir, hook)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn get_git_hooks_dir() -> anyhow::Result<PathBuf> {
    PathBuf::from(".git/hooks")
        .canonicalize()
        .with_context(|| "failed to find git hooks directory; are you in a git repository's root?")
}

fn install_hook(git_hooks_dir: &Path, hook: &str, force: bool) -> anyhow::Result<()> {
    let hook_path = git_hooks_dir.join(hook);
    if is_preco_installed_hook(&hook_path)? == IsPrecoInstalledHookResult::IsNotPrecoHook {
        if force {
            warn!(
                "overwriting non-preco hook {} since --force'd",
                hook_path.display()
            );
            std::fs::remove_file(&hook_path)?;
        } else {
            bail!(
                "hook {} already exists and --force is not set",
                hook_path.display()
            );
        }
    }
    let hook_contents = format!(
        "#!/bin/sh\n# {}\nexec {} run --git-hook={}\n",
        PRECO_HOOK_MARKER,
        std::env::current_exe()?.display(),
        shell_words::quote(hook),
    );
    std::fs::write(&hook_path, hook_contents)?;
    if cfg!(unix) {
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o744))?;
    }
    info!("installed hook {}", hook_path.display());
    Ok(())
}

#[derive(PartialEq)]
enum IsPrecoInstalledHookResult {
    IsPrecoHook,
    IsNotPrecoHook,
    NotThere,
}

fn is_preco_installed_hook(hook_path: &Path) -> anyhow::Result<IsPrecoInstalledHookResult> {
    if !hook_path.is_file() {
        return Ok(IsPrecoInstalledHookResult::NotThere);
    }
    match std::fs::read_to_string(hook_path)?.contains(PRECO_HOOK_MARKER) {
        true => Ok(IsPrecoInstalledHookResult::IsPrecoHook),
        false => Ok(IsPrecoInstalledHookResult::IsNotPrecoHook),
    }
}

fn remove_hook(git_hooks_dir: &Path, hook: &str) -> anyhow::Result<()> {
    let hook_path = git_hooks_dir.join(hook);
    match is_preco_installed_hook(&hook_path)? {
        IsPrecoInstalledHookResult::IsPrecoHook => {
            info!("removing hook {}", hook_path.display());
            std::fs::remove_file(&hook_path)?;
        }
        IsPrecoInstalledHookResult::NotThere => {
            debug!("hook {} is not installed", hook_path.display());
        }
        IsPrecoInstalledHookResult::IsNotPrecoHook => {
            debug!("hook {} is not a preco hook", hook_path.display());
        }
    }
    Ok(())
}
