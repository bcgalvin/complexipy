use ignore::Walk;
use std::collections::HashSet;
use std::path::Path;
use wax::walk::{Entry, FileIterator};
use wax::{Glob, any};

pub struct DiscoveredPaths {
    pub files: Vec<String>,
    pub failed_paths: Vec<String>,
}

fn resolved_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

fn record_ignore_error(error: &ignore::Error, fallback: &Path, failed_paths: &mut Vec<String>) {
    match error {
        ignore::Error::Partial(errors) => {
            for error in errors {
                record_ignore_error(error, fallback, failed_paths);
            }
        }
        ignore::Error::WithPath { path, .. } => failed_paths.push(resolved_path(path)),
        ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
            record_ignore_error(err, fallback, failed_paths);
        }
        ignore::Error::Loop { child, .. } => failed_paths.push(resolved_path(child)),
        _ => failed_paths.push(resolved_path(fallback)),
    }
}

pub fn get_paths_to_process(
    root_path: &str,
    to_exclude_paths: Vec<String>,
) -> Result<DiscoveredPaths, String> {
    let mut files = Vec::new();
    let mut failed_paths = Vec::new();
    let glob = Glob::new("**/*.py").unwrap();

    let normalized_root = root_path.replace('\\', "/");
    let root = Path::new(&normalized_root);
    let normalized_excludes: Vec<String> = to_exclude_paths
        .iter()
        .map(|s| s.replace('\\', "/"))
        .collect();

    let mut non_ignored = HashSet::new();
    for result in Walk::new(root) {
        match result {
            Ok(entry) => {
                if let Some(error) = entry.error() {
                    record_ignore_error(error, root, &mut failed_paths);
                }
                if let Some(path) = entry.path().to_str() {
                    non_ignored.insert(path.replace('\\', "/"));
                } else {
                    failed_paths.push(resolved_path(entry.path()));
                }
            }
            Err(error) => record_ignore_error(&error, root, &mut failed_paths),
        }
    }

    let exclude_refs: Vec<&str> = normalized_excludes.iter().map(|s| s.as_str()).collect();
    let non_excluded_entries = glob
        .walk(root)
        .not(any(exclude_refs))
        .map_err(|e| format!("Failed to apply exclude patterns: {}", e))?;

    for result in non_excluded_entries {
        match result {
            Ok(entry) => {
                if entry
                    .path()
                    .to_str()
                    .map(|s| non_ignored.contains(&s.replace('\\', "/")))
                    .unwrap_or(false)
                {
                    files.push(resolved_path(entry.path()));
                }
            }
            Err(error) => failed_paths.push(resolved_path(error.path().unwrap_or(root))),
        }
    }

    failed_paths.sort();
    failed_paths.dedup();
    Ok(DiscoveredPaths {
        files,
        failed_paths,
    })
}

#[cfg(test)]
mod tests;
