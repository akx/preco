use crate::cfg::pre_commit_hooks::StageOrUnknown;
use identify::mappings::Type;
use serde::Deserialize;
use std::fmt::Display;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrecommitConfig {
    pub minimum_pre_commit_version: Option<String>,
    #[serde(default)]
    pub fail_fast: bool,
    pub files: Option<String>,
    pub exclude: Option<String>,
    pub repos: Vec<Repo>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    #[serde(rename = "repo")]
    pub url: RepoURL,
    pub rev: String,
    pub hooks: Vec<HookConfiguration>,
}

#[derive(Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
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

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HookConfiguration {
    pub id: String,
    #[serde(flatten)]
    pub info: HookConfigurationInfo,
    #[serde(flatten)]
    pub overrides: HookDefinitionOverrides,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct HookConfigurationInfo {
    /// Additional id for command line
    pub alias: Option<String>,
    /// Override language version
    pub language_version: Option<String>,
    #[serde(default)]
    pub verbose: bool,
    pub log_file: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
/// See HookDefinition for documentation.
pub struct HookDefinitionOverrides {
    pub name: Option<String>,
    pub description: Option<String>,
    pub files: Option<String>,
    pub exclude: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::cfg::parsing::deserialize_type_list"
    )]
    pub types: Option<Vec<Type>>,
    #[serde(
        default,
        deserialize_with = "crate::cfg::parsing::deserialize_type_list"
    )]
    pub types_or: Option<Vec<Type>>,
    #[serde(
        default,
        deserialize_with = "crate::cfg::parsing::deserialize_type_list"
    )]
    pub exclude_types: Option<Vec<Type>>,
    pub additional_dependencies: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub stages: Option<Vec<StageOrUnknown>>,
    pub always_run: Option<bool>,
}
