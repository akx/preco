use crate::cfg::pre_commit_config::{HookConfiguration, Repo, RepoURL};
use crate::cfg::pre_commit_hooks::PrecommitHooks;
use anyhow::bail;
use rustc_hash::FxHasher;
use std::fs;
use std::hash::Hasher;
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
        if !pre_commit_hooks_path.exists() {
            bail!(
                "Could not find .pre-commit-hooks.yaml in {}",
                pre_commit_hooks_path.display()
            );
        }
        let rdr = fs::File::open(&pre_commit_hooks_path)?;
        let hooks: PrecommitHooks = serde_yaml::from_reader(rdr)?;
        Ok(LoadedCheckout {
            path: self.path,
            hooks,
        })
    }
}

pub fn get_checkout(repo: &Repo, hook: &HookConfiguration) -> anyhow::Result<Checkout> {
    let mut path_str = format!(
        "preco-checkouts/{}/{}",
        normalize_repo_url_to_path(&repo.url.to_string())?,
        normalize_repo_url_to_path(&repo.rev)?
    );
    if let Some(addl_deps) = &hook.overrides.additional_dependencies {
        if !addl_deps.is_empty() {
            path_str = format!("{}+{}", path_str, hash_additional_dependencies(addl_deps));
        }
    }
    Ok(Checkout {
        repo_url: repo.url.clone(),
        repo_rev: repo.rev.clone(),
        path: PathBuf::from(&path_str),
    })
}

fn hash_additional_dependencies(deps: &Vec<String>) -> String {
    // fxhash isn't cryptographically secure, but I don't think we need that here
    let mut hasher = FxHasher::default();
    for dep in deps {
        hasher.write(dep.as_bytes());
    }
    format!("{:x}", hasher.finish())
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
