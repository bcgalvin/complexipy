# Follow-up tooling

Inventory of functionality the realignment removes, recorded so replacements can
be scoped deliberately rather than rediscovered. Removal is the default; this file
is the record that makes aggressive removal safe.

This is not a rebuild checklist. Current scope is one developer on one machine,
local use only, with no portability or sharing requirement. Keep direct commands
and manual procedures unless a concrete local need or explicit request justifies
more machinery. External-project practices are examples, not requirements.

Every entry carries enough mechanics to rebuild the capability without reading the
deleted code. Deleted content is also recoverable from Git history; commit
`d690c9f` is the last revision before the realignment.

Disposition values:

- **Replace** - a candidate replacement if a local need is demonstrated; not an
  instruction to rebuild it now.
- **Decide** - worth having, but the shape is an open question.
- **Drop** - recorded for completeness; no replacement intended.

## Test and lint automation

**Disposition: Replace.** Keep the manual gate as the default.
Nothing runs it automatically after the realignment; that alone is not a reason
to build an automation framework.

### `.github/workflows/CI.yml`

Three jobs, triggered on `pull_request` only.

`lint` - installed dependencies without the project
(`uv sync --group dev --no-install-project --frozen`), ran `ruff check .`,
`ruff format --check .`, and `ty check .` with `uv run --no-sync` on every command,
then asserted through `importlib.metadata` that `complexipy` was **not** installed.
The point of that assertion: the lint environment must never build the extension,
or the type check silently starts reading the native module instead of the stub.
Preserve it in any replacement.

`quick-tests` - matrix of ubuntu 3.8/3.13, windows 3.8/3.13, macos 3.13. Ran
`uv sync --all-extras --frozen`, `maturin develop`, `pytest`, then two CLI
behaviors that **no pytest case covers**:

```
complexipy complexipy --failed
complexipy tests/src --exclude "exclude_dir/**" --ignore-complexity
complexipy tests/src --exclude "**/test_exclude*.py" --ignore-complexity
```

Self-dogfooding is only exercised here and is worth carrying. The two exclusion
invocations are not: both pass `--ignore-complexity`, and `ExitReport::success()`
resolves to `all_pass || ignore_complexity`, so their exit code was 0 whether or
not the glob matched anything. Reproducing them faithfully would reproduce a check
that cannot fail on the thing it appears to test. Exclusion in the analysis path
needs real coverage instead - nothing in either language currently walks a
directory through `run_analysis_shared`.

`rust-tests` - `cargo test --workspace --locked`,
`cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo fmt --all --check`, and four cross-target compile checks in separate Cargo
invocations so workspace feature unification cannot hide a missing feature gate:

```
cargo check -p complexipy-cli --locked
cargo check -p complexipy-core --no-default-features --locked
cargo check -p complexipy-core --no-default-features --features python --locked
cargo check -p complexipy-wasm --target wasm32-unknown-unknown --locked   # dies with the wasm crate
```

Only the CLI check survives the realignment. The wasm check dies with its target,
and the two `--no-default-features` variants die with the `runner` feature, which
workstream A collapses because `complexipy-wasm` was the only consumer that ever
disabled default features. A replacement CI job should carry the CLI check and not
reinstate the others: it still matters because the `serde(skip)` attributes on
`FunctionComplexity` and `FileComplexity` are gated on the `python` feature, so a
standalone CLI build serializes a different snapshot shape than the shipped one.
Cargo caching was keyed on `hashFiles('Cargo.lock')`.

### `.pi/` hooks

An unused harness for a different agent runner, removed without replacement in
kind. The gate design is worth recording because it is a good one:
`postToolUse` matchers fired on every edit, non-blocking, printing failures only.

- `*.rs` edits: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, `uv run maturin develop`. Note the rebuild came last,
  which is the correct order for leaving the extension current.
- `*.py` edits: `uv run pytest`, `uv run ruff check .`, `uv run ruff format --check .`,
  `uv run ty check .`, and per-file `uv run complexipy <path>` dogfooding.
- 600s timeout per hook.

If edit-triggered checks are wanted again, Claude Code hooks in
`.claude/settings.json` are the equivalent mechanism.

### `.pre-commit-config.yaml`

Removed in F, along with `.mdformat.toml`, the nine-setting `[tool.yamlfix]`
block in `pyproject.toml`, and the `pre-commit` dev dependency. The checkout had
no installed Git hook and no `core.hooksPath` override, so no uninstall was needed.

