use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub const DEFAULT_MAX_COMPLEXITY_ALLOWED: u64 = 15;

const CANDIDATES: [(&str, ConfigFileKind); 3] = [
    ("complexipy.toml", ConfigFileKind::Complexipy),
    (".complexipy.toml", ConfigFileKind::DotComplexipy),
    ("pyproject.toml", ConfigFileKind::Pyproject),
];

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum StringOrList<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> Default for StringOrList<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}

impl<T> StringOrList<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFileKind {
    Complexipy,
    DotComplexipy,
    Pyproject,
}

struct ConfigCandidate {
    path: PathBuf,
    kind: ConfigFileKind,
}

#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub path: PathBuf,
    pub value: toml::Value,
}

fn default_max_complexity_allowed() -> u64 {
    DEFAULT_MAX_COMPLEXITY_ALLOWED
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InlayHints {
    Always,
    #[default]
    Threshold,
    Never,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LspSection {
    #[serde(default)]
    pub inlay_hints: InlayHints,
    #[serde(default)]
    pub per_line_hints: bool,
    #[serde(default = "default_true")]
    pub diagnostics: bool,
}

impl Default for LspSection {
    fn default() -> Self {
        Self {
            inlay_hints: InlayHints::default(),
            per_line_hints: false,
            diagnostics: true,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LspConfig {
    #[serde(default = "default_max_complexity_allowed")]
    pub max_complexity_allowed: u64,
    #[serde(default)]
    pub exclude: StringOrList<String>,
    #[serde(default)]
    pub no_ignore: bool,
    #[serde(default)]
    pub lsp: LspSection,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            max_complexity_allowed: DEFAULT_MAX_COMPLEXITY_ALLOWED,
            exclude: StringOrList::default(),
            no_ignore: false,
            lsp: LspSection::default(),
        }
    }
}

pub fn read_complexipy_config(invocation_path: &str) -> Result<Option<ConfigSource>, String> {
    for (file_name, kind) in CANDIDATES {
        if let Some(candidate) = existing_candidate(Path::new(invocation_path), file_name, kind)? {
            return Ok(load_candidate_value(&candidate)?.map(|value| ConfigSource {
                path: candidate.path,
                value,
            }));
        }
    }

    Ok(None)
}

fn existing_candidate(
    invocation_path: &Path,
    file_name: &str,
    kind: ConfigFileKind,
) -> Result<Option<ConfigCandidate>, String> {
    let path = invocation_path.join(file_name);

    match fs::symlink_metadata(&path) {
        Ok(_) => Ok(Some(ConfigCandidate { path, kind })),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Failed to inspect {}: {}", path.display(), error)),
    }
}

fn load_candidate_value(candidate: &ConfigCandidate) -> Result<Option<toml::Value>, String> {
    let path = &candidate.path;
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    let value: toml::Value = toml::from_str(&content)
        .map_err(|error| format!("Failed to parse {}: {}", path.display(), error))?;

    Ok(match candidate.kind {
        ConfigFileKind::Complexipy | ConfigFileKind::DotComplexipy => Some(value),
        ConfigFileKind::Pyproject => value
            .get("tool")
            .and_then(|tool| tool.get("complexipy"))
            .cloned(),
    })
}
