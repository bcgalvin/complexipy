---
name: sync-upstream
description: Evaluate a specific upstream complexipy revision for deliberate local adoption, without automatically merging it or restoring upstream tooling.
---

# Evaluate upstream deliberately

Read `AGENTS.md`. This is a hard fork, not an upstream contribution workflow.
Ask for the upstream revision if the request does not identify one. Inspect
local status and remotes; `origin` is the fork, not upstream.

For an occasional comparison, fetch the requested ref directly without adding
a permanent remote or importing tags:

```bash
git fetch --no-tags https://github.com/rohaquinlop/complexipy.git <ref>
git rev-parse FETCH_HEAD
```

Record the resolved SHA and use it for subsequent reads; `FETCH_HEAD` changes
on later fetches. If a persistent upstream remote is explicitly requested, add
it with `git remote add --no-tags upstream <url>` before the first fetch. Do not
re-import inherited tags or change `origin` to point upstream.

Compare the relevant upstream commits and source against the fork and their
merge base. Explain what is useful, what conflicts with local behavior or API
contracts, and what should stay excluded. Scoring changes must be evaluated
against the conformance tests, not accepted on upstream authority.

An evaluation ends with findings. Apply selected changes only when requested;
do not auto-merge, switch branches, move tags or bump versions. Do not restore
PyPI publishing, a docs site, web/wasm/VS Code targets, collaborator workflows
or CI just because an upstream change assumes them. Run `verify` after an
adopted implementation and use `git-commit` only when committing is authorized.
