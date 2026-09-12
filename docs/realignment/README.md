# Fork realignment

Working tracker for resetting this hard fork away from its upstream public-project
assumptions. Temporary: deleting `docs/realignment/` is the last item in workstream G.

A six-lane explore sweep ran against this plan before execution;
[`explore-results.md`](explore-results.md) holds its evidence trail and the
pre-existing defects it surfaced. Corrections it made to this tracker are already
absorbed below.

[`design-issues-and-bugs.md`](design-issues-and-bugs.md) catalogs everything found
along the way that is not realignment work: small bugs, fixed in passing or
deferred with a reason, and design tensions recorded with enough context to find
later. Add to it as you go.

Removal is the default. [`follow-up-tooling.md`](follow-up-tooling.md) records
every capability being removed with enough mechanics to rebuild it, which is what
makes aggressive pruning safe. Add an entry there rather than keeping something
"just in case."

Branch: `fork-realignment`, cut from `main` at `d690c9f`.

Line counts below come from `git ls-files <path> | xargs wc -l` over tracked files.
Directories containing binaries (`vscode/complexipy/img/`) and files without a
terminal newline make those totals approximate. Rows marked **(external)** rest on
evidence outside this repository and are not verifiable from a checkout alone.

Citations. Markdown files are cited by section heading and source files by path
plus enclosing symbol (function, test, struct field, constant), never by line
number: A through D moved lines in every file this tracker names, and a line
reference goes stale silently while a symbol reference fails loudly. Completed
checklist items keep the citations they were executed against - the commit that
closed them is their record. `explore-results.md` reads against `d690c9f` and
carries execution notes where the tree has since moved;
`design-issues-and-bugs.md` uses symbols throughout because it outlives this
directory.

## Position

- Hard fork of `rohaquinlop/complexipy`. No upstream contribution is intended.
- Sole developer on one macOS arm64 machine. No outside collaborators,
  external distribution or portability target. Prefer direct local commands and
  small config; research options are not implementation requirements.
- Consumed from source by `recsys-code-quality` as a locally built wheel
  (CPython 3.14, macOS arm64). There is no PyPI target. **(external)**
- Docs become local markdown in this repository. There is no published site and
  no multi-language support.
- The VS Code extension and the browser demo are not needed.
- CI is not part of this local loop. Rebuild ideas are records, not commitments;
  automation or portability work needs a concrete local need or explicit request.

## Baseline facts

Observed 2026-09-12 at the branch point `d690c9f`. Rows that describe later state
say so.

| Fact | Value |
| -- | -- |
| Latest upstream commit contained | `030e207` (Robin Quintero, `chore(release): bump version to 8.0.1`), arrived through merge `9926391` |
| First local commit | `87ad610 fix(refactors): refuse loop guards when statements follow the chain` |
| Fork work | 11 commits, range `030e207..d690c9f`, all authored locally, all conventional-commit clean. One is the merge `9926391` |
| Declared version | `8.0.1` in the workspace `Cargo.toml`, which is upstream's released number |
| Upstream latest release | `8.0.1`. `CHANGELOG.md` dates it 2026-09-06; the GitHub release timestamp is 2026-09-07 **(external)** |
| Consumed artifact | `complexipy-8.0.0+rcq.2` wheel recorded in `../../wheelhouse/README.md` **(external)** |
| Inherited tags | 39 upstream tags, `0.2.0` through `8.0.0`, present in this checkout. No `8.0.1` tag, though `030e207` is contained |
| Remotes | Exactly one: `origin = https://github.com/bcgalvin/complexipy.git`, the fork. **There is no upstream remote**, which G and H both once assumed |
| Remote branches | `origin` has only `main`, still at `030e207`. Nothing past it has ever been pushed: eleven commits at the branch point, 23 at `39e1bd5`. Local remote-tracking refs were stale; B pruned 36 of them |
| Remote tags | `git ls-remote --tags origin` returns zero. The 39 inherited tags are local to this checkout |
| Local branches | `rcq` (at `9926391`) and `followup-batch-1` (at `d690c9f`) are both contained in `main` |
| Parent gitlink | `recsys-code-quality` pins `repos/complexipy` at `c613bdb` (`8.0.0+rcq.2`), seven commits before the branch point and nineteen before `39e1bd5`. Its `.gitmodules` entry has no `branch =` key |
| GitHub Actions | 0 workflow runs have ever executed on this fork **(external)** |
| Repo settings | Issues disabled, no Pages site, `main` unprotected. Wiki was enabled and unused and `homepageUrl` was `complexipy.com`; B disabled the Wiki and cleared the homepage **(external)** |
| Tracked YAML | Was eight. B removed five, C two, and F the last (`.pre-commit-config.yaml`); no tracked `.yaml`/`.yml` files remain |
| Refactor rules | Seven: C001, C002, C003, C004, C005, C007, C011. The ID space is non-contiguous |

## Decisions

1. **Distribution.** Consumed from source by `recsys-code-quality`. No PyPI
   publishing, no wheel matrix, no downstream notification.
1. **Docs.** Rewritten as local markdown under `docs/`. No site, no custom
   domain, no Spanish mirror, no publishing tooling.
1. **Collaborators.** None expected. All contributor-facing machinery is removed.
1. **VS Code and web demo.** Removed, along with the WASM target that exists to
   serve them.
