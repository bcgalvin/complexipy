---
name: sync-upstream
description: Evaluate a specific upstream complexipy revision for deliberate local adoption, without automatically merging it or restoring upstream tooling.
---

# Evaluate upstream deliberately

Read `AGENTS.md`, including its branch layout. This is a hard fork, not an
upstream contribution workflow. Fork work lives on `rcq`; `origin/main` mirrors
upstream `main`. Inspect local status and remotes before starting.

Advance the mirror and fetch it without tags, then record the resolved SHA and
use it for subsequent reads:

```bash
gh repo sync bcgalvin/complexipy -b main
git fetch --no-tags origin
git rev-parse origin/main
```

`gh repo sync` only fast-forwards the mirror; never sync, merge or push onto
`rcq` from GitHub. For a revision other than upstream `main`, fetch it directly
with `git fetch --no-tags https://github.com/rohaquinlop/complexipy.git <ref>`.
Do not re-import inherited tags.

Triage without touching the working tree:

```bash
git log --oneline --no-merges rcq..<sha>
git log --oneline --no-merges rcq..<sha> -- .github docs/es 'mkdocs*.yml' CHANGELOG.md CLAUDE.md AGENTS.md
git merge-tree --write-tree --name-only --messages rcq <sha>
git merge-tree --write-tree --name-only --messages --merge-base=<commit>^ rcq <commit>
```

The second command lists commits touching paths this fork never adopts; the
merge-tree forms preview a whole merge or one cherry-pick in memory.

Compare the relevant upstream commits and source against the fork and their
merge base. Explain what is useful, what conflicts with local behavior or API
contracts, and what should stay excluded. Scoring changes must be evaluated
against the conformance tests, not accepted on upstream authority.

An evaluation ends with findings. Apply selected changes only when requested;
do not auto-merge, switch branches, move tags or bump versions. Do not restore
PyPI publishing, a docs site, web/wasm/VS Code targets, collaborator workflows
or CI just because an upstream change assumes them. Run `verify` after an
adopted implementation and use `git-commit` only when committing is authorized.
Land adopted changes as ordinary commits on `rcq` (cherry-pick with `-x` where
clean, port where fork contracts differ; keep ported upstream code close to
verbatim), then record the evaluated SHA with
`git merge -s ours <sha>`, listing adopted and excluded work in the message.
When a sync is recorded with a merge, move the changelog baseline in
`cliff.toml` and `docs/changelog.md` to the recorded upstream commit in the
commit right after the merge (an `ours` merge carries no file changes), or
git-cliff will walk into upstream history through the merge.
