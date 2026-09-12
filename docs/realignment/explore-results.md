# Explore sweep results

Final analysis pass before executing the realignment. Six RepoPrompt explore agents
ran in parallel over separate lanes, plus one lane run directly against the
consuming repository, which is outside their workspace.

Lanes: CLI crate user-facing surface; Python package and stub; test suite (Python
and Rust); per-crate manifests and post-removal dead code; tracked-file
completeness; build and packaging. Plus: consumer coupling in
`recsys-code-quality`.

**Marking convention, which is the inverse of the tracker's.** In `README.md`,
unmarked claims are repository-verified and `(external)` flags the rest. Here,
**(verified)** marks a claim re-checked directly against the repository, and
unmarked means agent-reported and consistent with surrounding evidence but not
independently re-derived. Treat unmarked claims in this file as provisional.

Corrections this sweep makes to tracker instructions have been absorbed into
`README.md` rather than left here. What remains in this file is the evidence
trail, the pre-existing-defect inventory, and the watch items.

## Findings that change the plan

### 1. Removing `doc_url` breaks the consuming repository

`recsys-code-quality/scripts/complexipy_analysis/reduced_record.py:51` reads
`plan.doc_url`, and `:50` reads `plan.references`. The parent's
`tests/test_complexipy_analysis_helpers.py:26` loads the same helper. **(verified)**

`doc_url` alone is enough to raise `AttributeError` the first time that helper runs
against an 8.1.0 wheel; `references` compounds it if finding 4 is adopted, which it
now is.

This reclassifies the deferred outer-repository work. The tracker filed it as a
documentation update to `wheelhouse/README.md` and `docs/tools/complexipy.md`. It
is also a **code** fix in the consumer, and it gates the consumer's ability to use
8.1.0 at all.

### 2. There is no upstream remote, and `origin` carries no tags

`git remote -v` returns exactly one remote:
`origin = https://github.com/bcgalvin/complexipy.git`, the fork.
`git ls-remote --tags origin` returns **zero** tags, and `origin/main` is still at
`030e207` - the eleven local commits have never been pushed. **(verified)**

Three consequences, all absorbed into the tracker:

- G instructed setting `tagOpt = --no-tags` "on the upstream remote," and H's
  `sync-upstream` row said "Upstream remains the fetch remote." Neither has a
  target. If upstream is ever added as a remote, `--no-tags` belongs at add time,
  before the first fetch re-imports 39 tags.
- B asked whether `origin` carries the inherited tags. It does not, so G's local
  `git tag -d` is sufficient and no `git push --delete` is needed.
- `git remote prune origin` targets the fork. (Execution note: it pruned 36 stale
  remote-tracking refs inherited from the fork clone, so this was not the no-op
  predicted here - the remote side was right, the local side was not checked.)

### 3. This checkout is a git submodule, and `git submodule update` will revert it

`.git` is a file reading `gitdir: ../../.git/modules/repos/complexipy`. The parent
records `repos/complexipy` in `.gitmodules` with **no `branch =` key**, so it pins
by commit SHA. **(verified)**

The hazard is not the gitlink bump itself, which is routine. It is that any
`git submodule update --init` in the parent - including as part of an ordinary
pull - checks this submodule out at the recorded SHA, which is still
pre-realignment. Commits on `main` survive; an in-progress working tree does not
necessarily. That is a state-safety hazard across a multi-day realignment.

Two supporting facts: the parent already reports ` M repos/complexipy`
**(verified)**, so a dirty pointer is pre-existing noise and cannot signal that a
bump is pending; and the gitlink bump should land with the 8.1.0 bump rather than
in deferred work, or the parent pins a tree whose recorded version is 8.0.1 while
the working tree says 8.1.0.

Branch deletion is gitlink-safe: `rcq` is an ancestor of `main` and
`followup-batch-1` is identical to it, so neither can orphan a pin.

### 4. `complexipy.toml` overrides `[tool.complexipy]` - the pyproject block is dead

`crates/complexipy-cli/src/utils/toml.rs:7-16` tries `complexipy.toml`, then
`.complexipy.toml`, then `pyproject.toml`, and returns the **first hit with no
merge**. Root `complexipy.toml` exists, so `[tool.complexipy]` is never read from
the repository root. **(verified)**

