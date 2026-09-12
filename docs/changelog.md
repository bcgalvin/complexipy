# Changelog maintenance

This is local maintenance for one developer on one macOS arm64 machine.
Use git-cliff directly; no installer, wrapper or release pipeline is needed.

## Prerequisite

```bash
git-cliff --version
```

Use **2.14.1**. That local version check is sufficient. The current command is
`~/.local/bin/git-cliff`; root `cliff.toml` is the checked-in configuration.
There is no repository artifact manifest or checksum gate.

## Preview

Run from the repository root:

```bash
git-cliff --config cliff.toml --offline --no-exec \
  030e2079457412221087f520445e9f2a709faad6..HEAD
```

The range excludes the upstream baseline and includes the first fork commit
`87ad610`. Do not use `87ad610..HEAD` or add `--unreleased`, which can replace
an explicit range with the latest-tag range in git-cliff 2.14.1.

The config keeps merge summaries, routine changes and unknown subjects. It
prints first-line messages and breaking descriptions, not whole bodies or
session trailers. Groups and date-free release headings use ASCII. Dedicated
nonbreaking `chore(release): prepare X.Y.Z` and
`chore(release): refresh changelog` commits are omitted; reserve those subjects
for release bookkeeping, not implementation work. This is a convention, not
an automated commit validator.

## Regenerating for a release

For the first fork release, use `8.1.0`. Commit implementation work first.
Bump the workspace version and
regenerate/review `Cargo.lock`, then write a candidate:

```bash
git-cliff --config cliff.toml --offline --no-exec --tag 8.1.0 \
  030e2079457412221087f520445e9f2a709faad6..HEAD \
  > /tmp/complexipy-changelog.md
```

Check the command succeeded and read the candidate. Confirm the first fork fix,
the merge, and both D/E breaking descriptions are present, with no session
trailers or inherited release sections. If needed, compare against
`git log --oneline 030e207..HEAD`; `--context` provides git-cliff's parsed JSON.

Replace `CHANGELOG.md` with the reviewed candidate in full. Do not also copy its
two old handwritten Unreleased entries; their original commits already render.
Run the standing gate and review the diff before committing the version,
lockfile and changelog. Tag that commit separately. `--tag` labels generated
output; it does not create a Git tag. No publishing is involved.

The first regeneration is tracked in the [realignment plan](realignment/README.md);
the old changelog and version remain unchanged during preparation. If the config
changes later, rerun the preview and inspect the output; a bespoke verification
framework is not warranted for this local workflow.
