# Current work

Current implementation sequence and fork/parent coordination plan. This file
owns work order, dependencies and acceptance criteria. The
[maintenance catalog](maintenance/design-issues-and-bugs.md) owns defect details
and design questions; [AGENTS.md](../AGENTS.md) owns engineering conventions and
verification commands. This is a work plan, not a historical evidence record.
Update it as work lands and remove completed tasks rather than building a log.

## Current position

- Fork branch `rcq` includes `13a779b`: public `file_complexity(..., base_path=".")`,
  shared canonical roots for analysis and collectors, and path/population tests.
  No further public-base, collector-root or basename-compatibility work is due.
- The parent adopted release 9.0.0 (tag `9.0.0`, `cdfda9b`) in its commit
  `5e102b3`: its wheelhouse holds the verified wheel, its gitlink points at
  `cdfda9b`, and its provider guidance describes the 9.0.0 contracts. Later
  fork commits do not refresh the parent's wheel, gitlink or environments.
- The population/CLI batch now preserves emitted discovery and collector
  failures, rejects empty paths and bad TOML, aligns quiet gates, protects
  comparison state on incomplete collections and invalidates failed marker JSON.
  Its behavior is covered by real permission-error, state-reuse and collector
  contract tests; see the maintenance catalog's Fixed section.
- Upstream `main` through `fb8af35` is adopted selectively. Release 9.0.0
  carries the applicability-tier pin, real `enum.Enum` classes for the three
  enums, and the `complexipy lsp` language server on this fork's fail-closed
  config loader. After 9.0.0, upstream PR #262 (`134c71c`, per-rule suppression
  and `--select`/`--ignore`) was ported onto `rcq`; it is unreleased, so the
  parent gets it only with a later release. Excluded throughout: CI/release and
  riscv64 builds, the docs site and `docs/es`, the wasm crate, upstream changelog
  and agent-file edits, and upstream's stub rewrite.
- Remaining batches below are not implemented. Parent reassessment is complete;
  parent edits still await confirmation in that session. Coordination messages
  do not answer another session's approval prompt.
- A new runtime probe found that `ignore` suppresses ignore-file I/O errors
  internally. Emitted-error propagation is fixed, but complete filter validation
  remains open. Keep this limitation separate from suppression-marker parity.

## Engineering boundaries

One developer, one macOS arm64 machine, CPython 3.14+, one local parent consumer.
Backward compatibility is not required. Update contracts directly, with tests
and current documentation; do not add old-wheel fallbacks, legacy path modes,
private-API workarounds or automatic snapshot migration.

Keep changes small and grouped by behavior. No CI, portability work, historical
archives, artifact-manifest framework, generalized release automation or parent
runner integration is implied. Existing installed-wheel checks remain useful
because they exercise the actual consumer route.

## Fork sequence

### 1. Output precision - next

- Correct `utils/snapshot.rs` `format_function_location`: `path` already
  includes the filename. Replace the tests that expect `a.py/a.py:f` with the
  correct location. Change presentation only, not snapshot keys or schema.
- Make a failed post-analysis source read observable in
  `output/refactor.rs` `read_source_lines`, rather than silently losing the
  caret and original snippet through `.ok()`. Canonical labels already solve
  the former path-base ambiguity; do not reintroduce path compatibility logic.
- Address the catalog's console sort/header discrepancies with behavioral
  tests. Do not change scoring or snapshot identity as part of output cleanup.

### 2. Suppression and marker parity

Unify the recognition and location rules used by scoring, the all-markers
collector and the removable-marker collector. This is marker discovery inside
successfully read files, distinct from the now-reported file-discovery failures.

Anchor: `crates/complexipy-core/src/utils.rs` `find_noqa_comment`,
`collect_ignored_locations`, `filter_removable_ignores`, and the scorer's
`is_ignored` in `cognitive_complexity.rs`. Since the upstream `134c71c` port,
`find_noqa_comment` returns an `IgnoreDirective`: a rule-list marker keeps its
function and is skipped by both collectors. Build the parity fixes on that
type rather than on the earlier string markers, and keep rule-list markers in
the placement cases.

Acceptance includes ordinary and async definitions, decorator chains, markers
above the first decorator, between the last decorator and the definition, and
multiline annotated signatures. First reproduce reported-but-inert markers as
well as suppressed-but-unreported ones. Decide whether reported lines identify
the marker or definition and make removable-marker association honor that
decision; a range-only join cannot blindly assume an
above-decorator marker lies inside the function range. Pin the threshold and
`no_ignore` behavior without claiming that matching file labels proves complete
marker detection.

### 3. Deliberate scoring corrections

Address the catalog's expression-walker omissions in `utils.rs`
`count_bool_ops` and the relevant scorer arms. Derive each desired score and
line contribution before changing code, then update conformance tests and
`docs/scoring.md` together. Review affected refactor-plan measurements as well.
Do not adjust corpus totals just to make tests pass.

Class-body statements, nested classes and method/lambda recursion require
explicit scoring-contract decisions; they are not automatic extensions of the
boolean-walker fix. Record the chosen behavior and test it. Keep scoring work
separate from discovery and output changes so score differences remain clear.

### 4. Exact, fail-closed comparisons

