# Fork realignment

Working tracker for resetting this hard fork away from its upstream public-project
assumptions. Temporary: deleting `docs/realignment/` is the last item in workstream G.

Removal is the default. [`follow-up-tooling.md`](follow-up-tooling.md) records
every capability being removed with enough mechanics to rebuild it, which is what
makes aggressive pruning safe. Add an entry there rather than keeping something
"just in case."

Branch: `fork-realignment`, cut from `main` at `d690c9f`.

Line counts below come from `git ls-files <path> | xargs wc -l` over tracked files.
Directories containing binaries (`vscode/complexipy/img/`) and files without a
terminal newline make those totals approximate. Rows marked **(external)** rest on
evidence outside this repository and are not verifiable from a checkout alone.

## Position

- Hard fork of `rohaquinlop/complexipy`. No upstream contribution is intended.
- Sole developer. No outside collaborators are expected, ever.
- Consumed from source by `recsys-code-quality` as a locally built wheel
  (CPython 3.14, macOS arm64). There is no PyPI target. **(external)**
- Docs become local markdown in this repository. There is no published site and
  no multi-language support.
- The VS Code extension and the browser demo are not needed.
- CI is not part of the loop today and will be recreated from scratch on a
  separate branch after this realignment lands.

## Baseline facts

Observed 2026-09-12.

| Fact | Value |
| -- | -- |
| Latest upstream commit contained | `030e207` (Robin Quintero, `chore(release): bump version to 8.0.1`), arrived through merge `9926391` |
| First local commit | `87ad610 fix(refactors): refuse loop guards when statements follow the chain` |
| Fork work | 11 commits, range `030e207..d690c9f`, all authored locally, all conventional-commit clean. One is the merge `9926391` |
| Declared version | `8.0.1` in the workspace `Cargo.toml`, which is upstream's released number |
| Upstream latest release | `8.0.1`. `CHANGELOG.md` dates it 2026-09-06; the GitHub release timestamp is 2026-09-07 **(external)** |
| Consumed artifact | `complexipy-8.0.0+rcq.2` wheel recorded in `../../wheelhouse/README.md` **(external)** |
| Inherited tags | 39 upstream tags, `0.2.0` through `8.0.0`, present in this checkout. No `8.0.1` tag, though `030e207` is contained |
| Remote identity | Record `git remote -v` before starting. B prunes a remote and G sets a fetch policy on one; the document assumes they are the same remote and never establishes it |
| Remote branches | `origin` has only `main` **(external)**; local remote-tracking refs are stale |
| Local branches | `rcq` (at `9926391`) and `followup-batch-1` (at `d690c9f`) are both contained in `main` |
| GitHub Actions | 0 workflow runs have ever executed on this fork **(external)** |
| Repo settings | Issues disabled, Wiki enabled and unused, no Pages site, `homepageUrl` still `complexipy.com` **(external)** |
| Tracked YAML after `.github/` and mkdocs removal | `.pre-commit-config.yaml` only, which F then removes |
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

A, B, C, D, E, F, G, then H.

No automation runs the gates today: `.pi/` belongs to a harness this fork does not
use, `.pre-commit-config.yaml` carries no Rust or Python gate, and
`.github/workflows/CI.yml` triggers on `pull_request` only, which decision 8
abolishes. Verification during the realignment is therefore manual, using the
standing gate below, and `verify` is the first skill written in H.

Deletions precede rewrites so the rewrites describe the end state. B precedes G
because `release.yml` triggers on `push: tags: "*"` and G's closing version bump
is what the `release` skill tags.

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

- [ ] `vscode/` (15 files, ~2885 lines)
- [ ] `web/` (7 files, ~1250 lines)
- [ ] `build-wasm.sh`, `serve-web-version.sh`
- [ ] `crates/complexipy-wasm/` (2 files, 46 lines). The workspace uses
  `members = ["crates/*"]`, so no manifest edit is needed.