- **complexipy** - pinned `rohaquinlop/complexipy-pre-commit` at `v6.1.0`, so the
  configured check used **upstream's** binary, not this fork. Both checked-in
  analyzer configs declared `max-complexity-allowed = 15`; which one that
  upstream version would select was not verified. F removed the shadowed
  `[tool.complexipy]` block and kept root `complexipy.toml` unchanged. A
  replacement must be a `local` hook driving the locally built extension, or
  it tests the wrong program. **Replace.**
- **mdformat** - `--compact-tables`, with `mdformat-mkdocs>=0.2.1` and an exclusion
  for `SKILL.md`. That exclusion is not optional: mdformat has no frontmatter
  support and rewrites a skill's opening `---` as a thematic break and its closing
  `---` as a setext heading, destroying the YAML that makes the skill loadable.
  Any markdown formatter added later needs the same exclusion. **Decide** on a
  replacement; F's interim policy is no repository Markdown formatter. Maintain
  the surrounding style manually, check ASCII punctuation and `git diff --check`,
  and do not mistake those checks for Markdown validation. This policy and the
  frontmatter warning also live in `AGENTS.md` under "Code Style" and "Agent
  Configuration Layout", so they survive this inventory's eventual removal.
- **yamlfix** - formatted YAML. F removed the last tracked `.yaml`/`.yml` file;
  skill YAML frontmatter still exists but was not a target of this hook. **Drop.**

### Never existed, worth adding

Commit messages have never been machine-validated here. `pr-title.yml` checked
pull-request titles, not commits. With git-cliff parsing the log and no CI, a
`commit-msg` Conventional Commits check could add automatic validation if a
local need arises. Manual review remains sufficient for now. **Replace.**

## Release and distribution

**Disposition: Replace, at a fraction of the size.**

`.github/workflows/release.yml` (366 lines) built wheels across a full support
matrix and published them:

- linux `x86_64`/`x86`/`aarch64`/`armv7`/`s390x`/`ppc64le`, musllinux
  `x86_64`/`aarch64`, windows `x64`/`x86`, macos `x86_64`/`aarch64`, each against
  Python 3.8 through 3.14, via `PyO3/maturin-action@v1` with sccache
- an sdist, with `[tool.maturin] include = ["LICENSE"]` keeping the license in it
- the same unit-test matrix re-run before publishing
- `pypa/gh-action-pypi-publish` with trusted publishing (`id-token: write`,
  `skip-existing: true`)
- `notify-downstream`: `repository_dispatch` to `rohaquinlop/complexipy-pre-commit`
  and `rohaquinlop/complexipy-action` with the released version
- `deploy-docs`: `mkdocs build`, `mkdocs build -f mkdocs.es.yml`,
  `ghp-import -n -p -f site`

What this fork actually needs is one local wheel for one interpreter, which the
`vendor-build` skill covers:

```bash
CARGO_TARGET_DIR=<scratch>/cargo-target uvx --from 'maturin>=1.9.4,<2' maturin build \
  --release --locked --interpreter <project>/.venv/bin/python --out <project>/wheelhouse
```

Everything else - the matrix, the sdist, PyPI, the downstream dispatch - is gone
and is not coming back while distribution stays source-consumed.

`pr-title.yml` ran `amannn/action-semantic-pull-request` pinned by SHA. It returns
only if pull requests do. **Decide**, with the CI rebuild.

## Documentation publishing

**Disposition: Drop the publishing, replace the content.**

Removed: `mkdocs.yml`, `mkdocs.es.yml`, `docs/CNAME` (`complexipy.com`), the
`mkdocs-material` dev dependency, the full Spanish mirror (11 files, ~3411 lines,
including a 36K changelog translation), and `docs/img/`.

Capabilities that disappear with the site, in case a page later reads oddly
without them:

- **Snippet includes** - `--8<-- "CHANGELOG.md"` and `--8<-- "benchmarks/results.md"`
  let a page embed a file. `docs/changelog.md` was nothing but the first one.
- **Admonitions** (`!!! info`, `!!! note`) and **content tabs**
  (`=== "pyproject.toml"`) render as literal text in plain markdown. Retained
  prose needs de-MkDocs conversion, not relocation.
- **Mermaid** via `pymdownx.superfences` custom fences, **arithmatex** math,
  **footnotes**, **attr_list**, **emoji**, permalinked **toc**, and client-side
  **search** with a custom separator.