1. **Version.** Continue from `8.0.1`, abandon the `+rcq.N` local-version scheme.
   Bump to `8.1.0` as the closing commit of this realignment, since the artifact
   surface changes materially and `8.0.1` is upstream's number. Note that `8.1.0`
   re-acquires the same collision the moment upstream ships `8.1.0`; revisit if
   artifact provenance needs to survive a `sync-upstream` cycle.
1. **Branches.** `rcq` has nothing `main` lacks. Delete `rcq` and
   `followup-batch-1` once this work lands on `main`.
1. **Changelog.** Start fresh with git-cliff. The existing changelog tooling is
   not retained. Tool pinning and invocation are researched when the work is
   scheduled.
1. **Workflow.** Commit directly to `main` until this fork has CI that serves its
   own needs. Pull requests return only when updated GitHub Actions justify them.
1. **Skills.** The existing skills exist for cross-collaborator consistency,
   which is not a problem this fork has. Re-cut the slate around what this fork
   actually does.
1. **Shipped URLs.** Fully decouple. No external documentation URLs in tool
   output.
1. **`doc_url`.** Removed, not repurposed. The field goes from `RuleMetadata`,
   `RefactorPlan`, the stub, the JSON export schema, the CLI `References:` block,
   and SARIF, along with the `informationUri`/`helpUri` constants. `rule_id` still
   ships, so a consumer can map an id to local documentation itself.
1. **Benchmarks.** `benchmarks/` is removed. The harness is upstream's and
   baselines against upstream's PyPI 7.0.1 artifact, which is the wrong question
   for this fork. Replacement tooling baselines against a pinned `wheelhouse/`
   wheel.
1. **Local automation.** `.pi/` is an unused harness for a different agent runner
   and is deleted outright. `.pre-commit-config.yaml` and its supporting config go
   with it. Test automation is rebuilt afterward rather than migrated.

## Execution order

A through F, G preparation and H are complete. Next is G finalization and
coordinated parent adoption. G preparation resolved git-cliff and permanent document homes;
H has used and retired the predecessor skills. Keep the tracker until final
cleanup. The final version/tag and directory deletion are not part of H.

No automation runs the gates. Before B there were three candidates and none
qualified: `.pi/` belonged to a harness this fork does not use,
`.pre-commit-config.yaml` carried no Rust or Python test gate, and
`.github/workflows/CI.yml` triggered on `pull_request` only, which decision 8
abolishes. B deleted the first and third; F removed the pre-commit stack.
Verification during the realignment is therefore manual, using the standing gate
below. H wrote `verify` first; it records the procedure, not a scheduler.

Deletions precede rewrites so the rewrites describe the end state. B preceded G
because `release.yml` triggered on `push: tags: "*"` and G's closing version bump
is what the `release` skill tags; with the workflow now deleted, that hazard is
gone.

Review cadence: every workstream is reviewed by the oracle before committing,
each review in a **new chat with a self-contained briefing** - no reliance on a
prior thread. A thorough review follows once every workstream has landed. Each
workstream runs the applicable verification before its commit, corrects what
it itself falsifies rather than deferring that to E, and records anything it
finds in `design-issues-and-bugs.md`. Source, test, dependency and build/config
changes run the standing gate. Documentation/skill-only changes check the
instructions, references, formatting and `git diff --check` instead, and state
plainly that the build/test gate did not apply and was not run.

The standing gate, from `AGENTS.md`:

```
uv run maturin develop && uv run pytest
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
uv run ruff check . && uv run ruff format --check . && uv run ty check .
uv run python tests/contract/check_stub_contract.py --self-test
```

## Workstreams

### A. Remove the unshipped build targets

- [x] `vscode/` (15 files, ~2885 lines)
- [x] `web/` (7 files, ~1250 lines)
- [x] `build-wasm.sh`, `serve-web-version.sh`
- [x] `crates/complexipy-wasm/` (2 files, 46 lines). The workspace uses
  `members = ["crates/*"]`, so no manifest edit is needed.
- [x] The `wasm` feature in `complexipy-core`. It is declared `wasm = []` and gates
  exactly two sites: `CodeComplexity.version` in `classes.rs:117` and its
  initializer in `cognitive_complexity.rs:27`. The two consumers being deleted
  read `result.functions` and never `version`.
- [x] Collapse the `runner` feature. `complexipy-wasm` is the only consumer that
  sets `default-features = false`, so once it goes `runner` is on for every build
  and the feature is permanently-true dead configuration. Remove `default` and
  `runner` from `[features]` (keep `python`); make `ignore`, `globset`, `wax`, and
  `rayon` unconditional dependencies; strip the six `#[cfg(feature = "runner")]`
  attributes (`src/lib.rs:1,6,10,14,24` and `src/helpers.rs:1`) and the
  `#![cfg(feature = "runner")]` at `tests/collector_failures.rs:2`; and reduce
  `complexipy-python`'s dependency line to `features = ["python"]`.
  This also resolves a latent breakage: `tests/lib_surface.rs:7-12` imports
  runner-gated items with no `cfg`, so that crate's tests have never compiled
  without default features. With the feature gone there is no configuration in
  which those imports fail.
- [x] Workspace deps `wasm-bindgen`, `serde-wasm-bindgen`,
  `console_error_panic_hook` (`Cargo.toml:32-34`)
- [x] Regenerate `Cargo.lock` in the same commit. Every build path passes
  `--locked`, including A's own verify line, so between the deletion and the
  regeneration the standing gate is unrunnable.
- [x] `.gitignore`: `pkg/`, `vscode/complexipy/wasm/`, `web/wasm/`,
  `vscode/complexipy/complexipy-*.vsix`, `vscode/complexipy/.vscode/**`,
  `vscode/complexipy/.vscode-test/**`, `node_modules/`

