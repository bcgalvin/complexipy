"""Console-script entry point for the complexipy CLI and language server.

The entire CLI pipeline (configuration, analysis, snapshot, export
formats, diff and ratchet gates) runs in Rust. This module only
bootstraps the process and hands the arguments to the extension. A first
argument of ``lsp`` starts the language server instead, whose stdout
carries only protocol frames; ``complexipy -- lsp`` analyzes a path named
``lsp``.
"""

import signal
import sys
from pathlib import Path

from complexipy._complexipy import run_cli, run_lsp

LSP_ARGUMENT = "lsp"


def lsp_shadow_warning() -> str | None:
    """Return a hint when ``lsp`` names an existing path on a terminal."""
    if not Path(LSP_ARGUMENT).exists():
        return None

    stdin = sys.stdin
    if stdin is None or not stdin.isatty():
        return None

    return (
        f"complexipy is starting the language server; run "
        f"'complexipy -- {LSP_ARGUMENT}' to analyze ./{LSP_ARGUMENT}"
    )


def main() -> None:
    """Run the Rust CLI, or the language server for ``complexipy lsp``."""
    arguments = sys.argv[1:]

    if arguments[:1] == [LSP_ARGUMENT]:
        warning = lsp_shadow_warning()
        if warning is not None:
            print(warning, file=sys.stderr)

        signal.signal(signal.SIGINT, signal.SIG_DFL)
        sys.stdout.flush()
        sys.stderr.flush()
        sys.exit(run_lsp())

    sys.exit(run_cli(arguments))


if __name__ == "__main__":
    main()
