# git-cliff research for workstream G

**Adoption decision: keep this local and small.** One developer uses one
machine. The selected implementation is a locally installed git-cliff 2.14.1,
one `cliff.toml` and the manual commands in
[changelog maintenance](../changelog.md). A local version check is sufficient.
No wrapper, installer, artifact manifest, portability layer or bespoke test
framework is being adopted. Sections 1-7 retain research options and external
examples, not an implementation checklist. Section 8 records the actual choice.

Research date: 2026-09-12. Repository evidence is anchored at `5ec3c37`.
Three parallel explore agents covered official configuration semantics, official
installation/invocation tooling, and four public GitHub projects. The synthesis
below distinguishes verified behavior from recommendations; important findings
were checked again against official sources and this repository.

**Decision from the maintainer: use the 2.14 series. Target the exact release
2.14.1, not a floating `2.14` or `latest`.** GitHub identifies 2.14.1, dated
2026-09-01, as the latest release even though the website banner announces 2.14.0.
At research time the local executable reported 2.13.1; it was upgraded to
2.14.1 during implementation (see section 8). [Official release](https://github.com/orhun/git-cliff/releases/tag/v2.14.1),
[website announcement](https://git-cliff.org/blog/2.14.0/).

## Original proposal, narrowed during implementation

The research originally proposed the following toolchain. Only the root config
and direct generation are needed here; the additional machinery was rejected:

- One checked-in root `cliff.toml`, invoked explicitly.
- One exact executable/artifact pin, separate from complexipy's runtime dependencies.
- A read-only preview/check path and a separate explicit update path.
- Synthetic Git histories with expected output and exit status, plus an audit of
  the real fork range. Treat the config and template as executable policy.
- No GitHub metadata requirement, release-plz, publishing workflow, automatic
  commits, or tags created by the generator.
- Full regeneration of the fork-only changelog is the preferred starting point;
  do not adopt incremental prepend until its repeat-run behavior is tested.

These were research-stage proposals, not implementation requirements. Section 8
records the narrower adoption and chosen document homes. The historical tracker
was the execution plan; this research report did not regenerate `CHANGELOG.md`.

## 1. Inputs specific to this fork

Rechecked with Git inspection, not inferred from the tracker's older counts:

| Input at `5ec3c37` | Consequence for G |
| --- | --- |
| `030e207..HEAD` contains 29 commits | Use this as the initial coverage inventory, before deliberate filtering. |
| Full excluded baseline is `030e2079457412221087f520445e9f2a709faad6` | Record it in `docs/changelog.md`. |
| `87ad610..HEAD` contains 37 commits | It loses the first fork commit and adds nine inherited commits. Those nine are ancestors of `030e207`, so the correct range excludes them. |
| One merge, `9926391`, has a conventional `chore:` subject and no body | Conventional parsing does not itself imply merge removal. An empty body is not an invalid commit. |
| 18 of 29 messages carry `Claude-Session:` trailers | Suppress presentation of that metadata without destroying breaking-change information. |
| Five bodies are empty: `fa174cc`, `c613bdb`, `f6d4a14`, `9926391`, `5999826` | Subject-only entries must survive unless an explicit, separately justified rule skips them. |
| D (`39e1bd5`) and E (`b052287`) both have `BREAKING CHANGE:` footers | Test both. D's footer is no longer the sole breaking-change record. |
| 39 inherited tags remain; neither `.cliffignore` nor `.git-blame-ignore-revs` exists | Tag cleanup and future ignore files must not silently change coverage. |
| G specifies complexipy version `8.1.0` | Do not substitute an automatic SemVer bump for this explicit fork-version decision. |

D records removal of `doc_url`/`references`, SARIF URI keys, and console references.
E records the Python 3.14 floor and stricter typing of native results. Both need
readable migration detail, not merely a `[breaking]` badge.

The initial range excludes the baseline and its ancestry; it does **not** mean
first-parent-only history. A research report's suggestion that this range would
retain the nine inherited commits was rejected after comparing the actual Git
sets. Recompute these counts when G starts rather than hard-coding 29 as a
permanent expectation.

## 2. Pinning and distribution

### Distribution options considered, not adopted as repository tooling

Repository-managed artifact verification was not adopted; see section 8. The
research considered an official macOS arm64 archive with a recorded URL and
digest. Official binary releases provide checksum and signature instructions.
A version check does not establish artifact provenance, but it is sufficient
for this single-machine project's stated needs.
[Binary release guidance](https://git-cliff.org/docs/installation/binary-releases/).

The archive name for this target is
`git-cliff-2.14.1-aarch64-apple-darwin.tar.gz`. A public downstream lock already
records this exact release artifact, but G should verify against the upstream
release assets rather than copy another project's checksum as its trust source.
[mise's pinned artifact record](https://github.com/jdx/mise/blob/016fcd16a991c85e099d4f0b571bc44978a9eb94/mise.lock#L388-L426).

A source-built alternative is an exact Cargo install with `--locked` and a
dedicated install root. That also introduces Rust toolchain, target and build
feature inputs; the version alone is not a complete reproducible-build recipe.
[Official Cargo installation](https://git-cliff.org/docs/installation/crates-io/),
[Cargo install semantics](https://doc.rust-lang.org/cargo/commands/cargo-install.html).

### Why not immediately add an uv dependency?

At research time, PyPI's project and JSON metadata report 2.13.1; the versioned
2.14.1 JSON endpoint returns HTTP 404. Therefore a default-index
`git-cliff==2.14.1` pin is not currently available through either an uv dependency
group or `uvx`. Recheck availability during implementation; do not silently fall
back to 2.13.1. [PyPI project](https://pypi.org/project/git-cliff/).

If the selected version becomes available, a dedicated development/release group
plus `uv.lock` is a valid alternative. Such groups are not published as wheel
runtime dependencies, but do participate in project resolution. Isolated
`uv tool run --from 'git-cliff==2.14.1'` avoids project-environment coupling, but
its resolution is not recorded in this repository's `uv.lock`.
[uv dependency groups](https://docs.astral.sh/uv/concepts/projects/dependencies/#dependency-groups),
[uv development dependencies](https://docs.astral.sh/uv/concepts/projects/dependencies/#development-dependencies),
[uv tool versions](https://docs.astral.sh/uv/guides/tools/#requesting-specific-versions).

Do not add git-cliff to `[project].dependencies` or the analysis crates. Do not
introduce mise or pixi solely because the examples below use them.

## 3. Creating a maintainable `cliff.toml`

### Start from the selected version, not a rolling example

The 2.14.1 built-in config filters unconventional commits by default, does not
require conventional syntax, and does not protect breaking commits from parser
skips. It also contains project-opinionated skip rules and decorative group
labels. These are defaults to review, not a ready-made policy for this fork.
[Versioned built-in config](https://github.com/orhun/git-cliff/blob/v2.14.1/config/cliff.toml).

Recommended initial choices:

| Area | Proposed policy |
| --- | --- |
| Conventional parsing | `conventional_commits = true`; keep `split_commits = false`. |
| Historical inclusion | `filter_unconventional = false`, `filter_commits = false`, and a final visible `Other` group during the initial audit. |
| Strict validation | Decide separately whether `require_conventional = true` should apply to future commits. Do not confuse syntax validity with membership in an approved type list. |
| Breaking changes | `protect_breaking_commits = true`; put the breaking group before ordinary groups or skips. |
| Unknown classifications | With a catch-all parser, `fail_on_unmatched_commit` cannot enforce an allowed type vocabulary. A later strict policy needs explicit parsers without that fallback. |
| Message processing | Start with empty preprocessors and link parsers. Do not customize `processing_order` without a concrete need and a fixture. |
| Output | ASCII headings, preserved message spelling, explicit group order, no generic author/contributor or PR sections. |

The filtering, splitting and strictness options have different effects; parser
rules are ordered rather than combined into one classification policy.
[Git configuration reference](https://git-cliff.org/docs/configuration/git/).

Two implementation details matter: the first matching parser wins, and a body
regex sees an absent body as an empty string. Thus `body = ".*"` can capture
subject-only commits. `protect_breaking_commits` protects recognized breaking
commits from parser skip rules, not from an excluded range or a template that
fails to display their migration text.
[2.14.1 commit parsing implementation](https://github.com/orhun/git-cliff/blob/v2.14.1/git-cliff-core/src/commit.rs).

### Preserve breaking detail; omit session metadata by construction

Prefer a template that renders the parsed description, scope, commit identifier,
breaking status and nonempty `breaking_description`. Do not render raw messages,
whole bodies, or all footers by default. The structured context exposes these
separately, so removing every trailer with a regex is unnecessary.
[Template context](https://git-cliff.org/docs/templating/context/).

Recommendation: first prove that the real D/E messages render correctly and that
`Claude-Session:` is absent. If a preprocessor is still needed, target only the
observed trailer form and test multiline breaking footers before and after it.
Never delete everything after the first blank line. Keep the five empty-body
commits in the coverage audit; template conditionals should handle missing text.

### Keep presentation small

2.14 adds `commit_groups`, allowing group order to follow parser groups without
numbered HTML-comment tricks. It also adds optional Markdown formatting and
external body-template files. Use the ordered grouping facility, but keep the
small template inline initially. Leave Markdown formatting off until its exact
output is deliberately accepted; this is not a reason to reintroduce the removed
repository-wide formatting stack. No formatter should touch `SKILL.md`.
[2.14 feature announcement](https://git-cliff.org/blog/2.14.0/).

## 4. Range, tag and environment controls

### Range selection is part of the contract

For bootstrap generation, use the full baseline SHA followed by an explicit end
SHA. Resolve `HEAD` once for a preview/check so a moving branch cannot change the
input halfway through a release operation.

**Do not combine the baseline range with `--unreleased`.** In 2.14.1,
`determine_commit_range` replaces an explicit range with `last_tag..HEAD` when
`--unreleased` finds a last tag. A missing explicit config path can also fall back
to user/default configuration. Run from the repository root with the checked-in `--config cliff.toml`; if that
file is missing, stop rather than accepting a fallback configuration.
[2.14.1 range/config implementation](https://github.com/orhun/git-cliff/blob/v2.14.1/git-cliff/src/lib.rs).

Tag filtering is separate: `skip_tags` discards a matching release's commits;
`ignore_tags` removes a boundary while carrying its commits forward. Neither is a
substitute for selecting the fork baseline. Set tag spelling and ordering
explicitly; a numeric tag regex is not proof that a tag belongs to the fork.
[Tag configuration](https://git-cliff.org/docs/configuration/git/#tag_pattern).

Recommendation: use the tracker's bare `8.1.0` spelling, stable-version matching,
explicit oldest-first commit presentation, and test topological tag ordering on
a merged-branch fixture. Do not enable path filters, commit limits or tag limits
for bootstrap coverage. Decide whether the merge summary itself is useful only
after auditing the full set; skipping a merge summary does not remove its
ancestors.

The 2.14 series changes release assignment to Git graph reachability and adds
automatic `.git-blame-ignore-revs` exclusions. Audit that file and `.cliffignore`
as policy inputs whenever they appear. Breaking protection does not rescue a
commit removed before parsing by the blame-ignore filter.
[2.14.1 release changes](https://github.com/orhun/git-cliff/releases/tag/v2.14.1).

### Explicit flags do not eliminate all ambient inputs

Run from the repository root with the selected executable and a local config.
The configuration supports `GIT_CLIFF_*` environment overrides in addition to
CLI bindings, including output, prepend, range, template and config URL settings.
If local output is surprising, inspect those inputs. Automatic sanitization was
considered for a general-purpose wrapper but was not adopted here.
[Configuration overrides](https://git-cliff.org/docs/configuration/),
[CLI bindings](https://git-cliff.org/docs/usage/args/).

Use `--offline` for remote metadata and `--no-exec` for external processor
commands. They are separate controls: `--offline` does not block a
`--config-url` download, and `--no-exec` is not a network sandbox. Avoid remote
config entirely. Installation may need network access; subsequent generation
should not.
[2.14.1 config loading and processor controls](https://github.com/orhun/git-cliff/blob/v2.14.1/git-cliff/src/lib.rs).

Do not infer that an offline template can still obtain PR labels, remote
usernames or contributor metadata. Use commit identifiers and local Git history
instead. If links are retained, construct them for this fork without requiring
API enrichment. [GitHub integration](https://git-cliff.org/docs/integration/github/).

## 5. Practices from actual GitHub projects

These are observations at the linked commits, not endorsements of entire
workflows. Negative findings are limited to the inspected files.

| Project and evidence | Observed practice | Transfer to this fork |
| --- | --- | --- |
| git-cliff itself: [config](https://github.com/orhun/git-cliff/blob/854d89284412ac39e36ca5f20a4d9cb684423c2b/cliff.toml), [fixture runner](https://github.com/orhun/git-cliff/blob/854d89284412ac39e36ca5f20a4d9cb684423c2b/.github/actions/run-fixtures-test/action.yml), [release script](https://github.com/orhun/git-cliff/blob/854d89284412ac39e36ca5f20a4d9cb684423c2b/release.sh) | Synthetic repositories exercise configs; the runner compares generated Markdown and return codes against expected files. The release script regenerates, commits, then signs a tag. | Reuse the test design locally. Do not copy global Git config changes, `git add -A`, automatic commits or tag creation. |
| rattler-build: [config](https://github.com/prefix-dev/rattler-build/blob/ccfed6b80cfcf410847ddfa9c940129593a7054b/cliff.toml), [tool manifest](https://github.com/prefix-dev/rattler-build/blob/ccfed6b80cfcf410847ddfa9c940129593a7054b/pixi.toml), [release script](https://github.com/prefix-dev/rattler-build/blob/ccfed6b80cfcf410847ddfa9c940129593a7054b/scripts/release.py) | Tool resolution uses pixi; the script checks release preconditions, resolves a preview endpoint to a SHA, and exposes git-cliff stderr/failure. Later steps prepend, mutate versions and publish a release PR. | Borrow preflights, a stable endpoint and visible failures. Do not import its token, branch, PR or publishing machinery. No repeat-prepend idempotence proof was found in these files. |
| mise: [config](https://github.com/jdx/mise/blob/016fcd16a991c85e099d4f0b571bc44978a9eb94/cliff.toml), [lock](https://github.com/jdx/mise/blob/016fcd16a991c85e099d4f0b571bc44978a9eb94/mise.lock#L388-L426), [release workflow](https://github.com/jdx/mise/blob/016fcd16a991c85e099d4f0b571bc44978a9eb94/.github/workflows/release.yml#L520-L549) | The lock records 2.14.1 and platform checksums. Git-cliff is a fallback notes generator; errors are suppressed. Sponsor postprocessing removes the old section before writing one canonical block. | Borrow artifact locking and bounded replacement if mixed prose is needed. Reject the error-swallowing fallback for a correctness gate. |
| cargo-generate: [config](https://github.com/cargo-generate/cargo-generate/blob/f12632a2f9690baf181206db1bbba027e7545f48/cliff.toml), [release-plz config](https://github.com/cargo-generate/cargo-generate/blob/f12632a2f9690baf181206db1bbba027e7545f48/release-plz.toml), [release-PR workflow](https://github.com/cargo-generate/cargo-generate/blob/f12632a2f9690baf181206db1bbba027e7545f48/.github/workflows/release-pr.yml) | Explicit changelog-config wiring and a full-history checkout feed release-plz. The inspected workflow uses a version-tagged wrapper action, not a standalone git-cliff artifact pin. | Borrow explicit wiring and sufficient history, not release-plz. A wrapper version must not be mistaken for an independently pinned generator version. |

The most useful evidence is the upstream fixture runner, not the prevalence of
release automation. These downstream examples do not establish that blindly
repeating `--prepend` is safe or that their configurations have dedicated golden
tests.

## 6. Companion tooling considered, not adopted

### Separate preview, verification and mutation

Recommended wrapper responsibilities, with names/locations still to be chosen:

1. **Preflight:** verify executable version/provenance, repository root, existing
   config, valid baseline/end objects, expected tags and allowed environment.
   Release mutation should reject unrelated dirty work rather than stash it.
2. **Preview:** generate to stdout or a temporary candidate, preserving stderr
   and nonzero status. Keep `changelog.output` unset so a nominal check cannot
   accidentally write the real file.
3. **Check:** compare expected Markdown and parsed context; never modify the
   checkout, tag, commit or publish as a side effect.
4. **Update:** review the candidate and replace the destination only after all
   checks pass. Failure leaves the existing file intact. Repeating the operation
   must be a no-op or an explicit already-applied refusal, never duplicate text.

The CLI exposes stdout, `--output`, `--prepend` and JSON `--context`; there is no
dedicated general `--check` mode in the documented CLI. A successful context dump
is not proof that the Markdown template renders.
[CLI arguments](https://git-cliff.org/docs/usage/args/),
[context output](https://git-cliff.org/docs/usage/print-context/).

Prefer full regeneration from the fork baseline for a generated-only changelog.
G should replace the two manual Unreleased entries with their generated commit
entries, not carry both copies. If manual release prose is retained, give it a
separate durable source or explicitly bounded region; do not hand-edit a region
that the next run replaces. `--prepend` is a mutation, not a freshness check.
[Generation examples](https://git-cliff.org/docs/usage/examples/).

### Release sequencing and the self-reference problem

Generate for the explicitly chosen `8.1.0`, review and commit the version,
lockfile and changelog together, then tag that commit. Git-cliff's `--tag` labels
output; it does not create a Git tag. Automatic `--bump` interprets breaking
commits as a version-policy input, so it must not silently override the fork's
chosen version. [Bump semantics](https://git-cliff.org/docs/usage/bump-version/),
[upstream release ordering](https://github.com/orhun/git-cliff/blob/v2.14.1/release.sh).

Recommendation: exclude only the dedicated changelog/version bookkeeping commit
from the rendered entries, with a fixture proving that a breaking release commit
is not lost. Otherwise a changelog update creates another commit to document and
can never reach a stable check result. Keep the source coverage inventory separate
from this presentation omission.

Test generation before and after the real tag exists. A synthetic `--tag` uses
wall-clock release time in the implementation, so two same-day successful runs
are not a proof of cross-day reproducibility. Prefer date-free release headings
initially, or define an explicit stable date input before adding dates.
[2.14.1 release timestamp handling](https://github.com/orhun/git-cliff/blob/v2.14.1/git-cliff/src/lib.rs).

### Optional future CI, not part of this adoption

If CI returns, run the same local fixture/check entry point with complete relevant
history. The official Action recommends `fetch-depth: 0`. Pin both its action SHA
and its `version` input; they identify different components.
[Official Action](https://github.com/orhun/git-cliff-action).

The inspected Action installer downloads and extracts the selected archive without
checking the published checksum/signature. An immutable Action reference alone
is therefore not equivalent to artifact verification.
[Pinned installer source](https://github.com/orhun/git-cliff-action/blob/3d96a18cc4ec17e9dc69ddcc424ccafaf1f78ce2/install.sh).

Keep a future check job read-only, without publishing credentials or a commit/push
fallback. Do not reintroduce the CI/release stack F and B deliberately removed.

## 7. Possible verification framework, not adopted

The planned tests should run through the pinned executable, not a reimplementation
of its parser. Model small temporary repositories on the
[upstream fixture approach](https://github.com/orhun/git-cliff/blob/854d89284412ac39e36ca5f20a4d9cb684423c2b/.github/workflows/test-fixtures.yml),
using controlled identities, dates and tags without modifying global Git config.

| Case | Required assertion |
| --- | --- |
| Bootstrap range | Include `87ad610`; exclude `030e207` and its ancestry. Account for every selected SHA and every deliberate omission. |
| Real D/E messages | Both breaking descriptions survive; JSON/Python field removal and Python 3.14 requirements remain readable. |
| Trailers and empty bodies | No session URLs in rendered prose; subject-only commits still work; multiline breaking text is not truncated. |
| Classification | Specific rules beat general rules; unknown types are visible or fail according to the chosen policy. |
| Merge history | A branch merged after a release is assigned to the correct later release; merge-summary policy does not erase branch commits. |
| Ignore mechanisms | Exercise parser skips, `.cliffignore`, `.git-blame-ignore-revs`, and breaking protection separately. |
| Tag boundaries | No tags, first fork tag, later tag, irrelevant upstream tags, and multiple tags on one commit. |
| Repeat generation | Same inputs produce the same bytes; update twice does not duplicate headings or lose prior releases. |
| Release lifecycle | Before-tag preview and after-tag regeneration agree under the chosen date and bookkeeping-commit policy. |
| Ambient state | A nested invocation, hostile `GIT_CLIFF_*` override, missing config or shallow history cannot silently select different input. |
| Failure safety | Invalid TOML/template, missing ref or wrong executable version fails visibly and leaves the real changelog untouched. |
| Presentation | ASCII punctuation, stable headings/group order, correct empty-release behavior and no missing-template-field failures. |

On each version/config/template change: run fixtures, inspect a real fork-range
context, compare rendered output, review intentional snapshot updates, and record
the new pin. Schema/editor assistance is useful, but neither TOML validity nor a
rolling schema proves behavioral compatibility with the pinned binary.
[Configuration schema announcement](https://git-cliff.org/blog/2.14.0/).

## 8. Actual local adoption

- **Version:** `git-cliff --version` must report `2.14.1`. The global command at
  `~/.local/bin/git-cliff` was upgraded from the uv-managed 2.13.1 installation.
  No repository-managed installer or archive pin is needed.
- **Configuration:** root `cliff.toml`, with direct commands documented in
  `docs/changelog.md`. Keep merge/routine/unknown subjects, omit session trailers,
  preserve D/E breaking details, and use date-free ASCII headings. Review the
  rendered output manually.
- **Scope correction:** an initial wrapper, artifact manifest and extensive
  synthetic-history suite were overengineering for this project. They were
  removed before commit. The capabilities above are research evidence, not
  reasons to recreate them. `AGENTS.md` now records the single-developer,
  single-machine constraint and preference for the smallest sufficient solution.
- **Future Python tools:** if a real need arises, PEP 723 can declare their Python
  floor and dependencies. Do not add a Python wrapper just to run git-cliff.
- **Record homes:** G finalization moves the catalog, tooling records and this
  research into `docs/maintenance/`. The old tracker is retained in Git history
  at `e1a17fb`, not as a second live maintenance guide.
- **Release finalization:** replacing the inherited changelog, bumping/tagging
  8.1.0, cleaning refs and coordinating the parent are manual local work. H
  retired the old skills and added concise procedures; no release pipeline was
  introduced.

Research was read-only apart from this report. Preparation's final scope is
configuration and documentation, with manual native-tool verification recorded
in the historical tracker. Earlier validation of the discarded machinery is not evidence
that a verification framework remains in the repository. Formatting follows F's
manual Markdown policy, not a formatter hook.
