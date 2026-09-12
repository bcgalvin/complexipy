# Design issues and bugs

Catalog of everything the realignment turns up that is not itself realignment
work. Two kinds of entry:

- **Bugs** - small, scoped, and fixable in passing. Addressed as the workstream
  that opens the relevant file comes around, or deferred with a stated reason.
- **Design issues** - tension, friction, or debt that is not clearly a bug. Not
  scheduled. Recorded with enough context to find and evaluate later without
  rediscovering it.

Every entry names where the evidence is. Status values: **fixed** (with the
commit), **assigned** (to a workstream), **open**, **deferred** (with why).

`follow-up-tooling.md` records *removed capabilities* and what a replacement
needs; `explore-results.md` is the evidence trail of the analysis sweep. This
file is the one that stays actionable after `docs/realignment/` is deleted.

## Bugs

### Fixed

- **A relative `--output` resolved against the process CWD, not the invocation
  path.** `crates/complexipy-cli/src/utils/paths.rs` honoured `invocation_path`
  when `--output` was omitted but absolutized a relative `--output` against
  `current_dir()`, and created the directory during path *resolution*. Latent in
  CLI use (`run_cli` defaults the invocation path to `.`), exposed by the paths
  test writing `crates/complexipy-cli/rel-out/` into the checkout on every
  `cargo test`. **Fixed in `b69915b`**: join onto `invocation_path` first.

- **Six scoring rules asserted by prose were pinned by no test.** `raise`, `if`'s
  `else` (increments and nests), `elif` as a sibling clause, `break`/`continue`,
  boolean run grouping, and `with` transparency inside a loop. Two of these were
  stated wrongly on the first draft of `docs/scoring.md` and nothing in the repo
  would have caught a third. **Fixed in `2020084`**: `TestPaperConformance` now
  covers all six.

- **`--help` carried no descriptive text** - no `about`, no `help =` on any of
  21 options, and no doc comments to fall back on because comments are banned.
  **Fixed in `2020084`**.

### Assigned to a workstream

- **`RefactorPlan.references` is permanently empty.** `rules/types.rs:52` is the
  sole writer (`vec![]`), no rule overrides it, and it ships through the FFI
  contract and the JSON export regardless. Its only rendering path,
  `output_plan_references`, becomes unreachable once `doc_url` goes. **D**.

- **Three enums lie in the stub.** `RuleCategory` (`:13`), `Applicability`
  (`:22`) and `DiffStatus` (`:34`) in `complexipy/_complexipy.pyi` are declared
  `(Enum)` with string values. At runtime they are PyO3 simple enums: no
  `.name`, no `.value`, no iteration. The consuming repo already carries a
  `variants()` workaround. **D**.

- **Eight phantom constructors in the stub.** `classes.rs` has zero
  `#[pymethods]` and zero `#[new]`, so only `DiffEntry` is constructible, but the
  stub declares `__init__` for eight types with runtime-impossible Example
  blocks. Every attribute is also declared writable when all are read-only
  (`get_all` generates getters only). **D**.

- **The stub documents a removed feature.** `_complexipy.pyi:717` and `:755`
  describe `paths` as accepting Git repository URLs; git-URL analysis was
  removed in 8.0.0. `AGENTS.md` repeats the claim twice. `cache.rs:130-142`
  `looks_like_remote` normalizes cache keys for github/gitlab URLs that nothing
  can produce. **D** for the stub and dead code, **E** for AGENTS.md.

- **Native docstrings omit two parameters.** The `code_complexity` and
  `file_complexity` docstrings in the stub document only `code` / `file_path`
  and `base_path`, not `check_script` or `no_ignore`. **D**.

- **Dead code.** `crates/complexipy-cli/src/output.rs:158`
  `effective_sort_for_display` has no callers; `utils/snapshot.rs`
  `SnapshotEvaluation.snapshot_result` is computed and tested but never read by
  `run.rs`; `RuleMetadata` derives `Serialize, Deserialize` with no consumer.
  **D**.

- **`crates/complexipy-core/src/lib.rs:16-17`** claims its re-export block
  "mirrors `complexipy/__init__.py`'s `__all__`". It also exports
  `compute_staged_diff` and `run_analysis_shared`, and its `DiffEntry` /
  `DiffStatus` are different types from the `py_diff` ones Python sees. **E**.

- **Two config files at the repo root, one dead.** Discovery is first-hit-wins
  with no merge, so `complexipy.toml` shadows `[tool.complexipy]` in
  `pyproject.toml` entirely. They disagree on `paths`, `exclude`, `failed` and
  `quiet`. **F** retires one.

### Open

- **Statements in a class body that are not functions are scored nowhere.**
  `cognitive_complexity.rs` iterates a `ClassDef` body matching only
  `Stmt::FunctionDef`, and the module accumulator never sees a `ClassDef`.
  `class A:` containing `if x: pass` scores 0; the same `if` at module level
  scores 1. Found while deriving `docs/scoring.md`. Whether class-body control
  flow *should* count is a scoring-contract question, which is why this is
  open rather than fixed.