- [ ] The `wasm` feature in `complexipy-core`. It is declared `wasm = []` and gates
  exactly two sites: `CodeComplexity.version` in `classes.rs` and its
  initializer in `cognitive_complexity.rs`. The two consumers being deleted
  read `result.functions` and never `version`.
- [ ] Workspace deps `wasm-bindgen`, `serde-wasm-bindgen`,
  `console_error_panic_hook` (`Cargo.toml:32-34`)
- [ ] Regenerate `Cargo.lock`
- [ ] `.gitignore`: `pkg/`, `vscode/complexipy/wasm/`, `web/wasm/`,
  `vscode/complexipy/complexipy-*.vsix`, `vscode/complexipy/.vscode/**`,
  `vscode/complexipy/.vscode-test/**`, `node_modules/`

Verify: `cargo check -p complexipy-core --no-default-features --locked`,
`cargo check -p complexipy-core --no-default-features --features python --locked`,
`cargo check -p complexipy-cli --locked`, then the standing gate.

### B. Remove public-project, publishing, and unused tooling

- [ ] `.github/` in full: `workflows/CI.yml`, `workflows/release.yml`,
  `workflows/pr-title.yml`, `ISSUE_TEMPLATE/`, `PULL_REQUEST_TEMPLATE.md`,
  `FUNDING.yml`
- [ ] `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`
- [ ] `.pi/` (6 tracked files), and the `.pi/rust-validate/` entry in `.gitignore`
- [ ] `benchmarks/` (3 tracked files), and the `benchmarks/corpus/` entry in
  `.gitignore`
- [ ] GitHub settings: disable the Wiki, clear `homepageUrl` **(external)**
- [ ] Check whether `origin` carries the 39 inherited tags. If it does, G's local
  deletion buys nothing without also deleting them there. **(external)**
- [ ] `git remote prune origin`

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
carry two behaviors no pytest case covers - `complexipy complexipy --failed` and
the two `--exclude` glob validations - which the CI rebuild must carry or replace
with test coverage.

Verify: nothing to build. Confirm
`git grep -n "\.github\|\.pi/\|benchmarks/"` returns only intended references.

### C. Docs rewrite and teardown

Write, before deleting the source material:

- [ ] Scoring contract (from `understanding-scores.md`)
- [ ] Rule catalog (from `refactoring-rules.md`). Drop its `doc_url` JSON sample
  and the `print(f"  Docs: {plan.doc_url}")` example; decision 11 removes the
  field.
- [ ] Public Python API surface (from `api-reference.md`). Its `RefactorPlan` tree
  lists `doc_url: str`; same removal.
- [ ] Diff and snapshot semantics (from `usage-guide.md`). Its DiffStatus section
  is the only written statement that `DiffStatus` is not an `enum.Enum`, has
  no `.name`/`.value`, and formats as `DiffStatus.REGRESSED`. Commit `d690c9f`,
  the branch point, exists to correct exactly that.

Delete:

- [ ] Everything under `docs/` except `docs/realignment/`: `about.md`,
  `api-reference.md`, `benchmarks.md`, `changelog.md`, `CNAME`,
  `comparison-with-ruff.md`, `es/` (11 files, ~3411 lines), `img/`,
  `index.md`, `migration.md`, `refactoring-rules.md`,
  `understanding-scores.md`, `usage-guide.md`
- [ ] `mkdocs.yml`, `mkdocs.es.yml`
- [ ] `mkdocs-material` from the `dev` dependency group
- [ ] Regenerate `uv.lock`
- [ ] `.gitignore`: `site-es/`, `docs/_build/`

Retained prose carries MkDocs-only syntax (`!!! note` admonitions, `=== "tab"`
content tabs, `--8<--` snippet directives) that renders as literal text outside
MkDocs. De-MkDocs it rather than moving it verbatim.

The audience is this repository and the downstream agents in
`recsys-code-quality`. Installation, integrations, comparisons aimed at
prospective users, and anything addressed to contributors do not return.

