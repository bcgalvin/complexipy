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
    assert!(status.success(), "git {args:?} failed");
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "test@example.com"]);
    git(dir, &["config", "user.name", "Test"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "initial"]);
}

fn parse(args: &[&str]) -> CliArgs {
    let mut command = vec!["complexipy"];
    command.extend_from_slice(args);
    CliArgs::try_parse_from(command).expect("cli args should parse")
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
fn quiet_and_normal_runs_honor_ignore_complexity() {
    for quiet in [false, true] {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("complex.py"), COMPLEX).unwrap();
        let mut args = vec!["complex.py", "--snapshot-ignore", "--ignore-complexity"];
        if quiet {
            args.push("--quiet");
        }
        assert_eq!(
            run_at(parse(&args), dir.path().to_str().unwrap()),
            std::process::ExitCode::SUCCESS
        );
    }
}

#[test]
fn configured_empty_paths_and_malformed_candidates_fail_closed() {
    for config in ["quiet = true\n", "paths = []\n", "invalid ["] {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("complexipy.toml"), config).unwrap();
        fs::write(dir.path().join(".complexipy.toml"), "paths = ['.']\n").unwrap();
        assert_eq!(
            run_at(parse(&[]), dir.path().to_str().unwrap()),
            std::process::ExitCode::FAILURE
        );
        assert!(!dir.path().join("complexipy-snapshot.json").exists());
        assert!(!dir.path().join(".complexipy_cache").exists());
    }
}

#[test]
fn incomplete_analysis_never_creates_snapshot_or_cache() {
    for quiet in [false, true] {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("good.py"), SIMPLE).unwrap();
        let mut args = vec!["good.py", "missing.py", "--snapshot-create"];
        if quiet {
            args.push("--quiet");
        }
        assert_eq!(
            run_at(parse(&args), dir.path().to_str().unwrap()),
            std::process::ExitCode::FAILURE
        );
        assert!(!dir.path().join("complexipy-snapshot.json").exists());
        assert!(!dir.path().join(".complexipy_cache").exists());
    }
}

#[test]
fn incomplete_analysis_preserves_existing_state_and_exports_good_rows() {
    for quiet in [false, true] {
        for create in [false, true] {
            let dir = tempdir().unwrap();
            fs::create_dir(dir.path().join("pkg")).unwrap();
            let good = dir.path().join("pkg/good.py");
            let bad = dir.path().join("pkg/bad.py");
            fs::write(&good, COMPLEX).unwrap();
            fs::write(&bad, COMPLEX).unwrap();
            let root = dir.path().to_str().unwrap();
            assert_eq!(
                run_at(parse(&["pkg", "--snapshot-create"]), root),
                std::process::ExitCode::SUCCESS
            );
            let snapshot_path = dir.path().join("complexipy-snapshot.json");
            let cache_path = dir.path().join(".complexipy_cache/v/cache/functions");
            let snapshot = fs::read(&snapshot_path).unwrap();
            let cache = fs::read(&cache_path).unwrap();
            fs::write(&good, SIMPLE).unwrap();
            fs::write(&bad, "def broken(:\n").unwrap();
            let mut args = vec!["pkg", "--output-format", "json"];
            if quiet {
                args.push("--quiet");
            }
            if create {
                args.push("--snapshot-create");
            }
            assert_eq!(run_at(parse(&args), root), std::process::ExitCode::FAILURE);
            assert_eq!(fs::read(&snapshot_path).unwrap(), snapshot);
            assert_eq!(fs::read(&cache_path).unwrap(), cache);
            let rows: Vec<serde_json::Value> = serde_json::from_slice(
                &fs::read(dir.path().join("complexipy-results.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0]["path"], "pkg/good.py");
        }
    }
}

#[test]
fn complete_over_threshold_analysis_still_updates_cache() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.py"), SIMPLE).unwrap();
    let root = dir.path().to_str().unwrap();
    assert_eq!(
        run_at(parse(&["a.py", "--quiet"]), root),
        std::process::ExitCode::SUCCESS
    );
    let cache_path = dir.path().join(".complexipy_cache/v/cache/functions");
    let before = fs::read(&cache_path).unwrap();
    fs::write(dir.path().join("a.py"), COMPLEX).unwrap();
    assert_eq!(
        run_at(parse(&["a.py", "--quiet"]), root),
        std::process::ExitCode::FAILURE
    );
    assert_ne!(fs::read(&cache_path).unwrap(), before);
}

#[test]
fn a_valid_filtered_empty_population_succeeds() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("pkg")).unwrap();
    fs::write(dir.path().join("pkg/a.py"), COMPLEX).unwrap();
    assert_eq!(
        run_at(
            parse(&[
                "pkg",
                "--exclude",
                "a.py",
                "--quiet",
                "--report-ignored",
                "--output-format",
                "json"
            ]),
            dir.path().to_str().unwrap(),
        ),
        std::process::ExitCode::SUCCESS
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("complexipy-ignored.json")).unwrap(),
        "[]\n"
    );
}

#[test]
fn invalid_or_colliding_outputs_do_not_touch_marker_json_or_state() {
    for (formats, output) in [
        ("json", "complexipy-ignored.json"),
        ("csv,json", "results.json"),
    ] {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("a.py"),
            "def f():  # complexipy: ignore\n    pass\n",
        )
        .unwrap();
        let report = dir.path().join("complexipy-ignored.json");
        fs::write(&report, "sentinel").unwrap();
        let exit = run_at(
            parse(&[
                "a.py",
                "--report-ignored",
                "--output-format",
                formats,
                "--output",
                output,
                "--snapshot-create",
            ]),
            dir.path().to_str().unwrap(),
        );
        assert_eq!(exit, std::process::ExitCode::FAILURE);
        assert_eq!(fs::read_to_string(&report).unwrap(), "sentinel");
        assert!(!dir.path().join("results.json").exists());
        assert!(!dir.path().join("complexipy-snapshot.json").exists());
        assert!(!dir.path().join(".complexipy_cache").exists());
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
