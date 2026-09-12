# Follow-up tooling

Inventory of functionality the realignment removes, recorded so replacements can
be scoped deliberately rather than rediscovered. Removal is the default; this file
is the record that makes aggressive removal safe.

Every entry carries enough mechanics to rebuild the capability without reading the
deleted code. Deleted content is also recoverable from Git history; commit
`d690c9f` is the last revision before the realignment.

Disposition values:

- **Replace** - the capability is wanted; rebuild it for this fork.
- **Decide** - worth having, but the shape is an open question.
- **Drop** - recorded for completeness; no replacement intended.

## Test and lint automation

**Disposition: Replace.** This is the first thing to rebuild. Nothing runs these
gates automatically after the realignment.

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

Removed in full, along with `.mdformat.toml`, the nine-setting `[tool.yamlfix]`
block in `pyproject.toml`, and the `pre-commit` dev dependency.

- **complexipy** - pinned `rohaquinlop/complexipy-pre-commit` at `v6.1.0`, so the
  self-dogfooding gate ran **upstream's** binary against fork source. Threshold
  `max-complexity-allowed = 15` came from `[tool.complexipy]`. A replacement must
  be a `local` hook driving the locally built extension, or it tests the wrong
  program. **Replace.**
- **mdformat** - `--compact-tables`, with `mdformat-mkdocs>=0.2.1` and an exclusion
  for `SKILL.md`. That exclusion is not optional: mdformat has no frontmatter
  support and rewrites a skill's opening `---` as a thematic break and its closing
  `---` as a setext heading, destroying the YAML that makes the skill loadable.
  Any markdown formatter added later needs the same exclusion. **Decide.**
- **yamlfix** - formatted YAML. After the realignment there is no YAML left in the
  repository to format. **Drop.**

### Never existed, worth adding

Commit messages have never been machine-validated here. `pr-title.yml` checked
pull-request titles, not commits. With git-cliff parsing the log and no CI, a
`commit-msg` Conventional Commits check is the only guard. **Replace.**

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

## Rule documentation links

**Disposition: Drop, revisit only if a consumer asks.**

`RefactorPlan.doc_url` is removed from the model, the stub, and every construction
site, and the SARIF `informationUri`/`helpUri` constants go with it. Three
consumer-visible surfaces lose content:

- `--suggest-refactors` printed the URL under a `References:` heading, underlined
  and blue (`output_plan_references` in `crates/complexipy-cli/src/output/refactor.rs`).
  The `references` vector stays; no rule currently populates it, so the block
  disappears entirely.
- SARIF: `informationUri` on the tool component, `helpUri` on the complexity rule
  descriptor, and `helpUri` per refactor-plan rule. All three are optional in SARIF
  2.1.0, so omitting them is schema-valid.
- `--output-format json`: `doc_url` was serialized as part of each refactor plan,
  so this is a schema change for anything parsing that output.

`rule_id` still ships everywhere, so a consumer that wants documentation can map
the id to a local page itself. If a link is ever wanted back, reintroducing it
means a field on `RuleMetadata` and `RefactorPlan`, a stub entry, every struct
literal in the Rust tests, and relaxing the `starts_with("https://")` assertion in
`crates/complexipy-core/src/rules/registry/tests.rs` if the value is a repository
path rather than a URL.

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

The PR template's checklist - tests pass, code formatted, conventional title -
survives as the `verify` skill.

One item is a policy choice rather than dead weight: `SECURITY.md` routed
vulnerability reports to GitHub Security Advisories with a 7-day acknowledgement
commitment. Removing it leaves no reporting route. For a fork with no external
users that is correct; note it as a deliberate choice rather than an oversight.

## Changelog and release notes

**Disposition: Replace with git-cliff.**

The `release-notes` skill maintained `CHANGELOG.md` and its Spanish mirror by
hand, moved `## Unreleased` into a dated section at release time, created the tag,
and published through `gh release create`. It also verified sdist contents and
version consistency before tagging - a check worth carrying into the `release`
skill even though there is no sdist to publish.