The tracker framed these as two configs that disagree and asked to retire one. The
choice is not symmetric: retiring `complexipy.toml` changes real behavior - `paths`
goes from `["."]` to `["crates", "complexipy"]`, `exclude = ["tests/**"]` is lost,
and `failed`/`quiet` both flip - while retiring the pyproject block is a no-op.

One narrowing: this says nothing about what the pre-commit hook read, since that
hook runs upstream's binary and its discovery code is not in this repository. Both
files set `max-complexity-allowed = 15`, so `AGENTS.md`'s value claim is right and
only its provenance is wrong.

### 5. `references` is a second dead field, removable with `doc_url`

`crates/complexipy-core/src/rules/types.rs:52` is the sole writer and sets
`references: vec![]`; no rule in `complexity.rs` overrides it. The only non-empty
value anywhere is the CLI test fixture at `output/refactor/tests.rs:49`. **(verified)**

Once `doc_url` leaves `output_plan_references`, its guard
(`if doc_url.is_empty() && references.is_empty()`) can never be false in a real
run, so the entire `References:` block is unreachable. That answers the tracker's
open question about whether the helper survives: it does not.

Removing the field is a **two-place typing change** - `classes.rs:70` plus the stub
declaration and its `__init__` parameter - not three. The `#[pymodule]` list is for
adding or removing a *type*; `lib.rs` registers `RefactorPlan` with `add_class` and
never enumerates fields. Beyond typing it touches the same struct-literal sites as
`doc_url` (`registry/tests.rs`, `sarif/tests.rs`, `gitlab/tests.rs`,
`export_tests.rs`, `refactor/tests.rs`), `output_plan_references` itself, and - the
one that reaches consumers - the `--output-format json` schema a second time.

Sharper evidence than the field being empty: `refactor/tests.rs` populates
`references` with `"https://example.com/ref"` and asserts only `"References:"` and
the *`doc_url`* value. The `references` rendering path has no assertion at all.

### 6. The CI exclusion checks were vacuous, so "carry them over" carries nothing

The tracker and `follow-up-tooling.md` both record that CI exercised two behaviors
no pytest case covers: `complexipy complexipy --failed` and two `--exclude` glob
validations. The first is a real gate. The second is not: both exclusion
invocations pass `--ignore-complexity`, and `ExitReport::success()` resolves to
`all_pass || ignore_complexity`, so the exit code is 0 whether or not the glob
matched anything. They would have passed against a silently non-matching pattern.

They did prove path resolution, since `paths_ok` still gates the exit code. That is
the whole of what they proved.

Reproducing them faithfully in the CI rebuild would reproduce a check that cannot
fail on the thing it appears to test. The exclusion behavior needs real coverage
instead - see the test-coverage section below.

### 7. `docs/understanding-scores.md` contradicts the conformance tests, and C names it "the scoring contract"

Two scoring rules in the document that C designates as the source for the rewritten
scoring page are contradicted by `tests/main.py::TestPaperConformance`, which
`AGENTS.md` designates as the actual contract: **(verified)**

- **`with`.** `understanding-scores.md:150` annotates `with open(path) as f:` as
  "+1 (context manager treated as if)" and nests the following `if` at +2.
  `test_with_does_not_nest` asserts `with` plus a nested `if` totals **1**, pinning
  `with` at +0 with no nesting increment. That example's stated total of 6
  recomputes to 4.
- **`match`.** `:84` annotates `match value:` as "+0 (match itself doesn't count)".
  `test_match_top_level_structural_increment` asserts a `match` with two cases and
  no nested control flow totals **1**.

Transcribing the page would propagate two wrong scoring rules into the document
that replaces the algorithm's only prose specification. Re-derive it from
`TestPaperConformance` instead.

`docs/api-reference.md`, C's source for the API page, omits three public fields:
`reduction_is_measured` on `RefactorPlan`, `spliceable` on `CodeSuggestion`, and
`additional_refactor_plans` on `FunctionComplexity`. **(verified)** All three are in
the stub, and `reduction_is_measured` carries the measured-versus-estimated
semantic that `refactoring-rules.md` spends a section on.

## Additional scope for existing workstreams

