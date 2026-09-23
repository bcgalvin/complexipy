use complexipy_core::config::read_complexipy_config;
use serde::Deserialize;

use crate::types::Config;

pub fn get_complexipy_toml_config(invocation_path: &str) -> Result<Option<Config>, String> {
    let Some(source) = read_complexipy_config(invocation_path)? else {
        return Ok(None);
    };

    Config::deserialize(source.value)
        .map(Some)
        .map_err(|error| format!("Invalid config in {}: {}", source.path.display(), error))
}

#[cfg(test)]
mod tests;
