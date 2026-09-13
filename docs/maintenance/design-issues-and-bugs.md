# Design issues and bugs

Maintenance catalog, carried forward in full from the fork realignment.
Two kinds of entry:

- **Bugs** - scoped behavior defects. Address them when the requested work
  covers them, or defer with a stated reason.
- **Design issues** - tension, friction, or debt that is not clearly a bug.
  Recorded with enough context to find and evaluate later without rediscovering
  it; an entry alone does not schedule implementation.

Every entry names where the evidence is: source by path and enclosing symbol
(function, test, struct field, constant), markdown by section heading, never by
line number. This file outlives the realignment, and a line reference goes stale
silently while a symbol reference fails loudly. Status values: **fixed** (with
the commit or named change), **open**, **deferred** (with why).

[Current work](../current-work.md) owns the active sequence, fork/parent
responsibilities, dependencies and acceptance criteria. This catalog owns the
underlying issue details; update both when a planned issue is resolved.

[`follow-up-tooling.md`](follow-up-tooling.md) records removed capabilities and
possible replacements, not an implementation checklist. The completed tracker
and original `docs/realignment/explore-results.md` evidence trail remain in Git
history at `e1a17fb`. Historical workstream labels below identify that work;
they are not assignments to still-running workstreams.

## Parent-consumer priorities

The parent is a local consumer, not a distribution target. The usage below was
checked at parent commit `00b228a`. Its `docs/providers.md` (Supported surfaces
and qualification) excludes Complexipy from the main runner. Current use is
through direct exploration in
`.agents/skills/complexipy-explore/SKILL.md` and the native-record consumer in
`scripts/complexipy_analysis/reduced_record.py` (`main`, `plan_native`). Parent
paths in this section are relative to `recsys-code-quality`, not this fork.

The parent pins the 8.1.0 release and its current wheel; its serializer no longer
reads `doc_url` or `references`. A source change alone neither updates that wheel
nor installs it in a provider environment. On a requested refresh, check the
exact wheel contract and the real native-plan serializer together. Synthetic
parent tests do not establish native API compatibility on their own.

