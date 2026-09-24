# Python API

`complexipy/__init__.py` defines the public package surface over the extension.
Keep `__all__`, callable signatures and this page aligned. Interface changes
are coordinated with the local parent consumer rather than hidden behind
compatibility layers.

## Functions

```python
code_complexity(code: str, check_script: bool = False,
                no_ignore: bool = False) -> CodeComplexity
```

Analyze a source string.

```python
file_complexity(file_path: str, check_script: bool = False,
                no_ignore: bool = False, *,
                base_path: str = ".") -> FileComplexity
```

Analyze one file relative to `base_path`, an existing directory resolved from
the process working directory. Relative `file_path` values are looked up under
that root; absolute inputs remain absolute. Existing roots and inputs are
canonicalized, including symlinks. Results beneath the root have root-relative
paths; results outside it keep their canonical absolute paths. The default
root is the working directory, not an outside file's parent. Both `file_path`
and `base_path` must be strings; `pathlib.Path` objects are not accepted.

For an external-CWD survey, use
`file_complexity("src/example.py", base_path="/absolute/target")`. The native
`_complexipy.file_complexity(file_path, base_path, check_script, no_ignore)`
uses the same root semantics; the package wrapper makes `base_path` optional
and keyword-only. `tests/test_path_roots.py` pins public/native agreement and
root handling; core `tests/runner_paths.rs` pins explicit native roots and
`api/tests.rs` pins the Rust convenience API's outside-CWD absolute result.
The installed-wheel contract pins the public signature and result labels.

```python
collect_all_ignored_locations(
    paths: list[str], exclude: list[str], invocation_path: str = ".",
) -> tuple[list[IgnoredLocation], list[str]]

collect_removable_ignored_locations(
    paths: list[str], exclude: list[str], max_complexity_allowed: int,
    invocation_path: str = ".",
) -> tuple[list[RemovableIgnore], list[str]]
```