- The **`edit_uri`** "edit this page" affordance and the EN/ES language switcher.

Replacement is plain markdown under `docs/` covering the scoring contract, rule
catalog, public Python API, and diff/snapshot semantics. No site, no second
language.

## Benchmarks

**Disposition: Replace with our own tooling.** The existing harness is upstream's
and measures an upstream question.

`benchmarks/benchmark-cli.sh`, `benchmarks/generate_scaling_fixture.py`, and the
generated `benchmarks/results.md` are removed, along with the
`benchmarks/corpus/` entry in `.gitignore`.

What it did, worth keeping in a replacement:

- **Parity gate before timing.** Both CLIs exported JSON for every corpus repo
  with `--max-complexity-allowed 1000`; the script required byte-identical output
  (`cmp -s`) and matching exit codes before measuring. This is the part that makes
  a performance comparison meaningful, and it is the idea most worth preserving.
- **Corpus.** Real repositories shallow-cloned at pinned commits into
  `benchmarks/corpus/`, plus a single-file probe isolating startup cost from tree
  analysis.
- **Metrics.** Wall time via hyperfine (warmup 3, 5 runs; probe warmup 5, 20 runs)
  with stdout discarded; peak RSS via `/usr/bin/time -l`, 3 runs, maximum reported.
  Measured across four modes: default, `--quiet`, `--failed`, and full render to a
  file.
- **Scaling guard.** A generated fixture of 285 base functions timed at 1x/2x/4x
  (5 runs each) in `~/.cache/complexipy-benchmarks/scaling`, with the ratios
  recorded to catch superlinear regressions.
- **Environment block.** `results.md` recorded machine, OS, CPU, RAM, both CLI
  versions and build commits, uv and hyperfine versions, run counts, and date.

What must change:

- The baseline was `complexipy==7.0.1` installed from PyPI - upstream's artifact,
  answering "did porting the CLI to Rust help?". This fork's question is
  regression against its own prior build, so the baseline should come from a
  pinned wheel in `wheelhouse/`.
- Requires `git`, `hyperfine`, `uv`, and network access to clone three
  repositories, and is macOS-only (`/usr/bin/time -l`).
- The committed `results.md` is stale regardless: generated from upstream commit
  `1699696` with `/Users/rhafid/...` paths and tables labelled 8.0.0 against a
  declared 8.0.1.

## Rule documentation links and `references`

**Disposition: Drop, revisit only if a consumer asks.**

`RefactorPlan.doc_url` is removed from the model, the stub, and every construction
site, and the SARIF `informationUri`/`helpUri` constants go with it.
`RefactorPlan.references` went in the same commit (`39e1bd5`): `rules/types.rs`
`new_plan()` was its only writer and set `vec![]`, no rule overrode it, and the
consumer read it alongside `doc_url`. Three consumer-visible surfaces lose
content:

- `--suggest-refactors` printed the URL under a `References:` heading, underlined
  and blue. With both inputs gone the block was unreachable, so
  `output_plan_references` was deleted from
  `crates/complexipy-cli/src/output/refactor.rs` rather than left to guard two
  permanently empty values.
- SARIF: `informationUri` on the tool component, `helpUri` on the complexity rule
  descriptor, and `helpUri` per refactor-plan rule. All three are optional in SARIF
  2.1.0, so omitting them is schema-valid.
- `--output-format json`: `doc_url` and `references` were serialized as part of
  each refactor plan, so this is a schema change for anything parsing that
  output, recorded as the `BREAKING CHANGE:` footer on `39e1bd5`.

`rule_id` still ships everywhere, so a consumer that wants documentation can map
the id to a local page itself. If a link is ever wanted back, reintroducing it
means a field on `RuleMetadata` and `RefactorPlan`, a stub entry, every struct
literal in the Rust tests, and inverting
`tests/contract/cases/phantom_plan_fields.py`, which currently proves the
attribute is absent at both the ty and runtime levels. The
`starts_with("https://")` assertion that
`crates/complexipy-core/src/rules/registry/tests.rs` carried was deleted with the
field; a repository-path value would need no such check.

## Browser and editor surfaces

**Disposition: Drop.**

- `crates/complexipy-wasm/` - 46 lines of `wasm-bindgen` entry point over the same
  `code_complexity_shared()` the Python path uses. Rebuilding an editor or browser
  surface means re-adding a thin crate over that function, not reimplementing
  analysis.
