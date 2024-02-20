use serde::{Deserialize, Serialize};

pub type PrecommitHooks = Vec<PrecommitHook>;

fn default_as_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrecommitHook {
    /// Used in configuration
    pub id: String,
    /// Shown during hook execution
    pub name: String,
    /// For metadata purposes only
    #[serde(default)]
    pub description: String,
    /// The executable to run; may contain arguments that will not be
    /// overridden by user arguments
    pub entry: String,
    /// List of additional arguments to pass to the hook.
    /// Can be overridden by user configuration.
    #[serde(default)]
    pub args: Vec<String>,
    /// The language the hook is written in; used to install the hook's
    /// environment
    pub language: LanguageOrUnknown,
    /// Stages to run the hook in. If unspecified, run in all stages.
    pub stages: Option<Vec<StageOrUnknown>>,
    /// File type names to run the hook on (AND condition).
    pub types: Option<Vec<String>>,
    /// File type names to run the hook on (OR condition).
    pub types_or: Option<Vec<String>>,
    /// File type names to run the hook on.
    pub files: Option<String>,
    /// Whether filenames should be passed to the hook.
    #[serde(default = "default_as_true")]
    pub pass_filenames: bool,
    /// Run even if there are no matching files.
    #[serde(default)]
    pub always_run: bool,
    /// Require serial execution.
    #[serde(default)]
    pub require_serial: bool,
    /// Additional dependencies to install in the environment,
    /// if supported by the language.
    #[serde(default)]
    pub additional_dependencies: Vec<String>,
    /// Minimum version of pre-commit required to run this hook;
    /// ignored by preco.
    pub minimum_pre_commit_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Python,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LanguageOrUnknown {
    Language(Language),
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StageOrUnknown {
    Stage(Stage),
    Unknown(String),
}