Find whole-function `# complexipy: ignore` and `# noqa: complexipy` markers, and
the subset whose function no longer exceeds the threshold. A marker with a rule
list, such as `# complexipy: ignore[C007]`, keeps its function in analysis
results and is not reported here. A bare `# noqa` is **not** recognized. The
Python API has no refactor-rule selection: every rule is active, and inline rule
lists still remove plans (see [Refactor rules](rules.md#selecting-and-suppressing-rules)).

Both return a **two-tuple**: the results, and a list of paths that could not be
processed. Per-file failures are reported rather than aborting the walk, so
ignoring the second element silently discards them. `invocation_path` is the
existing-directory root for both input lookup and result labels, using the
same rules as `file_complexity`'s `base_path`. Explicit and directory-discovered
results identify a file the same way. Failed paths are absolute resolved paths
(canonical when the path exists), with no appended error text. An invalid
exclusion pattern reports the walked directory; emitted traversal errors report
the failing path, or the walk root if the walker supplies no path. Results from
successfully processed files survive those failures. Successful duplicate inputs
remain duplicated, and overlapping requests may repeat failed paths too; callers
must not assume either list has set semantics.

Directory discovery applies Python-extension, hidden/ignore-file and exclusion
filters. Explicit files bypass those discovery filters. These rules, mixed
success/failure results and root-relative lookup are pinned in core
`tests/runner_paths.rs` and `tests/test_path_roots.py`. Real unreadable-directory
failures and plain failure-path strings are pinned in
`tests/test_population_failures.py` and the installed-wheel runtime contract.
Both walkers' emitted errors, including malformed ignore-rule errors attached
to successful entries, are reported. The `ignore` dependency still suppresses
ignore-file I/O errors internally; an empty failure list does not prove every
filter file was read. Marker-recognition gaps are also separate from traversal
completeness. See [CLI rough edges](cli.md#known-rough-edges).

```python
compute_diff(
    current_files: list[FileComplexity], git_ref: str,
    invocation_path: str | None = None,
) -> list[DiffEntry]
has_regressions(entries: list[DiffEntry], max_complexity: int) -> bool
```

`invocation_path` is the directory git runs in; it defaults to the current
working directory (`TestDiff` in `tests/main.py` and `RUNTIME_CHECKS` in the
contract harness pin the two-argument call). See
[Diff and snapshots](diff-and-snapshots.md).

## Exceptions

Native analysis reports reading, UTF-8 decoding and parsing failures as
`ValueError` carrying the Rust error string: `code_complexity` on a syntax error,
`file_complexity` on a missing file or a path that is not a readable file
(`TestErrors` in `tests/main.py`). The stub and wrapper document this mapping.
Invalid file/collector roots also raise `ValueError`, including a missing root
or one that names a file. This is not a promise that every invalid API call
raises `ValueError`: argument conversion can fail before analysis.

The collectors return a missing path in the second tuple element rather than
raising (`tests/test_collector_failures.py`). `compute_diff` does not report Git
failures as exceptions; an unknown reference makes every supplied function `NEW`
(`compute_diff_git_error_skips_file` in `crates/complexipy-core/src/diff/tests.rs`).

## Types

`LineComplexity` - `line`, `complexity`.

`CodeSuggestion` - `replacement`, `applicability`, `description`, `spliceable`.

`RefactorPlan` - `kind`, `title`, `line_start`, `line_end`, `column_start`,
`current_complexity`, `estimated_reduction`, `estimated_complexity_after`,
`reduction_is_measured`, `rule_id`, `category`, `applicability`, `description`,
`explanation`, `suggestion`, `help`.

`FunctionComplexity` - `name`, `complexity`, `line_start`, `line_end`,
`line_complexities`, `refactor_plans`, `additional_refactor_plans`. Methods are
named `Class::method`; script mode adds a `<module>` entry.

`FileComplexity` - `path`, `file_name`, `functions`, `complexity`.

`CodeComplexity` - `functions`, `complexity`.

`IgnoredLocation` - `path`, `line`, `comment`.

`RemovableIgnore` - `path`, `line`, `comment`, `function`, `complexity`.

`DiffEntry` - `file_path`, `func_name`, `old_complexity`, `new_complexity`,
`status`.

`RuleCategory`, `Applicability`, `DiffStatus` - see the note below.

## Enums and result objects

**The enums are standard `enum.Enum` classes.** `RuleCategory`, `Applicability`
and `DiffStatus` are built by the extension with the functional `Enum` API and
report `complexipy` as their module. Each member's `.value` equals its `.name`,
iteration yields members in declaration order, and `cls(value)` or `cls[name]`
returns an existing member. A call without a value raises `TypeError`; an
unknown value raises `ValueError`. Members cannot be reassigned. Each enum has
one Rust definition, in the `complexipy-types` crate, shared by the engine and
the extension. The stub declares them as final `Enum` subclasses whose member
values are their names. `valid_usage.py` checks typed member reads, names,
values, iteration and lookup; `reassign_enums.py` pins rejected member
reassignment in ty. The installed-wheel runtime checks compare each enum's
member names and values, in order, against the installed stub, and check
lookups, rejected calls and rejected reassignment.

`Applicability` on a plan is the **rule's declared ceiling**, not what that plan
achieved. A rule declaring `MachineApplicable` can still emit help text with no
suggestion, and the console renderer prints the plan's applicability in the header
and the suggestion's in the body. Check `suggestion is not None` first, then read
`suggestion.applicability`.

**Result objects come from analysis, not constructors.** `CodeSuggestion`,
`LineComplexity`, `RefactorPlan`, `FunctionComplexity`, `FileComplexity`,
`CodeComplexity`, `IgnoredLocation` and `RemovableIgnore` reject construction
with `TypeError`. The stub uses a required `Never` parameter to reject direct
construction statically; it is not a token callers can obtain or pass at runtime.
`DiffEntry` is the exception and has a real constructor.

All twelve exported native types, including `DiffEntry` and the three enums,
reject subclassing. Their stub declarations are `@final`; `subclass_native.py`
pins each rejection in ty, and the installed-wheel runtime checks attempt to
subclass every exported native type.

Result attributes are read-only and the stub exposes them as getter-only
properties. List-valued getters return fresh Python lists; mutating one does
not alter the result. The installed-wheel runtime checks verify this behavior.

The installed-wheel contract checks all eight result types: `result_usage.py`
checks getter types, `assign_results.py` checks rejected assignments, and
`construct_results.py` checks rejected construction. `RUNTIME_CHECKS` compares
native getter names and value types to the installed stub, checks assignment
rejection and list-copy behavior, and checks that constructors reject empty,
correctly typed positional and field-keyword calls. `assign_readonly.py`
separately covers `DiffEntry`.

`additional_refactor_plans` includes cap drops and post-measurement drops, not all
candidate plans - see [Refactor rules](rules.md#how-plans-are-selected).

## Serialization surfaces

`--output-format json` serializes refactor plans in full. Six fields are
`serde(skip)` under the `python` feature - `line_start`, `line_end`,
`line_complexities`, `refactor_plans` and `additional_refactor_plans` on
`FunctionComplexity`, and `complexity` on `FileComplexity` - and the shipped wheel
always builds with that feature, so the snapshot file carries only
`{path, file_name, functions: [{name, complexity}]}`. CSV, JSON, SARIF and GitLab
each build their own payload rather than serializing these structs whole, so they
are unaffected. `line_complexities` reaches no CLI output format at all; read it
through this API.
