# Editor integration

`complexipy lsp` runs a Language Server Protocol server over stdio, so an editor
can show the same cognitive complexity analysis the CLI produces while you
write. The server uses the same Rust engine as `complexipy` and the Python API.
It was adopted from upstream's language server with this fork's configuration
contract; see [Configuration](#configuration).

Inside the editor you get:

- **Inlay hints** - a function's cognitive complexity at the end of its
  declaration, shown as `cognitive: 18`.
- **Hover** - anywhere inside a function: its name, its complexity, whether it
  exceeds the allowed threshold, and the title of the top refactor plan when
  one exists.
- **Diagnostics** - one warning per function above `max-complexity-allowed`,
  `cognitive complexity 18 exceeds the allowed 15`, on a range covering the
  whole function.

Results refresh as you type. The server uses full document sync and analyzes
only open documents, so there is no workspace-wide scan and no need to save.

## Starting the server

The editor starts the server as a subprocess with `complexipy lsp`, so the
`complexipy` command it runs must come from an environment with this fork
installed. This fork is not published; use one of these local routes:

- From this checkout, after `uv run maturin develop`:
  `/path/to/complexipy/.venv/bin/complexipy lsp`. Point the editor at that
  binary rather than at `uv run`, which would sync the environment on launch.
- From an environment that installed a locally built wheel (see the
  `vendor-build` skill): the absolute path of that environment's
  `bin/complexipy`, with the argument `lsp`.

Do not use `uvx complexipy` or `pip install complexipy`: both install the
upstream PyPI release, not this fork.

## Neovim

Neovim 0.11 and later can configure and enable the server with
`vim.lsp.config` and `vim.lsp.enable`:

```lua
vim.lsp.config("complexipy", {
  cmd = { "/path/to/env/bin/complexipy", "lsp" },
  filetypes = { "python" },
  root_markers = { "complexipy.toml", ".complexipy.toml", "pyproject.toml", ".git" },
})

vim.lsp.enable("complexipy")
```

On Neovim 0.10, start the server directly instead:

```lua
vim.lsp.start({ name = "complexipy", cmd = { "/path/to/env/bin/complexipy", "lsp" } })
```

Inlay hints are opt-in in Neovim. Enable them globally with
`vim.lsp.inlay_hint.enable(true)`, or per buffer with
`vim.lsp.inlay_hint.enable(true, { bufnr = 0 })`.

Upstream tested this Neovim setup. It has not been exercised on this fork's
local machine.

## Zed

Zed registers language servers in `settings.json`:

```json
{
  "lsp": {
    "complexipy": {
      "binary": {
        "path": "/path/to/env/bin/complexipy",
        "arguments": ["lsp"]
      }
    }
  },
  "languages": {
    "Python": {
      "language_servers": ["complexipy", "..."]
    }
  }
}
```

The `"..."` entry keeps the other Python language servers enabled. This setup
is unverified upstream and here: Zed launches only servers it already knows,
and the `binary` block overrides a registered server. If `complexipy` is not
offered as a Python server, the block has no effect until an extension
registers it.

## Configuration

The server reads the same files as the CLI, with the same loader
(`crates/complexipy-core/src/config.rs`): the first existing candidate of
`complexipy.toml`, `.complexipy.toml` and `pyproject.toml` (`[tool.complexipy]`)
in the workspace root. Files are not merged and there is no upward search.

```toml
max-complexity-allowed = 15

[lsp]
inlay-hints = "threshold"   # "always" | "threshold" | "never"
per-line-hints = false
diagnostics = true
```

In `pyproject.toml` the same keys live under `[tool.complexipy]` and
`[tool.complexipy.lsp]`.

| Key | Default | Description |
| -- | -- | -- |
| `max-complexity-allowed` | `15` | Functions strictly above this value are reported. A function at the limit passes. |
| `exclude` | `[]` | Glob patterns matched relative to the workspace root. Excluded files produce no hints and no diagnostics. |
| `no-ignore` | `false` | Analyze functions even when an inline ignore comment suppresses them. |
| `lsp.inlay-hints` | `"threshold"` | When to show the per-function hint: `"threshold"`, `"always"` or `"never"`. |
| `lsp.per-line-hints` | `false` | Also show a `+N` hint on every line with a nonzero complexity increment. |
| `lsp.diagnostics` | `true` | Publish warnings for functions above `max-complexity-allowed`. |

The server ignores the file's other keys (`paths`, `quiet`, `failed`, `sort`,
`color`, `output`, `output-format`, `cache-dir`, `snapshot-create`,
`snapshot-ignore`, `ignore-complexity`, `check-script`, `report-ignored` and
`diff`), and the CLI ignores the `lsp` table.