Verify: `uv sync --frozen` succeeds against the regenerated lock.

### D. Remove `doc_url` and the shipped URLs

Thirteen existing files plus one new contract case.

Origination:

- [ ] `crates/complexipy-core/src/rules/types.rs:27` (`RuleMetadata.doc_url`) and
  `:55` (`new_plan()` copies it)
- [ ] Seven `doc_url` literals in `crates/complexipy-core/src/rules/complexity.rs`
  (C001, C002, C003, C004, C005, C007, C011)
- [ ] `crates/complexipy-core/src/classes.rs:73` (`RefactorPlan.doc_url`)

Emission:

- [ ] `crates/complexipy-cli/src/output/refactor.rs:126,164-170`. Removing the
  argument leaves `output_plan_references` with only `plan.references`, which
  no rule populates, so the `References:` block stops rendering. Decide whether
  the helper survives at all.
- [ ] `crates/complexipy-cli/src/utils/sarif.rs`: `INFO_URI` (line 12, emitted at
  35 as `informationUri`), `HELP_URI` (13, emitted at 129 as `helpUri`), and
  `plan.doc_url` (167). All three keys are optional in SARIF 2.1.0
  **(external)**, so omitting them is schema-valid. Do not emit empty strings
  in their place.
- [ ] `crates/complexipy-core/src/utils.rs` (`output_json_shared`) serializes
  `refactor_plans` wholesale into a hand-built `json!` map, so the
  `serde(skip)` on `FunctionComplexity.refactor_plans` does not apply and
  `doc_url` really is in the `--output-format json` schema. This is a
  consumer-visible schema change; note it in the changelog.

Typing and tests:

- [ ] `complexipy/_complexipy.pyi:206,228`
- [ ] `crates/complexipy-core/src/rules/registry/tests.rs:33` (struct literal),
  `:243` (`plan.doc_url == meta.doc_url`), `:251`
  (`plan.doc_url.starts_with("https://")`)
- [ ] `crates/complexipy-core/src/utils/export_tests.rs:54` (struct literal)
- [ ] `crates/complexipy-cli/src/utils/gitlab/tests.rs:55` (struct literal only;
  GitLab output does not emit `doc_url`)
- [ ] `crates/complexipy-cli/src/output/refactor/tests.rs:49` (struct literal) and
  the `References:` assertions
- [ ] `crates/complexipy-cli/src/utils/sarif/tests.rs`: `:55` (struct literal),
  `:181` (shipped URL literal), `:250` (`assert_eq!(plan_rule["helpUri"], ...)`,
  which fails once the key goes absent)
- [ ] `tests/test_refactor_plans.py:292-317` (`test_rule_metadata_has_doc_url`
  goes entirely)
- [ ] Add a `tests/contract/cases/` case proving `doc_url` is gone. The harness
  currently covers `DiffEntry`, `DiffStatus`, `code_complexity`, and phantom
  helpers only; nothing exercises `RefactorPlan`. `phantom_import.py` is the
  prove-absence pattern. `EXPECTED_DIAGNOSTICS` in `check_stub_contract.py`
  must be edited alongside it.

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

- [ ] `README.md`: PyPI/Downloads/License badges pointing at upstream; the header
  logo, which is an upstream raw URL
  (`raw.githubusercontent.com/rohaquinlop/complexipy/.../complexipy_icon.svg`)
  whose only local counterpart, `docs/img/`, C deletes; all `complexipy.com`
  links; the Integrations section (GitHub Action, VS Code marketplace, and a
  pre-commit `rev: v5.1.0` that disagrees with `.pre-commit-config.yaml`'s
  `v6.1.0` and is repeated in `docs/index.md`); the Complexipy Teams link; the
  `pip install` instructions; the footer nav row's PyPI and upstream GitHub
  links; the "Built with ... by @rohaquinlop and contributors" line; and the
  top-nav anchors that dangle once those sections go
