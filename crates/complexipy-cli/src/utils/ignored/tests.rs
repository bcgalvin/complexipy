use std::fs;

use serde_json::Value;
use tempfile::tempdir;

use crate::types::OutputFormat;
use crate::utils::ignored::{IgnoredReport, handle_removable_ignores, handle_report_ignored};

const SIMPLE_MARKED: &str = "def simple(x):  # complexipy: ignore\n    return x + 1\n";

const COMPLEX_MARKED: &str = "def complex_fn(data):  # complexipy: ignore\n    if data:\n        for item in data:\n            if item:\n                return item\n    return None\n";

fn marked_file(dir: &std::path::Path, name: &str, content: &str) -> String {
    fs::write(dir.join(name), content).expect("should write");
    dir.join(name).to_string_lossy().into_owned()
}

#[test]
fn report_disabled_returns_empty() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", SIMPLE_MARKED);
    let report = dir.path().join("complexipy-ignored.json");
    fs::write(&report, "sentinel").unwrap();

    let IgnoredReport {
        locations,
        json_path,
        failed_paths,
    } = handle_report_ignored(
        false,
        &[file],
        &[],
        &[OutputFormat::Json],
        None,
        dir.path().to_str().unwrap(),
    )
    .expect("should succeed");
    assert!(failed_paths.is_empty());

    assert!(locations.is_empty());
    assert_eq!(json_path, None);
    assert_eq!(fs::read_to_string(report).unwrap(), "sentinel");
}

#[test]
fn report_writes_ignored_json_next_to_output() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", SIMPLE_MARKED);

    let IgnoredReport {
        locations,
        json_path,
        failed_paths,
    } = handle_report_ignored(
        true,
        &[file],
        &[],
        &[OutputFormat::Json],
        None,
        dir.path().to_str().unwrap(),
    )
    .expect("should succeed");
    assert!(failed_paths.is_empty());

    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].line, 1);
    assert_eq!(locations[0].comment, "# complexipy: ignore");

    let expected_path = dir.path().join("complexipy-ignored.json");
    assert_eq!(
        json_path,
        Some(expected_path.to_string_lossy().into_owned())
    );
    let content = fs::read_to_string(&expected_path).expect("should read");
    assert!(content.ends_with('\n'));
    let parsed: Vec<Value> = serde_json::from_str(&content).expect("should parse");
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["line"], 1);
    assert_eq!(parsed[0]["comment"], "# complexipy: ignore");
    assert!(parsed[0]["path"].as_str().expect("path").ends_with("a.py"));
}

#[test]
fn report_without_json_format_leaves_existing_json_untouched() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", SIMPLE_MARKED);
    let report = dir.path().join("complexipy-ignored.json");
    fs::write(&report, "sentinel").unwrap();

    let IgnoredReport {
        locations,
        json_path,
        failed_paths,
    } = handle_report_ignored(
        true,
        &[file],
        &[],
        &[OutputFormat::Csv],
        None,
        dir.path().to_str().unwrap(),
    )
    .expect("should succeed");
    assert!(failed_paths.is_empty());

    assert_eq!(locations.len(), 1);
    assert_eq!(json_path, None);
    assert_eq!(fs::read_to_string(report).unwrap(), "sentinel");
}

#[test]
fn report_without_comments_overwrites_stale_json_with_empty_array() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", "def plain(x):\n    return x\n");
    fs::write(
        dir.path().join("complexipy-ignored.json"),
        "[{\"stale\":true}]\n",
    )
    .unwrap();

    let IgnoredReport {
        locations,
        json_path,
        failed_paths,
    } = handle_report_ignored(
        true,
        &[file],
        &[],
        &[OutputFormat::Json],
        None,
        dir.path().to_str().unwrap(),
    )
    .expect("should succeed");
    assert!(failed_paths.is_empty());

    assert!(locations.is_empty());
    assert!(json_path.is_some());
    assert_eq!(
        fs::read_to_string(dir.path().join("complexipy-ignored.json")).unwrap(),
        "[]\n"
    );
}

#[test]
fn partial_reports_keep_good_rows_and_remove_stale_json() {
    let dir = tempdir().unwrap();
    let file = marked_file(dir.path(), "a.py", SIMPLE_MARKED);
    let root = dir.path().canonicalize().unwrap();
    let missing = root.join("missing.py").to_str().unwrap().to_string();
    let output = dir.path().join("complexipy-ignored.json");
    for paths in [vec![file.clone(), missing.clone()], vec![missing.clone()]] {
        fs::write(&output, "[{\"stale\":true}]\n").unwrap();
        let report = handle_report_ignored(
            true,
            &paths,
            &[],
            &[OutputFormat::Json],
            None,
            root.to_str().unwrap(),
        )
        .unwrap();
        assert_eq!(report.locations.len(), paths.len() - 1);
        assert_eq!(report.failed_paths, [missing.as_str()]);
        assert_eq!(report.json_path, None);
        assert!(!output.exists());
        let (removable, failed) =
            handle_removable_ignores(&paths, &[], 15, root.to_str().unwrap()).unwrap();
        assert_eq!(removable.len(), paths.len() - 1);
        assert_eq!(failed, [missing.as_str()]);
    }
}

#[test]
fn top_level_collector_errors_are_not_empty_reports() {
    let dir = tempdir().unwrap();
    let output = dir.path().join("complexipy-ignored.json");
    fs::write(&output, "[{\"stale\":true}]\n").unwrap();
    let missing = dir.path().join("missing");
    let root = missing.to_str().unwrap();
    assert!(
        handle_report_ignored(
            true,
            &[],
            &[],
            &[OutputFormat::Json],
            Some(dir.path().to_str().unwrap()),
            root,
        )
        .is_err()
    );
    assert!(!output.exists());
    assert!(handle_removable_ignores(&[], &[], 15, root).is_err());
}

#[test]
fn removable_ignores_below_threshold() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", SIMPLE_MARKED);

    let (removable, failed) =
        handle_removable_ignores(&[file], &[], 15, dir.path().to_str().unwrap()).unwrap();
    assert!(failed.is_empty());

    assert_eq!(removable.len(), 1);
    assert_eq!(removable[0].function, "simple");
    assert_eq!(removable[0].complexity, 0);
    assert_eq!(removable[0].line, 1);
}

#[test]
fn removable_ignores_above_threshold_excluded() {
    let dir = tempdir().expect("tempdir should work");
    let file = marked_file(dir.path(), "a.py", COMPLEX_MARKED);

    let (removable, failed) =
        handle_removable_ignores(&[file], &[], 2, dir.path().to_str().unwrap()).unwrap();
    assert!(failed.is_empty());

    assert!(removable.is_empty());
}
