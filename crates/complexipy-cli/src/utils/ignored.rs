use std::{fs, io::ErrorKind, path::Path};

use crate::types::OutputFormat;
use crate::utils::paths::resolve_output_paths;
use complexipy_core::classes::{IgnoredLocation, RemovableIgnore};
use complexipy_core::runner::{
    collect_all_ignored_locations_shared, collect_removable_ignored_locations_shared,
};
use complexipy_core::utils::ExportError;

#[derive(Default)]
pub struct IgnoredReport {
    pub locations: Vec<IgnoredLocation>,
    pub failed_paths: Vec<String>,
    pub json_path: Option<String>,
}

pub fn handle_report_ignored(
    report_ignored: bool,
    paths: &[String],
    exclude: &[String],
    output_formats: &[OutputFormat],
    output: Option<&str>,
    invocation_path: &str,
) -> Result<IgnoredReport, ExportError> {
    if !report_ignored {
        return Ok(IgnoredReport::default());
    }

    let collected = collect_all_ignored_locations_shared(paths, exclude, invocation_path);
    let json_path = if output_formats.contains(&OutputFormat::Json) {
        let output_paths = resolve_output_paths(output_formats, output, Path::new(invocation_path))
            .map_err(|error| ExportError::Io(error.to_string()))?;
        let (_, result_path) = output_paths
            .iter()
            .find(|(format, _)| format == &OutputFormat::Json)
            .expect("JSON format was requested");
        let dir = Path::new(result_path).parent().unwrap_or(Path::new("."));
        let path = dir.join("complexipy-ignored.json");
        if Path::new(result_path) == path {
            return Err(ExportError::Io(
                "Analysis JSON and ignored-marker JSON must have different output paths"
                    .to_string(),
            ));
        }
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(ExportError::Io(format!(
                    "Failed to invalidate ignored report {}: {}",
                    path.display(),
                    error
                )));
            }
        }
        Some(path)
    } else {
        None
    };

    let (locations, failed_paths) = collected.map_err(ExportError::Io)?;
    let json_path = if failed_paths.is_empty() {
        if let Some(path) = json_path {
            let data: Vec<serde_json::Value> = locations
                .iter()
                .map(|location| {
                    serde_json::json!({
                        "path": location.path,
                        "line": location.line,
                        "comment": location.comment,
                    })
                })
                .collect();
            let serialized = serde_json::to_string_pretty(&data).map_err(|error| {
                ExportError::Serialize(format!("Failed to serialize ignored locations: {error}"))
            })?;
            fs::write(&path, format!("{serialized}\n")).map_err(|error| {
                ExportError::Io(format!(
                    "Failed to write ignored locations to {}: {}",
                    path.display(),
                    error
                ))
            })?;
            Some(path.to_string_lossy().into_owned())
        } else {
            None
        }
    } else {
        None
    };

    Ok(IgnoredReport {
        locations,
        failed_paths,
        json_path,
    })
}

pub fn handle_removable_ignores(
    paths: &[String],
    exclude: &[String],
    max_complexity_allowed: u64,
    invocation_path: &str,
) -> Result<(Vec<RemovableIgnore>, Vec<String>), String> {
    collect_removable_ignored_locations_shared(
        paths,
        exclude,
        max_complexity_allowed,
        invocation_path,
    )
}

#[cfg(test)]
mod tests;
