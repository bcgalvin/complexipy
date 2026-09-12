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

`compute_diff` re-analyzes the reference revision with `git show` and pairs
functions by file and name (`compute_diff_reports_regressed_and_improved`,
`compute_diff_reports_removed_function` in
`crates/complexipy-core/src/diff/tests.rs`). The reference side is always
analyzed with `check_script` and `no_ignore` off (`analyse_content_to_map`,
source read), whatever the current side used: under `--check-script` every
file's `<module>` entry is `NEW`, and under `--no-ignore` every suppressed
function is `NEW`, so either flag can fail the ratchet on an untouched tree. Two
silent cases: a file or reference that git cannot show makes every function in
that file `NEW` (`compute_diff_marks_new_file_functions`, and
`compute_diff_git_error_skips_file`, which despite its name asserts the `NEW`
entry), and a reference version that fails to parse drops the whole file from
the result (`compute_diff_unparseable_old_content_skips_file`).
`has_regressions` reports whether any entry is `REGRESSED` or `NEW` with a new
complexity above the threshold; a regression that stays at or below it does not
count (`has_regressions_ratchet`).

The CLI equivalent is `--diff <ref>`, with `--staged` to compare the index
instead of the working tree; `--staged` alone compares against `HEAD`
(`resolve_diff_flags_staged_defaults_to_head`). Both replace the threshold gate
with the ratchet, and the staged form fails open when git cannot produce the
comparison - see [CLI](cli.md#exit-codes).

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
build your own mapping. The member comparison is pinned by `RUNTIME_CHECKS` in
`tests/contract/check_stub_contract.py`; the formatting claim rests on a runtime
check at review time, not a test.

`complexipy/_complexipy.pyi` declares `DiffStatus` as a plain class with `Final`
members and says the same in its docstring, so the stub and the runtime agree
here.

## Snapshots

`--snapshot-create` writes `complexipy-snapshot.json` recording every function
currently over the threshold. With a snapshot present, a run:

- passes functions already recorded that have not got worse;
- passes functions that improved, and rewrites the file with their new score -
  an entry leaves the file only once its function is at or below the threshold;
- fails new functions over the threshold;
- fails recorded functions that got more complex.

The comparison is against the recorded score, not the threshold, so raising the
threshold between runs lets a recorded regression drop out on the next passing
rewrite (source read of `handle_snapshot_watermark` and
`create_snapshot_file_shared`; `utils/snapshot/tests.rs` pins the pass and fail
cases).

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