### Workstream D - the typing surface is worse than `doc_url`

- **Eight phantom `__init__` declarations.**
  `crates/complexipy-core/src/classes.rs` contains zero `#[pymethods]` and zero
  `#[new]` blocks **(verified)**, so only `DiffEntry` - defined separately in
  `complexipy-python/src/lib.rs` - has a real constructor. The stub declares
  constructors for `CodeSuggestion` (:102), `LineComplexity` (:141), `RefactorPlan`
  (:209), `FunctionComplexity` (:313), `FileComplexity` (:403), `CodeComplexity`
  (:465), `IgnoredLocation` (:495), and `RemovableIgnore` (:532). Each stub
  docstring "Example" block is type-checkable and runtime-impossible.
- **Every class attribute is declared writable; all are read-only.** `get_all`
  generates getters only. `DiffEntry` is the only class modelling this correctly
  with `@property`, and `assign_readonly.py` is the only case that catches it.
- **Three enums carry the `DiffStatus` defect, not one.** `RuleCategory` (:13) and
  `Applicability` (:22) are also declared `(Enum)` with string values; the runtime
  MRO is `(class, object)`, `.value` and `.name` raise `AttributeError`, and
  `list(...)` raises `TypeError`. The consuming repo already carries a `variants()`
  workaround with the comment "PyO3 simple enums have no enum.Enum name/value
  readout." The stub is wrong and the consumer knows it.
- **The stub documents a removed feature.** `_complexipy.pyi:717` and `:755` both
  read "paths: List of file paths, directory paths, or Git repository URLs."
  **(verified)** Git-URL analysis was removed in 8.0.0 per `CHANGELOG.md`. This
  ships inside the wheel and is what the consumer's type checker reads. The same
  file's native `code_complexity`/`file_complexity` docstrings omit `check_script`
  and `no_ignore` from their `Args:` blocks.
- **The contract-harness gap is twelve members, not three.** The four case files
  reach `DiffEntry`, `DiffStatus`, `code_complexity`, `compute_diff`,
  `has_regressions`, and `CodeComplexity` transitively. Of eighteen `__all__`
  members, the unproven set is `file_complexity`,
  `collect_all_ignored_locations`, `collect_removable_ignored_locations`,
  `Applicability`, `CodeSuggestion`, `FileComplexity`, `FunctionComplexity`,
  `IgnoredLocation`, `LineComplexity`, `RefactorPlan`, `RemovableIgnore`, and
  `RuleCategory`.
- **The new contract case cannot follow the `phantom_import` pattern.** That case
  proves a *module-level* name is absent via `unresolved-import`. Proving a *field*
  is absent needs `unresolved-attribute` on a plan value, and since there is no
  `#[new]`, the case must reach a `RefactorPlan` through
  `code_complexity(...).functions[0].refactor_plans[0]`. The runtime half is not
  `EXPECTED_DIAGNOSTICS` but the `RUNTIME_CHECKS` string constant, which already
  carries the right `hasattr` idiom. Cases are outside root ty scope but inside
  Ruff's, so the new one must be lint- and format-clean.
- **Deleting `test_rule_metadata_has_doc_url` orphans its fixture.**
  `tests/fixtures/refactor_plans/metadata_validation.py` is loaded by that test
  alone. It sits outside `tests/src`, so the corpus total is unaffected and it is a
  clean delete.
- **SARIF `informationUri` has no test coverage at all.** Only `helpUri` is
  asserted. Removing it is silent either way. Note that
  `sarif_file_created_and_valid` does assert `doc["$schema"].is_string()`, so the
  `SCHEMA` watch item below is test-guarded.

### Workstream E - Python floor and stale invariants

- **`from __future__ import annotations`** at `complexipy/__init__.py:8`,
  `complexipy/cli.py:8`, `tests/test_refactor_plans.py:1`, and
  `tests/contract/check_stub_contract.py:8`. Under PEP 649 this opts out of
  deferred evaluation rather than being merely redundant, though nothing here
  introspects `__annotations__` at runtime, so the practical effect is nil.
- The stub imports `List`, `Optional`, `Tuple` with 51 usages; `tests/main.py:2`
  likewise. All should be builtin generics and unions at a 3.14 floor.