**A bad configuration disables analysis rather than selecting another one.**
Discovery is the CLI's: the same candidate is selected, and when it cannot be
read or parsed the server shows the error with `window/showMessage`, logs it to
stderr and publishes no hints, hover or warnings, just as the CLI refuses to
run. It never falls through to a later candidate or to defaults.

Validation is per consumer. The server checks the keys it reads
(`max-complexity-allowed`, `exclude`, `no-ignore` and the `lsp` table) and
treats a failure there the same way, for example an unknown `inlay-hints`
value. The CLI checks its own keys and ignores `lsp`, so one file can be valid
for one and invalid for the other: `paths = false` stops the CLI but not the
server.

The server reads the configuration at startup and again on
`workspace/didChangeConfiguration`. It shows an error again only when the
error changes, and logs it on every load. After fixing the file, ask the editor
to reload the server (`:LspRestart` in Neovim). The config tests in
`crates/complexipy-lsp/tests/protocol.rs` and
`test_a_bad_config_disables_analysis_without_falling_through` in
`tests/test_lsp.py` pin this behavior.

### Inlay hints

The hint sits at the end of the line that closes the function declaration: the
`def` line for a single-line signature (not a decorator line above it), or the
closing `)` or `):` line of a signature spanning several lines.

- `inlay-hints = "threshold"` (the default) shows `cognitive: N` only for
  functions **above** `max-complexity-allowed`.
- `inlay-hints = "always"` shows it for every function.
- `inlay-hints = "never"` suppresses every hint, including per-line hints.
- `per-line-hints = true` adds a `+N` hint at the end of every line with a
  nonzero increment.

The server reports the functions the CLI reports: top-level functions and
methods. A nested function is folded into its enclosing function, so hovering
inside it shows the parent.

### Diagnostics

- A function whose complexity **equals** `max-complexity-allowed` passes; only
  strictly greater values are reported.
- A document the parser cannot read produces one warning with the code
  `complexipy-parse-error` on its first line. Hints and hover keep answering
  from the last text that parsed, and the warning clears when the document
  parses again.

### Exclusions and ignores

- `exclude` globs are matched against the document path relative to the
  workspace folder as the client sends it. Paths are not canonicalized, so a
  symlinked workspace root and canonical document paths do not match.
- The CLI matches `exclude` relative to each walked directory and does not
  apply it to files named explicitly. The two agree for directories walked from
  the workspace root, such as `paths = ["."]`. With `paths = ["src"]`, the CLI
  excludes `src/legacy/x.py` for `legacy/**` and the server does not.
- A malformed glob is logged once per configuration load and skipped; the
  other patterns keep applying. A CLI run instead reports the walked directory
  as a failed path for the same list.
- A list too large to compile as one program is logged and applied one pattern
  at a time. The CLI reports the walked directory as failed for that list too.
- `helpers/exclude/tests.rs` pins the server's matcher against the walker's
  for walks rooted at the workspace root and the patterns both accept.
- Inline suppression comments (`# noqa: complexipy` and `# complexipy: ignore`)
  are honored as in the CLI unless `no-ignore = true`.

## Logs

The server writes only protocol frames to stdout; every log line goes to
stderr. Read it where the editor collects server logs (`:LspLog` in Neovim,
`zed: open log` in Zed).

## Limits

- The `--diff` ratchet, snapshots and exit-code gates remain CLI-only.
- Only Python documents are analyzed: language `python` or a path ending in
  `.py`.
- The server declares full document sync; a change carrying a range is
  ignored and logged.
- A workspace with several roots uses the first folder for configuration and
  `exclude` matching.
- Client `initializationOptions` are ignored; settings come from the
  configuration file.
- The `complexipy` console script reserves `lsp` as the first argument and
  ignores the arguments after it, so clients that append `--stdio` work. To
  analyze a path named `lsp`, run `complexipy -- lsp`; on a terminal,
  `complexipy lsp` prints that hint when such a path exists. The standalone
  Rust `complexipy` binary built from `complexipy-cli` has no `lsp` command.
- Ctrl-C stops a server started from a terminal.
