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

Analyze one file. This is a Python wrapper over the native entry point, and it
resolves `path` relative to the current working directory, or to the basename.
The native `_complexipy.file_complexity` accepts an explicit base path instead.

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
ignoring the second element silently discards them.

```python
compute_diff(current: list[FileComplexity], reference: str) -> list[DiffEntry]
has_regressions(entries: list[DiffEntry], max_complexity: int) -> bool
```

See [Diff and snapshots](diff-and-snapshots.md).

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
of those types has one - constructing any of them raises `TypeError`. `DiffEntry`
is the exception and is genuinely constructible. Every attribute on every type is
read-only, though the stub declares them writable. Do not write consumer code that
depends on either.

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
