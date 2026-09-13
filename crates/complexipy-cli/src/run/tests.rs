use std::fs;
use std::path::Path;
use std::process::Command;

use clap::Parser;
use tempfile::tempdir;

use super::run_at;
use crate::args::CliArgs;

const SIMPLE: &str = "def simple(x):\n    return x + 1\n";

const COMPLEX: &str = "def complex_func(data):\n    if data:\n        for item in data:\n            if item:\n                for x in item:\n                    if x:\n                        for y in x:\n                            if y:\n                                return y\n    return None\n";

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("git should run");
    assert!(status.success(), "git {:?} failed", args);
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "initial"]);
}

fn parse(args: &[&str]) -> CliArgs {
    let mut argv = vec!["complexipy"];
    argv.extend_from_slice(args);
    CliArgs::try_parse_from(argv).expect("cli args should parse")
}

#[test]
fn missing_paths_exits_failure() {
    let dir = tempdir().expect("tempdir should work");

    let exit = run_at(parse(&[]), dir.path().to_str().unwrap());

    assert_eq!(exit, std::process::ExitCode::FAILURE);
}

#[test]
fn clean_run_exits_success() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("simple.py");
    fs::write(&file, SIMPLE).expect("should write");

    let exit = run_at(
        parse(&[file.to_str().unwrap()]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::SUCCESS);
}

#[test]
fn failed_run_exits_failure() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("complex.py");
    fs::write(&file, COMPLEX).expect("should write");

    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--failed"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::FAILURE);
}

#[test]
fn snapshot_create_writes_snapshot_file() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("complex.py");
    fs::write(&file, COMPLEX).expect("should write");

    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--snapshot-create"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::SUCCESS);
    assert!(dir.path().join("complexipy-snapshot.json").exists());
}

#[test]
fn invalid_path_exits_failure() {
    let dir = tempdir().expect("tempdir should work");

    let exit = run_at(
        parse(&[dir.path().join("nope.py").to_str().unwrap()]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::FAILURE);
}

#[test]
fn diff_regression_exits_failure() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("a.py");
    fs::write(&file, SIMPLE).expect("should write");
    init_repo(dir.path());

    fs::write(&file, COMPLEX).expect("should write");
    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--diff", "HEAD"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::FAILURE);
}

#[test]
fn diff_clean_exits_success() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("a.py");
    fs::write(&file, COMPLEX).expect("should write");
    init_repo(dir.path());

    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--diff", "HEAD"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::SUCCESS);
}

#[test]
fn diff_only_leaves_the_threshold_gate_in_place() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("a.py");
    fs::write(&file, COMPLEX).expect("should write");
    init_repo(dir.path());

    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--diff-only", "HEAD"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::FAILURE);
}

#[test]
fn plain_flag_accepted() {
    let dir = tempdir().expect("tempdir should work");
    let file = dir.path().join("simple.py");
    fs::write(&file, SIMPLE).expect("should write");

    let exit = run_at(
        parse(&[file.to_str().unwrap(), "--plain"]),
        dir.path().to_str().unwrap(),
    );

    assert_eq!(exit, std::process::ExitCode::SUCCESS);
}

#[test]
fn relative_directory_is_resolved_from_invocation_root() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("pkg")).unwrap();
    fs::write(dir.path().join("pkg/simple.py"), SIMPLE).unwrap();
    let exit = run_at(
        parse(&["pkg", "--quiet", "--snapshot-ignore"]),
        dir.path().to_str().unwrap(),
    );
    assert_eq!(exit, std::process::ExitCode::SUCCESS);
}

#[test]
fn directory_exclusion_changes_the_threshold_gate_population() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("pkg")).unwrap();
    fs::write(dir.path().join("pkg/simple.py"), SIMPLE).unwrap();
    fs::write(dir.path().join("pkg/complex.py"), COMPLEX).unwrap();
    let invocation = dir.path().to_str().unwrap();
    let exit = run_at(parse(&["pkg", "--quiet", "--snapshot-ignore"]), invocation);
    assert_eq!(exit, std::process::ExitCode::FAILURE);
    let exit = run_at(
        parse(&[
            "pkg",
            "--quiet",
            "--snapshot-ignore",
            "--exclude",
            "complex.py",
        ]),
        invocation,
    );
    assert_eq!(exit, std::process::ExitCode::SUCCESS);
}

#[test]
fn analysis_and_marker_json_share_canonical_paths() {
    let invocation = tempdir().unwrap();
    let target = tempdir().unwrap();
    let source = "def marked(a):  # complexipy: ignore\n    return a\n";
    fs::write(target.path().join("source.py"), source).unwrap();
    std::os::unix::fs::symlink(target.path(), invocation.path().join("alias")).unwrap();
    let target_file = target.path().join("source.py").canonicalize().unwrap();
    for (root, expected) in [
        (target.path(), "source.py"),
        (invocation.path(), target_file.to_str().unwrap()),
    ] {
        let input = if root == target.path() { "." } else { "alias" };
        let exit = run_at(
            parse(&[
                input,
                "--quiet",
                "--snapshot-ignore",
                "--no-ignore",
                "--report-ignored",
                "--output-format",
                "json",
                "--output",
                "output/",
            ]),
            root.to_str().unwrap(),
        );
        assert_eq!(exit, std::process::ExitCode::SUCCESS);
        for name in ["complexipy-results.json", "complexipy-ignored.json"] {
            let rows: Vec<serde_json::Value> =
                serde_json::from_str(&fs::read_to_string(root.join("output").join(name)).unwrap())
                    .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["path"], expected);
        }
    }
}

#[test]
fn version_flag_handled_by_clap() {
    let error = CliArgs::try_parse_from(["complexipy", "--version"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
    assert_eq!(error.exit_code(), 0);
    assert!(!error.use_stderr());
    assert_eq!(
        error.to_string(),
        format!("complexipy {}\n", env!("CARGO_PKG_VERSION"))
    );
}
