use std::{fs, io::ErrorKind, path::Path};

use crate::types::Config;

pub fn get_complexipy_toml_config(invocation_path: &str) -> Result<Option<Config>, String> {
    let invocation_path = Path::new(invocation_path);

    for name in ["complexipy.toml", ".complexipy.toml"] {
        if let Some(content) = read_candidate(invocation_path, name)? {
            return toml::from_str(&content).map(Some).map_err(|error| {
                format!(
                    "Failed to parse {}: {}",
                    invocation_path.join(name).display(),
                    error
                )
            });
        }
    }

    let Some(content) = read_candidate(invocation_path, "pyproject.toml")? else {
        return Ok(None);
    };
    let config_path = invocation_path.join("pyproject.toml");
    let value: toml::Value = toml::from_str(&content)
        .map_err(|error| format!("Failed to parse {}: {}", config_path.display(), error))?;
    match value.get("tool").and_then(|tool| tool.get("complexipy")) {
        Some(section) => section
            .clone()
            .try_into()
            .map(Some)
            .map_err(|error| format!("Invalid config in {}: {}", config_path.display(), error)),
        None => Ok(None),
    }
}

fn read_candidate(invocation_path: &Path, name: &str) -> Result<Option<String>, String> {
    let path = invocation_path.join(name);
    match fs::symlink_metadata(&path) {
        Ok(_) => fs::read_to_string(&path)
            .map(Some)
            .map_err(|error| format!("Failed to read {}: {}", path.display(), error)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Failed to inspect {}: {}", path.display(), error)),
    }
}

#[cfg(test)]
mod tests;