- **`AGENTS.md:211` states the three-place FFI rule without the type/field
  qualifier** - "Changing one of those structs means updating three places in
  lockstep" **(verified)**. That is the exact rule the tracker corrected. It
  matches none of E's regex terms, so the wrong rule would outlive its correction
  while the `ffi-change` skill teaches the right one. E's regex also misses
  `AGENTS.md`'s "docs in `docs/` (EN + ES)" line, because the literal `docs/es`
  does not appear there, and the "PR titles" convention bullet, because it does not
  say "pull request". The lesson is that a keyword sweep cannot find claims whose
  wrongness has no keyword; E needs a read-through of the structural-invariant
  sections.
- **`crates/complexipy-core/src/lib.rs:16-17`** claims its re-export block "mirrors
  `complexipy/__init__.py`'s `__all__`". The genuine extras are two -
  `compute_staged_diff` and `run_analysis_shared`; `code_complexity` and
  `file_complexity` are in `__all__`. The stronger half of the point stands
  unchanged: core's `DiffEntry`/`DiffStatus` are different types from the `py_diff`
  ones Python sees. The neighbouring "compatibility promise ... major release"
  framing is upstream residue for a crate with `publish = false`.
- **`CLAUDE.md` is rewritten in E and invalidated by H.** Its skill bullet names
  `git-commit`, `create-pr`, and `release-notes`; H retires two of those and adds
  six. Either fold the skill list into H or accept the two-pass edit deliberately.
- **There is no `__version__` attribute** in the package or extension, so the
  version lives only in wheel metadata. `importlib.metadata.version("complexipy")`
  and `complexipy --version` both work, so this is a convenience gap rather than an
  identification gap.

### Build, packaging, and lint enforcement

- **`Cargo.lock` regeneration belongs in the same commit as the wasm deletion.**
  Every build path passes `--locked` - the contract harness runs
  `maturin build --profile dev --locked`, the wheelhouse build is
  `--locked --release`, and A's own verify line is three `cargo check --locked`
  invocations. Drift fails loudly rather than silently, but between the deletion
  and the regeneration the entire standing gate is unrunnable.
- **Clippy promotion moves from automation to a manual gate.** Workspace lints set
  `exit`, `dbg_macro`, `todo`, and `unimplemented` to `"warn"` **(verified)**, and
  `-D warnings` lives in both `CI.yml` and `.pi/hook-scripts/rust-checks.sh`, both
  deleted in B. The tracker's standing gate already carries
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, so nothing is
  lost that the gate does not restore. The residual option - setting the four to
  `"deny"` in the root manifest, which would also cover `cargo build` and
  `cargo check` and survive a skipped gate - is a hardening preference, not a gap.
- **The maturin floor is split.** PEP 517 requires `maturin>=1.9.4,<2.0`; the dev
  group declares `>=1.8.3`, resolved to 1.14.0; the wheelhouse artifact was built
  with 1.15.0. The contract harness shells `uv run --no-sync maturin build`, and
  `maturin develop` uses a plain `uv run`, so both take the dev-group binary rather
  than the PEP 517 one. `uv sync` does build the editable install through PEP 517
  and honours the higher floor. Two floors on two paths; raise the dev floor either
  way.
- **The parser is a network-fetched git dependency on a mutable tag.**
  `ruff_python_parser` and `ruff_python_ast` are declared as
  `{ git = "https://github.com/astral-sh/ruff.git", tag = "0.12.9" }`. `Cargo.lock`
  pins the resolved rev, so `--locked` protects reproducibility, but a fresh clone
  cannot build offline and any `cargo update` follows a tag that can be re-pointed.
  For a fork whose premise is local source builds with no registry, this is the
  largest build-reproducibility exposure in the repository.
- **Nothing pins the Rust toolchain or redirects build output.** No
  `rust-toolchain.toml`, no `.cargo/config.toml`, no `.python-version`. After CI is
  gone, nothing pins the toolchain and nothing directs `CARGO_TARGET_DIR` outside a
  checkout the parent treats as a read-only input. `vendor-build` carries that as
  prose; a committed `.cargo/config.toml` would enforce it.