These are prioritization principles, not a second execution order. Follow
[the current work plan](../current-work.md#fork-sequence) for the selected
batches; neither document authorizes unrelated implementation:

1. **Protect current file/plan consumption.** Preserve exact scoring and line
   attribution, complete native plan fields and explicit analysis failures.
   Address expression-walker omissions with deliberate scorer-contract changes;
   keep help-only plans distinct from applicable replacements. The parent owns
   source review and semantic judgments, not this analyzer.
1. **Make external-CWD surveys reliable.** Path resolution and analysis-walk /
   exclusion coverage are addressed by the path-root batch below; suppression
   parity and silent walker-entry failures remain. A caller running a survey
   needs to distinguish a filtered or failed population from a complete
   inventory. Include the empty-path config and quiet/ignore-complexity defects
   in the next CLI population/gate batch.
1. **Before trusting native comparisons, make failures explicit.** Cover path
   pairing, missing refs, reference parse failures, staged scope and matching
   analysis flags. Public `file_complexity(base_path=...)` now preserves
   repository-relative identity, but does not repair diff error handling. An
   empty comparison or all-`NEW` is not proof of a valid comparison. This is a
   prerequisite for reliable use of the optional diff surface, not a claim that
   the parent runner uses it today.
1. **Add controls only for demonstrated friction.** Explicit config selection
   or an independent snapshot path may help a requested workflow. Do not turn
   that into automatic target-config discovery: caller-selected scope and
   read-only inputs are current parent requirements. Standalone-CLI snapshot
   parity, benchmarks and extra output formats are not current adoption gates.

Related issues below retain the source anchors and unresolved design choices.
No CI, wheel archive, general release framework or parent-runner integration is
scheduled by this list. The removed-tooling inventory is not a competing roadmap.

## Bugs

### Fixed

- **Collector roots were ignored and file labels lost repository identity.**
  Fixed in the path-root batch: `runner.rs` `resolve_root` / `resolve_input`
  now give `run_analysis_shared`, `file_complexity_shared` and both collectors
  one canonical existing-directory root for relative lookup and result labels.
  Collectors previously resolved through process CWD and labeled against each
  input's parent; public file analysis collapsed out-of-CWD files to basenames.
  Public `file_complexity` now accepts keyword-only `base_path="."`, with the
  same semantics as the required native base. Files inside the root are
  root-relative, outside files canonical absolute; failures are absolute
  resolved paths and invalid roots fail the call. No legacy basename fallback
  remains. `tests/test_path_roots.py`, `runner_paths.rs`, Rust API tests and
  the installed-wheel cases pin defaults, explicit bases and failures.
  `run/tests.rs` `analysis_and_marker_json_share_canonical_paths` pins matching
  CLI JSON identities. This source change does not update the parent's wheel.

The enum/subclass and test fixes below landed with the local build and
verification contract change.

- **Enum construction and subclassing were over-promised by the stub.**
  `RuleCategory()`, `Applicability()` and `DiffStatus()` passed ty but raised
  `TypeError`. All twelve exported native types reject subclassing, not only
  the previously probed `LineComplexity`. The three enum stubs now have the
  required `Never` construction guard, and all twelve types are `@final`.
  `construct_enums.py` and `subclass_native.py` in `tests/contract/cases/`
  check rejected calls and bases; `valid_usage.py` checks typed members.
  `RUNTIME_CHECKS` verifies these limits against the installed native types
  and compares every enum's member names against the installed stub.
  The new negative cases failed against the unchanged parent 8.1.0 wheel,
  demonstrating the gap without changing that artifact. Static checks are
  stricter for code that already failed at runtime; native behavior and the
  public export list are unchanged.

- **Two tests could pass without checking their claimed behavior.**
  `crates/complexipy-cli/src/run/tests.rs` `version_flag_handled_by_clap` now
  requires `DisplayVersion`, exit code 0, stdout and the current Cargo package
  version text instead of accepting any clap error. In
  `tests/test_refactor_plans.py`, the renamed
  `test_flatten_condition_produces_nonempty_help` requires a C001 plan before
  asserting help-only output with nonempty help. The former
  `test_code_generation_produces_nonempty_snippets` skipped its assertions when
  no C001 plan existed, which was already the case: its fixture emitted C007
  instead. Its replacement, `flatten_condition_help.py`, has a fourth nested
  condition and intervening statements to meet C001's guards without a
  collapsible-if chain.
  This changes a rule fixture, not the scorer or the `tests/src/` corpus.

- **Generated changelog ended with a blank line at EOF.** The first G
  regeneration failed `git diff --check`. Root `cliff.toml` now normalizes only
  trailing newlines to one newline through its postprocessor. Native generation
  preserves the commit population and indented breaking descriptions while
  passing the whitespace check; no wrapper or formatter was added.

- **Changelog message handling was unspecified.** G preparation now uses
  local git-cliff 2.14.1 and root `cliff.toml`. A manual native-tool check at
  `5ec3c37` accounted for all 29 fork SHAs, including merge `9926391` and the
  five empty-body commits (`fa174cc`, `c613bdb`, `f6d4a14`, `9926391`,
  `5999826`). D/E migration descriptions survive; session trailers do not render.
  This is a manual check, not an automated contract suite. G finalization
  replaces the inherited changelog for 8.1.0; the direct maintenance procedure
  is in `docs/changelog.md`.

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

- **`RefactorPlan.references` was permanently empty** and `doc_url` pointed at a
  site this fork does not own. Both removed in D, along with the SARIF
  `informationUri`/`helpUri` constants and the console `References:` renderer.

- **Three stub enums claimed to be `enum.Enum`.** `RuleCategory`,
  `Applicability` and `DiffStatus` are PyO3 simple enums with no `.name`, no
  `.value` and no iteration. The stub now declares them as plain classes with
  typed members, and says so. Fixed in D.

- **The stub documented a removed feature.** Two collector docstrings described
  `paths` as accepting Git repository URLs, removed in 8.0.0. Fixed in D; the
  matching `AGENTS.md` claims and `cache.rs` `looks_like_remote` were handled
  subsequently by E, as recorded below.

- **Four consumer-visible removals needed a machine-readable record.** Dropping
  `doc_url` and `references` changes the `--output-format json` schema and the
  Python `RefactorPlan`, and removes SARIF `helpUri`/`informationUri` and the
  console `References:` block. The removals were recorded in D's
  `BREAKING CHANGE:` footer, which git-cliff now includes in the generated
  `CHANGELOG.md`.

- **`tests/main.py` was committed unformatted in `2020084`.** `ruff format --check` was omitted from that workstream's final gate, so the conformance
  tests added there shipped with over-long lines. Formatting corrected in D. The
  gate is only worth running in full.

- **`test_rule_metadata_has_doc_url` was slated for deletion with its fixture.**
  The tracker called for removing the test and the fixture only it loads,
  `tests/fixtures/refactor_plans/metadata_validation.py`. The test asserted more
  than `doc_url` - it guards that `RuleMetadata` identity fields reach the plan
  at all, a real historical regression. **Resolved in `39e1bd5`**: renamed
  `test_rule_metadata_reaches_the_plan`, its `doc_url` assertions replaced with
  absence assertions, the fixture still in use.

- **SARIF `informationUri` had no test.** Only `helpUri` was asserted, so
  removing either key was silent. **Fixed in `39e1bd5`**, which removed both and
  made `sarif/tests.rs` assert their absence. The pattern - an emitted key
  nothing checks - is worth remembering.

- **Six `--help` strings described behavior the code does not have.** `--diff`
  and `--diff-only` were backwards: `--diff` replaces the threshold gate with the
  regression ratchet and `--diff-only` never touches the exit code
  (`run/tests.rs` `diff_clean_exits_success`,
  `diff_only_leaves_the_threshold_gate_in_place`). `--sort` claimed to accept
  `name`; the value is `file_name`. `--max-complexity-allowed` claimed `0`
  disables the gate; `rows.rs` `is_function_passing` has no special case, so `0`
  is the strictest setting. `--suggest-refactors` claimed plans only for
  functions above the threshold; `render.rs` `output_file_entries` renders them
  for every listed row. `--report-ignored` claimed to report markers whose
  function no longer needs them; `handle_report_ignored` lists every marker, and
  the removable report is the automatic one. All six were written in C
  (`2020084`); `docs/cli.md` and this catalog's exit-code design issue repeated
  the `--diff` inversion. **Fixed in the review of C's pages**, which also
  corrected the documents.

- **`compute_diff` required `invocation_path` at runtime.** The one
  `#[pyfunction]` in `crates/complexipy-python/src/lib.rs` without a
  `#[pyo3(signature = ...)]`; PyO3 no longer defaults a trailing `Option`
  argument, so `compute_diff(current, "main")` raised `TypeError` while the
  stub, both docs pages and the upstream guide showed the two-argument call.
  Nothing called it in any test. **Fixed in the review of C's pages** with the
  signature attribute; `TestDiff` in `tests/main.py` and `RUNTIME_CHECKS` in the
  contract harness pin the two-argument call.

- **A third test that could not fail the way it claimed.**
  `select_non_overlapping_never_returns_overlapping_plans` in
  `rules/registry/tests.rs` gave its widest plan the highest effectiveness, so
  the selection collapsed to one plan and the pairwise assertion loop ran zero
  times. **Fixed in the review of C's pages**: the fixture now yields two
  survivors, asserted by id, and a sibling test pins that a spliceable plan
  beats a help-only plan of higher effectiveness, which nothing had pinned.

### Fixed in workstream E

Recorded by `refactor(fork)!: align identity and Python 3.14 contracts`.

- **Eight phantom constructors in the stub.** `classes.rs` has zero
  `#[pymethods]` and zero `#[new]` - the only `#[new]` in the tree is
  `DiffEntry`'s in `crates/complexipy-python/src/lib.rs` - so only `DiffEntry` is
  constructible, but the stub declares `__init__` for `CodeSuggestion`,
  `LineComplexity`, `RefactorPlan`, `FunctionComplexity`, `FileComplexity`,
  `CodeComplexity`, `IgnoredLocation`, and `RemovableIgnore`, with
  runtime-impossible Example blocks. Every attribute is also declared writable
  when all are read-only (`get_all` generates getters only; `DiffEntry` is the
  one type using `@property`). **Fixed in E**: getter-only properties, a
  typing-only `Never` construction guard, and removal of impossible examples.
  The installed-wheel contract now checks positive getter types and rejected
  assignment/construction for all eight result types, with runtime agreement.
  Runtime checks also compare native getter names and value types with the
  installed stub and verify list getters return independent copies.
  `docs/python-api.md` was updated in the same change.

- **Git-URL residue in code and AGENTS.md.**
  `crates/complexipy-cli/src/utils/cache.rs` `looks_like_remote`, called from
  `normalize_target`, passes github/gitlab URL targets through as cache keys when
  nothing can produce one; `AGENTS.md` credits `runner.rs` with git-URL walking
  twice (the Project Structure tree and the "Rust core" bullet).
  **Fixed in E**: removed the helper and its branch, and corrected both
  `AGENTS.md` descriptions.

- **Native docstrings omit two parameters.** The `code_complexity` and
  `file_complexity` docstrings in the stub document only `code` / `file_path`
  and `base_path`, not `check_script` or `no_ignore`. D corrected the collector
  docstrings' git-URL claim but not these parameter lists. **Fixed in E**:
  both native analysis docstrings now describe the two parameters.

- **Python-3.8 idioms the raised floor makes obsolete.**
  `from __future__ import annotations` in `complexipy/__init__.py`,
  `complexipy/cli.py`, `tests/test_refactor_plans.py`, and
  `tests/contract/check_stub_contract.py`; `typing.List`, `Optional`, and
  `Tuple` imported in `complexipy/_complexipy.pyi` (32 subscripted uses) and
  `List`/`Tuple` in `tests/main.py` (6), counted before E. Flagged by the explore
  sweep and absent from E's checklist
  until now. **Completed in E as policy cleanup**: remove the future imports
  and use builtin generics/unions. The old aliases were valid, not runtime
  defects. The floor is now `>=3.14` and the lockfile was regenerated.

- **The stub and wrapper promised unmapped analysis exceptions.** Native
  analysis failures are reported as `ValueError` carrying the Rust error string: `code_complexity` on a syntax error, `file_complexity` on a missing or
  unreadable file. `_complexipy.pyi` documents `SyntaxError`, `FileNotFoundError`,
  `PermissionError` and `UnicodeDecodeError`, and `complexipy/__init__.py`
  `file_complexity` repeats three of them. `tests/main.py` `_analyze_paths`
  caught the same four and would have let a `ValueError` propagate; the page
  review changed it to catch `ValueError`, and `TestErrors` pins the runtime.
  Whether the binding should map to those types is a design choice; the
  docstrings are wrong either way. Two more stub claims in the same family: the
  collectors' `invocation_path` is documented as "working directory for
  resolving relative paths" and is never read (see the collectors' invocation-path
  bug below), and the `additional_refactor_plans` docstring names only the cap
  where `registry.rs` `analyze` also counts plans whose measured reduction fell
  below one.
  **Fixed in E**: corrected these docstrings and updated the matching caveats in
  `docs/python-api.md` and `docs/rules.md`. Runtime behavior is unchanged.

