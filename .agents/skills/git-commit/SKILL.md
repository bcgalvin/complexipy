---
name: git-commit
description: Inspect, stage or commit complexipy changes with fork-specific Conventional Commits. Load before any git commit command; commit only when asked.
---

# Commit a local fork change

Read `AGENTS.md`. Inspect status, recent history, the working diff and any
already-staged changes. Preserve unrelated work. Commit one logical concern at
a time, staging explicit paths or hunks only; never `git add .` or `git add -A`.

## Message

Use `type(scope): description`, with an optional scope for broad changes.
Choose the type from the actual change: `feat`, `fix`, `refactor`, `perf`,
`test`, `docs`, `build`, `chore` or `revert`. Useful local scopes include `cli`,
`python`, `rules`, `diff`, `deps`, `agents`, `skills`, `realignment` and `release`.
Do not invent web, server, PR or CI scope to fit a generic template.

Use an imperative subject, no trailing period, at most 72 characters, and ASCII
punctuation throughout. Add a body when the reason or tradeoff is not clear
from the subject; there is no PR description to carry that explanation.
Wrap prose near 72 columns. Record compatibility breaks with `!` in the subject
and a `BREAKING CHANGE:` footer explaining the migration.

Root `cliff.toml` groups these commits but is not a validator: unknown subjects
survive in `Other`. It prints the first subject line and breaking descriptions,
not ordinary bodies or session trailers. Do not add AI attribution or session
trailers. Reserve `chore(release): prepare X.Y.Z` and
`chore(release): refresh changelog` for nonbreaking release bookkeeping only;
those exact subjects are omitted from the generated changelog.

## Before committing

- Review required manifest/lockfile changes together: workspace/member
  `Cargo.toml` changes with `Cargo.lock`, and dependency changes in
  `pyproject.toml` with `uv.lock`. Regenerate and review as needed, not after
  the commit. Pure configuration edits need not force unrelated lock changes.
- Run the applicable `verify` procedure and report skipped checks honestly.
  Complete any requested Oracle review and address its findings before commit.
- Stage only this change's explicit paths. Read the staged diff and run
  `git diff --cached --check`; confirm no unrelated staged content will be
  included. If pre-existing staging cannot be kept separate, ask rather than
  silently including or rearranging it.
- Commit only with user authorization, then inspect the commit and final
  status. Never push, amend, rebase, reset, discard work or move tags without
  an explicit request. A failed hook or check is not permission to bypass it.
