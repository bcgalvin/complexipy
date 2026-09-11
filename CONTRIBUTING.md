# Contributing to complexipy

Thanks for your interest in contributing! Here's how to get started.

## Reporting Bugs

Open a [bug report](https://github.com/bcgalvin/complexipy/issues/new?template=bug_report.md) using the issue template. Include steps to reproduce, expected vs. actual behavior, and your environment details.

## Suggesting Features

Open a [feature request](https://github.com/bcgalvin/complexipy/issues/new?template=feature_request.md) using the issue template. Describe the problem you're trying to solve and your proposed solution.

## Development Setup

1. Clone the repository:

    ```bash
    git clone https://github.com/bcgalvin/complexipy.git
    cd complexipy
    ```

1. Install dependencies:

    ```bash
    uv sync
    ```

1. Build the Rust extension:

    ```bash
    uv run maturin develop
    ```

## Running Tests

```bash
uv run pytest
cargo test --workspace --locked
```

Rebuild with `uv run maturin develop` after Rust source changes and before
running pytest, so the tests exercise the current extension.

## Linting, Formatting & Type Checking

```bash
uv run ruff check .
uv run ruff format --check .
uv run ty check .
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

Use `uv run ruff check --fix .` for safe lint fixes, including import sorting,
then `uv run ruff format .` for Python formatting. Review the resulting diff.
Ruff checks `E4`, `E7`,
`E9`, `F`, `I`, and `B` in package code and ordinary tests. Semantic fixtures in
`tests/src/**` and `tests/fixtures/**` are excluded from linting and formatting,
including when supplied explicitly, because they intentionally contain unusual
Python patterns.

All four Rust crates inherit workspace warnings for Clippy's `exit`,
`dbg_macro`, `todo`, and `unimplemented` lints. CI treats warnings as errors;
this is a focused policy, not the full restriction group.

## Cross-target Compile Checks

Install the target with `rustup target add wasm32-unknown-unknown`, then run:

```bash
cargo check -p complexipy-cli --locked
cargo check -p complexipy-core --no-default-features --locked
cargo check -p complexipy-core --no-default-features --features python --locked
cargo check -p complexipy-wasm --target wasm32-unknown-unknown --locked
```

PR CI runs these separately to check feature isolation. Compilation checks do
not establish runtime behavior on every target. The Rust CI job's Cargo
commands use `--locked` to preserve the checked-in dependency resolution.
Regenerate and review `Cargo.lock` after dependency or workspace-version changes,
and include required lockfile updates with the change.

## Pull Requests

- PR titles must follow [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): description` (e.g., `fix(diff): resolve path for nested invocation`). This is enforced by CI.
- Keep code clean - no comments. Use descriptive variable and function names instead.
- Run tests and linter before submitting.
- Keep changes focused. One feature or fix per PR.