- **The doc comment on `crates/complexipy-core/src/lib.rs`'s `classes`
  re-export block** claims it "mirrors `complexipy/__init__.py`'s `__all__`". It
  also exports
  `compute_staged_diff` and `run_analysis_shared`, and its `DiffEntry` /
  `DiffStatus` are different types from the `py_diff` ones Python sees.
  **Fixed in E**: removed the false comment and stated the separate Rust and
  Python compatibility contracts in `AGENTS.md`.

- **Two more stub claims contradicted the scoring tests.** `LineComplexity`
  described boolean operators as incrementing individually;
  `TestPaperConformance.test_boolean_runs_are_counted_per_operator_sequence`
  pins runs instead. `FunctionComplexity.name` excluded class names from method
  names and suggested separate nested-function results;
  `TestScorerContract.test_methods_are_named_class_method` and the scorer's
  function traversal disagree. **Fixed in E**: describe operator runs,
  `Class::method` names and nested-function aggregation.

### Fixed in workstream F

Recorded by `chore(tooling): remove the pre-commit stack`.

- **Two config files at the repo root, one dead.** Root `complexipy.toml`
  shadowed `[tool.complexipy]` in `pyproject.toml`; they disagreed on `paths`,
  `exclude`, `failed` and `quiet`. **Fixed in F**: remove the pyproject block,
  retaining root `complexipy.toml` unchanged. Discovery remains first-hit-wins
  with no merge (`crates/complexipy-cli/src/utils/toml.rs`
  `get_complexipy_toml_config`). The sibling tests
  `complexipy_toml_wins_over_every_other_candidate` and `candidates_are_not_merged`
  pin that behavior; pyproject config support remains available to consumers.

