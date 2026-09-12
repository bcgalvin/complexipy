# Python API

`complexipy/__init__.py` re-exports the extension module. Its `__all__` is a
compatibility promise: internal refactors keep these names and signatures stable,
and new exports belong in `__all__` and on this page.

## Functions

```python
code_complexity(code: str, check_script: bool = False,
                no_ignore: bool = False) -> CodeComplexity
```

Analyze a source string.

```python
file_complexity(file_path: str, check_script: bool = False,
                no_ignore: bool = False) -> FileComplexity
```

Analyze one file. This is a Python wrapper over the native entry point. The
result's `path` field is relative to the current working directory when the file
lies beneath it, and is the bare file name otherwise. The native
`_complexipy.file_complexity(file_path, base_path, check_script, no_ignore)`
takes the base directory explicitly instead. (Source read of
`complexipy/__init__.py` and `runner.rs` `file_complexity_shared`.)

```python
collect_all_ignored_locations(
    paths: list[str], exclude: list[str], invocation_path: str = ".",
) -> tuple[list[IgnoredLocation], list[str]]

collect_removable_ignored_locations(
    paths: list[str], exclude: list[str], max_complexity_allowed: int,
    invocation_path: str = ".",
) -> tuple[list[RemovableIgnore], list[str]]
```

Find `# complexipy: ignore` and `# noqa: complexipy` markers, and the subset whose
function no longer exceeds the threshold. A bare `# noqa` is **not** recognized.

Both return a **two-tuple**: the results, and a list of paths that could not be
processed. Per-file failures are reported rather than aborting the walk, so
ignoring the second element silently discards them. `invocation_path` is
accepted and ignored: `runner.rs` binds it as `_invocation_path`, relative
`paths` resolve against the process working directory, and each result's `path`
is relative to the parent of the directory you passed, or to a file's own parent
(source read; the stub describes the same behavior).

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
This is not a promise that every invalid API call raises `ValueError`: argument
conversion can fail before analysis, and the Python wrapper resolves paths first.

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

**The enums are not `enum.Enum`.** `RuleCategory`, `Applicability` and
`DiffStatus` are PyO3 simple enums: the MRO is `(cls, object)`, `.name` and
`.value` raise `AttributeError`, and the class is not iterable. Compare members
directly; to recover a name, build a mapping with `dir()`. The stub declares them
as plain classes with typed members. One typing gap remains: it permits
zero-argument enum construction even though the runtime rejects it. It also
permits subclassing `LineComplexity`, which the runtime rejects. Do not rely on
those operations; the result-constructor checks below do not cover them.

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

Result attributes are read-only and the stub exposes them as getter-only
properties. List-valued getters return fresh Python lists; mutating one does
not alter the result. The installed-wheel runtime checks verify this behavior.

The installed-wheel contract checks all eight result types: `result_usage.py`
checks getter types, `assign_results.py` checks rejected assignments, and
`construct_results.py` checks rejected construction. `RUNTIME_CHECKS` compares
native getter names and value types to the installed stub, checks assignment
rejection and list-copy behavior, and checks that constructors reject empty,
correctly typed positional and field-keyword calls. `assign_readonly.py` separately covers `DiffEntry`.

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