Verify: `cargo check -p complexipy-cli --locked`, then the standing gate.

The two `--no-default-features` cross-target checks are dropped with the feature
they exercised. They only ever distinguished a build shape `complexipy-wasm`
produced; with `runner` collapsed there is one core configuration plus the `python`
feature, and the CLI check is what still carries weight - the `serde(skip)`
attributes on `FunctionComplexity` and `FileComplexity` are gated on `python`, so a
standalone CLI build serializes a different snapshot shape than the shipped one.

### B. Remove public-project, publishing, and unused tooling

- [x] `.github/` in full: `workflows/CI.yml`, `workflows/release.yml`,
  `workflows/pr-title.yml`, `ISSUE_TEMPLATE/`, `PULL_REQUEST_TEMPLATE.md`,
  `FUNDING.yml`
- [x] `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- [x] `.pi/` (6 tracked files), and the `.pi/rust-validate/` entry in `.gitignore`
- [x] `benchmarks/` (3 tracked files), and the `benchmarks/corpus/` entry in
  `.gitignore`
- [x] GitHub settings: disable the Wiki, clear `homepageUrl` **(external)**
- [x] No action on remote tags: `origin` carries none, so G's local deletion is
  sufficient and no `git push --delete` is needed.
- [x] `git remote prune origin`

On `release.yml`: only the leaf job `notify-downstream` consumes
`CROSS_REPO_TOKEN`, and nothing depends on it. The `release` job needs the build
and test jobs, `deploy-docs` needs `release`, and the workflow triggers on tag
push and `workflow_dispatch`. So it is reachable. It cannot publish successfully
under this fork's identity, which depends on PyPI trusted-publisher configuration
outside this repository; with no publisher registered it fails at the OIDC token
exchange, before `skip-existing: true` is ever consulted. The sharper repo-local
reason to delete it rather than rely on it failing: a stray tag push fans out
roughly 120 matrix jobs (42 linux, 14 musllinux, 14 windows, 14 macos, 1 sdist,
35 unit-test) before it ever reaches `release`.

Everything here is recorded in `follow-up-tooling.md`. The CI job definitions
carry one behavior worth preserving: `complexipy complexipy --failed`. The two
`--exclude` glob validations are **not** worth reproducing. Both pass
`--ignore-complexity`, and `ExitReport::success()` resolves to
`all_pass || ignore_complexity`, so their exit code was 0 whether or not the glob
matched anything; they proved path resolution and nothing else. Exclusion in the
analysis path needs real coverage instead.

Verify: nothing to build. Confirm
`git grep -n "\.github\|\.pi/\|benchmarks/"` returns only intended references.

### C. Docs rewrite and teardown

Write, before deleting the source material:

- [x] Scoring contract. **Do not transcribe `understanding-scores.md`** - it
  contradicts `tests/main.py::TestPaperConformance`, which `AGENTS.md` designates
  as the real contract. Its `:150` scores `with` as "+1 (context manager treated
  as if)" and nests beneath it, while `test_with_does_not_nest` pins `with` at +0
  with no nesting; its `:84` scores `match` as "+0" while
  `test_match_top_level_structural_increment` pins it at +1. Re-derive the page
  from the conformance tests.
- [x] Rule catalog (from `refactoring-rules.md`). Drop its `doc_url` JSON sample
  and the `print(f"  Docs: {plan.doc_url}")` example; decision 11 removes the
  field.
- [x] Public Python API surface (from `api-reference.md`). Its `RefactorPlan` tree
  lists `doc_url: str` and `references: List[str]`; D removes both. The page also
  omits three public fields that must be added: `reduction_is_measured` on
  `RefactorPlan`, `spliceable` on `CodeSuggestion`, and
  `additional_refactor_plans` on `FunctionComplexity`.
- [x] Diff and snapshot semantics (from `usage-guide.md`). Its DiffStatus section
  is the only written statement that `DiffStatus` is not an `enum.Enum`, has
  no `.name`/`.value`, and formats as `DiffStatus.REGRESSED`. Commit `d690c9f`,
  the branch point, exists to correct exactly that.

Delete:

- [x] Everything under `docs/` except `docs/realignment/`: `about.md`,
  `api-reference.md`, `benchmarks.md`, `changelog.md`, `CNAME`,
  `comparison-with-ruff.md`, `es/` (11 files, ~3411 lines), `img/`,
  `index.md`, `migration.md`, `refactoring-rules.md`,
  `understanding-scores.md`, `usage-guide.md`
- [x] `mkdocs.yml`, `mkdocs.es.yml`
- [x] `mkdocs-material` from the `dev` dependency group
- [x] Regenerate `uv.lock`
- [x] `.gitignore`: `site-es/`, `docs/_build/`

Retained prose carries MkDocs-only syntax (`!!! note` admonitions, `=== "tab"`
content tabs, `--8<--` snippet directives) that renders as literal text outside
MkDocs. De-MkDocs it rather than moving it verbatim.

Do not diagnose the site build: it is already broken. B deleted
`benchmarks/results.md`, which `docs/benchmarks.md` includes and `mkdocs.yml`
resolves with `check_paths: true`, so `mkdocs build` now aborts. `mkdocs.es.yml`
omits `check_paths` and instead emits the literal `--8<--` line. Both files are
deleted here anyway.

The audience is this repository and the downstream agents in
`recsys-code-quality`. Installation, integrations, comparisons aimed at
prospective users, and anything addressed to contributors do not return.

Verify: `uv sync --frozen` succeeds against the regenerated lock.

### D. Remove `doc_url` and `references`, and decouple the shipped URLs

`references` goes in the same pass: `rules/types.rs:52` is its sole writer and sets
`vec![]`, no rule overrides it, and once `doc_url` leaves `output_plan_references`
its guard can never be false in a real run - so the `References:` block is
unreachable and the helper does not survive. Both fields hit the same struct
literals, and each is a separate `--output-format json` schema change.

Origination:

- [x] `crates/complexipy-core/src/rules/types.rs:27` (`RuleMetadata.doc_url`) and
  `:55` (`new_plan()` copies it)
- [x] Seven `doc_url` literals in `crates/complexipy-core/src/rules/complexity.rs`
  (C001, C002, C003, C004, C005, C007, C011)
- [x] `crates/complexipy-core/src/classes.rs:73` (`RefactorPlan.doc_url`) and
  `:70` (`RefactorPlan.references`)

Emission:

- [x] `crates/complexipy-cli/src/output/refactor.rs:126,164-170`. Removing the
  argument leaves `output_plan_references` with only `plan.references`, which
  no rule populates, so the `References:` block stops rendering. Decide whether
  the helper survives at all.
- [x] `gitlab` and `sarif` output formats: **kept**. The parent documents but does
  not invoke them; the decision and its reasoning are in
  [`design-issues-and-bugs.md`](design-issues-and-bugs.md). `docs/cli.md`
  therefore needs no edit.
- [x] `crates/complexipy-cli/src/utils/sarif.rs`: `INFO_URI` (line 12, emitted at
  35 as `informationUri`), `HELP_URI` (13, emitted at 129 as `helpUri`), and
  `plan.doc_url` (167). All three keys are optional in SARIF 2.1.0
  **(external)**, so omitting them is schema-valid. Do not emit empty strings
  in their place.
- [x] `crates/complexipy-core/src/utils.rs` (`output_json_shared`) serializes
  `refactor_plans` wholesale into a hand-built `json!` map, so the
  `serde(skip)` on `FunctionComplexity.refactor_plans` does not apply and
  `doc_url` really is in the `--output-format json` schema. This is a
  consumer-visible schema change. It is recorded as a `BREAKING CHANGE:` footer
  on D's commit rather than in `CHANGELOG.md`, because G truncates and
  regenerates that file from the commit log and a hand-written entry would be
  discarded. The footer is what git-cliff reads.

Typing and tests:

- [x] The pages C wrote already describe the post-D surface - `docs/python-api.md`
  and `docs/rules.md` omit `doc_url` and `references` deliberately. Confirm rather
  than re-edit them. If this workstream also retires the `gitlab`/`sarif` formats,
  `docs/cli.md` does need an edit.
- [x] `complexipy/_complexipy.pyi:206,228` for `doc_url` and the `references`
  declaration plus its `__init__` parameter
- [x] While the stub is open: `RuleCategory` (`:13`) and `Applicability` (`:22`)
  carry the same defect already recorded for `DiffStatus` (`:34`) - all three are
  declared `(Enum)` with string values against a PyO3 simple enum whose runtime has
  no `.name`, no `.value`, and no iteration. The consuming repo already carries a
  `variants()` workaround for exactly this.
- [x] Also in the stub: `:717` and `:755` document `paths` as accepting "Git
  repository URLs", a feature removed in 8.0.0. This ships inside the wheel and is
  what the consumer's type checker reads.
- [x] `crates/complexipy-core/src/rules/registry/tests.rs:33` (struct literal),
  `:243` (`plan.doc_url == meta.doc_url`), `:251`
  (`plan.doc_url.starts_with("https://")`)
- [x] `crates/complexipy-core/src/utils/export_tests.rs:54` (struct literal)
- [x] `crates/complexipy-cli/src/utils/gitlab/tests.rs:55` (struct literal only;
  GitLab output does not emit `doc_url`)
- [x] `crates/complexipy-cli/src/output/refactor/tests.rs:49` (struct literal) and
  the `References:` assertions
- [x] `crates/complexipy-cli/src/utils/sarif/tests.rs`: `:55` (struct literal),
  `:181` (shipped URL literal), `:250` (`assert_eq!(plan_rule["helpUri"], ...)`,
  which fails once the key goes absent)
- [x] `tests/test_refactor_plans.py` `test_rule_metadata_has_doc_url`. Planned as
  a deletion together with the fixture only it loads,
  `tests/fixtures/refactor_plans/metadata_validation.py`; done differently,
  because the test also guards that `RuleMetadata` identity fields reach the plan
  at all. It is now `test_rule_metadata_reaches_the_plan`, its `doc_url`
  assertions are absence assertions, and the fixture stays in use.
- [x] Add a `tests/contract/cases/` case proving `doc_url` is gone. The harness
  currently covers `DiffEntry`, `DiffStatus`, `code_complexity`, and phantom
  helpers only; nothing exercises `RefactorPlan`. `phantom_import.py` is the
  prove-absence pattern *in shape only*: it proves a module-level name is absent
  via `unresolved-import`, while proving a field is absent needs
  `unresolved-attribute` on a plan reached through
  `code_complexity(...).functions[0].refactor_plans[0]`, since there is no `#[new]`
  to construct one. The runtime half is the `RUNTIME_CHECKS` constant, not
  `EXPECTED_DIAGNOSTICS`. Cases are outside root ty scope but inside Ruff's, so the
  new one must be lint- and format-clean.

