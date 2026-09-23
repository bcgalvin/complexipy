# CLI

`complexipy [OPTIONS] [PATHS]...`. Every option carries help text, so
`complexipy --help` is authoritative for flag names and accepted values. It
prints no defaults: every option is optional there and the defaults are applied
in `resolve_config` (`utils/config.rs`) - threshold 15, `sort` `asc`, `color`
`auto`, every switch off. This page covers the behavior that a flag list cannot
convey.

## Configuration discovery

Config is read from the **first existing candidate** in the invocation directory:

1. `complexipy.toml`
1. `.complexipy.toml`
1. `pyproject.toml`, `[tool.complexipy]`

**First hit wins outright - the files are not merged, and there is no upward
search** (`crates/complexipy-cli/src/utils/toml/tests.rs`). A candidate that
cannot be read or parsed is fatal, even when CLI paths/options are supplied:
the error goes to stderr and analysis does not start. A valid `pyproject.toml`
without `[tool.complexipy]` means no config. The tests
`a_malformed_complexipy_toml_stops_discovery`,
`unreadable_candidates_fail_instead_of_falling_through` and
`malformed_later_candidates_fail_closed` pin these distinctions. Three
consequences worth internalizing:

- If a `complexipy.toml` exists, a `[tool.complexipy]` block in the same
  `pyproject.toml` is never read. This repository keeps only `complexipy.toml`;
  the loader still supports `[tool.complexipy]` for other projects.
- Discovery is relative to the **invocation directory**, not the analyzed path.
  Running the tool from outside a target means the target's own thresholds are
  silently ignored.
- A typo that breaks `complexipy.toml` cannot silently select another threshold.
  A final missing or empty path list is also an error, regardless of whether a
  config loaded. CLI paths can supply a list omitted by valid config
  (`toml_empty_or_missing_paths_requires_cli_paths`). A nonempty requested
  directory with no files selected by filters remains a valid empty analysis
  (`a_valid_filtered_empty_population_succeeds` in `run/tests.rs`).

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

## Path resolution and population

Relative targets resolve from the invocation directory, normally the process
working directory. The directory must exist; an invalid invocation root fails
the run. Roots and existing input paths are canonicalized, including symlinks.
Analysis and marker reports use the same file identity: paths beneath the root
are root-relative, and outside-root paths are absolute. Failed targets are
reported as plain absolute resolved paths, canonical when they exist. Failure
entries never append diagnostic text; an invalid exclusion pattern identifies
the walked directory, and a walker error without a path identifies its root.

Directory discovery selects `.py` files that survive hidden-file, `.ignore`,
`.gitignore` (inside a Git repository) and `--exclude` filtering. Exclusion globs
are relative to each walked directory, not the invocation root. **Explicit file
arguments bypass all those discovery filters**, including the `.py` extension
filter. Overlapping inputs are not deduplicated.

Core `tests/runner_paths.rs` pins these populations and failure paths;
`run/tests.rs` pins relative-directory lookup, an exclusion changing the
threshold gate's result, and matching analysis/marker JSON identities for
in-root and outside-root targets (including symlink aliases). Emitted errors
from both walkers, including ignore-rule errors attached to directory entries,
are retained alongside good rows. `tests/test_population_failures.py` pins real
unreadable-directory errors through both Python collectors and CLI execution.
Failure paths are deduplicated for CLI diagnostics, never successful rows.
The dependency's silent ignore-file I/O failures remain a limitation below.

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
- **1** - a gate failed, or the run itself failed: a missing/empty path list,
  malformed/unreadable discovered config, a `cache-dir` that is not a non-empty
  string, an unwritable output, or an
  invocation root that is not an existing directory. `run_at` returns the same
  failure code for both (source read of `run.rs`;
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
forces the threshold check to pass while leaving the others intact, in both
quiet and normal modes (`quiet_and_normal_runs_honor_ignore_complexity`).
The path gate includes analysis and both requested/automatic collector failures;
quiet mode changes presentation, not the set of checks.

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
  path (`utils/ignored.rs`; `utils/ignored/tests.rs`). A complete empty collection
  writes `[]`, replacing any preceding report. Failed/partial collection removes
  that requested JSON file and reports failure, rather than presenting stale or
  partial data as a complete inventory. This completeness check belongs to the
  marker collector: an analysis-only parse failure still fails the run but does
  not invalidate a completed marker inventory. The full output-format selection
  is validated before marker output; analysis JSON cannot use the same path as
  `complexipy-ignored.json`. No marker JSON file is touched when the report or
  JSON output is not requested. Its scanner
  (`collect_ignored_locations`) is a separate line-based pass that looks for
  `def ` lines only, so two placements that do suppress are never listed: a
  marker above the first decorator, and any marker on an `async def` (source
  read; recorded in the realignment catalog).
- **The removable-marker report** - markers whose function no longer exceeds the
  threshold - is collected automatically in both normal and quiet runs, but
  displayed only outside quiet mode. It has no flag to disable it, and costs a
  second walk of the path set plus parsing of files containing recognized
  markers (`run.rs`, `collect_removable_ignores_from_file` in core `runner.rs`).
  Its failures contribute to the same path gate as analysis and the all-marker
  report; a top-level collection error is fatal rather than an empty report.

## Where the tool writes

Three paths; only the snapshot has no override:

- `.complexipy_cache/` in the invocation directory. `--cache-dir` moves it. It
  ships its own `.gitignore` containing `*`, so it does not appear in
  `git status`.
- `complexipy-snapshot.json` in the invocation directory. **No flag redirects
  this**, and a passing snapshot check rewrites it - see
  [Diff and snapshots](diff-and-snapshots.md).
- Whatever `--output` resolves to.

On an incomplete analysis or collector result, snapshot creation/watermark
updates and previous-function cache replacement are skipped, in both display
modes. Existing state stays byte-identical; no snapshot baseline or cached delta
is used for that partial display. Good rows may still be exported, accompanied
by stderr failure diagnostics and exit 1. Analysis exports have no embedded
completeness flag: consumers must check the exit code. A complete over-threshold
analysis is not frozen merely because its threshold gate fails. `run/tests.rs` pins new
state prevention, existing-state byte preservation and complete-run cache updates.

## Known rough edges

- The `ignore` dependency suppresses I/O errors reading ignore files internally.
  An unreadable `.ignore` can therefore leave its rules unapplied without a
  failed-path entry, even though emitted traversal/ignore-rule errors are now
  reported. Failure reporting is not yet proof of complete filter validation;
  this is recorded separately in the maintenance catalog.

- `--color` is currently inert. Output is always colored; only `--plain` produces
  clean text, and it drops everything but path, name and score.
- `-s file_name` orders by function name in the console while the CSV export
  orders by path.
- Every non-quiet `--diff-only` run prints "--diff and --diff-only both set",
  because `run.rs` tests the flags after `resolve_diff_flags` has already
  cleared `--diff`. When both really are set, `--diff`'s reference is dropped in
  favour of `--diff-only`'s.
- `--max-complexity-allowed 0` is the strictest setting, not a disabled gate:
  every comparison is `complexity <= max` with no special case for zero
  (`rows.rs` `is_function_passing`). The help text used to say otherwise.
- `--diff` with `--check-script` or `--no-ignore` reports `NEW` entries for
  functions nobody touched - see [Diff and snapshots](diff-and-snapshots.md).
