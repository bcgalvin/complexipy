---
name: vendor-build
description: Build and verify a local complexipy wheel for the parent recsys-code-quality project when a vendored wheel refresh is requested.
---

# Build the local consumer wheel

Read `AGENTS.md`, then the parent's `../../AGENTS.md` and
`../../wheelhouse/README.md` for consumer safety and the existing provenance table.
Run commands from this repository's root. Use the same machine's CPython 3.14
and installed maturin; do not add a distribution or environment-management layer.

Before building, confirm the source commit and workspace version. Build a
consumer artifact from a clean, committed tree so its recorded SHA identifies
its contents. Do not overwrite a wheel with different source under the same
version. Until the first post-realignment 8.1.0 release, do not build a consumer
wheel: the interim 8.0.1 version collides with upstream. The parent's older
`rcq` branch, upstream-remote and `+rcq.N` instructions are historical, not
build defaults.

Choose an absolute scratch directory outside this checkout and run:

```bash
CARGO_TARGET_DIR="<absolute-scratch>/cargo-target" uv run maturin build \
  --locked --release --interpreter "$PWD/.venv/bin/python" \
  --out ../../wheelhouse
```

Use the exact emitted wheel path, not a glob that might pick an older wheel:

```bash
uv run python tests/contract/check_stub_contract.py \
  --wheel "<exact-wheel-path>" --self-test
```

Inspect the receipt as well as the exit status: `passed` and
`origin.source_stub_matches_wheel` must both be true. The latter is reported,
not enforced by the harness. If validation fails, report the exact wheel as
unverified and do not mark it current or adopt it in the parent. This checks
selected installed-wheel typing and runtime promises; it does not replace the
source gate in `verify`.
Compare the wheel's METADATA version with the workspace version and the rebuilt
`uv run complexipy --version`. Rebuild with `uv run maturin develop` first if the
local extension is stale.

Record the wheel filename, source commit, version, build command/interpreter and
SHA-256 in the parent's existing wheelhouse table when that parent edit is in
scope. Do not create another receipt system. Installing the wheel into the
parent provider environment and changing its gitlink are separate adoption
steps, not implied by building it. Check consumer compatibility before adoption:
in particular, the parent reduced-record writer must no longer read the removed
`plan.doc_url` and `plan.references` fields. Do not change the parent as a side
effect of this skill unless that work was requested.