A *field* change is a two-place typing change: `classes.rs` plus
`_complexipy.pyi`. `crates/complexipy-python/src/lib.rs` registers `RefactorPlan`
with `add_class` and never enumerates fields, and `classes.rs` uses `get_all`. The
three-place `#[pymodule]` lockstep applies to adding or removing a *type*.

While the contract case is open, capture a pre-existing stub over-promise nearby:
`_complexipy.pyi:34` declares `class DiffStatus(Enum)`, which type-checks
`.name`/`.value` against a runtime that has neither.

Verify: the standing gate. Then
`rg -n 'complexipy\.com|rohaquinlop|complexipy-teams'` across the tree to catch
any residual URL neither D nor E enumerated.

### E. Rewrite the identity surfaces

Completed after the review of C's pages at `7025244`, in
`refactor(fork)!: align identity and Python 3.14 contracts`. A fresh Oracle review
found no production regression; its stale-index finding and getter/constructor
test improvements were applied. The separately verified enum-construction and
subclassing typing gap remains open in the catalog.

- [x] Replace `README.md`'s upstream logo, badges, site links, PyPI installation,
  integration recipes, footer and dangling navigation with the fork relationship,
  local development commands and links to the six reference pages. Retain the
  upstream attribution and unchanged `LICENSE`.
