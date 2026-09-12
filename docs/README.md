# complexipy documentation

Local reference for this fork. There is no published site; these are plain
markdown files read from the repository.

- [Scoring contract](scoring.md) - how cognitive complexity is computed, and what
  does and does not increment. Derived from `TestPaperConformance` and
  `TestScorerContract` in `tests/main.py`, which are authoritative.
- [Refactor rules](rules.md) - the seven `--suggest-refactors` rules, how plans
  are ranked and capped, and what `reduction_is_measured` means.
- [CLI](cli.md) - configuration discovery and precedence, output paths, exit
  codes, inline ignores, and where the tool writes.
- [Python API](python-api.md) - the `__all__` surface, the types it exposes, the
  exceptions it raises, and the enum, result-object and typing contracts.
- [Diff and snapshots](diff-and-snapshots.md) - the two ratchets, and the
  `DiffStatus` comparison contract.
- [Changelog maintenance](changelog.md) - local git-cliff version check,
  configuration and manual generation.

`AGENTS.md` at the repository root covers commands, architecture, and
conventions. Ongoing maintenance records:

- [Design issues and bugs](maintenance/design-issues-and-bugs.md) - the full
  catalog, including parent-consumer priorities and open/deferred issues.
- [Removed tooling](maintenance/follow-up-tooling.md) - what was removed and
  what a future replacement would need; not a commitment to rebuild it.
- [git-cliff research](maintenance/changelog-git-cliff.md) - research evidence
  and the deliberately small local solution adopted from it.

The completed realignment tracker and evidence trail are retained in Git
history at `e1a17fb:docs/realignment/`. Workstream letters A-H and numbered
decisions in these records refer to that revision's `README.md` tracker.
