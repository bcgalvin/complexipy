use std::fs;
use std::path::Path;
use std::slice;

use complexipy_core::classes::FileComplexity;
use complexipy_core::runner::file_complexity_shared;
use complexipy_core::{
    collect_all_ignored_locations, collect_removable_ignored_locations, run_analysis_shared,
};
use tempfile::tempdir;

const MARKED: &str = "def f(a):  # complexipy: ignore\n    return a\n";

fn write(root: &Path, name: &str, content: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn paths(files: &[FileComplexity]) -> Vec<String> {
    let mut paths: Vec<_> = files.iter().map(|file| file.path.clone()).collect();
    paths.sort();
    paths
}

#[test]
fn directory_exclusions_match_relative_to_an_absolute_walk_root() {
    let dir = tempdir().unwrap();
    write(dir.path(), "sub/a.py", MARKED.as_bytes());
    write(dir.path(), "keep/b.py", MARKED.as_bytes());
    let root = dir.path().canonicalize().unwrap();
    for pattern in ["sub/**", "**/a.py"] {
        let root = root.to_str().unwrap();
        let (files, failed) = run_analysis_shared(
            &[root.to_string()],
            &[pattern.to_string()],
            false,
            false,
            root,
        )
        .unwrap();
        assert!(failed.is_empty());
        assert_eq!(paths(&files), ["keep/b.py"]);
    }
    write(dir.path(), "pkg/a.py", MARKED.as_bytes());
    for (pattern, expected) in [
        ("a.py", Vec::<String>::new()),
        ("pkg/a.py", vec!["pkg/a.py".to_string()]),
    ] {
        let (files, failed) = run_analysis_shared(
            &["pkg".to_string()],
            &[pattern.to_string()],
            false,
            false,
            root.to_str().unwrap(),
        )
        .unwrap();
        assert!(failed.is_empty());
        assert_eq!(paths(&files), expected);
    }
}

#[test]
fn native_file_analysis_uses_the_explicit_root() {
    let dir = tempdir().unwrap();
    write(dir.path(), "pkg/a.py", MARKED.as_bytes());
    let root = dir.path().to_str().unwrap();
    let absolute = dir.path().join("pkg/a.py").canonicalize().unwrap();
    for input in ["pkg/a.py", absolute.to_str().unwrap()] {
        let file = file_complexity_shared(input, root, false, false).unwrap();
        assert_eq!(file.path, "pkg/a.py");
    }
    for invalid in [dir.path().join("missing"), absolute] {
        assert!(
            file_complexity_shared("pkg/a.py", invalid.to_str().unwrap(), false, false).is_err()
        );
    }
}

#[test]
fn all_walkers_resolve_relative_targets_and_share_file_identity() {
    let dir = tempdir().unwrap();
    write(dir.path(), "pkg/a/utils.py", MARKED.as_bytes());
    write(dir.path(), "pkg/b/utils.py", MARKED.as_bytes());
    let root = dir.path().to_str().unwrap();
    for inputs in [
        vec!["pkg".to_string()],
        vec!["pkg/a/utils.py".to_string(), "pkg/b/utils.py".to_string()],
        vec![
            "./pkg/a/../a/utils.py".to_string(),
            "pkg/b/utils.py".to_string(),
        ],
    ] {
        let (files, failed) = run_analysis_shared(&inputs, &[], false, false, root).unwrap();
        assert!(failed.is_empty());
        assert_eq!(paths(&files), ["pkg/a/utils.py", "pkg/b/utils.py"]);
        let (ignored, failed) = collect_all_ignored_locations(&inputs, &[], root).unwrap();
        assert!(failed.is_empty());
        assert_eq!(
            ignored
                .iter()
                .map(|row| row.path.as_str())
                .collect::<Vec<_>>(),
            ["pkg/a/utils.py", "pkg/b/utils.py"]
        );
        let (removable, failed) =
            collect_removable_ignored_locations(&inputs, &[], 15, root).unwrap();
        assert!(failed.is_empty());
        assert_eq!(
            removable
                .iter()
                .map(|row| row.path.as_str())
                .collect::<Vec<_>>(),
            ["pkg/a/utils.py", "pkg/b/utils.py"]
        );
    }
}

#[test]
fn outside_root_files_and_symlink_targets_keep_absolute_identity() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    write(outside.path(), "outside.py", MARKED.as_bytes());
    let target = outside.path().join("outside.py");
    std::os::unix::fs::symlink(&target, root.path().join("alias.py")).unwrap();
    let expected = target.canonicalize().unwrap().to_str().unwrap().to_string();
    for input in [target.to_str().unwrap().to_string(), "alias.py".to_string()] {
        let invocation = root.path().to_str().unwrap();
        let (files, failed) =
            run_analysis_shared(slice::from_ref(&input), &[], false, false, invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(paths(&files), [expected.as_str()]);
        let (ignored, failed) =
            collect_all_ignored_locations(slice::from_ref(&input), &[], invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(ignored[0].path, expected);
        let (removable, failed) =
            collect_removable_ignored_locations(&[input], &[], 15, invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(removable[0].path, expected);
    }
}

#[test]
fn all_walkers_filter_directories_but_process_explicit_files() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    write(dir.path(), ".ignore", b"ignored.py\n");
    write(dir.path(), ".gitignore", b"gitignored.py\n");
    let filtered = [
        "excluded.py",
        "ignored.py",
        "gitignored.py",
        ".hidden.py",
        ".hidden/nested.py",
        "notes.txt",
        "stub.pyi",
    ];
    for name in filtered.into_iter().chain(["keep.py"]) {
        write(dir.path(), name, MARKED.as_bytes());
    }
    let invocation = dir.path().to_str().unwrap();
    let excludes = ["excluded.py".to_string()];
    let (files, failed) =
        run_analysis_shared(&[".".to_string()], &excludes, false, false, invocation).unwrap();
    assert!(failed.is_empty());
    assert_eq!(paths(&files), ["keep.py"]);
    let (ignored, failed) =
        collect_all_ignored_locations(&[".".to_string()], &excludes, invocation).unwrap();
    assert!(failed.is_empty());
    assert_eq!(
        ignored
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        ["keep.py"]
    );
    let (removable, failed) =
        collect_removable_ignored_locations(&[".".to_string()], &excludes, 15, invocation).unwrap();
    assert!(failed.is_empty());
    assert_eq!(
        removable
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        ["keep.py"]
    );
    for input in filtered {
        let inputs = [input.to_string()];
        let (files, failed) =
            run_analysis_shared(&inputs, &excludes, false, false, invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(paths(&files), [input]);
        let (ignored, failed) =
            collect_all_ignored_locations(&inputs, &excludes, invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(ignored[0].path, input);
        let (removable, failed) =
            collect_removable_ignored_locations(&inputs, &excludes, 15, invocation).unwrap();
        assert!(failed.is_empty());
        assert_eq!(removable[0].path, input);
    }
}

#[test]
fn failures_are_absolute_and_valid_rows_survive() {
    let dir = tempdir().unwrap();
    write(dir.path(), "pkg/good.py", MARKED.as_bytes());
    write(dir.path(), "pkg/bad.py", b"\xff");
    write(
        dir.path(),
        "pkg/malformed.py",
        b"def broken(:  # complexipy: ignore\n    pass\n",
    );
    let root = dir.path().canonicalize().unwrap();
    let invocation = root.to_str().unwrap();
    let expected: Vec<_> = ["pkg/bad.py", "pkg/malformed.py"]
        .map(|name| root.join(name).to_str().unwrap().to_string())
        .into();
    for inputs in [
        vec!["pkg".to_string()],
        vec![
            "pkg/good.py".to_string(),
            "pkg/bad.py".to_string(),
            "pkg/malformed.py".to_string(),
        ],
    ] {
        let (files, mut failed) =
            run_analysis_shared(&inputs, &[], false, false, invocation).unwrap();
        assert_eq!(paths(&files), ["pkg/good.py"]);
        failed.sort();
        assert_eq!(failed, expected);
        let (ignored, failed) = collect_all_ignored_locations(&inputs, &[], invocation).unwrap();
        assert_eq!(ignored.len(), 2);
        assert_eq!(failed, [expected[0].clone()]);
        let (removable, mut failed) =
            collect_removable_ignored_locations(&inputs, &[], 15, invocation).unwrap();
        assert_eq!(removable.len(), 1);
        assert_eq!(removable[0].path, "pkg/good.py");
        failed.sort();
        assert_eq!(failed, expected);
    }
    let missing = ["missing.py".to_string()];
    let expected = root.join("missing.py").to_str().unwrap().to_string();
    assert_eq!(
        run_analysis_shared(&missing, &[], false, false, invocation)
            .unwrap()
            .1,
        [expected.as_str()]
    );
    assert_eq!(
        collect_all_ignored_locations(&missing, &[], invocation)
            .unwrap()
            .1,
        [expected.as_str()]
    );
    assert_eq!(
        collect_removable_ignored_locations(&missing, &[], 15, invocation)
            .unwrap()
            .1,
        [expected]
    );
}

#[test]
fn invalid_invocation_roots_fail_before_processing_even_empty_inputs() {
    let dir = tempdir().unwrap();
    write(dir.path(), "file.py", MARKED.as_bytes());
    for name in ["missing", "file.py"] {
        let root = dir.path().join(name);
        let root = root.to_str().unwrap();
        assert!(run_analysis_shared(&[], &[], false, false, root).is_err());
        assert!(collect_all_ignored_locations(&[], &[], root).is_err());
        assert!(collect_removable_ignored_locations(&[], &[], 15, root).is_err());
    }
}

#[test]
fn overlapping_inputs_preserve_multiplicity() {
    let dir = tempdir().unwrap();
    write(dir.path(), "pkg/a.py", MARKED.as_bytes());
    let root = dir.path().to_str().unwrap();
    let inputs = ["pkg".to_string(), "pkg/a.py".to_string()];
    let (files, failed) = run_analysis_shared(&inputs, &[], false, false, root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(paths(&files), ["pkg/a.py", "pkg/a.py"]);
    let (ignored, failed) = collect_all_ignored_locations(&inputs, &[], root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(
        ignored
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        ["pkg/a.py", "pkg/a.py"]
    );
    let (removable, failed) = collect_removable_ignored_locations(&inputs, &[], 15, root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(
        removable
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        ["pkg/a.py", "pkg/a.py"]
    );
}

#[test]
fn symlink_roots_produce_the_same_relative_identity() {
    let dir = tempdir().unwrap();
    let alias_dir = tempdir().unwrap();
    write(dir.path(), "pkg/a.py", MARKED.as_bytes());
    let alias = alias_dir.path().join("root");
    std::os::unix::fs::symlink(dir.path(), &alias).unwrap();
    let root = alias.to_str().unwrap();
    let inputs = ["pkg".to_string()];
    let (files, failed) = run_analysis_shared(&inputs, &[], false, false, root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(paths(&files), ["pkg/a.py"]);
    let (ignored, failed) = collect_all_ignored_locations(&inputs, &[], root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(ignored[0].path, "pkg/a.py");
    let (removable, failed) = collect_removable_ignored_locations(&inputs, &[], 15, root).unwrap();
    assert!(failed.is_empty());
    assert_eq!(removable[0].path, "pkg/a.py");
}

#[test]
fn invalid_excludes_report_the_absolute_directory_as_failed() {
    let dir = tempdir().unwrap();
    write(dir.path(), "pkg/a.py", MARKED.as_bytes());
    let root = dir.path().canonicalize().unwrap();
    let invocation = root.to_str().unwrap();
    let inputs = ["pkg".to_string()];
    let excludes = ["[".to_string()];
    let (files, analysis_failed) =
        run_analysis_shared(&inputs, &excludes, false, false, invocation).unwrap();
    assert!(files.is_empty());
    let (ignored, ignored_failed) =
        collect_all_ignored_locations(&inputs, &excludes, invocation).unwrap();
    assert!(ignored.is_empty());
    let (removable, removable_failed) =
        collect_removable_ignored_locations(&inputs, &excludes, 15, invocation).unwrap();
    assert!(removable.is_empty());
    let prefix = format!("{}: ", root.join("pkg").display());
    for failed in [analysis_failed, ignored_failed, removable_failed] {
        assert_eq!(failed.len(), 1);
        assert!(failed[0].starts_with(&prefix));
        assert!(failed[0].len() > prefix.len());
    }
}

#[test]
fn directory_analysis_forwards_script_and_suppression_flags() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "source.py",
        b"if ready:\n    pass\ndef marked(a):  # complexipy: ignore\n    if a:\n        return 1\n",
    );
    let root = dir.path().to_str().unwrap();
    for check_script in [false, true] {
        for no_ignore in [false, true] {
            let (files, failed) =
                run_analysis_shared(&[".".to_string()], &[], check_script, no_ignore, root)
                    .unwrap();
            assert!(failed.is_empty());
            assert_eq!(files.len(), 1);
            assert_eq!(
                files[0].functions.iter().any(|f| f.name == "<module>"),
                check_script
            );
            assert_eq!(
                files[0].functions.iter().any(|f| f.name == "marked"),
                no_ignore
            );
        }
    }
}
