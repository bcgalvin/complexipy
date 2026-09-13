# Current work

Current implementation sequence and fork/parent coordination plan. This file
owns work order, dependencies and acceptance criteria. The
[maintenance catalog](maintenance/design-issues-and-bugs.md) owns defect details
and design questions; [AGENTS.md](../AGENTS.md) owns engineering conventions and
verification commands. This is a work plan, not a historical evidence record.
Update it as work lands and remove completed tasks rather than building a log.

## Current position

- Fork `main` includes `13a779b`: public `file_complexity(..., base_path=".")`,
  shared canonical roots for analysis and collectors, and path/population tests.
  No further public-base, collector-root or basename-compatibility work is due.
- The parent's adopted wheel is still 8.1.0 from `7f27ffb`, not current fork
  source. Source commits do not refresh its wheel, gitlink or environments.
- The batches below are not implemented. Parent reassessment is complete;
  parent edits still await confirmation in that session. Coordination messages
  do not answer another session's approval prompt.
- The latest scope additions were checked against source and existing tests,
  not reproduced at runtime. Implementation must establish failing regression
  cases before calling those defects fixed.

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

### 1. Population failures and CLI integrity - next

Make incomplete work observable from discovery through the CLI exit code.

- Return discovered files and traversal failures from
  `crates/complexipy-core/src/helpers/exclude.rs` `get_paths_to_process`.
  Capture errors from both existing walkers, preserving their successful
  discovery/exclusion behavior; do not rewrite them merely to introduce an
  abstraction. Preserve successful rows and propagate failures through analysis
  and both collectors in `runner.rs`; a small internal result is sufficient.
- Stop dropping collector failures in
  `crates/complexipy-cli/src/utils/ignored.rs` `handle_report_ignored` and
  `handle_removable_ignores`. Aggregate every failure source; decide separately
  whether identical diagnostics are merged for presentation. Never deduplicate
  successful input results: input multiplicity remains intentional.
- Choose a consistent failed-entry shape: current collectors mix bare paths and
  `path: error` strings. Preserve absolute resolved failure-path identity,
  canonical when the path exists. Update `AGENTS.md`, `docs/python-api.md`,
  binding/stub docstrings and installed-wheel cases for the chosen contract,
  following `ffi-change` if the public contract changes.
- Reject a final empty configured `paths` list in `utils/config.rs`
  `resolve_config`. Distinguish an absent configuration from a malformed or
  unreadable candidate in `utils/toml.rs`. Do not reject a valid requested
  directory merely because discovery filters select no files.
- Make `--quiet` affect presentation, not required checks or exit status.
  Fix its `--ignore-complexity` discrepancy. Resolve the removable-report
  check set deliberately: it currently runs only outside quiet mode, so simply
  adding its failures to the gate would create another quiet-dependent result.
- Prevent snapshot creation/watermark rewrite and previous-function cache
  advancement when the analysis population is incomplete. `run.rs` `run_at`
  currently invokes these writes regardless of `failed_paths`. Cache loading
  and replacement are coupled in `remember_previous_functions`; choose a
  load-only path or omit cached deltas on partial runs. Useful partial
  console/export results may remain available, but must retain clear failure
  diagnostics and a nonzero outcome.
- A successful, complete requested marker JSON report must overwrite its file
  with `[]` when empty. `handle_report_ignored` currently leaves an earlier
  `complexipy-ignored.json` untouched. Define and test failed/partial-report
  output behavior too; neither stale data nor an unsuccessful empty scan may
  masquerade as a new complete inventory.

Update `docs/cli.md` and the relevant API/report documentation in the same
change. Replace tests that intentionally pin superseded behavior, including
`a_malformed_complexipy_toml_falls_through_to_the_next_candidate`, with assertions
for the chosen failure contract.

Acceptance:

- Reproduce an emitted walker error on this machine and retain good files from
  the same request. Check core analysis, both core collectors, both public
  Python collector bindings and CLI orchestration. This does not require a new
  Python directory-analysis API.
- Cover failures from both CLI collector routes, repeated diagnostics,
  malformed/unreadable config, empty requested paths, and a valid empty
  filtered population.
- Check quiet/non-quiet gate agreement and unchanged snapshot/cache bytes
  after incomplete analysis. Do not freeze those stores merely because a
  complete analysis exceeds the complexity threshold.
- Cover nonempty-to-empty marker JSON reuse and failed collection in a reused
  output directory. Keep existing canonical-path, exclusion and multiplicity
  regressions passing.

### 2. Output precision

- Correct `utils/snapshot.rs` `format_function_location`: `path` already
  includes the filename. Replace the tests that expect `a.py/a.py:f` with the
  correct location. Change presentation only, not snapshot keys or schema.
