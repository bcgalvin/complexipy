# complexipy

> Cognitive complexity analyzer for Python - measures how hard code is for humans to understand.

This is the single source of truth for agent instructions in this repository.
`CLAUDE.md` is a pointer to this file plus a short Claude-Code-specific section -
put anything tool-agnostic here, not there. See [Keeping This File Current](#keeping-this-file-current).

## Scope and engineering defaults

This is local-only work by one developer on one machine: macOS arm64 with
CPython 3.14+. The parent project consumes locally built wheels on that same
machine. There is no external distribution, sharing or portability requirement.

- Backward compatibility is not required. Change contracts directly when the
  task calls for it, update documentation and tests, and coordinate adoption
  with the parent. Do not add legacy modes or shims to preserve old behavior.
- Prefer a direct command, existing local tool and small config over a wrapper,
  installer or framework. For git-cliff, a local `--version` check is sufficient.
- Do not add artifact manifests, download/checksum machinery, cross-platform
  support, environment isolation layers, CI or generalized release automation
  merely because other projects use them. Add machinery only for a demonstrated
  local problem or an explicit request; explain that need first.
- Test actual behavior and regressions that matter here, not hypothetical
  distribution, portability or hostile-machine scenarios. Keep the existing
  analyzer and installed-wheel contract gates: they verify the actual local
  consumer, not a speculative deployment target.
- Treat research as evidence to select from, not a checklist to implement.
  Prefer manual verification for infrequent local maintenance when it is clear
  and sufficient.
- If a standalone Python tool is actually needed, use PEP 723 metadata for its
  Python requirement and dependencies (`[]` for stdlib-only scripts), and run it
  with `uv run --script`. Keep its Python floor aligned with this project.
  That convention is not a reason to wrap a working native CLI in Python.

## Tech Stack

- **Language:** CPython 3.14+ (package shell) + Rust (engine, CLI, diff)
- **Rust toolchain:** rustup's latest stable, edition 2024; no pinned release
- **Framework:** clap (CLI args), owo-colors/syntect/comfy-table (terminal output)
- **Package Manager:** uv (Python), Cargo (Rust)
- **Build:** maturin (Rust -> Python extension)
- **Docs:** plain markdown in `docs/`, read from the repository
- **Distribution:** local source-built wheels for `recsys-code-quality`, not PyPI;
  the current validation target is CPython 3.14 on macOS arm64

The analysis engine is Rust; the CLI and public Python API are thin wrappers over a
PyO3 extension module (`complexipy._complexipy`). Scoring follows G. Ann Campbell's
SonarSource cognitive complexity paper.

## Project Structure

```
complexipy/
+-- crates/                       # Cargo workspace (root Cargo.toml is virtual)
|   +-- complexipy-core/          # engine: algorithm, types, rules, runner, diff
|   |   `-- src/
|   |       +-- cognitive_complexity.rs   # AST walking + scoring algorithm
|   |       +-- classes.rs                # Data types (FunctionComplexity, RefactorPlan, ...)
|   |       +-- config.rs                 # Config discovery shared by the CLI and the LSP
|   |       +-- refactor_plans.rs         # ComplexityRegion tree + build_refactor_plans()
|   |       +-- rules/                    # Clippy-style refactor rule system
|   |       |   +-- types.rs              # RefactorRule trait + RuleMetadata
|   |       |   +-- complexity.rs         # Concrete rules (C001-C005, C007, C011)
|   |       |   `-- registry.rs           # Registration, filtering, ranking, overlap
|   |       +-- runner.rs                 # Local file/dir walk + shared entry points
|   |       +-- diff.rs                   # git-diff comparison (compute_diff, DiffEntry)
|   |       +-- api.rs                    # Rust-level code_complexity / file_complexity
|   |       +-- utils.rs                  # CSV/JSON writers, snapshot I/O, AST helpers
|   |       `-- helpers/exclude.rs        # Walker exclusion + the LSP's path matcher
|   +-- complexipy-types/         # shared enums; `python` feature builds enum.Enum classes
|   +-- complexipy-cli/           # CLI: clap args, output rendering, run orchestration
|   +-- complexipy-lsp/           # stdio language server: diagnostics, inlay hints, hover
|   `-- complexipy-python/        # PyO3 module (_complexipy) + py_diff wrappers
|
+-- complexipy/                   # Python package: thin re-export layer over Rust
|   +-- __init__.py               # Public API: imports _complexipy, file_complexity wrapper
|   +-- cli.py                    # Console-script bootstrap -> run_cli, or run_lsp for `lsp`
|   +-- py.typed                  # PEP 561 marker
|   `-- _complexipy.pyi           # Type stubs for the Rust extension
|
+-- tests/                        # pytest test suite
|   +-- main.py                   # Core tests + paper conformance
|   +-- src/                      # Test fixture .py files (excluded from collection)
|   +-- fixtures/refactor_plans/  # Rule-behaviour fixtures
|   +-- contract/                 # Installed-wheel stub/runtime contract harness
|   `-- test_*.py                 # Utility module tests
|
`-- docs/                         # Local markdown reference (no site)
```

## Commands

Verification is manual: no repository-managed Git hooks or CI run the gates below.
For a parent wheel refresh, use `vendor-build`: the parent requires an external
build environment and Cargo output, without an editable project install in the
provider checkout. That artifact-build route is separate from the development
commands below.
The `verify` skill sequences these checks and adds a built-CLI smoke invocation.
For documentation/skill-only changes, the build and test gate does not apply:
check the instructions, references, Markdown structure, ASCII punctuation and
`git diff --check`, and explicitly report that the standing gate was not run.

### Setup

```bash
uv sync
```

Rust comes from rustup's stable channel with the `clippy` and `rustfmt`
components (`rustup component add clippy rustfmt` if either is missing).

`uv sync` creates `.venv`. Workspace builds unify core's `python` feature, so
Cargo links pyo3 into test binaries; `.cargo/config.toml` sets `PYO3_PYTHON` to
`.venv/bin/python` so those builds use the project interpreter rather than the
first `python3` on `PATH`. Run `uv sync` before workspace Cargo commands. An
exported `PYO3_PYTHON`, or maturin's `--interpreter`, takes precedence.

### Build (Rust extension)

```bash
uv run maturin develop
```

**After editing any `crates/**/*.rs`, rebuild before running pytest** - otherwise
pytest exercises the previously built `.so`, and both passing and failing results
are meaningless.

### Test

```bash
uv run pytest
cargo test --workspace --locked
```

Installed-wheel typing contract (builds a wheel, installs it into a fresh
venv, type-checks `tests/contract/cases/` from a neutral directory with ty,
and checks runtime agreement; `--self-test` also proves wrong expectations
fail):

```bash
uv run python tests/contract/check_stub_contract.py --self-test
```

Single test:

```bash
uv run pytest tests/main.py::TestFiles::test_match
uv run pytest tests/test_refactor_plans.py::test_long_elif_chain_on_single_variable_recommends_match
uv run pytest -k refactor
cargo test -p complexipy-core --locked rules::registry
```

### Lint, Format & Type Check

```bash
uv run ruff check .
uv run ruff format --check .
uv run ty check .
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

Use `uv run ruff check --fix .` for safe lint fixes, including import sorting,
then `uv run ruff format .` for Python formatting. Review the resulting diff.
Ruff enables
`E4`, `E7`, `E9`, `F`, `I`, and `B`, including ordinary tests. The semantic
fixture trees `tests/src/**` and `tests/fixtures/**` are excluded from linting
and formatting; `force-exclude` also protects explicitly supplied fixture paths.
Excluded explicit paths are skipped, so a successful command does not mean those
files were checked.

All five crates inherit `[workspace.lints]` from the root `Cargo.toml`. The
rustc table forbids `unsafe_code` and warns on `unnameable_types` (a public
signature must not expose a type that users outside the crate cannot name) and
`elided_lifetimes_in_paths` (write a borrowing type as `Options<'_>`).
`unreachable_pub` stays off: `dead_code` already reports unused `pub` items in
private modules, and the lint cannot see into the cli and lsp crates, whose
roots export every module. The Clippy table enables the `pedantic`
group, which also covers lints that Clippy later moves into it, at
`priority = -1` so that individual entries override it. It allows twelve
pedantic lints that do not pay off in these unpublished crates:
`cast_possible_truncation`, `cast_possible_wrap`, `cast_precision_loss`,
`cast_sign_loss`, `format_collect`, `format_push_string`, `implicit_hasher`,
`missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`,
`struct_excessive_bools` and `too_many_lines`. It warns on these restriction
lints: `exit`, `dbg_macro`, `todo`, `unimplemented`, `panic` and `unreachable`;
`mod_module_files` (modules use `foo.rs` plus a `foo/` directory, never
`mod.rs`); `iter_over_hash_type` (output order must not depend on hashing);
`allow_attributes` and `allow_attributes_without_reason` (the suppression rule
under Code Style); `print_stdout` and `print_stderr` (the output rule there);
and `string_slice` (slicing text by byte offset needs a proven char boundary, so
prefer `str::get` or char-based methods). Root `clippy.toml` sets
`avoid-breaking-exported-api = false` because no crate is published,
`allow-panic-in-tests = true`, and `excessive-nesting-threshold = 6`, the
deepest block nesting in the code today (extract a function rather than nest
deeper). It bans `std::env::set_current_dir` because test binaries share one
process working directory. Lints are declared as warnings, so
promotion to errors comes from the `-D warnings` flag on the Clippy commands and
nothing else enforces it automatically. The restriction, nursery and cargo
groups are not enabled. `cargo clippy --fix --workspace --all-targets --locked`
applies the machine-applicable fixes. To count findings, for example after
`rustup update`, run Clippy without `-D warnings`; with it, Cargo stops after
the first failing crate.

ty treats `possibly-unresolved-reference`, `possibly-missing-attribute`,
`unused-ignore-comment`, and `redundant-cast` as errors. Every remaining warning
also causes failure, including unknown configured rules. Warning-driven failure
does not enable disabled rules; retain the explicit rule list. Tests remain excluded
from the root ty check; severity policy does not verify native/stub parity, which is
what the `tests/contract/` harness covers for its cases.
The analysis Python version is inferred from `requires-python` (currently 3.14).
Ruff also infers its Python target from that metadata. The contract harness uses
its running interpreter to build and install the wheel, so run it on CPython 3.14+.

Re-run rule, warning, and Python-target controls when upgrading ty.

### Feature-isolation lint check

One configuration is checked separately so workspace feature unification cannot
hide a missing feature gate or a warning that exists only without `python`. The
CLI check compiles and lints core without `python`, which matters because the
`serde(skip)` attributes on `FunctionComplexity` and `FileComplexity` are gated
on that feature: a standalone CLI build serializes a different snapshot shape
than the shipped extension does. It is a compile and lint check, not runtime
coverage, and it cannot see that shape difference.

```bash
cargo clippy -p complexipy-cli --locked -- -D warnings
```

### Changelog

Use the locally installed git-cliff directly. Confirm
`~/.local/bin/git-cliff --version` reports `2.14.1`; a newer Homebrew
`git-cliff` can shadow it on `PATH`. Root `cliff.toml` holds the configuration.
The short manual procedure is in `docs/changelog.md`, including the upstream
baseline to move when a sync is recorded with a merge. No installer, wrapper or
automated release pipeline is maintained here.

### Run

Root `complexipy.toml` is the sole checked-in analyzer config. It keeps the
self-dogfooding threshold at 15 and excludes `tests/**`.

```bash
uv run complexipy <path>
uv run complexipy . --diff rcq --max-complexity-allowed 15
uv run complexipy complexipy --failed          # dogfood the tool on itself
uv run complexipy lsp                          # stdio language server
cargo run -p complexipy-lsp --locked           # the same server from the tree
```

## Branches and remotes

- `rcq` is the fork's only working branch. Commits, release tags and the
  parent's gitlink live on it, and `origin/rcq` on the public GitHub fork is its
  backup. Local `origin/HEAD` points at it, pinned with
  `remote.origin.followRemoteHEAD=never`.
- The GitHub default branch is `main`, so the fork's landing page and its
  "Sync fork" button act on the mirror. Never use GitHub's sync on `rcq`: it
  would merge upstream there, or overwrite it with `--force`.
- `origin/main` is a pristine mirror of upstream `main`, advanced only by
  GitHub's fork sync (`gh repo sync bcgalvin/complexipy -b main`). Keep no local
  `main` branch; never commit or push to `main`.
- Upstream tags are never fetched. Fetch with `git fetch --no-tags origin`.
- The `sync-upstream` skill evaluates `origin/main` against `rcq` and records
  each adoption with a merge.

## Architecture

### Layering

```
complexipy/cli.py        console-script bootstrap: sys.argv -> run_cli(), or run_lsp() for `lsp`
complexipy/__init__.py   public API: re-exports _complexipy names + file_complexity wrapper
  `- complexipy._complexipy  PyO3 module (crates/complexipy-python)
       +- run_cli -> complexipy_cli::run::run_at()   clap args -> RunConfig -> display/exit
       +- run_lsp -> complexipy_lsp::run_server()    stdio LSP loop, GIL released
       +- code_complexity / file_complexity         engine entry points (complexipy-core)
       `- compute_diff / has_regressions            diff ratchet (complexipy-core)
```

### The FFI contract

Shared analysis types cross into Python through
`crates/complexipy-core/src/classes.rs` (`FileComplexity`, `FunctionComplexity`,
`LineComplexity`, `RefactorPlan`, `CodeSuggestion`, `IgnoredLocation`,
`RemovableIgnore`, `CodeComplexity`). Adding or removing a type changes
`crates/complexipy-core/src/classes.rs`, the `#[pymodule]` export list in
`crates/complexipy-python/src/lib.rs`, and `complexipy/_complexipy.pyi`. A field
change updates its Rust definition and the stub plus all Rust struct literals;
`add_class` does not enumerate fields. The core crate's `python` feature gates
both `#[pyclass]` and `serde(skip)` on the shared types and enables
`complexipy-types/python`.

`RuleCategory`, `Applicability` and `DiffStatus` have one Rust definition each in
`crates/complexipy-types/src/lib.rs`; core re-exports them from `classes.rs` and
`diff.rs`. Behind its `python` feature, `complexipy-types/src/python.rs` builds
real `enum.Enum` classes whose values equal the member names, converts members
in both directions, and the module registers the classes with `m.add`. A member
change updates the Rust enum, its Python member table and the stub. The Python
`DiffEntry` is defined in `py_diff` in `crates/complexipy-python/src/lib.rs`,
with conversions to core's `DiffEntry`.

The eight result structs in `classes.rs` have getters but no Python constructors.
Their stub properties are read-only; a required `Never` argument to `__new__`
rejects direct construction statically, including zero-argument calls. It is a
typing-only guard, not a runtime token API. The three enums are standard
`enum.Enum` classes: calling one looks up an existing member by value, and
members cannot be reassigned. `DiffEntry` has a real constructor. All twelve
exported native types reject subclassing; their stub classes are `@final`. Keep
these promises covered by the installed-wheel contract's positive
getter/member, negative assignment/construction/reassignment/subclass, and
runtime checks.

Function changes must also keep the binding, stub, wrapper and public exports in
sync. Use explicit `#[pyo3(signature = ...)]` for defaulted arguments and test
omitted-argument and keyword calls at runtime. A Rust `Option` alone does not
specify a Python default.

`complexipy/__init__.py` is the public Python API surface: `code_complexity`,
`file_complexity`, `collect_all_ignored_locations`,
`collect_removable_ignored_locations`, `compute_diff`, `has_regressions`, and the
`DiffEntry` / `DiffStatus` types. `run_cli` and `run_lsp` are process bootstraps
in `_complexipy`, declared in the stub but kept out of `__all__`. Keep the
implementation, exports, documentation and tests aligned when deliberately
changing a contract. New exports belong in
`__init__.py` + `__all__` and on `docs/python-api.md`. The Rust re-exports in
`crates/complexipy-core/src/lib.rs` have their own contract tests in
`crates/complexipy-core/tests/lib_surface.rs`; they include Rust-only entry
points and core diff types, so they are not a copy of Python's `__all__`.
Neither surface requires backward-compatibility shims or a major-release
ceremony; report changed behavior and coordinate the parent's wheel adoption.

File analysis and both ignored-location collectors use a canonical existing
root for relative input lookup and result identity. Public `file_complexity`
accepts keyword-only `base_path="."`; the native function requires `base_path`.
Collectors and `run_analysis_shared` use `invocation_path`. Files inside the
root have root-relative paths; files outside it have canonical absolute paths.
Failed entries are plain absolute resolved paths, canonical when they exist,
never `path: message` strings. A failed exclusion setup identifies its directory;
a walker error without a path identifies the walk root. Emitted traversal and
ignore-rule errors join read/parse failures without discarding successful rows.
An invalid root fails the call. Directory inputs apply discovery/exclusion
filters; explicit files bypass those filters. Keep this shared contract covered
by `tests/test_path_roots.py`, `tests/test_population_failures.py`,
`crates/complexipy-core/tests/runner_paths.rs` and the installed-wheel harness.
Do not equate reported walker errors with complete filter validation: the
`ignore` dependency suppresses ignore-file I/O errors internally; see the catalog.

CLI analysis, requested marker collection and automatic removable-marker
collection contribute to one failure gate in both quiet and normal modes.
Deduplicate failure diagnostics, not successful input rows. Any incomplete
collection prevents snapshot creation/watermark updates and cache replacement;
partial rows may still be exported with a nonzero exit and stderr diagnostics.
Partial runs do not load cached deltas or evaluate snapshots. Complete requested
marker JSON writes even an empty array; a failed marker collection invalidates
the requested marker JSON file instead of leaving a stale complete inventory.
Missing/empty resolved CLI path lists and malformed/unreadable discovered TOML
candidates fail before analysis; absent config and valid filtered-empty targets
are not errors. Discovery is `read_complexipy_config` in
`crates/complexipy-core/src/config.rs`, shared with the language server: the
first existing candidate decides, and a read or parse failure, or a failed
validation of the keys a consumer reads, is never replaced by a later candidate
or by defaults. Each consumer validates only its own keys, so one file can be
valid for the CLI and invalid for the server, or the reverse. The server shows
a failure with `window/showMessage` when the error changes and publishes no
results until the configuration reloads.

### Rust core

- `crates/complexipy-core/src/cognitive_complexity.rs` - the algorithm. Parses with
  `ruff_python_parser`, walks the AST, and accumulates structural / nesting / boolean
  increments. While scoring, it also records a tree of `ComplexityRegion`s.
- `crates/complexipy-core/src/refactor_plans.rs` - defines `ComplexityRegion` /
  `RegionKind` / `ComplexityResult` and `build_refactor_plans()`, which lazily builds
  a `OnceLock<RuleRegistry>` and delegates to it. Scoring produces regions; regions
  produce refactor plans. Keep that direction - rules consume regions to find
  structure. Expression parsing inside rules and re-parsing a source splice to
  measure its reduction are allowed.
- `crates/complexipy-core/src/rules/` - the refactor rule system (see below).
- `crates/complexipy-core/src/runner.rs` - local file/dir expansion, exclusion
  globs, and the shared entry points (`run_analysis_shared`, `file_complexity_shared`,
  the ignored-location collectors).
- `crates/complexipy-core/src/diff.rs` - git diff comparison, `DiffEntry` /
  `DiffStatus`, staged diff, regression ratchet.
- `crates/complexipy-core/src/config.rs` - config discovery, `StringOrList`, the
  default threshold and the language server's `LspConfig`.
- `crates/complexipy-core/src/helpers/exclude.rs` - two exclusion matchers: the
  walker's pattern program in `get_paths_to_process`, and `is_path_excluded`,
  which the language server uses for open documents. The walker matches
  relative to each walked directory and skips explicit files; the server
  matches relative to the workspace root. `helpers/exclude/tests.rs` pins them
  together for walks from that root, so a change to one has to keep the other
  in agreement. The server skips malformed patterns; the walker fails the
  directory.
- `crates/complexipy-core/src/api.rs` - Rust-level `code_complexity` /
  `file_complexity` (mirrors the Python public API).
- `crates/complexipy-core/src/utils.rs` - CSV/JSON writers, snapshot file I/O, and
  AST helpers (`count_bool_ops`, noqa/ignore-comment scanning).

### Refactor rules (`--suggest-refactors`)

A rule implements `RefactorRule::check(region, source, index, def_names,
function_complexity) -> Option<RefactorPlan>` plus a `&'static RuleMetadata`.
`index` is a `LineIndex`; `def_names` is the collected set of definition names.
`RuleMetadata::new_plan()` prefills the identity fields so
`id` / `category` / `applicability` / `description` can only ever come from
metadata; rules fill in the dynamic fields via `..metadata().new_plan()`.

`RuleRegistry::analyze()` then, in order: collects plans from active rules over
the region tree recursively, drops any plan with `estimated_reduction < 1` as noise, sorts by
spliceable desc -> `effectiveness` desc -> reduction desc -> line asc (a
machine-applicable replacement beats a help-only plan of higher
effectiveness), resolves overlapping line ranges by keeping the
higher-spliceable/higher-effectiveness/higher-reduction plan, and caps at 5
plans per function.

`effectiveness` in `RuleMetadata` is the single source of truth for ranking - the
registry reads it via `effectiveness_by_rule_id()`, not a separate hardcoded
ranking switch. Adding a rule is therefore: write the struct + `impl RefactorRule` in
`crates/complexipy-core/src/rules/complexity.rs`, set its `effectiveness` tier, register it in
`RuleRegistry::register_defaults()`, and document it in `docs/rules.md`.
Update `crates/complexipy-core/src/rules/registry/tests.rs` as well:
`fixture_for`, the checked-rule count
in `every_registered_rule_produces_a_plan_consistent_with_its_own_metadata`,
`effectiveness_matches_documented_tiers` and `rule_applicability_tiers_are_pinned`.
`MaybeIncorrect` requires a test for each known failure shape; the tier test
rejects it until such a rule exists. Add behavioral fixtures and assertions
under `tests/fixtures/refactor_plans/` and `tests/test_refactor_plans.py`.
The `add-refactor-rule` skill is the task procedure for this lockstep.

Rule selection lives in one `RuleSet` (`rules/types.rs`), carried with
`check_script`, `no_ignore` and `with_plans` in `AnalysisOptions` through the
scorer, runner, CLI, language server and Python bindings; `rules::default_registry()`
is the shared registry. `--ignore` wins over `--select`, a bare marker drops the
function, and a rule-list marker (`# complexipy: ignore[C007]`) subtracts rules
for one function only and is never reported by the marker collectors. Rule ids
must stay a letter followed by digits: `utils.rs` `is_rule_id` treats any other
bracketed text as a reason for a whole-function suppression.

Guiding principle for rule output: never emit a suggestion the tool cannot stand
behind. If a heuristic isn't confident, emit `help` text rather than a wrong
`suggestion`, and never print a complexity number the code knows is fabricated.

### Crate split

The workspace splits the build across five crates:

- `complexipy-types` - the shared `RuleCategory`, `Applicability` and
  `DiffStatus` enums. Its optional `python` feature builds them as Python
  `enum.Enum` classes.
- `complexipy-core` - the engine; depends on types. One optional feature,
  `python`, which adds the pyo3 `#[pyclass]` attributes **and the `serde(skip)`
  attributes** to the shared types and enables `complexipy-types/python`.
  Everything else is unconditional.
- `complexipy-cli` - clap args + output rendering; depends on core.
- `complexipy-lsp` - the stdio language server (`lsp-server`, `lsp-types`);
  depends on core. Its `complexipy-lsp` binary is for running the server from
  the tree.
- `complexipy-python` - PyO3 module; depends on core (`python`), types
  (`python`, for the enum classes), the cli crate (for `run_cli`) and the lsp
  crate (for `run_lsp`). Built by maturin via `manifest-path` in pyproject.toml.

Dependency direction is one-way: python -> cli and lsp -> core -> types. Never
the reverse. Adding a dependency means adding it to the crate that uses it.

## Testing

- `tests/main.py` - core suite: asserts exact complexity totals for the fixtures in
  `tests/src/`, plus SonarSource paper conformance. If you change the algorithm, these
  hardcoded numbers are the contract you're renegotiating - update them deliberately,
  never to make a run go green.
- `tests/test_*.py` - one file per Python-side concern; refactor-plan fixtures and
  behaviour tests against the public API.
- `tests/fixtures/refactor_plans/` - fixtures for rule behaviour, deliberately kept out
  of the `tests/src/` complexity corpus so rule work doesn't perturb the asserted
  totals.
- `tests/test_lsp.py` - drives `complexipy lsp` as a subprocess client. Read the
  child's stdout with `os.read` on the raw fd; a buffered read swallows frames.
  `crates/complexipy-lsp/tests/protocol.rs` covers the same protocol over an
  in-memory connection.
- `tests/contract/` - the installed-wheel stub contract harness. `cases/*.py` are
  deliberately wrong or right consumer snippets with expected ty diagnostics encoded
  in `check_stub_contract.py`; they are not collected by pytest and are outside the
  root ty scope, but they are inside Ruff's scope, so cases must stay lint- and
  format-clean (type errors only). Adding a stub declaration means adding a case that
  proves it.
- Rust tests live next to their module. Public-API tests go in the crate's
  `tests/` directory; tests that need private items are a `mod tests;` child module
  in a sibling file (e.g. `crates/complexipy-core/src/utils.rs` ->
  `crates/complexipy-core/src/utils/tests.rs`). No `#[path]`
  wiring - a new test file is invisible until the owning module declares it.
- `pyproject.toml` sets `python_files = ["test_*.py", "main.py"]`, so `tests/main.py` is
  a test module (not a script), and `norecursedirs = ["tests/src"]` keeps the fixture
  `.py` files from being collected.

## Code Style

- No explanatory comments in code. The code must speak for itself. Required
  PEP 723 metadata in a standalone script is configuration and is permitted.
- Suppress a lint with `#[expect(lint, reason = "...")]` on the smallest item
  that covers it, never `#[allow]`; an expectation that stops firing fails the
  gate. The reason is one short clause naming the external constraint. It is
  lint data read by the compiler, not an explanatory comment. Use
  `#[cfg_attr(test, expect(...))]` or `#[cfg_attr(feature = "python", expect(...))]`
  when a lint fires in one build only; a lint that fires inside PyO3-generated
  code needs the expectation on the enclosing module.
- Only `crates/complexipy-cli/src/run.rs` writes to stdout, and only it and
  `crates/complexipy-lsp/src/server.rs` write to stderr; each carries a
  module-level expectation for the print lints. Stdout is the LSP protocol
  stream while the server runs, including inside the Python process through
  `run_lsp`, so core, types and the bindings never print.
- ASCII punctuation only. Never use Unicode dashes (em dash U+2014, en
  dash U+2013, horizontal bar U+2015) in code, comments, docs, or commit
  messages. Use ASCII `-`.
- Docstrings only when necessary, and only about what the function does. Never changelog or history notes.
- Conventional Commits: `type(scope): description` (e.g., `fix(diff): resolve path for nested invocation`).
- Markdown has no repository formatter for now. Preserve the surrounding style
  manually, keep punctuation ASCII, and run `git diff --check`. These checks do
  not validate Markdown structure or rendering. See the `SKILL.md` warning below
  before introducing any formatter.
- Ruff for linting and formatting (line-length 80, indent-width 4; ordinary tests included, `tests/src/**` and `tests/fixtures/**` excluded).

## Key Files

- `crates/complexipy-core/src/cognitive_complexity.rs` - Core algorithm: parses Python AST via ruff, computes cognitive complexity with nesting/structural/boolean increments
- `crates/complexipy-core/src/refactor_plans.rs` - `ComplexityRegion` tree, `build_refactor_plans()` registry entry point
- `crates/complexipy-core/src/rules/types.rs` - `RefactorRule` trait, `RuleMetadata`, `new_plan()`
- `crates/complexipy-core/src/rules/registry.rs` - Rule registration, noise filtering, effectiveness ranking, overlap resolution
- `crates/complexipy-core/src/diff.rs` - Git diff comparison, `DiffEntry`/`DiffStatus`, `compute_diff`, `has_regressions`
- `crates/complexipy-core/src/runner.rs` - Shared entry points: `run_analysis_shared`, `file_complexity_shared`, ignored-location collectors
- `crates/complexipy-python/src/lib.rs` - PyO3 module `_complexipy`, pyfunctions, `py_diff` wrappers
- `crates/complexipy-lsp/src/server.rs` - language server loop, config loading, debounced publishing
- `crates/complexipy-core/src/config.rs` - config discovery shared by the CLI and the language server
- `crates/complexipy-types/src/python.rs` - Python `enum.Enum` classes and conversions for the shared enums
- `crates/complexipy-cli/src/run.rs` - `run_at()`: config -> analysis -> snapshot -> display -> exit code
- `crates/complexipy-cli/src/utils/config.rs` - `resolve_config()`: merges CLI args + TOML into `RunConfig`
- `crates/complexipy-cli/src/output.rs` - Console display, `handle_display`, `handle_results_storage`
- `crates/complexipy-cli/src/utils/paths.rs` - Output path resolution for CSV/JSON/GitLab/SARIF exports
- `crates/complexipy-cli/src/utils/snapshot.rs` - `evaluate_snapshot()`, `SnapshotEvaluation`, watermark logic
- `complexipy/__init__.py` - Public Python API surface (`__all__` and file wrapper)
- `complexipy/_complexipy.pyi` - Type stubs for the Rust extension module
- `tests/main.py` - Core test suite including SonarSource paper conformance tests

## Conventions

- **Package manager:** Always use `uv` - `uv run pytest`, `uv run ruff`, `uv run complexipy`
- **Build inputs:** Keep the dev-group maturin constraint aligned with
  `[build-system].requires`. uv's explicit cache keys cover manifests, Rust
  sources and the packaged Python sources, stubs and `py.typed` marker; keep
  that list aligned when adding packaged inputs.
- **Parser dependencies:** Keep both Ruff crates on the same explicit Git
  revision. A parser upgrade is a deliberate scoring-contract change, not a
  side effect of refreshing a mutable tag.
- **Rust toolchain:** Track rustup's latest stable; there is no
  `rust-toolchain.toml`, no `rust-version` and no clippy `msrv`. Each stable
  release changes the lint set, so after `rustup update` run the full `verify`
  gate before other work and fix new lint findings or renamed lint names in
  one dedicated commit, for example `chore(rust): adopt Rust 1.99 lints`. Keep
  that commit separate from feature work and from a parser revision bump.
  `resolver = "3"` makes `cargo update` prefer dependency versions that the
  installed rustc supports.
- **Cargo lockfile:** Regenerate and review `Cargo.lock` after dependency or workspace-version changes, and include required lockfile updates with the change. Every Cargo command here that resolves dependencies passes `--locked`, so drift fails rather than silently resolving. `[tool.maturin] locked = true` extends this to `maturin develop` and uv's automatic rebuilds: after a manifest edit they fail until `Cargo.lock` is deliberately regenerated (`cargo update --workspace` or `cargo update -p <crate>`). After a manifest change, `cargo clippy --workspace --lib --locked -- -D unused_crate_dependencies` catches declared dependencies that no library uses; it is limited to library targets because bins and integration tests report false positives.
- **Commits:** Only commit when explicitly asked. Never auto-commit. Stage explicit paths - never `git add -A` or `git add .`
- **Commit subjects:** Must follow Conventional Commits. There is no automatic
  validator. git-cliff retains unknown subjects in `Other` rather than silently
  losing history; that is not validation. Review the generated changelog before
  committing release artifacts.

## Agent Configuration Layout

Each piece of agent config has exactly one real copy; the other paths point at it.

- `AGENTS.md` (this file) is canonical. `CLAUDE.md` is a stub that imports it via
  `@AGENTS.md` and holds only Claude-Code-specific mechanics.
- `.agents/skills/` holds the real skill files, so any tool that reads `.agents/` sees
  plain files. `.claude/skills` is a symlink to `../.agents/skills` - **do not replace
  it with copies.** Add a new skill once, under `.agents/skills/<name>/SKILL.md`.

The local skills are short procedures, not automation entry points:

- `verify`: applicable gate and built-CLI smoke; docs-only checks when appropriate.
- `git-commit`: fork-specific messages and explicit-path staging, only when asked.
- `vendor-build`: build and check a wheel for the same-machine parent consumer.
- `release`: local version, changelog and authorized tag; no publishing.
- `add-refactor-rule`: rule metadata, registration, fixtures and ranking tests.
- `ffi-change`: binding/stub/export changes and the existing wheel contract.
- `sync-upstream`: deliberate comparison, not automatic adoption.

Keep each skill to one `SKILL.md` unless a concrete task needs more. Do not
reintroduce issue/PR/publishing workflows or duplicate the canonical commands
and invariants into a separate tooling framework.

Never run `mdformat` over `SKILL.md`: it rewrites the opening `---` as a
thematic break and the closing `---` as a setext heading, silently destroying
the YAML frontmatter that makes a skill loadable. Any future Markdown formatter
must exclude `(^|/)SKILL\.md$`. There is no formatter hook at present.

## Keeping This File Current

Treat this file as part of the change, not as documentation to catch up on later.

- If a change alters a **command**, a **structural invariant** (the FFI three-place
  contract, the region -> rule direction, Rust tests as `mod tests;` siblings), an **architectural
  boundary**, or a **convention**, update this file in the same commit - scope it
  `docs(agents)` when the doc edit stands alone. If a gate command changes, update
  `.agents/skills/verify/SKILL.md` in the same change: it repeats those commands
  in execution order and must not become a stale copy.
- New refactor rule, export format, or CLI flag: check whether Project Structure, Key
  Files, or Architecture now describe something that no longer exists.
- Do not restate any of this in `CLAUDE.md`. That file imports this one via
  `@AGENTS.md`; anything duplicated there will drift. It holds only
  Claude-Code-specific mechanics (which skills to load, subagent and plan-mode habits).

## Anti-Patterns

- Do not add explanatory comments to code. Use descriptive variable/function
  names instead; required PEP 723 script metadata is the narrow exception.
- Do not add docstrings that describe changelog or history. Docstrings describe what a function does.
- Do not commit without explicit user instruction.
- Do not use `pip` or `python -m` - use `uv run` for all commands.
- Do not modify `crates/` (Rust) without understanding what the `python` feature
  gates - it controls both the `#[pyclass]` attributes and the `serde(skip)`
  attributes, so a build without it serializes a different shape.
- Do not run pytest against stale Rust changes - `uv run maturin develop` first.
- Do not adjust asserted complexity totals in `tests/main.py` to make a run pass.
- Do not duplicate agent config. `CLAUDE.md` imports this file; `.claude/skills` is a
  symlink. Copies drift.
