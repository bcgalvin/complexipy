use ignore::Walk;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;
use wax::walk::{Entry, FileIterator};
use wax::{Glob, Program, any};

pub fn is_path_excluded(path: &str, root: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let normalized_path = path.replace('\\', "/");
    let normalized_root = root.replace('\\', "/");
    let relative = relative_to(&normalized_path, normalized_root.trim_end_matches('/'))
        .unwrap_or(normalized_path.as_str());

    patterns
        .iter()
        .any(|pattern| pattern_matches(relative, pattern))
}

fn pattern_matches(relative: &str, pattern: &str) -> bool {
    any([normalized_pattern(pattern).as_ref()])
        .map(|program| program.is_match(relative))
        .unwrap_or(false)
}

fn normalized_pattern(pattern: &str) -> Cow<'_, str> {
    if pattern.contains('\\') {
        Cow::Owned(pattern.replace('\\', "/"))
    } else {
        Cow::Borrowed(pattern)
    }
}

fn relative_to<'a>(path: &'a str, root: &str) -> Option<&'a str> {
    if root.is_empty() {
        return Some(path.trim_start_matches('/'));
    }

    if path == root {
        return Some("");
    }

    path.strip_prefix(root)?.strip_prefix('/')
}

pub fn invalid_exclude_patterns(patterns: &[String]) -> Vec<String> {
    patterns
        .iter()
        .filter(|pattern| any([normalized_pattern(pattern.as_str()).as_ref()]).is_err())
        .cloned()
        .collect()
}

pub fn exclude_list_overflows(patterns: &[String]) -> bool {
    if patterns.is_empty() || !invalid_exclude_patterns(patterns).is_empty() {
        return false;
    }

    let normalized: Vec<String> = patterns
        .iter()
        .map(|pattern| normalized_pattern(pattern).into_owned())
        .collect();
    let pattern_refs: Vec<&str> = normalized.iter().map(|s| s.as_str()).collect();

    any(pattern_refs).is_err()
}

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
