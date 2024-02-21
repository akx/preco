use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PrecommitConfig {
    pub minimum_pre_commit_version: Option<String>,
    #[serde(default)]
    pub fail_fast: bool,
    pub files: Option<String>,   // TODO: unimplemented
    pub exclude: Option<String>, // TODO: unimplemented
    pub repos: Vec<Repo>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    #[serde(rename = "repo")]
    pub url: RepoURL,
    pub rev: String,
    pub hooks: Vec<Hook>,
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
pub struct Hook {
    pub id: String,
    pub additional_dependencies: Option<Vec<String>>,
    pub args: Option<Vec<String>>, // TODO: unimplemented
    pub exclude: Option<String>,   // TODO: unimplemented
}