The public file base exists; do not schedule it again. The remaining work is
in `crates/complexipy-core/src/diff.rs`, its CLI orchestration and Python API.

- Replace suffix/basename guessing in `resolve_git_path` with an exact mapping
  between file identity and repository-relative Git paths. Define which root
  relative records use and reject mismatched/outside-repository inputs rather
  than pairing them heuristically.
- Distinguish genuinely new/deleted files and valid empty comparisons from
  invalid references, Git execution failures and reference parse failures.
  Return explicit failure information and make the ratchet fail closed.
- Use matching script/suppression settings on both sides. Staged comparisons
  already include whole-file deletions; apply caller-selected scope and
  exclusions without a working-tree-based filter dropping index-only entries.
  Non-staged `compute_diff` receives only current records and cannot discover
  absent whole files unless that API is deliberately expanded. Do not promise
  that extra discovery as part of preserving staged deletions.
- Define one explicit repository-root contract for the diff API and CLI,
  separate from the CLI invocation directory used for configuration, cache and
  snapshot writes. Reuse or rename existing API context where sufficient, but
  expose the distinction to CLI callers. Do not make changing CWD into the
  target or automatic target-config discovery the solution.
- Fix the spurious `--diff-only` warning by inspecting the original flags. This
  small diagnostic fix may land earlier without waiting for the diff redesign.

Update `docs/cli.md`, `docs/diff-and-snapshots.md` and affected Python API
contracts in the same change, replacing tests that pin superseded behavior.

Acceptance includes duplicate basenames, nested/external invocation, missing
refs, reference parse errors, valid new/deleted files, excluded staged files,
matching flags, valid no-change comparisons and nonzero outcomes on failures.
Use the `ffi-change` procedure for any binding/stub/error-contract changes.

### Remaining catalog work

Resolve the newly identified ignore-file I/O validation question before claiming
complete filter coverage. The dependency suppresses these errors before the
fork's discovery code sees them. Reproduce unreadable local and inherited/Git
ignore inputs, decide the supported validation boundary and test it; do not
silently replace the existing two-walker selection behavior with a new framework.
This is separate from the completed emitted-error propagation work.

No open issue is silently removed by this sequence. Continue through the
[open bugs](maintenance/design-issues-and-bugs.md#open) and
[design issues](maintenance/design-issues-and-bugs.md#design-issues), resolving
or explicitly deferring each according to actual local need. This includes
color behavior, applicability semantics, feature-dependent serialization,
SARIF metadata, dead code/comments and remaining verification limitations.

Config selection, independent snapshot paths, provenance and other design
choices need a concrete decision, not speculative machinery. Existing deferred
items keep their stated reasons until deliberately revisited. The removed-tooling
inventory is not a commitment to rebuild CI, benchmarks or distribution tooling.

## Parent work

Paths in this section are relative to `recsys-code-quality`, not this fork.
Parent documentation work can proceed independently once its edit approval is
resolved; adopting changed runtime behavior requires a new release wheel.

### Independent guidance corrections

- Fix the stale section reference in
  `.agents/skills/complexipy-explore/SKILL.md`.
- Make required guidance available to the bounded evaluation in
  `scripts/evaluate_skill.py` `snapshot_project` / `complexipy_prompt`.
  Prefer self-contained parent guidance for the evaluated surface and optional
  fork-reference links. If more files are genuinely required, align both the
  snapshot inputs and read allowlist with tests, not a new copying framework.
  Guidance must describe the evaluated wheel, not merely current fork HEAD.

### After the 9.0.0 adoption

The wheel swap, record, gitlink and guidance updates are done. The exact-wheel
contract, source-stub equality, hash agreement, real plan and suggestion
serialization and unchanged input checkouts were checked. Still open:

- Run an external-CWD CLI capture on an aliased (symlinked) target and compare
  it with the canonical prefix while retaining multiplicity checks; public
  explicit-root lookup has not yet been exercised that way from the parent.
- Keep `compare_artifacts.py` `population` lexical. Supply each capture's
  canonical target prefix, including symlink resolution; do not repair retained
  artifact paths by resolving them against today's filesystem.
- Use fresh or deliberately selected snapshot state. Changed path keys must
  not silently reinterpret or migrate an earlier snapshot.
- Installing the wheel into a provider environment remains explicit.

Emitted file-discovery/processing failures now reach the CLI gate, but complete
filter validation still needs the ignore-file I/O decision above. Complete
suppression inventories await marker parity; trustworthy native comparisons
await the diff batch. Current bounded direct use can adopt earlier with those
limitations still documented. No parent runner integration or historical
evidence archive is required.

## Verification and completion

For implementation batches, run the applicable standing gate in `AGENTS.md`,
including rebuilding before pytest after Rust changes and the existing
installed-wheel contract. Extend its cases when the API changes. Use Oracle at
design boundaries when useful and review each implemented batch before its
authorized commit; apply findings and rerun affected checks.

For documentation-only changes, check source references, links, Markdown
structure, ASCII punctuation and `git diff --check`. The build/test gate does
not apply; report that explicitly. Do not run a Markdown formatter on skills.
Keep this plan and the maintenance catalog consistent in the same change, and
communicate the committed fork revision and remaining adoption dependencies to
the parent. Do not treat a commit, coordination message or built wheel as
permission to change another session's inputs or resolve its approval prompt.