- [ ] `pyproject.toml`: authors, `[project.urls]`, `description`, the 16-entry
  PyPI-discovery `keywords` list, classifiers (currently stopping at 3.12),
  and `requires-python` raised from `>=3.8` to `>=3.14`
- [ ] Regenerate `uv.lock`. It pins `requires-python = ">=3.8"` at line 3 and
  carries three resolution markers (`<3.9`, `==3.9.*`, `>=3.10`) with
  version-split `pre-commit` and `pytest` entries, plus `mkdocs-material` in
  the dev group from C.
- [ ] `Cargo.toml`: authors, homepage, documentation, repository
- [ ] `AGENTS.md`. Rather than working a list, run
  `rg -n 'wasm|web/|vscode|mkdocs|docs/es|\.github|benchmark|pre-commit|pull request|3\.8' AGENTS.md`
  and resolve every hit. Known dead content: the `### WASM / web demo` command
  block (`./build-wasm.sh`, `./serve-web-version.sh`) at lines 182-183, the
  `### Docs` block (`uv run mkdocs serve`) at 189, and the `### Benchmarks`
  block; the four-crate dual-target model, the Key Files and Architecture
  entries for `crates/complexipy-wasm/src/lib.rs`, and the Project Structure
  entries for `web/`, `vscode/`, `.github/workflows/`; the Tech Stack lines
  ("Python 3.8+", "wasm-pack", "MkDocs Material (EN + ES)"); the Cross-target
  compile checks block, whose
  `cargo check -p complexipy-wasm --target wasm32-unknown-unknown` becomes an
  invalid command; the CI lint-job paragraph; the pre-commit hooks bullet under
  Code Style; the rule-authoring instruction to document in both
  `docs/refactoring-rules.md` and `docs/es/refactoring-rules.md`; the
  PR-title-enforced-by-CI and `gh` issue/PR conventions; and the
  "Contributors on Windows need symlink support" note at line 366, which
  decision 3 abolishes.
- [ ] `CLAUDE.md`: the skill list and anything that assumes the removed workflow

Raising the Python floor is not metadata-only. `AGENTS.md` records that ty infers
its analysis version from `requires-python`, `[tool.ruff]` sets no
`target-version` so Ruff retargets with it, and
`tests/contract/check_stub_contract.py` builds its wheel with
`--interpreter sys.executable`, so the harness needs a 3.14 interpreter afterward.

`LICENSE` is not edited. MIT requires retaining the upstream copyright notice, and
`[tool.maturin] include = ["LICENSE"]` keeps it in the sdist. Record the fork
relationship in `README.md` instead.

`AGENTS.md` requires updating itself in the same commit as a structural change,
which is in tension with deferring all rewrites to E. Either update it per
workstream or accept the deviation deliberately.

Verify: `uv sync --frozen`, `uv run ruff check .`, `uv run ty check .`, and the
contract harness on 3.14.

### F. Remove the pre-commit stack

Clean break. All three hooks go, and test automation is rebuilt afterward rather
than migrated. `follow-up-tooling.md` carries what each did.

- [ ] `.pre-commit-config.yaml`
- [ ] `.mdformat.toml` (a `[plugin.mkdocs]` block, vestigial once the site is gone)
- [ ] The nine-setting `[tool.yamlfix]` block in `pyproject.toml`
- [ ] `pre-commit` from the `dev` dependency group, and regenerate `uv.lock`
- [ ] Install the Git hook removal if a `.git/hooks/pre-commit` shim is present

Three things must come back in the rebuild, and are recorded as such:

- A `local` complexipy hook driving the **locally built** extension. The removed
  hook pinned `rohaquinlop/complexipy-pre-commit` at `v6.1.0`, so the
  self-dogfooding gate ran upstream's binary against fork source.
- A decision on markdown formatting. If any formatter returns, it needs the
  `SKILL.md` exclusion: mdformat has no frontmatter support and rewrites a skill's
  opening `---` as a thematic break and its closing `---` as a setext heading,
  destroying the YAML that makes the skill loadable.