- **`-s file_name` sorts by function name in the console, by path in CSV.**
  `output/rows.rs:65` sorts `f.name`; `export_tests.rs` proves the CSV path
  sorts by file path. Two implementations of one flag value. Console side
  unverified beyond the source read.

- **Duplicate file headers in console output.** Grouping is by consecutive
  same-path entries, so any sort that interleaves paths repeats a header.
  Probably not specific to `--top` since truncation runs after row building.
  Unverified; reproduce first.

- **`--suggest-refactors` degrades silently on a failed source read.**
  `read_source_lines` ends in `.ok()`, so the caret span and `Original:` snippet
  vanish with no warning. The trigger is narrow - it skips the join for paths
  starting with `/`, so absolute out-of-tree targets are safe - but relative
  targets like `../repos/foo` and Windows-style absolutes are exposed.

- **Two tests that cannot fail the way they claim to.**
  `crates/complexipy-cli/src/run/tests.rs:149-153` `version_flag_handled_by_clap`
  asserts only `result.is_err()`, satisfied by a parse failure as readily as a
  version display. `tests/test_refactor_plans.py:320-327` wraps its whole body in
  `if flatten_plan:`, so it passes with zero assertions if C001 stops firing; the
  neighbouring test shows the right guard.

- **SARIF `informationUri` has no test.** Only `helpUri` is asserted. D removes
  both, so this closes itself, but the pattern - an emitted key nothing checks -
  is worth remembering.

- **Deleting `test_rule_metadata_has_doc_url` orphans a fixture.**
  `tests/fixtures/refactor_plans/metadata_validation.py` is loaded by that test
  alone. Outside `tests/src`, so the corpus total is unaffected. **D** deletes
  both.

### Deferred

- **`--color` is completely inert.** `output/render.rs:16,24,40` computes
  `color_enabled` and nothing reads it but two tests, which give false confidence
  that the flag works. Every renderer calls owo-colors unconditionally;
  `Color::Auto` hardcodes true with no `is_terminal()` check even though the same
  file uses `is_terminal()` for width; `NO_COLOR` and `CLICOLOR` are ignored.
  The only clean-text mode is `--plain`, which drops everything but path, name
  and score, so **the console surface cannot produce machine-readable refactor
  output at all** - a consumer must use `--output-format json`. Deferred past
  the realignment because it is a behavior change that wants its own
  verification, and D changes the JSON schema twice; the two output surfaces
  should not move at once.

## Design issues

### Configuration is resolved against the wrong directory

Config discovery (`crates/complexipy-cli/src/utils/toml.rs:7-16`) joins three
candidate filenames onto the invocation path, first hit wins, no merge, no
upward search, and never consults the analyzed target. A target's own
`[tool.complexipy]` thresholds are silently ignored whenever the tool runs from
outside it - which is exactly the parent project's mandated pattern of running
providers from an external working directory with output redirected. The
parent therefore cannot honour a target's own configuration without copying it.

Resolution is a design choice, not a patch: target-root lookup, upward search,
an explicit `--config` flag, or a documented "config is always the caller's"
stance. Each changes what existing invocations mean.

### The tool writes into the tree it analyzes

Three write locations, two of them not redirectable:

- `.complexipy_cache/` in the invocation directory, shipping its own
  `.gitignore` containing `*` and a `CACHEDIR.TAG`. `git status --porcelain`
  shows nothing after a run, which defeats a git-status-based before/after
  snapshot of the target. `--cache-dir` moves it.
- `complexipy-snapshot.json`, hardcoded to the invocation directory with **no
  override flag**, and rewritten by a passing check (a documented ratchet, not a
  bug - but it means running in a directory that already holds a snapshot
  silently updates it).
- `--output`, which is redirectable and was the subject of the fixed bug above.

The parent treats analyzed targets as read-only inputs. The cache's self-hiding
is the sharpest tension: it is designed to be invisible, and invisibility is
precisely what a target-safety snapshot cannot tolerate.

### Serialized shape depends on a Cargo feature

Every `serde(skip)` in `classes.rs` is `#[cfg_attr(feature = "python", serde(skip))]` - five fields on `FunctionComplexity`, one on `FileComplexity`.
The shipped extension always enables `python`, so its snapshot format is
`{path, file_name, functions: [{name, complexity}]}`. A standalone
`cargo build -p complexipy-cli` binary writes five extra fields per function
for the same input. Feature unification decides which shape a given build
gets, and the divergence compiles cleanly in both directions, so the compile
checks that exist to catch feature-gate mistakes cannot see it. The surviving
`cargo check -p complexipy-cli --locked` is kept for this reason but only
proves the shape compiles.

A serialization contract should not be a side effect of a bindings feature.
The honest fix is to make the skip unconditional or to make it a runtime
choice, and either one is a format change.

### `applicability` conflates what a rule can do with what a plan did

