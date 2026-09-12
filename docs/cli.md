# CLI

`complexipy [OPTIONS] [PATHS]...`. Every option carries help text, so
`complexipy --help` is authoritative for flag names and accepted values. It
prints no defaults: every option is optional there and the defaults are applied
in `resolve_config` (`utils/config.rs`) - threshold 15, `sort` `asc`, `color`
`auto`, every switch off. This page covers the behavior that a flag list cannot
convey.

## Configuration discovery

Config is read from the **first** of these that is found *and parses* in the
invocation directory:

1. `complexipy.toml`
1. `.complexipy.toml`
1. `pyproject.toml`, `[tool.complexipy]`

**First hit wins outright - the files are not merged, and there is no upward
search** (`crates/complexipy-cli/src/utils/toml/tests.rs`). A candidate that
fails to parse is not fatal: the error goes to stderr and discovery moves on to
the next candidate, then to the built-in defaults
(`a_malformed_complexipy_toml_falls_through_to_the_next_candidate`). Three
consequences worth internalizing:

- If a `complexipy.toml` exists, a `[tool.complexipy]` block in the same
  `pyproject.toml` is never read. This repository has both; only the former
  applies.
- Discovery is relative to the **invocation directory**, not the analyzed path.
  Running the tool from outside a target means the target's own thresholds are
  silently ignored.
- A typo that breaks `complexipy.toml` changes the threshold to the next
  candidate's, or to 15, with only a stderr line to show for it - and a config
  file that parses but has no `paths` key makes a bare `complexipy` analyze
  nothing and exit 0 (`resolve_config` only errors on missing paths when no
  config file loaded at all).

Keys are kebab-case and mirror the flag names. `paths`, `exclude` and
`output-format` accept a bare string or a list. `diff` is a **table**, not the
string the `--diff` flag takes:

```toml
paths = ["src"]
exclude = ["tests/**"]
max-complexity-allowed = 15

[diff]
branch = "main"
staged = false
```

CLI arguments override the file (`utils/config/tests.rs`). Four options are
CLI-only with no config key: `--plain`, `--suggest-refactors`, `--top` and
`--diff-only`; `--staged` is the `staged` key of the `[diff]` table. (Source read
of `Config` in `types.rs` against `CliArgs` in `args.rs`.)

## Machine-readable output

`--output-format` takes any of `csv`, `json`, `gitlab`, `sarif`, comma-separated.
Default filenames are `complexipy-results.csv`, `.json`, `.gitlab.json` and
`.sarif`.

`--output` sets the destination, and its rules are strict:

- `-` is rejected. Writing machine-readable output to stdout is not supported.
- A trailing separator means a directory, and parent directories are created.
- Selecting more than one format requires a directory destination. An existing
  file, or a path without a trailing separator, is an error.

The JSON payload is hand-built per record rather than a serialization of the
internal types, so it carries `{path, file_name, function_name, complexity, refactor_plans}`. `line_complexities` reaches no output format; read it through
the [Python API](python-api.md).

## Exit codes

- **0** - analysis ran and every gate passed.
- **1** - a gate failed, or the run itself failed: no paths and no config file,
  a `cache-dir` that is not a non-empty string, an unwritable output. `run_at`
  returns the same failure code for both (source read of `run.rs`;
  `run/tests.rs` pins `missing_paths_exits_failure`).
- **2** - a usage error rejected the arguments before anything ran. This is
  clap's code; the console script forwards it.

