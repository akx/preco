use crate::run_hook::RunHookResult;
use std::ffi::OsStr;
use std::path::Path;
use std::process::ExitStatus;
use tracing::error;

pub(crate) struct RunCommandOutput<'a> {
    pub(crate) command: &'a str,
    pub(crate) stdout: Option<String>,
    pub(crate) stderr: Option<String>,
    pub(crate) status: ExitStatus,
}

impl RunCommandOutput<'_> {
    pub(crate) fn print_output_if_failed(&self) {
        if self.status.success() {
            return;
        }
        error!("command '{}' returned {}", self.command, self.status);
        if let Some(stdout) = &self.stdout {
            error!("stdout:\n{}", stdout);
        }
        if let Some(stderr) = &self.stderr {
            error!("stderr:\n{}", stderr);
        }
    }
    pub(crate) fn to_run_hook_result(&self) -> RunHookResult {
        if self.status.success() {
            RunHookResult::Success
        } else {
            RunHookResult::Failure
        }
    }
}

pub(crate) fn run_command<'a, K, V>(
    command: &'a str,
    work_dir: &Path,
    env_set: &[(K, V)],
    env_remove: &[K],
    verbose: bool,
) -> anyhow::Result<RunCommandOutput<'a>>
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // TODO: windows (we don't have sh -c there)
    let mut builder = std::process::Command::new("sh");
    builder.arg("-c").arg(command).current_dir(work_dir);
    for (key, value) in env_set {
        builder.env(key, value);
    }
    for key in env_remove {
        builder.env_remove(key);
    }
    if verbose {
        let status = builder.status()?;
        Ok(RunCommandOutput {
            command,
            stdout: None,
            stderr: None,
            status,
        })
    } else {
        let output = builder.output()?;
        Ok(RunCommandOutput {
            command,
            stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
            stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            status: output.status,
        })
    }
}
