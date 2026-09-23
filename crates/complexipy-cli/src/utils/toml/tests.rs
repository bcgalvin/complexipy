use std::fs;
use std::path::Path;

use tempfile::tempdir;

use super::get_complexipy_toml_config;

const COMPLEXIPY_TOML: &str = "max-complexity-allowed = 7\n";
const DOT_COMPLEXIPY_TOML: &str = "max-complexity-allowed = 8\n";
const PYPROJECT_TOML: &str = "[tool.complexipy]\nmax-complexity-allowed = 9\n";

fn threshold(dir: &Path) -> Option<u64> {
    get_complexipy_toml_config(dir.to_str().unwrap())
        .unwrap()
        .map(|config| config.max_complexity_allowed)
}

#[test]
fn complexipy_toml_wins_over_every_other_candidate() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(dir.path().join("complexipy.toml"), COMPLEXIPY_TOML).expect("should write");
    fs::write(dir.path().join(".complexipy.toml"), DOT_COMPLEXIPY_TOML).expect("should write");
    fs::write(dir.path().join("pyproject.toml"), PYPROJECT_TOML).expect("should write");

    assert_eq!(threshold(dir.path()), Some(7));
}

#[test]
fn dot_complexipy_toml_wins_over_pyproject() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(dir.path().join(".complexipy.toml"), DOT_COMPLEXIPY_TOML).expect("should write");
    fs::write(dir.path().join("pyproject.toml"), PYPROJECT_TOML).expect("should write");

    assert_eq!(threshold(dir.path()), Some(8));
}

#[test]
fn pyproject_tool_section_is_the_last_candidate() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(dir.path().join("pyproject.toml"), PYPROJECT_TOML).expect("should write");

    assert_eq!(threshold(dir.path()), Some(9));
}

#[test]
fn pyproject_without_a_tool_section_yields_no_config() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = \"x\"\n",
    )
    .expect("should write");

    assert_eq!(threshold(dir.path()), None);
}

#[test]
fn candidates_are_not_merged() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(dir.path().join("complexipy.toml"), COMPLEXIPY_TOML).expect("should write");
    fs::write(
        dir.path().join("pyproject.toml"),
        "[tool.complexipy]\nquiet = true\n",
    )
    .expect("should write");

    let config = get_complexipy_toml_config(dir.path().to_str().unwrap())
        .unwrap()
        .expect("should load");

    assert_eq!(config.max_complexity_allowed, 7);
    assert!(!config.quiet);
}

#[test]
fn a_malformed_complexipy_toml_stops_discovery() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(
        dir.path().join("complexipy.toml"),
        "max-complexity-allowed = \"x\"\n",
    )
    .expect("should write");
    fs::write(dir.path().join("pyproject.toml"), PYPROJECT_TOML).expect("should write");

    let error = get_complexipy_toml_config(dir.path().to_str().unwrap()).unwrap_err();
    assert!(error.contains("Failed to parse"));
    assert!(error.contains("complexipy.toml"));
}

#[test]
fn unreadable_candidates_fail_instead_of_falling_through() {
    use std::os::unix::fs::PermissionsExt;

    for name in ["complexipy.toml", ".complexipy.toml", "pyproject.toml"] {
        let dir = tempdir().unwrap();
        let path = dir.path().join(name);
        fs::write(&path, "").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let result = get_complexipy_toml_config(dir.path().to_str().unwrap());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let error = result.unwrap_err();
        assert!(error.contains("Failed to read"));
        assert!(error.contains(name));
    }
}

#[test]
fn malformed_later_candidates_fail_closed() {
    for (name, content) in [
        (".complexipy.toml", "paths = false"),
        ("pyproject.toml", "invalid ["),
        ("pyproject.toml", "[tool.complexipy]\npaths = false"),
    ] {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(name), content).unwrap();
        assert!(get_complexipy_toml_config(dir.path().to_str().unwrap()).is_err());
    }
}

#[test]
fn discovery_does_not_search_upward() {
    let dir = tempdir().expect("tempdir should work");
    fs::write(dir.path().join("complexipy.toml"), COMPLEXIPY_TOML).expect("should write");
    let child = dir.path().join("child");
    fs::create_dir(&child).expect("should create");

    assert_eq!(threshold(&child), None);
}