### Open

- **Statements in a class body that are not functions are scored nowhere.**
  `cognitive_complexity.rs` iterates a `ClassDef` body matching only
  `Stmt::FunctionDef`, and the module accumulator never sees a `ClassDef`.
  `class A:` containing `if x: pass` scores 0; the same `if` at module level
  scores 1, and a class nested in a class body is dropped together with its
  methods, which are never reported. Found while deriving `docs/scoring.md`.
  Whether class-body control flow *should* count is a scoring-contract
  question, which is why this is open rather than fixed.

- **`-s file_name` sorts by function name in the console, by path in CSV.**
  `output/rows.rs` `sort_functions` sorts `function.name` for `Sort::FileName`;
  `export_tests.rs` `csv_file_name_sorts_by_path_keeping_function_order` proves
  the CSV path sorts by file path. Two implementations of one flag value.
  Console side unverified beyond the source read.

- **Duplicate file headers in console output.** Grouping is by consecutive
  same-path entries, so any sort that interleaves paths repeats a header.
  Probably not specific to `--top` since truncation runs after row building.
  Unverified; reproduce first.

- **`--suggest-refactors` degrades silently on a failed source read.**
  `crates/complexipy-cli/src/output/refactor.rs` `read_source_lines` ends in
  `.ok()`, so the caret span and `Original:` snippet vanish with no warning if
  the file cannot be read after analysis. The path-root contract now gives it
  in-root relative or outside-root absolute file labels, so the reconstruction
  is correct for this local workflow. The former raw `../repos/foo` and
  Windows-path explanations are not current triggers; the unreported read
  failure remains. This conclusion is a source read, not a runtime reproduction.

- **Dead code.** `crates/complexipy-cli/src/output.rs`
  `effective_sort_for_display` has no callers; `utils/snapshot.rs`
  `SnapshotEvaluation.snapshot_result` is computed and tested but never read by
  `run.rs`; `RuleMetadata` derives `Serialize, Deserialize` with no consumer;
  `utils/ignored.rs` `handle_report_ignored` takes `_no_ignore`. The collectors'
  `invocation_path` is now used for path resolution; it is not dead code.

