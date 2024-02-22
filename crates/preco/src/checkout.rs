use crate::cfg::pre_commit_config::{HookConfiguration, Repo, RepoURL};
use crate::cfg::pre_commit_hooks::PrecommitHooks;
use crate::dirs::get_checkouts_dir;
use crate::helpers;
use anyhow::{bail, Context};
use std::fs;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct Checkout {
    pub(crate) repo_url: RepoURL,
    pub(crate) repo_rev: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct LoadedCheckout {
    pub(crate) path: PathBuf,
    pub(crate) hooks: PrecommitHooks,
}

impl Checkout {
    pub fn ensure_checkout_cloned(&self) -> anyhow::Result<()> {
        match &self.repo_url {
            RepoURL::Local => {
                bail!("not implemented: local repos");
                // Nothing to do here
            }
            RepoURL::Meta => {
                bail!("not implemented: meta repos");
                // Nothing to do here
            }
            RepoURL::Url(url) => {
                if !(url.starts_with("http://") || url.starts_with("https://")) {
                    bail!("unsupported URL scheme: {}", url);
                }
                // TODO: need to lock (fslock?) around this if parallel
                if !self.path.exists() {
                    fs::create_dir_all(&self.path)?;
                    debug!("cloning {} to {}", &url, &self.path.display());
                    std::process::Command::new("git")
                        .args(["-c", "advice.detachedHead=false"])
                        .arg("clone")
                        .arg("--depth=1")
                        .arg("--branch")
                        .arg(&self.repo_rev)
                        .arg(url)
                        .arg(&self.path)
                        .status()?;
                }
            }
        }
        Ok(())
    }

    pub fn load(self) -> anyhow::Result<LoadedCheckout> {
        let pre_commit_hooks_path = self.path.join(".pre-commit-hooks.yaml");
        let rdr = fs::File::open(&pre_commit_hooks_path)
            .with_context(|| format!("unable to open {}", pre_commit_hooks_path.display()))?;
        let hooks: PrecommitHooks = serde_yaml::from_reader(rdr)
            .with_context(|| format!("could not parse {}", pre_commit_hooks_path.display()))?;
        Ok(LoadedCheckout {
            path: self.path,
            hooks,
        })
    }
}

pub fn get_checkout(repo: &Repo, hook: &HookConfiguration) -> anyhow::Result<Checkout> {
    let mut path_str = format!(
        "{}/{}/{}",
        get_checkouts_dir().to_str().unwrap(),
        normalize_repo_url_to_path(&repo.url.to_string())?,
        normalize_repo_url_to_path(&repo.rev)?
    );
    if let Some(addl_deps) = &hook.overrides.additional_dependencies {
        if !addl_deps.is_empty() {
            path_str = format!(
                "{}+{}",
                path_str,
                helpers::hash_additional_dependencies(addl_deps)
            );
        }
    }
    Ok(Checkout {
        repo_url: repo.url.clone(),
        repo_rev: repo.rev.clone(),
        path: PathBuf::from(&path_str),
    })
}

fn normalize_repo_url_to_path(url: &str) -> anyhow::Result<String> {
    let mut s = String::new();
    for c in url.chars() {
        if c == '/' {
            s.push('_');
        } else if c == ':' {
            s.push_str("__");
        } else if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() {
            s.push(c);
        } else {
            s.push_str(&format!("u{:02x}", c as u32));
        }
    }
    Ok(s)
}
