# Changelog maintenance

This is local maintenance for one developer on one macOS arm64 machine.
Use git-cliff directly; no installer, wrapper or release pipeline is needed.

## Prerequisite

```bash
git-cliff --version
```

Use **2.14.1**. That local version check is sufficient. The current command is
`~/.local/bin/git-cliff`; a newer Homebrew `git-cliff` earlier on `PATH` can
shadow it, so call that path directly. Root `cliff.toml` is the checked-in
configuration. There is no repository artifact manifest or checksum gate.

## Preview

Run from the repository root:

```bash
~/.local/bin/git-cliff --config cliff.toml --offline --no-exec \
  fb8af353dc9b63607013ffcce9da9e2fed917d6f..HEAD
```

The range excludes upstream history through the last recorded upstream merge,
`fb8af35`, and includes the first fork commit `87ad610`. When a later upstream
sync is recorded with a merge, move this baseline and the `cliff.toml` header to
that upstream commit in the commit right after the recording merge; otherwise
the range walks into upstream history through the merge. Do not use
`87ad610..HEAD` or add `--unreleased`, which can replace an explicit range with
the latest-tag range in git-cliff 2.14.1.

The config keeps merge summaries, routine changes and unknown subjects. It
prints first-line messages and breaking descriptions, not whole bodies or
session trailers. Groups and date-free release headings use ASCII. Dedicated
nonbreaking `chore(release): prepare X.Y.Z` and
`chore(release): refresh changelog` commits are omitted; reserve those subjects
for release bookkeeping, not implementation work. This is a convention, not
an automated commit validator.

## Regenerating for a release

The first fork release is `8.1.0`; use the requested version for later releases.
The command below illustrates 8.1.0. Commit implementation work first.
Bump the workspace version and
regenerate/review `Cargo.lock`, then write a candidate. Substitute an existing
absolute scratch directory outside the checkout, using the agent's session
scratchpad when available:

```bash
~/.local/bin/git-cliff --config cliff.toml --offline --no-exec --tag 8.1.0 \
  fb8af353dc9b63607013ffcce9da9e2fed917d6f..HEAD \
  > "<absolute-scratch>/complexipy-changelog.md"
```

Check the command succeeded and read the candidate. Confirm the first fork fix,
the merge, and the field-removal/Python-floor breaking descriptions are present,
with no session trailers or inherited release sections. If needed, compare against
`git log --oneline fb8af35..HEAD`; `--context` provides git-cliff's parsed JSON.

Replace `CHANGELOG.md` with the reviewed candidate in full. Do not hand-maintain
additional entries that regeneration would overwrite; commit messages are the
input.
Run the standing gate and review the diff before committing the version,
lockfile and changelog. Tag that commit separately. `--tag` labels generated
output; it does not create a Git tag. No publishing is involved.

The realignment's first regeneration replaces the inherited changelog rather
than duplicating its old handwritten entries. Its completed plan is preserved
in Git history at `e1a17fb:docs/realignment/README.md`. If the config changes,
rerun the preview and inspect the output. At the first multi-release regeneration,
also check the blank line between release sections. A bespoke verification
framework is not warranted for this local workflow.
