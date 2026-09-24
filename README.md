# complexipy

Cognitive complexity analysis for Python, implemented in Rust with a thin Python
API and CLI.

This repository is a hard fork of
[`rohaquinlop/complexipy`](https://github.com/rohaquinlop/complexipy), maintained
for `recsys-code-quality`. It is consumed from source as a locally built wheel,
not published to PyPI. There is no documentation site, editor extension, or
upstream contribution workflow; editors can run the `complexipy lsp` language
server. This is local-only work by one developer on one machine, with no
external distribution, sharing or portability requirement.
Maintenance favors direct local commands and small configuration over tooling
frameworks; [AGENTS.md](AGENTS.md#scope-and-engineering-defaults) records that scope.

## Local development

Use CPython 3.14 or later, uv, and current stable Rust through rustup.
The current consumer uses CPython 3.14 on macOS arm64; other platforms are not
part of the local verification target.

```bash
uv sync --frozen
uv run maturin develop
uv run complexipy complexipy --failed
```

Rebuild the extension after changing Rust before running Python tests. See
[AGENTS.md](AGENTS.md#commands) for the complete verification commands.

## Python API

```python
from complexipy import code_complexity

result = code_complexity("def f(value):\n    if value:\n        return 1\n")
for function in result.functions:
    print(function.name, function.complexity)
```

Analysis results expose read-only attributes and are returned by the API, not
constructed directly. [Python API](docs/python-api.md) documents the full
surface and its limitations.

## Reference

- [Documentation index](docs/README.md)
- [Scoring contract and known limits](docs/scoring.md)
- [Refactor rules](docs/rules.md)
- [CLI and configuration](docs/cli.md)
- [Diff and snapshots](docs/diff-and-snapshots.md)
- [Editor integration](docs/editors.md)

Scoring follows G. Ann Campbell's cognitive complexity model. This project is
independent of SonarSource and is not endorsed by it.

## License

[MIT](LICENSE). The upstream copyright notice is retained.