- **Directory walker entry errors disappear from the population.**
  `crates/complexipy-core/src/helpers/exclude.rs` `get_paths_to_process` drops
  `ignore::Walk` errors with `Err(_) => None` and wax entry errors via
  `entry.ok()`. An unreadable subtree can therefore vanish without a failed
  path in analysis or either collector. The path-root batch covers missing
  inputs, invalid globs and read/parse failures after discovery, not this
  discovery-time failure. Carry walker errors into the shared result contract
  before treating an empty `failed_paths` list as proof of a complete survey;
  do not pin silent omission as desirable behavior.

- **CLI marker reports discard collection failures.**
  `crates/complexipy-cli/src/utils/ignored.rs` `handle_report_ignored` drops
  the collector's failed-path vector, although it propagates a top-level
  error. `handle_removable_ignores` drops the vector and converts a top-level
  error to an empty report with `unwrap_or_default`. The core collector
  contract therefore does not reach the CLI intact. The latter report only
  runs outside quiet mode in `run.rs` `run_at`; adding its failures to the exit
  gate without choosing a consistent check set would create a new
  quiet-dependent outcome. Source-confirmed; add end-to-end regression tests
  in the population/CLI batch.

- **An empty marker report can leave stale JSON behind.**
  `utils/ignored.rs` `handle_report_ignored` writes `complexipy-ignored.json`
  only for nonempty locations. Reusing the output directory after markers
  disappear leaves the preceding report intact. The sibling
  `report_without_comments_writes_no_file` test pins fresh-directory omission,
  not safe reuse. A complete requested JSON report should overwrite with `[]`;
  failed or partial collection must remain distinguishable from a successful
  empty inventory. Source/test-read finding; add nonempty-to-empty and failed
  collection reuse cases.

- **Incomplete analysis can advance persistent comparison state.**
  `run.rs` `run_at` calls `evaluate_snapshot` and `handle_display` before
  evaluating `failed_paths`. `utils/snapshot.rs` can create or watermark a
  snapshot; `output.rs` `handle_display` calls
  `utils/cache.rs` `remember_previous_functions`, which replaces the stored
  function population for the target key. Successful files from a partial
  analysis can therefore advance state before the run reports failure.
  Snapshot merging retains absent files; this is not a claim that failed
  files are deleted from the snapshot. The population batch must prevent
  state advancement on incomplete analysis while keeping useful partial
  results and diagnostics. Source-confirmed; pin unchanged snapshot/cache
  bytes on failure before closing this issue.

- **Snapshot diagnostics duplicate the filename.**
  `utils/snapshot.rs` `format_function_location` joins `file_name` onto `path`,
  even though `FileComplexity.path` already identifies the file. Sibling tests
  explicitly expect `a.py/a.py:f` in watermark messages. Correct presentation
  and those expectations without changing `build_function_key` or the stored
  snapshot schema. This is a source/test-confirmed defect, not a new path-root
  regression.

- **Library diff failures are indistinguishable from valid results.**
  `crates/complexipy-core/src/diff.rs` `compute_diff` emits all current functions
  as `NEW` if `file_content_at_ref` fails, but drops a file entirely when
  `analyse_content_to_map` cannot parse its reference content. `run_git` hides
  stderr and returns `None` for timeouts or spawn failures; a nonzero Git exit
  becomes a failed result that `file_content_at_ref` also collapses to `None`.
  Callers receive no failed-path list or error explaining either outcome.
  A genuinely new file or a file with no functions can also yield those shapes,
  so result-shape checks alone cannot establish success. Resolve the error
  contract explicitly and test missing refs, unreadable reference content and
  parse failures before treating this as a trustworthy parent comparison route.
  This is separate from the CLI exit-code hole below.

- **The ratchet gate fails open.** With `--diff`, or a bare `--staged` (which
  `resolve_diff_flags` turns into `--diff HEAD`), `run.rs` sets `enforce_diff`
  so `ExitReport::success` ignores the threshold check, then computes `diff_ok`
  only when it has entries. `compute_staged_diff` returns `None` outside a git
  repository and an empty list for a reference git cannot resolve, so in both
  cases `diff_ok` stays true and an over-threshold tree exits 0 having applied
  neither gate. The non-staged path is safer only by accident: an unresolvable
  reference makes every function `NEW`, which the ratchet does catch.
  Documented in `docs/cli.md`; not pinned, since a passing test would ratify
  the hole.

- **A config file without `paths` analyzes nothing and exits 0.**
  `resolve_config` returns `MissingPaths` only when no config file loaded at
  all; `Config.paths` is `#[serde(default)]`, so a file that parses but has no
  `paths` key yields an empty path list, a run over nothing, and
  `ExitCode::SUCCESS`. The error string ("You need to define paths ...") and
  `docs/cli.md`'s exit-code list both promised otherwise until the page review
  narrowed the claim. Not pinned.