- [x] `pyproject.toml`: identify the fork maintainer and repository, replace the
  description, remove the PyPI-discovery keywords and classifiers, and raise
  `requires-python` from `>=3.8` to `>=3.14`.
- [x] Regenerate `uv.lock`: remove the older-interpreter branches and their
  compatibility dependencies. `pre-commit` remains until F; this is not a tool
  upgrade pass.
- [x] `Cargo.toml`: identify the fork maintainer and repository/homepage. Remove
  the site documentation URL and its `documentation.workspace` inheritance in
  all three crate manifests. Leave the workspace version at `8.0.1` for G.
- [x] Correct `AGENTS.md`'s Python floor, both Git-URL walking claims, the
  type-versus-field FFI rule, the function-default binding contract, and the
  contributor-facing Windows note. Preserve the Rust stable-export compatibility
  promise separately from Python's exports. Keep the region-to-rule direction
  with the qualification that expression parsing and splice measurement are
  allowed. Normalize the edited file to ASCII.
- [x] Remove `utils/cache.rs::looks_like_remote` and its call in
  `normalize_target`, residue from removed Git-URL analysis. Remove the false
  core `lib.rs` comment claiming its stable re-exports mirror Python `__all__`.
- [x] Review `CLAUDE.md`: no E-owned edits needed. Its skill list stays until H.
  Pre-commit commands and the `SKILL.md` exclusion in `AGENTS.md` stay until F,
  which owns removing that stack in the same change as its guidance.
- [x] Modernize the Python annotations: remove the four future-annotations
  imports from the package, refactor tests and contract harness; replace the
  stub's `List`/`Optional`/`Tuple` and `tests/main.py`'s `List`/`Tuple` with
  builtin generics and unions. This is policy cleanup, not a claim that those
  aliases are invalid on 3.14.
- [x] Replace the eight phantom result constructors with a typing-only required
  `Never` argument to `__new__`; expose result attributes as getter-only
  properties. Remove runtime-impossible construction examples. Preserve the
  genuine `DiffEntry` constructor and all existing public fields and signatures.
- [x] Correct the stub and wrapper exception documentation, both ignored
  `invocation_path` descriptions, and the `additional_refactor_plans` count.
  Document `check_script` and `no_ignore` on both native analysis functions.
- [x] Correct two further stub contradictions found during the E review:
  `LineComplexity` counts boolean runs, not individual operators, and
  `FunctionComplexity.name` includes `Class::method`; nested functions are not
  separate results. These agree with `TestPaperConformance` and
  `TestScorerContract.test_methods_are_named_class_method`.
- [x] Update `docs/python-api.md`, `docs/rules.md` and the documentation index
  together with the stub so their warnings about defects fixed here do not
  become stale in reverse. Correct the stale DiffStatus formatting-test note
  and the nonexistent single-test command in `AGENTS.md` found by the Oracle.
- [x] Extend the installed-wheel contract with positive getter types for all
  rejected assignments and construction, and runtime
  rejection checks. Compare getter names and runtime value types against the
  installed stub, check independent list copies, and reject correctly typed
  positional construction as well as empty and keyword calls. Run every positive
  diagnostic case at runtime, not just `valid_usage.py`.

The floor change retargets Ruff and ty through `requires-python`; the contract
harness uses `sys.executable` for wheel building and installation. Verification
uses the existing CPython 3.14.3 environment. Cargo metadata edits do not change
resolved dependencies or workspace versions; review `Cargo.lock` for unexpected
drift rather than upgrading dependencies.

Outside E: pre-commit/config removal (F), version/changelog and permanent catalog
homes (G), skills (H), and all catalogued scoring/CLI behavior defects. The
maturin floor mismatch and Python-source cache-key question remain recorded design
issues rather than incidental edits to `pyproject.toml`.

Verified: the standing gate (144 pytest tests, 300 Rust tests, lint/format/type
checks and the eight-case installed-wheel contract self-test), the separate CLI
feature-isolation check, `uv sync --frozen`, built-CLI smoke with exact output and
both threshold exits, the Markdown hook, and whitespace/ASCII checks. The
extension was rebuilt before pytest. Version and `Cargo.lock` remain unchanged;
no wheel was vendored into the parent.

