---
name: release
description: Prepare a local complexipy version, generated changelog and optional Git tag when a fork release is requested; no publishing.
---

# Prepare a local release

Read `AGENTS.md` and `docs/changelog.md`. Use the requested version; ask if it is
unspecified. Before editing release files, inspect `git status` and check whether
the requested tag exists. Stop and ask if it exists or unrelated changes remain;
do not stash, discard or include them. Finish and commit implementation work
first, then begin from a clean tree so verification matches the release commit.

1. Change `[workspace.package].version` in root `Cargo.toml`. The member crates
   inherit it and `pyproject.toml` declares a dynamic version through maturin;
   do not invent a second literal version.
2. Regenerate `Cargo.lock` with `cargo update --workspace`, then review the diff.
   This is a deliberate lockfile-writing command, unlike the gate's `--locked`
   commands. Check all three workspace package records match the new version
   and investigate unrelated dependency changes.
3. Confirm `~/.local/bin/git-cliff --version` is 2.14.1; a newer Homebrew
   `git-cliff` can shadow it on `PATH`. Follow the direct candidate
   generation and review procedure in `docs/changelog.md`, using the requested
   version for `--tag`. Keep the explicit fork baseline. Do not add an installer,
   wrapper, handwritten duplicate entries or a publishing step.
4. Run `verify`, including the rebuild, then check `uv run complexipy --version`
   and `uv run python -c 'from importlib.metadata import version; print(version("complexipy"))'`
   against the requested version. A stale extension can report the old version
   even when the manifests are correct.
5. Complete any requested Oracle review before committing. Load `git-commit`
   and commit the reviewed version, lockfile and changelog only when authorized.
   Reserve `chore(release): prepare X.Y.Z` for that bookkeeping, not implementation.
6. Only when tagging is authorized, create the bare version tag on the verified
   release commit (`git tag <version> <commit>`). Confirm its resolved commit.
   An existing tag is a reason to stop and ask, not to delete or move it.

`git-cliff --tag` labels output; it does not create a Git tag. Do not push,
publish a GitHub release, build an sdist, upload to PyPI, or update the parent
as part of this procedure. Use `vendor-build` when a local wheel is requested.