- **A malformed config file is skipped, not rejected.** `load_toml_config` and
  `load_pyproject_config` print the parse error and return `None`, so
  `get_complexipy_toml_config` falls through to the next candidate and then to
  the built-in defaults. A typo in `complexipy.toml` silently changes the
  threshold. Pinned as current behavior by
  `a_malformed_complexipy_toml_falls_through_to_the_next_candidate`, because
  `docs/cli.md` now documents it; failing closed is the better design and would
  invert that test.

- **The `--diff-only` warning fires on every `--diff-only` run.** `run.rs`
  checks `diff_only.is_some() && diff.is_none()` after `resolve_diff_flags`,
  which always clears `diff` when `diff_only` is set, so
  `diff_flags_warning` ("--diff and --diff-only both set") prints for a bare
  `--diff-only`. The check has to run on the pre-resolution flags. Also
  undocumented until the page review: with both flags set, `--diff`'s
  reference is discarded.

- **Suppression and the marker report disagree.** `is_ignored` runs
  `find_noqa_comment` from the function's range start, which includes
  decorators, so a marker on the line above the first decorator suppresses.
  `collect_ignored_locations` is a separate line scanner that only starts from
  `def ` lines, so that placement is never reported, and neither is any marker
  on an `async def`. Two placements suppress nothing: a marker between two
  decorators (the `@` branch stops at the first non-decorator line) and a marker
  after an annotated parameter in a multi-line signature
  (`signature_has_marker` stops at the first line containing any colon).
  `docs/cli.md` documents the reachable behavior; the working placements are
  pinned by `test_ignore_marker_placements_that_suppress`, the gaps are not.

- **Staged comparisons use repository-wide scope.** `run.rs` `run_at` calls
  `compute_staged_diff` with only the reference and invocation path, not the
  configured analysis paths or exclusions. In `diff.rs`, `staged_python_files`
  selects staged `*.py` paths across the repository. A narrow analysis request
  can therefore compare files outside its intended population. The parent's
  exploration skill requires caller-selected scope; native staged comparison
  is not safe for an excluded-file task merely because CLI analysis paths are
  narrow. Decide how comparison scope is passed and test a staged file outside
  that scope before treating this route as suitable for such tasks.

- **`--diff` compares against a reference analyzed with fixed flags.**
  `analyse_content_to_map` in `diff.rs` always runs with `check_script` and
  `no_ignore` off, while the current side uses the run's flags. Under
  `--check-script` every file gains a `NEW` `<module>` entry; under
  `--no-ignore` every suppressed function is `NEW`. Either can fail the ratchet
  on an untouched tree. `compute_staged_diff` is immune because both sides go
  through the same function. Documented in `docs/diff-and-snapshots.md`; not
  pinned.

- **`--quiet` drops `--ignore-complexity`.** `handle_display` in
  `crates/complexipy-cli/src/output.rs` returns `has_success_functions(...)` on
  the quiet path without consulting `ignore_complexity`, while the non-quiet path
  computes `all_pass || ignore_complexity` in `render.rs` `output_summary`.
  `--quiet --ignore-complexity` therefore exits 1 on an over-threshold function
  that the same run without `--quiet` passes. Documented as a rough edge in
  `docs/cli.md`; deliberately not pinned, since the fix is a one-line behavior
  change.

- **The boolean-run walker does not descend through every expression.**
  `utils.rs` `count_bool_ops` recurses into comparisons, positional call
  arguments, tuples, lists, sets, dict values, ternaries, lambdas and
  comprehensions, and nothing else; `not` goes through
  `count_different_childs_type`, which follows only a directly nested `and`,
  `or` or `not`, so `not g(a and b)` scores 0 for the run and
  `not (1 if a else 2)` loses the ternary's structural increment too.
  `(a and b) + 1`, `d[a and b]`, `g(k=a and b)`, `g(*(a and b))`, an f-string
  and a walrus target (`if (n := a and b):`) all score 0 for the run, and the
  `Stmt::Match` arm in `cognitive_complexity.rs` never walks the subject or a
  `case` guard, so `match a and b:` and `case 1 if a and b:` score nothing for
  theirs. The paper charges every operator sequence. Documented as current
  behavior in `docs/scoring.md` ("Expression walker limits") and deliberately
  not pinned.

- **Recursion detection sees bare-name calls only.** `cognitive_complexity.rs`
  `RecursionFinder` matches an `Expr::Call` whose callee is an `Expr::Name` equal
  to the function name, and does not descend into lambdas or nested scopes, so
  `self.m()` inside method `m`, and a self-call inside a `lambda`, are not
  recursion. Whether method recursion should count is a scoring-contract
  question, like the class-body item above; documented, not pinned.