### F. Remove the pre-commit stack

Clean break. All three hook definitions are removed; automation will be rebuilt
rather than migrated. `follow-up-tooling.md` carries their behavior and replacement
requirements. Recorded by `chore(tooling): remove the pre-commit stack`.

- [x] Remove `.pre-commit-config.yaml` and `.mdformat.toml` (the latter contained
  only the obsolete `[plugin.mkdocs]` setting).
- [x] Remove the nine-setting `[tool.yamlfix]` block in `pyproject.toml`.
- [x] Remove `pre-commit` from the `dev` dependency group and regenerate `uv.lock`.
  The lockfile loses only pre-commit and nine orphaned dependencies: `cfgv`,
  `distlib`, `filelock`, `identify`, `nodeenv`, `platformdirs`, `python-discovery`,
  `pyyaml`, `virtualenv`. All retained third-party package records are unchanged.
- [x] Prune `.gitignore` of obsolete packaging, installer, coverage, translation
  and legacy IDE patterns, plus the redundant `libcomplexipy.dylib*` entry.
  Retain `target/` (maturin wheels and Cargo builds), `dist/` as a safeguard for
  other local packaging frontends, `.venv/`, the Python native extension and
  bytecode patterns, pytest/analyzer caches, and existing local editor/OS
  exclusions. Add explicit `.ruff_cache/` coverage instead of relying on Ruff's
  self-hiding directory. No ignored files were deleted.
- [x] Confirm no Git hook needs uninstalling: the submodule's hooks directory,
  under the parent's `.git/modules/repos/complexipy/hooks/`, contains only
  `.sample` files; `core.hooksPath` has no override. No Git config was changed.
- [x] Remove the dead `[tool.complexipy]` block; retain root `complexipy.toml`
  byte-for-byte. The removed block used `paths = ["crates", "complexipy"]`,
  `failed = true`, `quiet = true` and no exclusion. The retained config uses
  `paths = ["."]`, `failed = false`, `quiet = false`, `exclude = ["tests/**"]`.
  Discovery is first-hit-wins with no merge (`crates/complexipy-cli/src/utils/toml.rs`
  `get_complexipy_toml_config`), so removing the shadowed block changes no active
  setting. Consumer support for pyproject config remains, pinned by the sibling
  discovery tests.
- [x] Update `AGENTS.md` and `docs/cli.md` alongside the config removal. Interim
  Markdown policy: no repository formatter; preserve surrounding style manually,
  check ASCII punctuation and `git diff --check`. These checks are not Markdown
  validation. Keep the `SKILL.md` frontmatter hazard in the permanent agent guide,
  not only this temporary tracker.

Still deferred to the tooling rebuild:

- A `local` complexipy hook driving the **locally built** extension. The removed
  definition pinned `rohaquinlop/complexipy-pre-commit` at `v6.1.0`, not the fork.
- Decide whether to restore a Markdown formatter. Any formatter needs the
  `SKILL.md` exclusion: mdformat rewrites a skill's opening `---` as a thematic
  break and its closing `---` as a setext heading, destroying its YAML frontmatter.
- A `commit-msg` Conventional Commits check. `pr-title.yml` checked pull-request
  titles, not commits; G's planned git-cliff generation still has no validator.

Validation on CPython 3.14.3/macOS arm64: `uv lock --check`, `uv sync --frozen`,
`maturin develop`, 144 pytest tests, 300 Rust tests (`--workspace --locked`),
Clippy with `-D warnings`, Cargo format check, the standalone CLI compile check,
Ruff lint and format checks, ty, and the installed-wheel contract `--self-test`
all pass.
The full standing gate applies because F changes tooling dependencies and config,
not only documentation. `Cargo.lock` and root `complexipy.toml` are unchanged.

A temporary built-CLI smoke test loaded copies of the before/after project config:
identical output and exit status, `tests/**` excluded, threshold overrides at 1
and 2 returning 1 and 0 for a function of complexity 2. The existing ignored-path
inventory is unchanged; `pre-commit` is absent from the synced environment.
Changed files pass ASCII and `git diff --check` checks. No Markdown formatter was
run, following the new interim policy.

A case-insensitive reference sweep for `pre-commit`, `pre_commit`, `mdformat` and
`yamlfix` found only the permanent frontmatter warning, realignment records,
historical `CHANGELOG.md` entries, and a conditional downstream-hook example in
`release-notes/SKILL.md` ("Downstream Release Verification"). That example invoked
no removed tool at F's completion; H subsequently removed the entire skill.
`.claude/settings.json` has attribution settings only, not hooks. No live command references to the removed tooling remain.

Fresh Oracle review completed before commit (three reviewers). Follow-ups:
record the residue sweep, add the explicit lock freshness check, retain `dist/`,
move manual-verification guidance to "Commands", and tidy wrapping. No
source/test changes or unresolved F blockers; no G/H implementation or
parent-repository changes are included.

### G. Changelog, version, and branch cleanup

#### Preparation before H

- [x] Use the locally installed **git-cliff 2.14.1**, confirmed with
  `git-cliff --version`. This is sufficient for one developer on one machine;
  no repository installer, artifact manifest, wrapper or portability layer.
- [x] Add root `cliff.toml` and the short manual procedure in
  [`../changelog.md`](../changelog.md). Keep merge summaries, routine/unknown
  subjects and D/E breaking descriptions; do not render bodies/session trailers.
  Reserve nonbreaking release-bookkeeping subjects for changelog/version work.
