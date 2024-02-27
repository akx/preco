use crate::file_matching::PathBufSet;
use crate::run_hook::RunHookResult;
use std::borrow::Cow;
use std::ffi::OsStr;

use rayon::prelude::*;
use std::path::Path;
use std::process::ExitStatus;
use std::thread::available_parallelism;
use tracing::{error, trace_span};

#[cfg(unix)]
const MAX_COMMAND_LENGTH: usize = 131_072;
#[cfg(windows)]
const MAX_COMMAND_LENGTH: usize = 8192;

pub(crate) struct RunCommandOutput {
    pub(crate) command: String,
    pub(crate) stdout: Option<String>,
    pub(crate) stderr: Option<String>,
    pub(crate) status: ExitStatus,
}

impl RunCommandOutput {
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
    #[allow(dead_code)]
    pub(crate) fn to_run_hook_result(&self) -> RunHookResult {
        if self.status.success() {
            RunHookResult::Success
        } else {
            RunHookResult::Failure
        }
    }
}

pub(crate) fn run_command<K, V>(
    command: String,
    work_dir: &Path,
    env_set: &[(K, V)],
    env_remove: &[K],
    verbose: bool,
) -> anyhow::Result<RunCommandOutput>
where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    // TODO: windows (we don't have sh -c there)
    let mut builder = std::process::Command::new("sh");
    builder.arg("-c").arg(command.clone()).current_dir(work_dir);
    for (key, value) in env_set {
        builder.env(key, value);
    }
    for key in env_remove {
        builder.env_remove(key);
    }
    let run_span = trace_span!("run command", command = command);
    let _enter = run_span.enter();

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

pub(crate) struct RunCommandOnFilesOutput {
    pub(crate) results: Vec<anyhow::Result<RunCommandOutput>>,
}

impl RunCommandOnFilesOutput {
    pub(crate) fn print_output_if_failed(&self) {
        for result in &self.results {
            match result {
                Ok(res) => res.print_output_if_failed(),
                Err(e) => error!("error running command: {}", e),
            }
        }
    }
    pub(crate) fn to_run_hook_result(&self) -> RunHookResult {
        if self
            .results
            .iter()
            .all(|res| res.as_ref().map_or(false, |res| res.status.success()))
        {
            RunHookResult::Success
        } else {
            RunHookResult::Failure
        }
    }
}

pub(crate) fn run_command_on_files<K, V>(
    command: &str,
    work_dir: &Path,
    env_set: &[(K, V)],
    env_remove: &[K],
    verbose: bool,
    files: Option<&PathBufSet>,
    serial: bool,
) -> RunCommandOnFilesOutput
where
    K: AsRef<OsStr> + Sync,
    V: AsRef<OsStr> + Sync,
{
    let target_count = if serial {
        1
    } else {
        match available_parallelism() {
            Ok(n) => n.get(),
            _ => 1,
        }
    };
    let filenames: Option<Vec<Cow<str>>> =
        files.map(|files| files.iter().map(|f| f.to_string_lossy()).collect());
    let commands = match filenames {
        Some(filenames) => generate_commands(command, &filenames, target_count),
        None => {
            // "pass_filenames" not set, I suppose
            vec![command.to_string()]
        }
    };

    let results = if serial {
        commands
            .into_iter()
            .map(|command| run_command(command, work_dir, env_set, env_remove, verbose))
            .collect()
    } else {
        commands
            .into_par_iter()
            .map(|command| run_command(command, work_dir, env_set, env_remove, verbose))
            .collect()
    };
    RunCommandOnFilesOutput { results }
}

fn generate_commands(
    command_prefix: &str,
    files: &Vec<Cow<str>>,
    target_count: usize,
) -> Vec<String> {
    let prefix_and_space_len = command_prefix.len() + 1;
    let mut quoted_filename_sets = get_quoted_filenames(prefix_and_space_len, files);
    if quoted_filename_sets.len() < target_count {
        // TODO: can these accidentally be longer than MAX_COMMAND_LENGTH?
        quoted_filename_sets = split_to_target_count(quoted_filename_sets, target_count);
    }
    quoted_filename_sets
        .into_iter()
        .filter(|quoted_filenames| !quoted_filenames.is_empty())
        .map(|quoted_filenames| format!("{} {}", command_prefix, quoted_filenames.join(" ")))
        .collect()
}

fn split_to_target_count(sets: Vec<Vec<Cow<str>>>, target_count: usize) -> Vec<Vec<Cow<str>>> {
    let mut split_sets: Vec<Vec<Cow<str>>> = Vec::new();
    for _ in 0..target_count {
        split_sets.push(Vec::new());
    }
    let mut i = 0;
    for filenames in sets {
        for filename in filenames {
            split_sets[i].push(filename);
            i = (i + 1) % split_sets.len();
        }
    }
    split_sets
}

fn get_quoted_filenames<'a>(
    reserve_size: usize,
    files: &'a Vec<Cow<'a, str>>,
) -> Vec<Vec<Cow<'a, str>>> {
    let mut quoted_filename_vecs = Vec::new();
    let mut current_quoted_filenames = Vec::new();
    let mut current_spaced_len = 0;
    for file in files {
        let quoted = shell_words::quote(file.as_ref());
        let this_len_with_space = quoted.len() + 1;
        let next_len = reserve_size + current_spaced_len + this_len_with_space;
        if next_len >= MAX_COMMAND_LENGTH {
            quoted_filename_vecs.push(current_quoted_filenames);
            current_quoted_filenames = Vec::new();
            current_spaced_len = 0;
        }
        current_spaced_len += this_len_with_space;
        current_quoted_filenames.push(quoted);
    }
    if !current_quoted_filenames.is_empty() {
        quoted_filename_vecs.push(current_quoted_filenames);
    }
    quoted_filename_vecs
}