- **Plain `//` comments in Rust against the no-comments rule.** `AGENTS.md`
  (Code Style, Anti-Patterns) bans comments in code, and nothing enforces it. 33
  lines across four files carry them: `rules/complexity.rs` (9, production code,
  two blocks in `generate_loop_guard_suggestion`), `rules/complexity/tests.rs`
  (19), `rules/registry/tests.rs` (3, above the `checked == 7` assertion in
  `every_registered_rule_produces_a_plan_consistent_with_its_own_metadata`,
  reading "if a 9th rule is added" while seven are registered), and
  `output/render/tests.rs` (2). The path-root batch removed the two stale
  `api/tests.rs` comments with their old basename assertion. Doc comments
  (`///` and `//!`) are not counted.
  H's `add-refactor-rule` skill now cites that registry test as one of its
  explicit gates and warns against copying the comment pattern. Removing the
  comments remains a code change outside H; the defect is not closed by guidance.

### Deferred

- **`--color` is completely inert.** `output/render.rs` `handle_console_settings`
  computes `ConsoleSettings.color_enabled` and nothing reads it but two tests,
  which give false confidence
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

### Diff path pairing is heuristic

The public-file identity loss is fixed by the path-root batch (see Fixed).
From an external CWD, use `file_complexity("a/utils.py", base_path=repo_root)`
for repository-relative identity. No private binding workaround is required.

`diff.rs` `resolve_git_path` still tries suffixes at the reference, then a
unique tracked basename. A malformed or mismatched caller-supplied path can
become an all-`NEW` result or pair a record with a different same-named file.
The explicit public base avoids losing identity before comparison; it does not
make this heuristic or the silent diff failures safe. Exact path pairing and
failure reporting remain part of the diff batch.

### Explicit configuration selection is unavailable

Config discovery (`get_complexipy_toml_config` in
`crates/complexipy-cli/src/utils/toml.rs`) joins three candidate filenames onto
the invocation path, first hit wins, no merge, no upward search, and never
consults the analyzed target. This matches the parent's caller-controlled
external-CWD route; it is not itself proof that discovery uses the wrong root.
If a task explicitly selects a target's config, there is no `--config` path:
use equivalent CLI settings or place config in the external invocation directory.

An explicit config option could remove that friction without silently adopting
ambient target settings. Automatic target-root lookup or upward search would
change existing invocations and requires a separate decision. Neither is a
current parent requirement.

### Write locations and invocation-directory state

Three write locations, one of them not redirectable:

- `.complexipy_cache/` in the invocation directory, shipping its own
  `.gitignore` containing `*` and a `CACHEDIR.TAG`. `git status --porcelain`
  shows nothing after a run, which defeats a git-status-based before/after
  snapshot of the target. `--cache-dir` moves it.
- `complexipy-snapshot.json`, hardcoded to the invocation directory with **no
  override flag**, and rewritten by a passing check (a documented ratchet, not a
  bug - but it means running in a directory that already holds a snapshot
  silently updates it).
- `--output`, which is redirectable and was the subject of the fixed bug above.

These are invocation-directory writes, not unavoidable writes into every
analyzed target. The parent runs from an external CWD and redirects cache and
output there. `--snapshot-create=false --snapshot-ignore=true` prevents snapshot
creation and watermark rewrite, but `evaluate_snapshot` still loads any existing
snapshot. A separate snapshot path would be useful only for a workflow that needs
independent snapshot state. Do not run inside a target to obtain Git context and
then assume it remains read-only. The cache's self-hiding also means Git status
alone cannot detect every write; parent checkout observations cover more.

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
conjunction of the threshold, path and snapshot checks. `--diff <ref>` replaces
the threshold check with the regression ratchet (`diff_ok && paths_ok && snapshot_ok`), so a run exits 0 with functions over the limit as long as none
regressed or appeared above it (`run/tests.rs` `diff_clean_exits_success`).
`--diff-only` prints the comparison and leaves the gates alone
(`diff_only_leaves_the_threshold_gate_in_place`). `--ignore-complexity` forces
the threshold check to pass but keeps the others - except under `--quiet`, where
it is not read at all (bug above). Neither is wrong, but a consumer scripting on
the exit code has to know which gate set the flags selected, and nothing in the
output says. This entry, `docs/cli.md`, and C's `--help` text all had `--diff`
and `--diff-only` the wrong way round until the review of C's pages.

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

### Every non-quiet run walks and parses the tree twice

`run.rs` calls `handle_removable_ignores` whenever `--quiet` is off, which
re-runs `get_paths_to_process` and re-parses every file to build the
removable-marker report; `--report-ignored` adds a third walk. The report is
unconditional and unsuppressible, so on a large tree the default invocation
costs roughly twice the analysis it displays. Documented in `docs/cli.md`;
whether it should be a flag, or reuse the analysis results, is a design
choice.