- [x] Decide to regenerate the whole changelog at finalization, replacing the
  old preamble and both handwritten Unreleased entries rather than duplicating
  their source commits. The existing file stays unchanged during preparation.
- [x] Reserve `docs/maintenance/design-issues-and-bugs.md`,
  `docs/maintenance/follow-up-tooling.md` and
  `docs/maintenance/changelog-git-cliff.md` as permanent homes. Move the records
  only after H has used them; no duplicate copies now.

The research is [`changelog-git-cliff.md`](changelog-git-cliff.md). An initial
implementation over-applied its suggestions and added an installer, wrapper and
large test suite. Those uncommitted additions were removed after the maintainer
clarified the scope. `AGENTS.md` now records the local-only, single-developer,
single-machine rule and the requirement for a concrete need before adding such
machinery. Future Python scripts, if needed, should use PEP 723; this workflow
needs no Python script.

Manual validation at `5ec3c37`: native git-cliff 2.14.1 selected all 29 fork
commits exactly once, retained the merge and five empty-body entries, preserved
D/E migration details, and rendered ASCII without session trailers. This is
manual verification, not a checked-in contract framework. The existing standing
gate was rerun after simplification: 144 Python tests, 300 Rust tests, lint,
format/type checks, CLI feature isolation and the installed-wheel contract all
pass. A fresh Oracle review of the simplified diff preceded the commit; its
document-currency findings were applied. ASCII and whitespace checks pass;
no Markdown formatter was run.

#### Finalization after H

- [ ] Delete the 39 inherited upstream tags from this checkout. They remain on
  upstream.
- [ ] There is no upstream remote to set a fetch policy on, and `origin` carries
  no tags, so local deletion holds. If upstream is ever added as a remote,
  `--no-tags` belongs at add time - `git fetch` auto-follows tags reachable from
  fetched history and would re-import all 39 on the first fetch.
- [ ] Regenerate `030e207..HEAD` using the direct command in `docs/changelog.md`
  and review the candidate before replacing `CHANGELOG.md`. Not
  `87ad610..HEAD`, which loses the first fork fix and includes inherited history.
  Keep D's two field removals and E's Python floor visible, along with the
  merge and five empty-body commits. Omit session trailers from rendered prose.
- [x] H retired `release-notes`, which hand-maintained `CHANGELOG.md` plus the
  Spanish mirror and published through `gh release create`. The new `release`
  skill uses the direct local git-cliff procedure, with no publishing step.
- [ ] Bump to `8.1.0` in the workspace `Cargo.toml`. `pyproject.toml` has no
  literal version: it declares `dynamic = ["version"]` and maturin resolves it
  through `manifest-path` to `[workspace.package]`.
- [ ] Regenerate `Cargo.lock` again. It carries a `version = "8.0.1"` record per
  workspace crate, and `AGENTS.md` requires regenerating after a
  workspace-version change.
- [ ] Tag `8.1.0`.
- [ ] Bump the parent's submodule gitlink for `repos/complexipy`. This belongs
  here rather than in deferred work: the parent pins `c613bdb`, whose
  `Cargo.toml` says `8.0.0+rcq.2` - the scheme decision 5 abandons - and which
  predates the branch point by seven commits. Until the bump lands, a parent
  `git submodule update` checks out that tree.
- [ ] Delete `rcq` and `followup-batch-1` after the work lands on `main`.
- [ ] Sweep `.agents/skills/` when finalizing: remove `release`'s temporary
  tracker clause and `vendor-build`'s pre-8.1.0 embargo; update `add-refactor-rule`
  to the catalog's permanent path and retire the temporary `realignment` scope
  example in `git-commit`. Check other live links to the moved records too.
- [ ] Delete `docs/realignment/`, after rehoming what outlives it: any unfinished
  `follow-up-tooling.md` entries, and `design-issues-and-bugs.md` in full, which
  is not realignment work. Move the records to the reserved `docs/maintenance/`
  paths and update links before deleting this directory.

The former tooling and document-home choices are settled above. Finalization
waits for H, not for a repository-managed git-cliff distribution mechanism.

Do not build a vendored wheel mid-realignment. Without the `+rcq.N` segment it
would report `8.0.1`, which is indistinguishable from upstream's release while
behaving differently. The next wheel should be `8.1.0`.

Verify: `cargo test --workspace --locked` after the lock regeneration, then
`uv run maturin develop && uv run complexipy --version` - without the rebuild,
`--version` reports the previously installed extension and will happily print
`8.0.1` after a correct bump.

### H. Re-cut the skills

Implemented as seven short `SKILL.md` procedures, with no scripts, generated
assets, installer, CI or release framework. `.claude/skills` retains its tracked
symlink to `.agents/skills`.

- [x] Retire `create-issue`, `create-pr` and `release-notes`. No issue/PR/publishing
  workflow is needed for one developer on one local machine.
- [x] Make `git-commit` fork-specific: local scopes, explicit staging, explanatory
  bodies without relying on PRs, manifest/lockfile pairing, breaking descriptions
  and git-cliff's reserved bookkeeping subjects. It is guidance, not a validator.
- [x] Write `verify` first, then the remaining five procedures below.
- [x] Update `AGENTS.md`'s skill catalog and `CLAUDE.md`'s skill loading guidance.
  Correct the abbreviated rule signature and add the three registry test gates
  to the canonical instructions rather than preserving their omission.