- A `commit-msg` Conventional Commits check. Commit messages have never been
  machine-validated here - `pr-title.yml` checked pull-request titles - and
  git-cliff now depends on the convention with nothing else guarding it.

Also settle the duplicate complexipy config while here: root `complexipy.toml`
(`paths = ["."]`, `exclude = ["tests/**"]`, `failed = false`, `quiet = false`) and
`pyproject.toml`'s `[tool.complexipy]` (`paths = ["crates", "complexipy"]`,
`failed = true`, `quiet = true`) disagree on exactly those four settings. Retire
one.

Verify: `uv sync --frozen`; confirm `git status` is clean without hook
intervention.

### G. Changelog, version, and branch cleanup

- [ ] Delete the 39 inherited upstream tags from this checkout. They remain on
  upstream.
- [ ] Set `tagOpt = --no-tags` (or an equivalent fetch policy) on the upstream
  remote. `git tag -d` is local-only and `git fetch` auto-follows tags
  reachable from fetched history, so the first `sync-upstream` run restores
  them otherwise. `git remote prune origin` does not touch tags.
- [ ] Truncate `CHANGELOG.md` at and below `## [8.0.1]`, then regenerate
  `030e207..HEAD` with git-cliff. **Not** `87ad610..HEAD`, which resolves to
  19 commits including all nine inherited upstream ones and excludes
  `87ad610` itself.
- [ ] Decide what happens to the two hand-written `## Unreleased` entries. They
  describe `fa174cc` (stub path and module-total documentation) and `87ad610`
  (C002 loop-guard refusal). Regenerating duplicates those two - and picks up
  `8a35091` and `0f691e7`, two user-visible `fix:` commits the hand-written
  section never recorded. The drift is an argument for regenerating, not
  against. The file preamble's "links to its GitHub release notes" also
  becomes false with no release target.
- [ ] Record the `--output-format json` schema change from D as a breaking entry.
- [ ] Confirm git-cliff's merge-commit handling. One of the 11 commits is the
  merge `9926391`; git-cliff commonly filters merges, so the regenerated body
  may carry ten entries from a correct range.
- [ ] Retire the `release-notes` skill, which hand-maintains `CHANGELOG.md` plus
  the Spanish mirror and publishes through `gh release create`.
- [ ] Bump to `8.1.0` in the workspace `Cargo.toml`. `pyproject.toml` has no
  literal version: it declares `dynamic = ["version"]` and maturin resolves it
  through `manifest-path` to `[workspace.package]`.
- [ ] Regenerate `Cargo.lock` again. It carries a `version = "8.0.1"` record per
  workspace crate, and `AGENTS.md` requires regenerating after a
  workspace-version change.
- [ ] Tag `8.1.0`.
- [ ] Delete `rcq` and `followup-batch-1` after the work lands on `main`.
- [ ] Delete `docs/realignment/`, after folding any unfinished
  `follow-up-tooling.md` entries into wherever they will actually be tracked.

Do not build a vendored wheel mid-realignment. Without the `+rcq.N` segment it
would report `8.0.1`, which is indistinguishable from upstream's release while
behaving differently. The next wheel should be `8.1.0`.

Verify: `cargo test --workspace --locked` after the lock regeneration, then
`uv run maturin develop && uv run complexipy --version` - without the rebuild,
`--version` reports the previously installed extension and will happily print
`8.0.1` after a correct bump.

### H. Re-cut the skills

`.claude/skills` is a symlink to `.agents/skills` and must stay a symlink.

Retire:

- `create-issue`: issues are disabled and there are no reporters.
- `create-pr`: no PR workflow until CI exists. Rebuild it with the CI branch.
- `release-notes`: superseded by git-cliff and the absence of a release target.

Keep and retune:

- `git-commit`: now the primary guard on the commit convention that git-cliff
  parses. It currently instructs reading `CONTRIBUTING.md` and `.github/`
  templates, which B deletes, and lists `web` among example scopes. Its
  lockfile-pairing section (`pyproject.toml` with `uv.lock`, `Cargo.toml` with
  `Cargo.lock`) is worth keeping and is what C, E, F, and G depend on.

Proposed. `verify` is written first:

| Skill | Encodes |
| -- | -- |
| `verify` | The standing gate above. `maturin develop` before pytest is the one hard ordering constraint and the most-repeated trap in `AGENTS.md`; the contract harness builds its own wheel into a fresh venv and is independent of it. |
| `vendor-build` | Wheel into `../../wheelhouse/` with `CARGO_TARGET_DIR` outside the checkout, stub and runtime parity check, provenance table. Currently prose in another repository. |
| `release` | Bump the workspace `Cargo.toml` (the only literal), regenerate `Cargo.lock`, regenerate the changelog with git-cliff, rebuild, tag. It must not reintroduce a publish step. Carry over the removed `release-notes` skill's version-consistency check. |
| `add-refactor-rule` | The rule lockstep: struct and `impl` in `complexity.rs`, `register_defaults()`, effectiveness tier, docs entry, fixture test - plus the three hardcoded gates in `rules/registry/tests.rs` that `AGENTS.md` omits: a new arm in `fixture_for()` (which panics on an unknown id), the literal `assert_eq!(checked, 7, ...)`, and the expected-tier table in `effectiveness_matches_documented_tiers`. Its comment says "if a 9th rule is added" while seven are registered; correct that while writing the skill. |
| `ffi-change` | For a new type: `classes.rs`, the `#[pymodule]` export list, `_complexipy.pyi`, `complexipy/__init__.py` plus `__all__`, the `lib.rs` stable re-export block, and `crates/complexipy-core/tests/lib_surface.rs`. For a field: `classes.rs` plus the stub, and every Rust struct-literal site. `py_diff` types (`DiffEntry`, `DiffStatus`) live in `complexipy-python/src/lib.rs`; a field change there locksteps with the stub only, but a new type still needs `add_class`. Every variant adds a `tests/contract/cases/` case. |
| `sync-upstream` | Evaluating an upstream release as deliberate maintenance. Upstream remains the fetch remote, with `--no-tags`. |

`add-refactor-rule` and `ffi-change` are worth building precisely because
`AGENTS.md` states those invariants as prose a subagent can skip, and in both
cases the prose is already incomplete.

## Deferred

Scope for rebuilt tooling lives in [`follow-up-tooling.md`](follow-up-tooling.md).

- **Test automation.** `verify` skill first, then a rebuilt pre-commit config.
  Nothing checks this repository automatically until they exist.
- **CI rebuild.** Separate branch after this work lands. Written for this fork
  rather than adapted from upstream's.
- **Pull requests.** Adopted only when that CI gives them a purpose.
- **Benchmarks.** Own tooling, baselined against a pinned `wheelhouse/` wheel.
- **Outer-repository catch-up.** `wheelhouse/README.md` and
  `docs/tools/complexipy.md` sections 9.6 and 9.7 are updated after the first
  meaningful version bump, not during this work.
- **git-cliff tooling choice.** Pinning and invocation are researched when the
  changelog work is scheduled. The direction is settled; the mechanism is not.

## Invariants that survive the realignment

- `LICENSE` keeps the upstream copyright notice.
- The asserted complexity totals in `tests/main.py` are the algorithm contract,
  not numbers to adjust for a green run.
- Scoring produces regions; regions produce refactor plans. Rules never re-parse
  source **to find structure** - they consume regions. The qualifier is
  load-bearing: `complexity.rs` calls `parse_expression` at lines 305, 601, and
  1055, and the registry re-parses spliced source to measure reductions.
- `uv run maturin develop` before pytest after any `crates/**/*.rs` change.
- The public Python API in `complexipy/__init__.py` and its `__all__`.
- The typing lockstep in the `ffi-change` row above, which differs for types and
  fields.
