use std::env;
use std::env::JoinPathsError;
use std::ffi::OsString;
use std::path::PathBuf;

pub(crate) fn prepend_to_path_envvar(prepend: &str) -> Result<OsString, JoinPathsError> {
    let path = env::var_os("PATH").expect("PATH unset in ambient environment");
    let mut paths = env::split_paths(&path).collect::<Vec<_>>();
    paths.insert(0, PathBuf::from(prepend));
    env::join_paths(paths)
}

pub(crate) fn append_args(entry: &str, args: &[String]) -> anyhow::Result<String> {
    let mut command = shell_words::split(entry)?;
    for arg in args {
        command.push(shell_words::quote(arg).to_string());
    }
    Ok(shell_words::join(&command))
}