- **uv `cache-keys` cover no Python source.** The five entries are
  `pyproject.toml`, `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`, and
  `**/*.rs`; custom keys replace uv's defaults. The omission is anomalous and the
  fix is free, but the obvious failure mode may not exist: the project is an
  editable install with the compiled extension landing in the source tree, so
  Python edits are live at the next import with no stale copy to invalidate. Add
  `complexipy/**/*.py` and the `.pyi`, but do not claim a stale-install bug without
  reproducing one.
- **`include = ["LICENSE"]` under `[tool.maturin]` is inert today**, since no sdist
  is built here and the license reaches the wheel through metadata. It stops being
  inert if the `release` skill carries over the retired `release-notes` skill's
  `maturin sdist` verification step.
- **Both surviving core cross-target checks are vestigial, not one.** After the
  wasm crate goes, `runner` is on for every build path, so
  `cargo check -p complexipy-core --no-default-features` verifies a shape nothing
  builds - and `--no-default-features --features python` does too, since
  `complexipy-python` always takes `["python", "runner"]`. Only the CLI check
  retains a subject, and the `serde` finding below is why it matters.
  Independently, `cargo test -p complexipy-core --no-default-features` has never
  compiled: `tests/lib_surface.rs:7-12` imports runner-gated items with no `cfg`,
  and `cargo check` skips `tests/`. **Resolved in the plan:** workstream A now
  collapses the `runner` feature outright and drops both checks, which also
  removes the configuration in which `lib_surface.rs` fails to compile.

### Serialized output differs by build shape

Every `serde(skip)` in `classes.rs` is `#[cfg_attr(feature = "python", serde(skip))]`

- on `FunctionComplexity`'s `line_start`, `line_end`, `line_complexities`,
  `refactor_plans`, and `additional_refactor_plans`, and on `FileComplexity.complexity`.
  With the `python` feature off, none applies.

`docs/usage-guide.md` documents the snapshot format as
`{path, file_name, functions: [{name, complexity}]}` - the skipped shape. A
snapshot written by a standalone `cargo build -p complexipy-cli` binary would carry
five extra fields per function. Feature unification decides which: a workspace
build unifies `python` on through `complexipy-python`, while a single-package build
does not.

`AGENTS.md` names feature unification as the hazard its separate cross-target
invocations exist to catch, but those are compile checks and this divergence
compiles cleanly in both directions. This sits between the manifest lane and the
test lane, which is why no single lane found it. Confirm by diffing a snapshot from
both builds.

### Test coverage the removals expose

- **Exclusion in the analysis path has no coverage**, and the CI checks that
  appeared to cover it were vacuous (finding 6).
  `crates/complexipy-core/tests/collector_failures.rs:110` does exercise exclusion,
  but through the collector entry point and only with a `**/bad.py` pattern -
  neither CI pattern shape is covered anywhere.
- **Nothing exercises the analysis walk through production code.**
  `tests/main.py:12-50` reimplements it with `path.rglob("*.py")` and calls
  `file_complexity` per file, bypassing `run_analysis_shared`; every
  `crates/complexipy-cli/src/run/tests.rs` case passes a single file. The collector
  walk is covered; the analysis walk is not.
- **`tests/src/exclude_dir/` must stay** even though its only consumer is deleted:
  `tests/main.py` asserts a hardcoded corpus total at line 68 that walks
  `tests/src` recursively. Confirm those two files score non-zero before assuming
  deletion would change the total, but do not delete them casually.
- **`version_flag_handled_by_clap`** (`run/tests.rs:149-153`) asserts only
  `result.is_err()`, so it passes identically for a version display, a help
  display, or a parse failure. `sarif/tests.rs` compares the driver version to
  `env!("CARGO_PKG_VERSION")`, which is self-referential. Nothing meaningfully
  tests versioning during the 8.1.0 bump.
- **A vacuous test:** `tests/test_refactor_plans.py:320-327` wraps its entire body
  in `if flatten_plan:`, so it passes with zero assertions if C001 stops firing.
  The neighbouring test guards against exactly this with
  `assert func.refactor_plans, "fixture produced no plans to validate"` - the
  codebase already knows the pattern.