The hand-maintained changelog had already drifted: `## Unreleased` described two
commits while four fork commits had landed, missing `8a35091` and `0f691e7`.
Generation from the log is the fix.

## Skills

- `create-issue` - **Drop.** Issues are disabled; there are no reporters.
- `create-pr` - **Decide.** Rebuild alongside CI if pull requests return.
- `release-notes` - **Replace.** Superseded by git-cliff and the `release` skill.

## Pre-existing defects

Not caused by the realignment and not removed capabilities - live bugs the explore
sweep surfaced. Recorded here because this file is what outlives
`docs/realignment/`. Each entry is self-contained so the set can be lifted out
wholesale when the directory goes.

Several bear directly on how `recsys-code-quality` consumes this tool; those are
marked **consumer-facing**.

### Fixed before workstream A

**A relative `--output` resolved against the process CWD, and a test wrote into
the checkout on every gate run.** Fixed.

`crates/complexipy-cli/src/utils/paths.rs` took an `invocation_path` and used it
when `--output` was omitted, but absolutized a relative `--output` with
`std::path::absolute` against the **process CWD**, ignoring the parameter. The two
branches disagreed about what a relative path is relative to, and
`fs::create_dir_all` runs during path *resolution*, so merely resolving a relative
output created a directory before any analysis.

In ordinary CLI use the divergence was latent, because `run_cli` defaults
`invocation_path` to `"."`. It surfaced through
`crates/complexipy-cli/src/utils/paths/tests.rs`, which passes a tempdir as the
invocation path and `"rel-out/"` as the output: the result was
`crates/complexipy-cli/rel-out/`, untracked and un-gitignored, recreated by every
`cargo test --workspace` - a command in the standing gate, so every workstream's
verification recreated it inside a checkout the parent treats as a read-only
input.

Resolution: `std::path::absolute(invocation_path.join(output))`, matching the
no-`--output` branch. An absolute `--output` is unaffected, since `Path::join`
replaces on an absolute argument, and CLI behavior is unchanged because
`invocation_path` is `"."` there. Verified by removing the directory and
confirming the test no longer recreates it.

### Fold into workstream D

These are residue of the same removed feature D is already cleaning up, or dead
code in files D already opens.

- **`looks_like_remote`** (`crates/complexipy-cli/src/utils/cache.rs:130-142`)
  normalizes cache keys for `github.com`/`gitlab.com` URLs. Nothing in the tree
  clones or fetches a URL; git-URL analysis was removed in 8.0.0. Same category as
  the stale `_complexipy.pyi` docstrings and the false `AGENTS.md` claims.
- **`effective_sort_for_display`** (`crates/complexipy-cli/src/output.rs:158`) has
  no callers; `handle_display` inlines the identical logic.
- **`SnapshotEvaluation.snapshot_result`**
  (`crates/complexipy-cli/src/utils/snapshot.rs`) is computed and unit-tested, but
  `run.rs` reads only `should_run` and `watermark_success`.
- **`RuleMetadata` derives `Serialize, Deserialize`** with no evident consumer, in
  a file D edits anyway.

### Fold into workstream C

**`--help` contains no descriptive text.** `crates/complexipy-cli/src/args.rs` has
no `about`/`long_about` and no `help =` on any of the 21 options; since `AGENTS.md`
bans code comments, clap has no doc comments to fall back on either. Upstream
offloaded all explanation to the docs site. C is where that site disappears, so C
is where `--help` has to become the discovery surface - otherwise there is a window
with no user-facing documentation at all. Confirm the absence first with
`rg -n 'help\s*=|about|///' crates/complexipy-cli/src/args.rs`.

### Defer past the realignment

Real behavior changes. Each needs its own change, its own verification, and in two
cases a design decision. Keeping them out of a deletion-heavy realignment keeps
"did the realignment break this?" separable from "did my fix break this?".

