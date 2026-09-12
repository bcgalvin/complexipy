//! Per-file failures must surface in the collectors' failed-path list.
#![cfg(feature = "runner")]

use std::fs;
use std::path::Path;
use std::slice;

use tempfile::{TempDir, tempdir};

use complexipy_core::{collect_all_ignored_locations, collect_removable_ignored_locations};

const VALID_MARKED: &str = "def simple(a):  # complexipy: ignore\n    return a\n";
const MALFORMED_MARKED: &str = "def broken(:  # complexipy: ignore\n    return 1\n";
const MALFORMED_UNMARKED: &str = "def broken(:\n    return 1\n";
const INVALID_UTF8: &[u8] = b"def f():  # complexipy: ignore\n    return '\xff'\n";

fn write(dir: &TempDir, name: &str, content: &[u8]) -> String {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("should write fixture");
    path.to_str().expect("utf-8 path").to_string()
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn file_names(paths: &[String]) -> Vec<String> {
    sorted(
        paths
            .iter()
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .expect("file name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
    )
}

#[test]
fn explicit_invalid_utf8_file_is_reported_by_both_collectors() {
    let dir = tempdir().expect("tempdir");
    let bad = write(&dir, "bad.py", INVALID_UTF8);

    let (locations, failed) =
        collect_all_ignored_locations(slice::from_ref(&bad), &[], ".").expect("should run");
    assert!(locations.is_empty());
    assert_eq!(failed, vec![bad.clone()]);

    let (removable, failed) =
        collect_removable_ignored_locations(slice::from_ref(&bad), &[], 15, ".")
            .expect("should run");
    assert!(removable.is_empty());
    assert_eq!(failed, vec![bad]);
}

#[test]
fn explicit_marked_malformed_file_keeps_markers_but_fails_removable_parse() {
    let dir = tempdir().expect("tempdir");
    let malformed = write(&dir, "malformed.py", MALFORMED_MARKED.as_bytes());

    let (locations, failed) =
        collect_all_ignored_locations(slice::from_ref(&malformed), &[], ".").expect("should run");
    assert_eq!(locations.len(), 1);
    assert_eq!(locations[0].line, 1);
    assert!(failed.is_empty());

    let (removable, failed) =
        collect_removable_ignored_locations(slice::from_ref(&malformed), &[], 15, ".")
            .expect("should run");
    assert!(removable.is_empty());
    assert_eq!(failed, vec![malformed]);
}

#[test]
fn unmarked_malformed_file_is_not_a_removable_failure() {
    let dir = tempdir().expect("tempdir");
    let unmarked = write(&dir, "unmarked.py", MALFORMED_UNMARKED.as_bytes());

    let (removable, failed) =
        collect_removable_ignored_locations(&[unmarked], &[], 15, ".").expect("should run");
    assert!(removable.is_empty());
    assert!(failed.is_empty());
}

#[test]
fn directory_scan_retains_valid_rows_and_reports_each_failed_file() {
    let dir = tempdir().expect("tempdir");
    write(&dir, "valid.py", VALID_MARKED.as_bytes());
    write(&dir, "malformed.py", MALFORMED_MARKED.as_bytes());
    write(&dir, "bad.py", INVALID_UTF8);
    let root = dir.path().to_str().expect("utf-8 path").to_string();

    let (locations, failed) =
        collect_all_ignored_locations(slice::from_ref(&root), &[], ".").expect("should run");
    assert_eq!(locations.len(), 2);
    assert_eq!(file_names(&failed), vec!["bad.py"]);

    let (removable, failed) =
        collect_removable_ignored_locations(&[root], &[], 15, ".").expect("should run");
    assert_eq!(removable.len(), 1);
    assert_eq!(removable[0].function, "simple");
    assert_eq!(file_names(&failed), vec!["bad.py", "malformed.py"]);
}

#[test]
fn excluded_invalid_files_are_omitted_rather_than_failed() {
    let dir = tempdir().expect("tempdir");
    write(&dir, "valid.py", VALID_MARKED.as_bytes());
    write(&dir, "bad.py", INVALID_UTF8);
    let root = dir.path().to_str().expect("utf-8 path").to_string();

    let exclude = ["**/bad.py".to_string()];

    let (locations, failed) =
        collect_all_ignored_locations(slice::from_ref(&root), &exclude, ".").expect("should run");
    assert_eq!(locations.len(), 1);
    assert!(failed.is_empty());

    let (removable, failed) =
        collect_removable_ignored_locations(&[root], &exclude, 15, ".").expect("should run");
    assert_eq!(removable.len(), 1);
    assert!(failed.is_empty());
}

#[test]
fn missing_path_reporting_is_preserved() {
    let missing = "definitely/not/here.py".to_string();

    let (locations, failed) =
        collect_all_ignored_locations(slice::from_ref(&missing), &[], ".").expect("should run");
    assert!(locations.is_empty());
    assert_eq!(failed, vec![missing]);
}