- **`tests/contract/check_stub_contract.py` is referenced by nothing executable** -
  not by CI, not by pre-commit, not collectible by pytest. `AGENTS.md` documents it
  and it is run by hand. The `verify` skill is where that changes.
- **Nothing in the standing gate runs the CLI.** The gate covers pytest, cargo
  test/clippy/fmt, ruff, ty, and the contract harness - no invocation of the built
  binary. Today that is covered by `.pi/hook-scripts/py-complexipy.sh` and CI's
  `complexipy complexipy --failed`, both deleted in B. Since the pre-existing
  defects below sit precisely in that surface, `verify` should carry one.

## Pre-existing defects surfaced

The sweep found roughly a dozen live bugs that are not realignment work. They are
recorded with their evidence and sequencing in
[`follow-up-tooling.md`](follow-up-tooling.md#pre-existing-defects), which is the
file that outlives this directory - one was fixed before workstream A, four fold
into D, one into C, and the rest are deferred past the realignment.

The three that bear on how `recsys-code-quality` consumes this tool: `--color` is
completely inert so the console surface cannot produce clean text; config discovery
is CWD-only so a target's own thresholds are ignored when running from an external
working directory; and the tool writes into the tree it analyzes three ways, one of
them invisible to `git status`.

## Verified clean

- **The wheel payload is unaffected by every planned removal.** Contents are
  `complexipy/{__init__.py,cli.py,_complexipy.pyi,py.typed,*.so}` plus dist-info.
  Nothing on the removal list is currently shipped.
- **All four crates already set `publish = false`** and inherit every identity
  field from the workspace, so fixing the root manifest fixes all four and the
  version bump needs no per-crate edit.
- **Version plumbing is clean.** `--version` derives from the workspace version via
  `#[command(version)]`, and SARIF's driver version uses `env!("CARGO_PKG_VERSION")`.
- **The `wasm` feature gates exactly two sites** (`classes.rs:117`,
  `cognitive_complexity.rs:27` - the `#[cfg]` attributes, with the fields on the
  following lines), no test references them, and `tests/lib_surface.rs` only takes
  `size_of`, so the field removals compile through untouched.
- **pyo3 0.29 supports CPython 3.14.** The `wheelhouse/` artifact is a working
  `cp314-cp314-macosx_11_0_arm64` wheel built from this tree with maturin 1.15.0.
- **No `build.rs` anywhere, and no sdist is ever produced** in this repository.
- **Raising the Python floor changes no wheel tag.** There is no abi3 or
  `py-limited-api` configuration. The one real effect is maturin's interpreter
  discovery, which the contract harness avoids by pinning `--interpreter sys.executable`.
- **Pytest collection, Rust test wiring, and test filesystem hygiene are correct**
  apart from the `rel-out` case. All 20 sibling Rust test files are declared
  (14 in the CLI crate, 6 in core); no real test file is invisible to collection;
  Python tests write only through `tmp_path`.
- **No test or fixture references a path being deleted.** The only textual mentions
  are inert strings, plus a GitLab provenance comment at `tests/src/test.py:2`.

## Watch items

- **`crates/complexipy-cli/src/utils/sarif.rs:11` `SCHEMA` must survive.** It is
  the SARIF-mandated `$schema` URL, unrelated to upstream identity. D's
  verification grep does not match it, but a broad "strip all URLs" pass would
  delete it. `sarif_file_created_and_valid` asserts it is a string, so a deletion
  would at least fail a test.
- **`gitlab` and `sarif` output formats lost their only plausible consumers** when
  B removed CI. No longer a watch item: it is now a decision in D, which opens
  `sarif.rs` regardless.
- **uv's `**/*.rs` cache key has no gitignore awareness**, so it nominally includes
  generated `.rs` under `target/`. Whether uv filters that internally was not
  established.
- **Raising the floor shifts ty's inferred analysis target**, which can change the
  diagnostics encoded in `EXPECTED_DIAGNOSTICS` in the contract harness. Run it
  before and after the bump.
- **`.claude/skills` symlink status was not confirmed.** H asserts it must stay a
  symlink and `AGENTS.md` forbids replacing it with copies, but whether it is
  tracked was not established by the completeness lane. One
  `ls -la .claude/ && git ls-files .claude/` settles whether a fresh clone gets it.
