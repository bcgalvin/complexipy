# CLI

`complexipy [OPTIONS] [PATHS]...`. Every option carries help text, so
`complexipy --help` is authoritative for flag names and defaults; this page covers
the behavior that a flag list cannot convey.

## Configuration discovery

Config is read from the **first** of these found in the invocation directory:

1. `complexipy.toml`
1. `.complexipy.toml`
1. `pyproject.toml`, `[tool.complexipy]`

**First hit wins outright - the files are not merged, and there is no upward
search.** Two consequences worth internalizing:

- If a `complexipy.toml` exists, a `[tool.complexipy]` block in the same
  `pyproject.toml` is never read. This repository has both; only the former
  applies.
- Discovery is relative to the **invocation directory**, not the analyzed path.
  Running the tool from outside a target means the target's own thresholds are
  silently ignored.

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

CLI arguments override the file. Three options are CLI-only with no config key:
`--plain`, `--suggest-refactors`, `--top`.

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
- **1** - a gate failed.
- **2** - the run itself failed.

Which gates apply depends on the flags. Normally the result is the threshold
check, the path check and the snapshot check. **`--diff-only` drops the threshold
check entirely**, so a run can exit 0 with functions over the limit as long as
nothing regressed against the reference. `--ignore-complexity` forces the
threshold check to pass while leaving the others intact.

## Inline ignores

Two markers are recognized on a function's `def` line:

```python
# complexipy: ignore
# noqa: complexipy
```

A bare `# noqa` is **not** recognized. An ignored function is excluded from
results entirely rather than reported with a zero score. Decorated functions are
handled, with the marker on the `def` line.

`--no-ignore` disregards every marker. `--report-ignored` lists markers whose
function no longer exceeds the threshold, so they can be removed; it writes
`complexipy-ignored.json` only when `--output-format json` is also set, placing it
beside the resolved JSON output path.

## Where the tool writes

Three paths, only one of which is redirectable:

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