Which gates apply depends on the flags (`ExitReport::success` in `types.rs`).
Normally the result is the threshold check, the path check and the snapshot
check. **`--diff <ref>` replaces the threshold check with the regression
ratchet**: the run fails only when a function regressed, or is new, above the
threshold, so it exits 0 with pre-existing functions over the limit
(`diff_clean_exits_success`, `diff_regression_exits_failure`). **`--diff-only <ref>` prints the same comparison and leaves the exit code exactly as it would
be without the flag** (`diff_only_leaves_the_threshold_gate_in_place`); it is
the visual-only form, not the stricter one. **`--staged` on its own selects the
same ratchet gate as `--diff HEAD`** (`resolve_diff_flags` supplies the
reference; `resolve_diff_flags_staged_defaults_to_head`), so a bare
`complexipy --staged` never enforces the threshold either. `--ignore-complexity`
forces the threshold check to pass while leaving the others intact - except under
`--quiet`, where it is not consulted at all; see the rough edges below.

The ratchet gate fails open. When the comparison cannot be made - `--staged`
outside a git repository, or a staged reference git cannot resolve - `run.rs`
leaves `diff_ok` true while `enforce_diff` still drops the threshold check, so an
over-threshold tree exits 0 having applied neither gate (source read; recorded
in the realignment catalog).

## Inline ignores

Two markers are recognized, case-insensitively:

```python
# complexipy: ignore
# noqa: complexipy
```

A marker suppresses a function when it sits on the `def` line, on the line
immediately above the definition's first line (the `def`, or the first decorator
when there is one), on a decorator line, or inside a multi-line signature before
the first line that contains a colon - an annotated parameter counts
(`test_ignore_marker_placements_that_suppress` pins the four working placements;
`find_noqa_comment` in `crates/complexipy-core/src/utils.rs` is the source). A
marker between two decorators, or after an annotated parameter in a multi-line
signature, does not suppress. A bare `# noqa` is **not** recognized. An ignored
function is excluded from results entirely rather than reported with a zero
score (`test_noqa_complexipy_ignore`).

`--no-ignore` disregards every marker. Two reports exist and they are not the
same thing:

- **`--report-ignored`** lists recognized markers. With `--output-format json` it
  also writes them to `complexipy-ignored.json` beside the resolved JSON output
  path (`utils/ignored.rs`; `utils/ignored/tests.rs`). Its scanner
  (`collect_ignored_locations`) is a separate line-based pass that looks for
  `def ` lines only, so two placements that do suppress are never listed: a
  marker above the first decorator, and any marker on an `async def` (source
  read; recorded in the realignment catalog).
- **The removable-marker report** - markers whose function no longer exceeds the
  threshold - runs automatically at the end of every run that is not `--quiet`,
  with no flag to request or suppress it, and costs a second full walk and parse
  of the path set (source read of `run.rs`).

## Where the tool writes

Three paths; only the snapshot has no override:

- `.complexipy_cache/` in the invocation directory. `--cache-dir` moves it. It
  ships its own `.gitignore` containing `*`, so it does not appear in
  `git status`.
- `complexipy-snapshot.json` in the invocation directory. **No flag redirects
  this**, and a passing snapshot check rewrites it - see
  [Diff and snapshots](diff-and-snapshots.md).
- Whatever `--output` resolves to.

## Known rough edges

- `--color` is currently inert. Output is always colored; only `--plain` produces
  clean text, and it drops everything but path, name and score.
- `-s file_name` orders by function name in the console while the CSV export
  orders by path.
- `--quiet --ignore-complexity` exits 1 on an over-threshold function that the
  same run without `--quiet` passes: the quiet path in `output.rs`
  `handle_display` never reads `ignore_complexity`.
- Every non-quiet `--diff-only` run prints "--diff and --diff-only both set",
  because `run.rs` tests the flags after `resolve_diff_flags` has already
  cleared `--diff`. When both really are set, `--diff`'s reference is dropped in
  favour of `--diff-only`'s.
- `--max-complexity-allowed 0` is the strictest setting, not a disabled gate:
  every comparison is `complexity <= max` with no special case for zero
  (`rows.rs` `is_function_passing`). The help text used to say otherwise.
- `--diff` with `--check-script` or `--no-ignore` reports `NEW` entries for
  functions nobody touched - see [Diff and snapshots](diff-and-snapshots.md).
