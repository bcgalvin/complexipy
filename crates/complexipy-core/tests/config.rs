use complexipy_core::config::{ConfigSource, InlayHints, LspConfig, read_complexipy_config};

use std::fs as fs_sync;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) {
    fs_sync::write(dir.path().join(name), content).expect("should write config");
}

fn source(dir: &TempDir) -> Option<ConfigSource> {
    read_complexipy_config(&dir.path().to_string_lossy()).expect("config should load")
}

fn lsp_config(dir: &TempDir) -> LspConfig {
    source(dir)
        .expect("a config file should be found")
        .value
        .try_into()
        .expect("config should convert to LspConfig")
}

#[test]
fn lsp_config_defaults_without_section() {
    let config: LspConfig = toml::from_str("max-complexity-allowed = 20").unwrap();

    assert_eq!(config.max_complexity_allowed, 20);
    assert_eq!(config.lsp.inlay_hints, InlayHints::Threshold);
    assert!(!config.lsp.per_line_hints);
    assert!(config.lsp.diagnostics);
    assert!(!config.no_ignore);
}

#[test]
fn lsp_config_rust_default_matches_the_documented_threshold() {
    let config = LspConfig::default();

    assert_eq!(config.max_complexity_allowed, 15);
    assert_eq!(config.lsp.inlay_hints, InlayHints::Threshold);
    assert!(!config.lsp.per_line_hints);
    assert!(config.lsp.diagnostics);
}

#[test]
fn lsp_config_defaults_on_empty_table() {
    let config: LspConfig = toml::from_str("").unwrap();

    assert_eq!(config.max_complexity_allowed, 15);
    assert_eq!(config.exclude.into_vec(), Vec::<String>::new());
}

#[test]
fn lsp_config_reads_kebab_case_keys() {
    let config: LspConfig = toml::from_str(
        r#"
        max-complexity-allowed = 12
        exclude = ["legacy/*"]
        no-ignore = true

        [lsp]
        inlay-hints = "always"
        per-line-hints = true
        diagnostics = false
        "#,
    )
    .unwrap();

    assert_eq!(config.max_complexity_allowed, 12);
    assert_eq!(config.exclude.into_vec(), vec!["legacy/*".to_string()]);
    assert!(config.no_ignore);
    assert_eq!(config.lsp.inlay_hints, InlayHints::Always);
    assert!(config.lsp.per_line_hints);
    assert!(!config.lsp.diagnostics);
}

#[test]
fn lsp_config_ignores_unknown_keys() {
    let config: LspConfig = toml::from_str(
        r#"
        snapshots = true

        [lsp]
        unknown-key = 1
        inlay-hints = "never"
        "#,
    )
    .unwrap();

    assert_eq!(config.lsp.inlay_hints, InlayHints::Never);
}

#[test]
fn lsp_config_rejects_unknown_inlay_hint_value() {
    let result: Result<LspConfig, _> = toml::from_str("[lsp]\ninlay-hints = \"sometimes\"");

    assert!(result.is_err());
}

#[test]
fn candidates_are_tried_in_the_documented_order() {
    let dir = TempDir::new().unwrap();
    write(&dir, "complexipy.toml", "max-complexity-allowed = 1");
    write(&dir, ".complexipy.toml", "max-complexity-allowed = 2");
    write(
        &dir,
        "pyproject.toml",
        "[tool.complexipy]\nmax-complexity-allowed = 3\n",
    );

    for (expected, name) in [
        (1, "complexipy.toml"),
        (2, ".complexipy.toml"),
        (3, "pyproject.toml"),
    ] {
        let source = source(&dir).unwrap();
        assert!(source.path.ends_with(name));
        let config: LspConfig = source.value.try_into().unwrap();
        assert_eq!(config.max_complexity_allowed, expected);
        fs_sync::remove_file(dir.path().join(name)).unwrap();
    }

    assert!(source(&dir).is_none());
}

#[test]
fn reader_prefers_complexipy_toml() {
    let dir = TempDir::new().unwrap();
    write(&dir, "complexipy.toml", "max-complexity-allowed = 3");
    write(&dir, ".complexipy.toml", "max-complexity-allowed = 9");

    assert_eq!(lsp_config(&dir).max_complexity_allowed, 3);
}

#[test]
fn reader_falls_back_to_dot_variant() {
    let dir = TempDir::new().unwrap();
    write(&dir, ".complexipy.toml", "max-complexity-allowed = 9");

    assert_eq!(lsp_config(&dir).max_complexity_allowed, 9);
}

#[test]
fn reader_reads_pyproject_section() {
    let dir = TempDir::new().unwrap();
    write(
        &dir,
        "pyproject.toml",
        "[tool.complexipy]\nmax-complexity-allowed = 7\n\n[tool.other]\nx = 1\n",
    );

    assert_eq!(lsp_config(&dir).max_complexity_allowed, 7);
}

#[test]
fn reader_returns_none_without_config_files() {
    let dir = TempDir::new().unwrap();

    assert!(source(&dir).is_none());
}

#[test]
fn reader_returns_none_without_pyproject_section() {
    let dir = TempDir::new().unwrap();
    write(&dir, "pyproject.toml", "[tool.ruff]\nline-length = 80\n");

    assert!(source(&dir).is_none());
}

#[test]
fn reader_rejects_a_malformed_first_candidate() {
    let dir = TempDir::new().unwrap();
    write(&dir, "complexipy.toml", "max-complexity-allowed = ");
    write(&dir, ".complexipy.toml", "max-complexity-allowed = 11");

    let error = read_complexipy_config(&dir.path().to_string_lossy()).unwrap_err();

    assert!(error.contains("Failed to parse"));
    assert!(error.contains("complexipy.toml"));
}

#[test]
fn reader_does_not_inspect_later_candidates() {
    let dir = TempDir::new().unwrap();
    write(&dir, "complexipy.toml", "max-complexity-allowed = 4");
    write(&dir, "pyproject.toml", "invalid [");

    assert_eq!(lsp_config(&dir).max_complexity_allowed, 4);
}

#[test]
fn readers_agree_on_all_three_sources() {
    let dir = TempDir::new().unwrap();

    write(&dir, "complexipy.toml", "max-complexity-allowed = 4");
    let direct = lsp_config(&dir);
    assert_eq!(direct.max_complexity_allowed, 4);
    fs_sync::remove_file(dir.path().join("complexipy.toml")).unwrap();

    write(&dir, ".complexipy.toml", "max-complexity-allowed = 4");
    assert_eq!(lsp_config(&dir), direct);
    fs_sync::remove_file(dir.path().join(".complexipy.toml")).unwrap();

    write(
        &dir,
        "pyproject.toml",
        "[tool.complexipy]\nmax-complexity-allowed = 4",
    );
    assert_eq!(lsp_config(&dir), direct);
}
