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
(source read; the stub's description of the parameter is wrong).

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

Every failure inside the extension is a plain `ValueError` carrying the Rust
error string: `code_complexity` on a syntax error, `file_complexity` on a missing
file or a path that is not a readable file (`TestErrors` in `tests/main.py`). The collectors do not raise for
a missing path - it lands in the second tuple element
(`tests/test_collector_failures.py`) - and `compute_diff` never raises; an
unknown reference makes every function `NEW` (`compute_diff_git_error_skips_file`
in `crates/complexipy-core/src/diff/tests.rs`). The stub and the wrapper
docstring promise `SyntaxError`, `FileNotFoundError`, `PermissionError` and
`UnicodeDecodeError`; none of those is ever raised.

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

## Enums, and what the stub still gets wrong

**The enums are not `enum.Enum`.** `RuleCategory`, `Applicability` and
`DiffStatus` are PyO3 simple enums: the MRO is `(cls, object)`, `.name` and
`.value` raise `AttributeError`, and the class is not iterable. Compare members
directly; to recover a name, build a mapping with `dir()`. The stub declares them
correctly as plain classes with typed members.

`Applicability` on a plan is the **rule's declared ceiling**, not what that plan
achieved. A rule declaring `MachineApplicable` can still emit help text with no
suggestion, and the console renderer prints the plan's applicability in the header
and the suggestion's in the body. Check `suggestion is not None` first, then read
`suggestion.applicability`.

**Still wrong: the constructors do not exist.** The stub declares `__init__` for
`CodeSuggestion`, `LineComplexity`, `RefactorPlan`, `FunctionComplexity`,
`FileComplexity`, `CodeComplexity`, `IgnoredLocation` and `RemovableIgnore`. None
of those types has one - constructing any of them raises `TypeError`
(`tests/contract/check_stub_contract.py` `RUNTIME_CHECKS` pins `LineComplexity`;
the others follow from the absence of any `#[new]` in `classes.rs`). `DiffEntry` is the
exception and is genuinely constructible. Every attribute on every type is
read-only, though the stub declares them writable; the contract case
`assign_readonly.py` pins this for `DiffEntry` only. Do not write consumer code
that depends on either.

**Also wrong: the exception docstrings** (see [Exceptions](#exceptions)), the
collectors' `invocation_path` description (see above), and the
`additional_refactor_plans` docstring, which mentions only the cap - see
[Refactor rules](rules.md#how-plans-are-selected).

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
