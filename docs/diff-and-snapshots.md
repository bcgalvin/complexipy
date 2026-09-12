# Diff and snapshots

Two independent ratchets. The diff compares against a git reference; the snapshot
compares against a committed file.

## Diff

```python
from complexipy import compute_diff, has_regressions, DiffStatus, file_complexity

current = [file_complexity(p) for p in paths]
entries = compute_diff(current, "origin/main")

if has_regressions(entries, 15):
    raise SystemExit("Complexity regressions detected")
```

`compute_diff` re-analyzes the reference revision and pairs functions by file and
name. `has_regressions` reports whether any entry regressed past the threshold.
The CLI equivalent is `--diff <ref>`, with `--staged` to compare the index.

`DiffEntry` carries `file_path`, `func_name`, `old_complexity`, `new_complexity`
and `status`, all read-only. Either complexity may be `None`, for a function that
is new or removed.

### The DiffStatus comparison contract

`DiffStatus` has five members - `REGRESSED`, `IMPROVED`, `UNCHANGED`, `NEW`,
`REMOVED` - and is a PyO3 simple enum, not an `enum.Enum` and not a `str`
subclass. This is the single easiest thing to get wrong, because the failure is
silent: a status comparison that never matches means regressions pass unnoticed
rather than raising.

```python
e.status == DiffStatus.REGRESSED   # True
e.status == "REGRESSED"            # False
f"{e.status}"                      # 'DiffStatus.REGRESSED'
```

There is no `.name` and no `.value`, and formatting yields the qualified
`DiffStatus.REGRESSED` rather than `REGRESSED`, so comparing against a string
fails even after formatting. Compare against members. If you need plain names,
build your own mapping.

Note that `complexipy/_complexipy.pyi` declares `DiffStatus` as an `Enum`
subclass. The stub is wrong; the runtime above is authoritative.

## Snapshots

`--snapshot-create` writes `complexipy-snapshot.json` recording every function
currently over the threshold. With a snapshot present, a run:

- passes functions already recorded that have not got worse;
- passes functions that improved, and rewrites the file to drop them;
- fails new functions over the threshold;
- fails recorded functions that got more complex.

`--snapshot-ignore` skips the comparison. The file is meant to be committed.

Two behaviors worth knowing before wiring this into anything:

- **A passing run rewrites the file.** The rewrite is the ratchet - improved
  functions are removed automatically - and it merges, preserving entries for
  files outside the current run. It is not gated behind `--snapshot-create`.
- **The path is fixed.** It is always `complexipy-snapshot.json` in the
  invocation directory, with no flag to redirect it. Combined with the point
  above, running the tool in a directory that already holds a snapshot will
  rewrite that snapshot.

The snapshot format is `{path, file_name, functions: [{name, complexity}]}`. The
other fields on `FunctionComplexity` are `serde(skip)` under the `python` feature
that the shipped wheel always enables.
