use crate::cfg::pre_commit_hooks::StageOrUnknown;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrecommitConfig {
    pub minimum_pre_commit_version: Option<String>,
    #[serde(default)]
    pub fail_fast: bool,
    pub files: Option<String>,
    pub exclude: Option<String>,
    pub repos: Vec<Repo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    #[serde(rename = "repo")]
    pub url: RepoURL,
    pub rev: String,
    pub hooks: Vec<HookConfiguration>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
#[serde(untagged)]
#[serde(rename_all = "snake_case")]
pub enum RepoURL {
    Local,
    Meta,
    Url(String),
}

impl Display for RepoURL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RepoURL::Local => "local".to_string(),
            RepoURL::Meta => "meta".to_string(),
            RepoURL::Url(url) => url.to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HookConfiguration {
    pub id: String,
    #[serde(flatten)]
    pub info: HookConfigurationInfo,
    #[serde(flatten)]
    pub overrides: HookDefinitionOverrides,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HookConfigurationInfo {
    /// Additional id for command line
    pub alias: Option<String>, // TODO: unimplemented.
    /// Override language version
    pub language_version: Option<String>,
    #[serde(default)]
    pub verbose: bool,
    pub log_file: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// See HookDefinition for documentation.
pub struct HookDefinitionOverrides {
    pub name: Option<String>,
    pub description: Option<String>,
    pub files: Option<String>,
    pub exclude: Option<String>,
    pub types: Option<Vec<String>>,
    pub types_or: Option<Vec<String>>,
    pub exclude_types: Option<Vec<String>>,
    pub additional_dependencies: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub stages: Option<Vec<StageOrUnknown>>,
    pub always_run: Option<bool>,
}
