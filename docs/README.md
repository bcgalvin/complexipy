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

`AGENTS.md` at the repository root covers commands, architecture, and
conventions. [`realignment/`](realignment/) tracks the in-progress fork
realignment and is deleted when that work completes.