| Skill | Implemented procedure |
| -- | -- |
| `verify` | Standing gate, CLI feature-isolation compile check and built `complexipy complexipy --failed` smoke. Rebuild before pytest. Distinguishes the root editable-install ty check from the installed-wheel harness's selected guarantees. Documentation/skills-only changes use manual content/structure/ASCII and diff checks, not the build gate. |
| `vendor-build` | Direct `maturin build --locked --release` into `../../wheelhouse/`, with an external `CARGO_TARGET_DIR`. Check the exact emitted wheel using the existing harness's `--wheel` option and inspect its source-stub-match receipt field (not an enforced assertion). Reuse the parent's provenance table when in scope, not a new receipt system. No parent install or gitlink change is implied. |
| `release` | Workspace version, deliberate `cargo update --workspace`, direct git-cliff candidate review, rebuild and agreement of workspace/lock/installed/CLI versions. Commit and bare local tag only when authorized. No publish or sdist step. |
| `add-refactor-rule` | Metadata, registration, effectiveness, docs and behavioral fixtures; update `fixture_for`, the checked-rule count and `effectiveness_matches_documented_tiers`. Do not copy the catalogued stale "9th rule" comment. Its removal remains a code change outside H. |
| `ffi-change` | Separate type, field, function and public-export paths. New Python-visible types need native registration and stubs; package and stable Rust exports change only when applicable. Fields include affected literals and conversions, including `py_diff` conversions. Add/update contract cases and runtime checks for the changed promise. |
| `sync-upstream` | Fetch a requested ref by upstream URL with `--no-tags`, record its SHA and compare deliberately. A persistent remote is optional and also uses `--no-tags`. Evaluation does not authorize a merge or the return of upstream deployment tooling. |

The parent wheel guide still describes `rcq`, `+rcq.N` and upstream as the fetch
remote, and its reduced-record writer still reads `plan.references`/`plan.doc_url`.
H verified those facts and warns against adopting those stale instructions.
The first wheel waits for G's 8.1.0 release; adoption also requires the consumer
compatibility fix. H does not build a wheel or edit the parent.

Verified: all seven skills pass the skill-creator frontmatter/name/scaffold
validator. Each contains only `SKILL.md`; the shared symlink is unchanged.
Commands and source references were inspected, retired-skill references swept,
and surviving changed files checked as Markdown-only and ASCII-only.
`git diff --check` is clean. No Markdown formatter was run. The standing build
and test gate did not apply to this documentation-only change and was not run.
Fresh Oracle review completed before commit. Applied the relevant corrections:
clean-tree/existing-tag release preflight, a same-change update rule for the
verification command list, explicit catalog and contract-case wiring, and a G
cleanup sweep for temporary skill guidance. The changelog candidate example now
uses a scratch directory compatible with agent instructions. Wheel validation
failure must be reported before any adoption; the requested direct wheelhouse
build remains a manual local procedure, not a new artifact-promotion system.

## Deferred

Possible rebuild ideas live in [`follow-up-tooling.md`](follow-up-tooling.md).
They are not an implementation checklist. Under the current single-machine scope,
only a demonstrated local need or explicit request justifies adopting one.

- **Test automation.** H supplies the manual `verify` procedure. A hook is an
  optional future response to a demonstrated local need, not the next required
  step.
- **CI and pull requests.** Not planned under the single-developer, single-machine
  mandate. Reconsider only if the workflow changes and creates an actual need.
- **Benchmarks.** Own tooling, baselined against a pinned `wheelhouse/` wheel.
- **Outer-repository catch-up.** `wheelhouse/README.md` and
  `docs/tools/complexipy.md` sections 9.6 and 9.7 are updated after the first
  meaningful version bump, not during this work. Note this is not documentation
  alone: `scripts/complexipy_analysis/reduced_record.py` reads `plan.references`
  and `plan.doc_url` while building its reduced record - still true at `39e1bd5`,
  checked against the parent checkout - so it raises `AttributeError` against an
  8.1.0 wheel until it is fixed. The gitlink bump is *not* deferred; see G.
- **git-cliff upkeep.** Check the local version, keep `cliff.toml` small and
  manually review generated output. No further tooling layer is planned.

## Working hazard

This checkout is a git submodule of `recsys-code-quality`, pinned by SHA with no
`branch =` key in the parent's `.gitmodules`. Any `git submodule update --init` in
the parent - including as part of an ordinary pull - checks this submodule out at
the recorded SHA, `c613bdb`, seven commits before the branch point. Commits on
`fork-realignment` and `main` survive; an in-progress working tree does not
necessarily. The parent already reports
` M repos/complexipy`, so its dirty state cannot signal anything about this work.

## Invariants that survive the realignment

- `LICENSE` keeps the upstream copyright notice.
- The asserted complexity totals in `tests/main.py` are the algorithm contract,
  not numbers to adjust for a green run.
- Scoring produces regions; regions produce refactor plans. Rules never re-parse
  source **to find structure** - they consume regions. The qualifier is
  load-bearing: `complexity.rs` calls `parse_expression` in
  `equality_dispatch_subject`, `find_nested_try`, and `collect_free_names`, and
  `registry.rs` `measure_reduction` re-parses spliced source with `parse_module`
  to measure reductions.
- `uv run maturin develop` before pytest after any `crates/**/*.rs` change.
- The public Python API in `complexipy/__init__.py` and its `__all__`.
- The typing lockstep in the `ffi-change` row above, which differs for types and
  fields.