- Make a failed post-analysis source read observable in
  `output/refactor.rs` `read_source_lines`, rather than silently losing the
  caret and original snippet through `.ok()`. Canonical labels already solve
  the former path-base ambiguity; do not reintroduce path compatibility logic.
- Address the catalog's console sort/header discrepancies with behavioral
  tests. Their output work is not a prerequisite for the next population fix.

### 3. Suppression and marker parity

Unify the recognition and location rules used by scoring, the all-markers
collector and the removable-marker collector. This is marker discovery inside
successfully read files, distinct from batch 1's file-discovery failures.

Anchor: `crates/complexipy-core/src/utils.rs` `find_noqa_comment`,
`collect_ignored_locations`, `filter_removable_ignores`, and the scorer's
`is_ignored` in `cognitive_complexity.rs`.

Acceptance includes ordinary and async definitions, decorator chains, markers
above the first decorator, between the last decorator and the definition, and
multiline annotated signatures. First reproduce reported-but-inert markers as
well as suppressed-but-unreported ones. Decide whether reported lines identify
the marker or definition and make removable-marker association honor that
decision; a range-only join cannot blindly assume an
above-decorator marker lies inside the function range. Pin the threshold and
`no_ignore` behavior without claiming that matching file labels proves complete
marker detection.

### 4. Deliberate scoring corrections

Address the catalog's expression-walker omissions in `utils.rs`
`count_bool_ops` and the relevant scorer arms. Derive each desired score and
line contribution before changing code, then update conformance tests and
`docs/scoring.md` together. Review affected refactor-plan measurements as well.
Do not adjust corpus totals just to make tests pass.

Class-body statements, nested classes and method/lambda recursion require
explicit scoring-contract decisions; they are not automatic extensions of the
boolean-walker fix. Record the chosen behavior and test it. Keep scoring work
separate from discovery and output changes so score differences remain clear.

### 5. Exact, fail-closed comparisons

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

- Correct `wheelhouse/README.md` to follow the fork's `vendor-build` procedure:
  dedicated external build environment, exported `UV_PROJECT_ENVIRONMENT` and
  `CARGO_TARGET_DIR`, setup with `uv sync --locked --no-install-project` when
  needed, then `uv run --no-sync`. Build into scratch and validate before
  adopting; do not install the project editably into the provider checkout.
- Fix the stale section reference in
  `.agents/skills/complexipy-explore/SKILL.md`.
- Make required guidance available to the bounded evaluation in
  `scripts/evaluate_skill.py` `snapshot_project` / `complexipy_prompt`.
  Prefer self-contained parent guidance for the evaluated surface and optional
  fork-reference links. If more files are genuinely required, align both the
  snapshot inputs and read allowlist with tests, not a new copying framework.
  Guidance must describe the evaluated wheel, not merely current fork HEAD.

### New-wheel adoption

This depends on a requested clean, committed fork release and validated wheel,
not on resolving every backlog issue. Use the `release` procedure for a new
workspace version reflecting the breaking path/API changes; do not rebuild
current source under the existing 8.1.0 artifact identity.

- Update current capability/API guidance in `docs/tools/complexipy.md` and the
  explore skill to use public `file_complexity(..., base_path=...)`; remove the
  private-binding workaround when the adopted wheel supports the new contract.
- Do not change `scripts/complexipy_analysis/reduced_record.py` `main` solely
  for this batch: it passes a string path and constructs its own location
  instead of consuming `result.path`.
- Keep `compare_artifacts.py` `population` lexical. Supply each capture's
  canonical target prefix, including symlink resolution; do not repair retained
  artifact paths by resolving them against today's filesystem.
- Use fresh or deliberately selected snapshot state. Changed path keys must
  not silently reinterpret or migrate an earlier snapshot.

Minimum acceptance for adoption:

- Exact-wheel installed contract, source-stub equality, and agreement of
  version, source revision and the recorded wheel hash.
- Real native plan **and suggestion** serialization through `plan_native`, not
  only synthetic fixtures.
- Public explicit-root lookup and path identity; an external-CWD CLI capture
  using an aliased target, compared with its canonical prefix while retaining
  multiplicity checks.
- Input checkouts unchanged by building and provider execution.
- Update the current wheel metadata and parent gitlink together. Installation
  into a provider environment remains explicit, not implied by wheel creation.

Complete file-population claims await batch 1; complete suppression inventories
await batch 3; trustworthy native comparisons await batch 5. Current bounded
direct use can adopt earlier with those limitations still documented. No parent
runner integration or historical evidence archive is required.

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