- **`--color` is completely inert. (consumer-facing)**
  `crates/complexipy-cli/src/output/render.rs:16,24,40` computes `color_enabled`
  and stores it; nothing reads it but two tests at `render/tests.rs:228,237`, which
  give false confidence that the flag works. Every renderer calls owo-colors
  directly, so ANSI is emitted unconditionally. `Color::Auto` hardcodes true with
  no `is_terminal()` check - though the same file calls `is_terminal()` in
  `terminal_width()` - and `NO_COLOR`/`CLICOLOR` are not consulted. The only clean
  text mode is `--plain`, which discards everything but path, name, and complexity.
  Net effect: there is no way to get clean machine-readable refactor output from
  the console surface; a consumer must use `--output-format json`. Do not stack
  this on D, which already changes that JSON schema twice.
- **Config discovery is CWD-only, never target-relative. (consumer-facing)**
  `crates/complexipy-cli/src/utils/toml.rs:7-16` joins the three candidate
  filenames onto `invocation_path`, which `run_cli` defaults to `"."`. No upward
  search, no target-root lookup. A target's own `[tool.complexipy]` thresholds are
  silently ignored when the tool runs from an external working directory, which is
  exactly the parent's mandated invocation pattern. Fixing this is a design
  decision - upward search, target-root lookup, or an explicit flag - not a patch.
- **The tool writes into the tree it analyzes. (consumer-facing)**
  `.complexipy_cache/` is created in the invocation directory and ships its own
  `.gitignore` containing `*` plus a `CACHEDIR.TAG`, so `git status --porcelain`
  shows nothing after a run - which defeats a git-status-based before/after target
  snapshot. `--cache-dir` redirects it, and has no help text. The snapshot path is
  hardcoded to `<invocation>/complexipy-snapshot.json` with no override flag at
  all. Decide whether the fix is flags, different defaults, or both.
- **`-s file_name` sorts by function name** (`crates/complexipy-cli/src/output/rows.rs:65`).
  The sharper framing: `complexipy-core`'s `export_tests.rs` proves the CSV path
  sorts `file_name` by path, so console output and CSV export disagree on what the
  same flag value means. Unverified - `rows.rs` was outside the review slice.
- **Duplicate file headers.** Grouping is by consecutive same-path entries, so a
  path can get two headers. Probably not specific to `--top`, since truncation runs
  after row building. Unverified; reproduce before fixing.
- **`--suggest-refactors` can degrade silently.** `read_source_lines` ends in
  `.ok()`, so a failed read yields `None` and both the caret span and the
  `Original:` snippet vanish with no warning. The mechanism is certain; the trigger
  is not - the join is skipped when the path starts with `/`, so an out-of-tree
  absolute target likely takes the safe branch. Real exposure is Windows-style
  absolute paths and relative targets like `../repos/foo`.
- **CLI output is not ASCII.** Emoji in `run.rs` and `render.rs`, box-drawing
  U+2500, status glyphs, and the documented rule-category marks. This is *not* a
  style-rule violation - `AGENTS.md` scopes its rule to Unicode dashes, and the
  category marks are asserted as `\u{25b2}` in tests. It matters only because the
  parent captures this output as text, and because `render.rs` guards one emoji
  behind `cfg!(windows)` while leaving the status glyphs unguarded.

## Replacement priorities

1. **`verify` skill** - the local gate, replacing what CI and `.pi/` ran. Nothing
   checks this repository automatically until it exists.
1. **`.pre-commit-config.yaml` rebuild** - a `local` complexipy hook on the built
   extension, plus `commit-msg` Conventional Commits validation that git-cliff
   depends on.
1. **`vendor-build` skill** - the one genuinely needed half of `release.yml`.
1. **CI rebuild** - separate branch, written for this fork rather than adapted.
   Carry over the dependency-only lint assertion, the three surviving cross-target
   checks, and the CLI exclusion and `--failed` behaviors no pytest case covers.
1. **Benchmarks** - own tooling, baselined against a pinned `wheelhouse/` wheel,
   keeping the parity gate and the scaling guard.
1. **Rule documentation links** - only if a downstream consumer asks for them.

The pre-existing defects above are sequenced separately: one fixed before A, four
folded into D, one into C, and the rest deferred past the realignment.