### Local verification lost an isolation property

The deleted CI lint job ran ruff and ty in an environment with the extension
deliberately *not* installed, and asserted that through `importlib.metadata`,
so ty always read the stub. The standing gate's `uv run ty check .` runs with
the editable project installed, so ty reads the native module. The contract
harness (`tests/contract/check_stub_contract.py`) checks stub/runtime parity from
a neutral directory. It has eleven diagnostic cases, including positive getter
and enum-member types, negative assignment/construction for all eight result
types, construction rejection for three enums and subclass rejection for all
twelve native types, plus separate runtime checks. These are selected promises,
not exhaustive API coverage. H's `verify` skill distinguishes these checks.
No CI rebuild is planned under the single-machine mandate; a separate root-lint
environment remains an unadopted option, not a prerequisite for the existing
wheel harness.

### The parser is a network-fetched git dependency

**Fixed in the local build and verification contract change:** both
`ruff_python_parser` and `ruff_python_ast` now declare revision
`ef422460de726c5b896c09c364d02a4db24bcaf0` explicitly in `Cargo.toml`, replacing
the mutable `0.12.9` tag. This is the same revision already resolved in
`Cargo.lock`; the five Ruff package source identifiers changed, not their code
or versions. Deliberate lockfile regeneration can no longer follow a moved tag.

An uncached checkout still needs the Git source fetched. That is acceptable
for this machine's local builds; vendoring or offline bootstrap machinery is
not required. A future parser upgrade must review scoring changes explicitly.

### Build tooling constraints and cache inputs

**Fixed in the local build and verification contract change:** PEP 517 and the
`dev` group now both require `maturin>=1.9.4,<2.0`; `uv.lock` retains maturin
1.14.0. Previously the dev floor was `>=1.8.3`, allowing different tooling
through `maturin develop` and PEP 517. No dependency version was upgraded.

`[tool.uv] cache-keys` now includes `complexipy/**/*.py`,
`complexipy/**/*.pyi` and `complexipy/py.typed` alongside manifests and Rust
sources. The [uv cache documentation](https://docs.astral.sh/uv/concepts/cache/#dynamic-metadata)
specifies file globs for rebuild/reinstall invalidation. Previously packaged
Python/stub-only edits were missing from this explicit key. This configuration
does not replace rebuilding the extension after Rust edits or checking the
exact installed wheel; it is not an exhaustive cache-behavior test suite.

### Test coverage has structural holes

- **Analysis-walk and exclusion gaps closed in the path-root batch.**
  `crates/complexipy-core/tests/runner_paths.rs` calls `run_analysis_shared`
  and both collectors on directories and explicit files. It pins canonical
  identities, walk-root-relative exclusions, hidden/ignore/extension filters,
  explicit-file bypass, invalid globs, mixed valid/failed files and forwarded
  scoring flags. CLI `run/tests.rs` pins an exclusion changing the threshold
  gate and matching analysis/marker JSON paths. `tests/main.py` still uses its
  own fixture walk; it is no longer the only analysis-population exercise.
  Silent walker-entry errors remain open, as recorded above.
- The installed-wheel cases now touch all eighteen public exports, but cover
  selected signatures, getter types and failure modes rather than every input
  combination or runtime behavior.
- `tests/src/exclude_dir/` exists only to feed the deleted CI checks but must
  stay: `tests/main.py` asserts a corpus total that includes it.

### Removed features leave residue

Git-URL analysis was removed in 8.0.0. D removed its stub-docstring residue;
E corrected both `AGENTS.md` claims and removed `cache.rs::looks_like_remote`.
The same hazard applies to `doc_url`, `references`, the wasm target and the
`runner` feature unless each removal sweeps for its own references. H's `verify`
skill now calls for a live-reference sweep on removal/rename changes; that is
manual guidance, not an enforced check.

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
B removed the last CI. Neither is integrated into the parent runner; the
parent's `docs/tools/complexipy.md` (Current interactive capability map) selects
direct Python and CLI JSON/CSV surfaces, not these formats.

**Decided in D: both stay.** Every other removal in this realignment has been
upstream-project machinery - CI, the docs site, the editor extension,
contributor templates. These are product capability, and the parent's charter is
to retain provider-native output rather than narrow it. They are stable, tested,
and cost nothing to keep; removing them later is cheaper than re-adding them.

Recorded because the reasoning is a judgment call rather than a fact, and
because the criterion the tracker set - "keep them only if the parent ingests
them directly" - pointed the other way. If the maintenance cost ever becomes
real, this is the entry to revisit. Note also that SARIF has a known defect: it
emits a literal `uriBaseId` of `"%SRCROOT%"` with no `originalUriBaseIds` block.