`plan.applicability` always comes from the rule's metadata; no rule sets it
per plan. C002, C005 and C007 declare `MachineApplicable` and then fall back to
help-only text in several documented cases. The console renderer prints the
plan's applicability in the header and the suggestion's in the body, so a plan
can display as safe to apply directly above a `Help:` block with nothing to
apply. Consumers have to know to test `suggestion is not None` first.

The model has one field doing two jobs. A rule-level capability and a
plan-level outcome are different things, and the `references` and `doc_url`
fields being dead weight on the same struct suggests `RefactorPlan` grew by
accretion.

### The exit code means different things under different flags

`ExitReport::success()` in `crates/complexipy-cli/src/types.rs` is normally the
conjunction of the threshold, path and snapshot checks. `--diff-only` drops
the threshold check entirely, so a run exits 0 with functions over the limit
as long as nothing regressed. `--ignore-complexity` forces the threshold check
to pass but keeps the others. Neither is wrong, but a consumer scripting on the
exit code has to know which gate set the flags selected, and nothing in the
output says.

### The scoring contract lived only in prose, and the prose drifted

`docs/understanding-scores.md` scored `with` as an increment that nests and
`match` as free; the conformance tests said the opposite, and a worked example
totalled 6 where the algorithm gives 4. Nobody noticed because a wrong scoring
page is indistinguishable from a right one until someone recomputes it. C
replaced it with a page derived from `TestPaperConformance` and pinned the
previously unguarded rules, but the general hazard remains: any future
statement of the algorithm has to be derived from the tests, never written
alongside them.

A related note that is intended behavior, recorded so nobody files it as a
bug: `elif` and `else:` followed by `if` are the same AST shape (`orelse=[If]`)
and score 2 and 4 respectively. The scorer treats an `elif` chain as sibling
clauses, which is what the paper prescribes.

### Local verification lost an isolation property

The deleted CI lint job ran ruff and ty in an environment with the extension
deliberately *not* installed, and asserted that through `importlib.metadata`,
so ty always read the stub. The standing gate's `uv run ty check .` runs with
the editable project installed, so ty reads the native module. The contract
harness (`tests/contract/check_stub_contract.py`) is what still checks
stub/runtime parity, and it covers four cases against eighteen `__all__`
members. The `verify` skill should say this plainly; a rebuilt CI should
restore the dependency-only environment.

### The parser is a network-fetched git dependency on a mutable tag

`ruff_python_parser` and `ruff_python_ast` are declared as
`{ git = "https://github.com/astral-sh/ruff.git", tag = "0.12.9" }`.
`Cargo.lock` pins the resolved revision, so `--locked` builds are
reproducible, but a fresh clone cannot build offline and `cargo update` follows
a tag that can be re-pointed. For a fork whose premise is local source builds,
this is the largest reproducibility exposure in the tree.

### Build tooling has two floors and a partial cache key

PEP 517 requires `maturin>=1.9.4,<2.0`; the `dev` group declares `>=1.8.3`.
`maturin develop` and the contract harness use the dev-group binary, `uv sync`
uses the PEP 517 one - two floors on two paths, and the lower one predates
CPython 3.14. Separately, `[tool.uv] cache-keys` lists Rust sources and
manifests but no Python source; under an editable install that is probably
harmless, but the omission is anomalous and the fix is free.

### Test coverage has structural holes

- Nothing exercises the analysis walk through `run_analysis_shared`.
  `tests/main.py` reimplements the walk with `rglob` and calls
  `file_complexity` per file; every `run/tests.rs` case passes a single file.
  The collector walk is covered; the analysis walk is not.
- Exclusion has no analysis-path coverage. The CI checks that appeared to
  cover it passed `--ignore-complexity`, so their exit code could not fail on a
  non-matching glob.
- The contract harness leaves twelve of eighteen `__all__` members unproven.
- `tests/src/exclude_dir/` exists only to feed the deleted CI checks but must
  stay: `tests/main.py` asserts a corpus total that includes it.

### Removed features leave residue

Git-URL analysis was removed in 8.0.0. Its residue survived in the stub
docstrings, two `AGENTS.md` claims, and `cache.rs::looks_like_remote`. The
same pattern will apply to `doc_url`, `references`, the wasm target and the
`runner` feature unless each removal sweeps for its own references. Worth a
grep step in the `verify` skill.

### Provenance after the realignment

Decision 5 bumps to `8.1.0` and abandons the `+rcq.N` local-version segment.
The moment upstream ships its own `8.1.0`, a fork build and an upstream build
report the same version with different behavior, and there is no
`__version__` attribute or other runtime marker to tell them apart
(`importlib.metadata.version` and `--version` both report only the number).
`sync-upstream` will also conflict in four files because A collapsed a feature
upstream still has. Both are known and accepted; both will need attention the
first time upstream is pulled.

### Output formats without consumers

`gitlab` and `sarif` existed for GitLab Code Quality and GitHub code scanning.
B removed the last CI, so neither has a consumer unless the parent ingests
them directly. Decision pending in D, which opens `sarif.rs` regardless.
