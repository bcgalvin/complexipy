use std::fs;

use tempfile::tempdir;

use crate::api::{code_complexity, file_complexity};

const SIMPLE: &str = "def simple(x):\n    return x + 1\n";

#[test]
fn code_complexity_returns_total_and_functions() {
    let result = code_complexity(SIMPLE, false, false).expect("should analyze");

    assert_eq!(result.complexity, 0);
    assert_eq!(result.functions.len(), 1);
    assert_eq!(result.functions[0].name, "simple");
    assert_eq!(result.functions[0].complexity, 0);
}

#[test]
fn code_complexity_measures_nesting() {
    let code = "def complex_func(data):\n    if data:\n        for item in data:\n            if item:\n                return item\n    return None\n";

    let result = code_complexity(code, false, false).expect("should analyze");

    assert_eq!(result.functions[0].complexity, 6);
}

#[test]
fn code_complexity_check_script_reports_module() {
    let code = "if True:\n    x = 1\n";

    let result = code_complexity(code, true, false).expect("should analyze");

    assert!(
        result
            .functions
            .iter()
            .any(|function| function.name == "<module>")
    );
}

#[test]
fn code_complexity_no_ignore_includes_suppressed() {
    let code = "def marked(x):  # complexipy: ignore\n    if x:\n        return 1\n    return 0\n";

    let ignored = code_complexity(code, false, false).expect("should analyze");
    assert_eq!(ignored.functions.len(), 0);

    let analyzed = code_complexity(code, false, true).expect("should analyze");
    assert_eq!(analyzed.functions.len(), 1);
    assert_eq!(analyzed.functions[0].complexity, 1);
}

#[test]
fn code_complexity_syntax_error() {
    let result = code_complexity("def broken(:\n", false, false);

    assert!(result.is_err());
}

#[test]
fn file_complexity_accepts_an_absolute_path() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("mymodule.py");
    fs::write(&file, SIMPLE).expect("should write");

    let result = file_complexity(file.to_str().unwrap(), false, false).expect("should analyze");

    assert_eq!(result.file_name, "mymodule.py");
    assert_eq!(result.functions[0].name, "simple");
    assert_eq!(result.functions[0].complexity, 0);
}

#[test]
fn file_complexity_outside_cwd_keeps_absolute_path() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("outside.py");
    fs::write(&file, SIMPLE).expect("should write");

    let result = file_complexity(file.to_str().unwrap(), false, false).expect("should analyze");

    assert_eq!(result.path, file.canonicalize().unwrap().to_str().unwrap());
    assert_eq!(result.file_name, "outside.py");
    assert_eq!(result.functions[0].name, "simple");
}

#[test]
fn file_complexity_missing_file() {
    let dir = tempdir().expect("tempdir should work");

    let result = file_complexity(dir.path().join("nope.py").to_str().unwrap(), false, false);

    assert!(result.is_err());
}

#[test]
fn file_complexity_syntax_error() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("broken.py");
    fs::write(&file, "def broken(:\n").expect("should write");

    let result = file_complexity(file.to_str().unwrap(), false, false);

    assert!(result.is_err());
}
