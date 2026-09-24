---
name: verify
description: Run complexipy's local verification gate after implementation, or check a documentation-only change without rebuilding the analyzer.
---

# Verify local changes

Read `AGENTS.md` for the canonical commands and exclusions. Work from the
repository root. This is a manual procedure for the existing local environment,
not a request to create a runner, hook or CI job.

Cargo takes `PYO3_PYTHON` from `.cargo/config.toml`, which names the project
`.venv`; run `uv sync` first if `.venv` is missing, and do not export an ad hoc
value.

For Python dependency changes, regenerate/review `uv.lock` deliberately and
run `uv lock --check` before the gate, so `uv run` does not silently repair drift.

For source, tests, dependencies or build/config changes, run the standing gate:

```bash
uv run maturin develop
uv run pytest
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
uv run ruff check .
uv run ruff format --check .
uv run ty check .
uv run python tests/contract/check_stub_contract.py --self-test
cargo check -p complexipy-cli --locked
uv run complexipy complexipy --failed
```

Rebuild before pytest whenever Rust changed; the last command must also use the
rebuilt extension. Check each exit status, not the status of a trailing `tail`
or `grep`. A CLI smoke failure needs investigation, not a threshold adjustment.
Run focused regression tests as well when the change needs them.

The root ty check sees the editable native installation; it is not proof that
stubs match runtime. The separate installed-wheel harness tests selected typing
and runtime promises from a neutral directory, not the whole API. The standalone
CLI check verifies compilation without core's `python` feature, not parity of
its serialized output with the extension.

For documentation/skill-only changes, the build and test gate does not apply.
Check instructions against their sources, paths and Markdown structure manually;
keep punctuation ASCII and run `git diff --check`. Never run mdformat on a
`SKILL.md`. For a removed or renamed capability, search for remaining live
references without rewriting clearly historical records.

Report what ran, failures and omissions explicitly. Do not call checks passed
when they were skipped, reuse stale build results, or commit as part of this
skill.