- **Merge-surface divergence.** Removing the crate also let workstream A collapse
  core's `runner` feature, which upstream still has. A future `sync-upstream` will
  therefore conflict in `crates/complexipy-core/Cargo.toml`,
  `crates/complexipy-core/src/lib.rs`, `src/helpers.rs`, and
  `tests/collector_failures.rs`. The resolution is always to keep this fork's
  unconditional form.
- The core `wasm` feature gated exactly one thing: `CodeComplexity.version`.
  Neither consumer read it.
- `build-wasm.sh` - `wasm-pack build --target web --out-name complexipy_wasm`, then
  copied `pkg/*.{js,d.ts,wasm}` into `web/wasm/` and `vscode/complexipy/wasm/`.
- `web/` - CodeMirror demo, served on :8080 by `serve-web-version.sh`.
- `vscode/` - extension published under `rohaquinlop`, consuming `result.functions`
  to draw inline complexity decorations.

## Contributor policy surfaces

**Disposition: Drop.**

`.github/ISSUE_TEMPLATE/` (bug report, feature request, `blank_issues_enabled`),
`PULL_REQUEST_TEMPLATE.md`, `CODE_OF_CONDUCT.md` (Contributor Covenant),
`CONTRIBUTING.md`, and `FUNDING.yml` (GitHub Sponsors for `rohaquinlop`) all serve
an audience this fork does not have. Issues are already disabled on the
repository, so the templates were unreachable and CONTRIBUTING's issue links were
already dead.

The PR template's test/format checklist survives as the manual `verify` skill;
commit subjects are reviewed through `git-commit`, not a PR-title check.

One item is a policy choice rather than dead weight: `SECURITY.md` routed
vulnerability reports to GitHub Security Advisories with a 7-day acknowledgement
commitment. Removing it leaves no reporting route. For a fork with no external
users that is correct; note it as a deliberate choice rather than an oversight.

## Changelog and release notes

**Disposition: Replace with git-cliff.** G preparation uses a local 2.14.1
version check, root `cliff.toml` and direct commands in `docs/changelog.md`.
No installer, wrapper, artifact manifest or bespoke test framework is retained.
The inherited changelog is replaced at G finalization, after H.

The `release-notes` skill maintained `CHANGELOG.md` and its Spanish mirror by
hand, moved `## Unreleased` into a dated section at release time, created the tag,
and published through `gh release create`. H removed it. The new local `release`
skill retains version consistency checks before tagging, not sdist verification
or publishing: there is no sdist consumer.

The hand-maintained changelog had already drifted: `## Unreleased` described two
commits while four fork commits had landed, missing `8a35091` and `0f691e7`.
Generation from the log is the fix.

## Skills

- `create-issue` - **Drop, completed in H.** No local need for an issue workflow.
- `create-pr` - **Drop, completed in H.** No PR or CI workflow is planned.
- `release-notes` - **Replace, completed in H.** The new `release` skill uses
  git-cliff directly and does not publish.

H also retuned `git-commit` and added `verify`, `vendor-build`,
`add-refactor-rule`, `ffi-change` and `sync-upstream`. Each is a single
`SKILL.md`, not a script or an automation layer. `AGENTS.md` indexes them.

## Pre-existing defects

Moved to [`design-issues-and-bugs.md`](design-issues-and-bugs.md), which is the
single catalog for bugs and design issues found during the realignment. This
file records removed capabilities only.

## Replacement priorities

1. **Local procedures, completed in H.** `verify` and `vendor-build` preserve
   the useful checks without scheduling them. `verify` includes the standalone
   CLI compile check and built-CLI smoke; the existing wheel harness covers
   selected stub/runtime promises.
1. **Hooks/CI, not planned.** The earlier rebuild proposals are not commitments.
   Reconsider only for a demonstrated need. git-cliff does not require a
   commit-msg hook; unknown subjects are retained and the changelog is reviewed
   manually. Do not restore no-default-features or vacuous exclusion checks
   merely because they appeared in deleted automation.
1. **Benchmarks** - own tooling, baselined against a pinned `wheelhouse/` wheel,
   keeping the parity gate and the scaling guard.
1. **Rule documentation links** - only if a downstream consumer asks for them.

Pre-existing defects are sequenced in `design-issues-and-bugs.md`.
