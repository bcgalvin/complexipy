use crate::classes::{FileComplexity, IgnoredLocation, RemovableIgnore};
use crate::cognitive_complexity::function_level_cognitive_complexity_shared;
use crate::helpers::exclude::get_paths_to_process;
use crate::utils::{collect_ignored_locations, filter_removable_ignores};
use rayon::prelude::*;
use ruff_python_parser::parse_module;
use std::path;

use crate::cognitive_complexity::code_complexity_shared;

struct ProcessOptions {
    exclude: Vec<String>,
    check_script: bool,
    no_ignore: bool,
}

type ComplexitiesAndFailedPaths = (Vec<FileComplexity>, Vec<String>);

fn resolve_root(root: &str) -> Result<path::PathBuf, String> {
    let resolved = path::Path::new(root)
        .canonicalize()
        .map_err(|error| format!("Failed to resolve path root '{}': {}", root, error))?;
    if !resolved.is_dir() {
        return Err(format!("Path root is not a directory: '{}'", root));
    }
    Ok(resolved)
}

fn resolve_input(root: &path::Path, input: &str) -> path::PathBuf {
    let resolved = root.join(input);
    resolved.canonicalize().unwrap_or(resolved)
}

fn path_string(path: &path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn relative_label(file_path: &str, root: &path::Path) -> String {
    path::Path::new(file_path)
        .strip_prefix(root)
        .ok()
        .and_then(|path| path.to_str())
        .unwrap_or(file_path)
        .to_string()
}

pub fn run_analysis_shared(
    paths: &[String],
    exclude: &[String],
    check_script: bool,
    no_ignore: bool,
    invocation_path: &str,
) -> Result<ComplexitiesAndFailedPaths, String> {
    let invocation_root = resolve_root(invocation_path)?;
    let mut successful = Vec::new();
    let mut failed_paths = Vec::new();

    for input in paths {
        let path_obj = resolve_input(&invocation_root, input);
        let path = path_string(&path_obj);
        if !path_obj.is_file() && !path_obj.is_dir() {
            failed_paths.push(path);
            continue;
        }

        let opts = ProcessOptions {
            exclude: exclude.to_vec(),
            check_script,
            no_ignore,
        };

        let (mut complexities, mut f_paths) = if path_obj.is_dir() {
            evaluate_dir_shared(&path, &opts, &invocation_root)
        } else {
            match analyze_file_shared(&path, &opts, &invocation_root) {
                Ok(file_complexity) => (vec![file_complexity], vec![]),
                Err(_) => (vec![], vec![path.to_string()]),
            }
        };
        complexities.iter_mut().for_each(|f| {
            f.functions
                .sort_by(|a, b| a.complexity.cmp(&b.complexity).then(a.name.cmp(&b.name)))
        });
        complexities.sort_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then(a.file_name.cmp(&b.file_name))
                .then(a.complexity.cmp(&b.complexity))
        });
        successful.append(&mut complexities);
        failed_paths.append(&mut f_paths);
    }

    Ok((successful, failed_paths))
}

fn evaluate_dir_shared(
    path: &str,
    opts: &ProcessOptions,
    invocation_path: &path::Path,
) -> ComplexitiesAndFailedPaths {
    let files_paths_to_process = match get_paths_to_process(path, opts.exclude.clone()) {
        Ok(paths) => paths,
        Err(e) => return (vec![], vec![format!("{}: {}", path, e)]),
    };

    let results: Vec<Result<FileComplexity, String>> = files_paths_to_process
        .par_iter()
        .map(|file_path| analyze_file_shared(file_path, opts, invocation_path))
        .collect();

    let mut complexities = Vec::new();
    let mut failed_paths = Vec::new();
    for (file_path, result) in files_paths_to_process.into_iter().zip(results) {
        match result {
            Ok(file_complexity) => complexities.push(file_complexity),
            Err(_) => failed_paths.push(file_path),
        }
    }
    (complexities, failed_paths)
}

fn analyze_file_shared(
    path: &str,
    opts: &ProcessOptions,
    invocation_path: &path::Path,
) -> Result<FileComplexity, String> {
    analyze_file_at(path, invocation_path, opts.check_script, opts.no_ignore)
}

pub fn file_complexity_shared(
    file_path: &str,
    base_path: &str,
    check_script: bool,
    no_ignore: bool,
) -> Result<FileComplexity, String> {
    let root = resolve_root(base_path)?;
    let file_path = path_string(&resolve_input(&root, file_path));
    analyze_file_at(&file_path, &root, check_script, no_ignore)
}

fn analyze_file_at(
    file_path: &str,
    base_path: &path::Path,
    check_script: bool,
    no_ignore: bool,
) -> Result<FileComplexity, String> {
    let path = path::Path::new(file_path);
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid file name: {}", file_path))?;
    let code = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;
    let code_complexity = code_complexity_shared(&code, check_script, no_ignore)
        .map_err(|e| format!("Failed to process file '{}': {}", file_path, e))?;
    Ok(FileComplexity {
        path: relative_label(file_path, base_path),
        file_name: file_name.to_string(),
        complexity: code_complexity.complexity,
        functions: code_complexity.functions,
    })
}

pub fn collect_file_ignored_locations(
    file_path: &str,
    base_path: &path::Path,
) -> Result<Vec<IgnoredLocation>, String> {
    let code = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;
    let locations = collect_ignored_locations(&code);
    Ok(locations
        .into_iter()
        .map(|(line, comment)| IgnoredLocation {
            path: relative_label(file_path, base_path),
            line,
            comment,
        })
        .collect())
}

pub fn collect_all_ignored_locations_shared(
    paths: &[String],
    exclude: &[String],
    invocation_path: &str,
) -> Result<(Vec<IgnoredLocation>, Vec<String>), String> {
    collect_locations(
        paths,
        exclude,
        invocation_path,
        collect_file_ignored_locations,
    )
}

pub fn collect_removable_ignored_locations_shared(
    paths: &[String],
    exclude: &[String],
    max_complexity_allowed: u64,
    invocation_path: &str,
) -> Result<(Vec<RemovableIgnore>, Vec<String>), String> {
    collect_locations(paths, exclude, invocation_path, |file_path, root| {
        collect_removable_ignores_from_file(file_path, root, max_complexity_allowed)
    })
}

trait Located {
    fn sort_key(&self) -> (String, u64);
}

impl Located for IgnoredLocation {
    fn sort_key(&self) -> (String, u64) {
        (self.path.clone(), self.line)
    }
}

impl Located for RemovableIgnore {
    fn sort_key(&self) -> (String, u64) {
        (self.path.clone(), self.line)
    }
}

fn collect_locations<T, F>(
    paths: &[String],
    exclude: &[String],
    invocation_path: &str,
    collect_file: F,
) -> Result<(Vec<T>, Vec<String>), String>
where
    T: Located + Send,
    F: Fn(&str, &path::Path) -> Result<Vec<T>, String> + Copy + Sync,
{
    let root = resolve_root(invocation_path)?;
    let mut all_locations = Vec::new();
    let mut failed_paths = Vec::new();

    for input in paths {
        let path_obj = resolve_input(&root, input);
        let path_str = path_string(&path_obj);

        if path_obj.is_dir() {
            let files = match get_paths_to_process(&path_str, exclude.to_vec()) {
                Ok(paths) => paths,
                Err(e) => {
                    failed_paths.push(format!("{}: {}", path_str, e));
                    continue;
                }
            };
            let results: Vec<Result<Vec<T>, String>> = files
                .par_iter()
                .map(|file_path| collect_file(file_path, &root))
                .collect();
            for (file_path, result) in files.into_iter().zip(results) {
                match result {
                    Ok(locs) => all_locations.extend(locs),
                    Err(_) => failed_paths.push(file_path),
                }
            }
        } else if path_obj.is_file() {
            match collect_file(&path_str, &root) {
                Ok(locs) => all_locations.extend(locs),
                Err(_) => failed_paths.push(path_str.to_string()),
            }
        } else {
            failed_paths.push(path_str.to_string());
        }
    }

    all_locations.sort_by_key(Located::sort_key);
    Ok((all_locations, failed_paths))
}

fn collect_removable_ignores_from_file(
    file_path: &str,
    base_path: &path::Path,
    max_complexity_allowed: u64,
) -> Result<Vec<RemovableIgnore>, String> {
    let code = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;
    let locations = collect_ignored_locations(&code);
    if locations.is_empty() {
        return Ok(vec![]);
    }
    let parsed = parse_module(&code).map_err(|e| format!("Failed to parse code: {}", e))?;
    let ast_body = parsed.into_suite();
    let (functions, _) =
        function_level_cognitive_complexity_shared(&ast_body, &code, false, true, false);
    let removable = filter_removable_ignores(&locations, &functions, max_complexity_allowed);
    Ok(removable
        .into_iter()
        .map(|(line, comment, function, complexity)| RemovableIgnore {
            path: relative_label(file_path, base_path),
            line,
            comment,
            function,
            complexity,
        })
        .collect())
}
