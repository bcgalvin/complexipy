---
name: vendor-build
description: Build and verify a local complexipy wheel for the parent recsys-code-quality project when a vendored wheel refresh is requested.
---

# Build the local consumer wheel

Read `AGENTS.md`, then the parent's `AGENTS.md` and `wheelhouse/README.md`
(`../../` from this repository's root) for consumer safety and the current
artifact. Use this machine's CPython 3.14 and an external environment reserved
for fork build tools, with maturin and ty. Check that maturin satisfies the
shared build-system and dev-group requirement `>=1.9.4,<2.0`.
No distribution or environment-management layer is needed.

Before building, confirm the source commit and workspace version. Build a
consumer artifact from a clean, committed tree so its recorded SHA identifies
its contents. Do not overwrite a wheel with different source under the same
version. Build from the requested fork release on `rcq`; `origin` is the fork,
not upstream.

From this repository's root, select absolute environment and scratch paths
outside the input checkouts. Export both settings for the build and the contract
harness, whose nested ty invocation also uses the project environment. Use these
exports only in the build shell; do not carry them into the development gate.

```bash
export UV_PROJECT_ENVIRONMENT="<absolute-external-build-env>"
export CARGO_TARGET_DIR="<absolute-scratch>/cargo-target"
```

If that build environment needs preparation, use
`uv sync --locked --no-install-project` with those settings first. Sync removes
packages outside this project's lockfile: do not point it at the parent's
provider environment. Do not use ordinary syncing `uv run` or `maturin develop`
for this route: an editable install can write the extension into the source
checkout even when the environment itself is external.

Build into external scratch, not directly into the parent's current wheelhouse:

```bash
uv run --no-sync maturin build \
  --locked --release --interpreter "$UV_PROJECT_ENVIRONMENT/bin/python" \
  --out "<absolute-scratch>/wheels"
```

Use the exact emitted wheel path, not a glob that might pick an older wheel:

```bash
uv run --no-sync python tests/contract/check_stub_contract.py \
  --wheel "<absolute-exact-wheel-path>" --self-test
```

Inspect the contract result as well as the exit status: `passed` and
`origin.source_stub_matches_wheel` must both be true. The latter is reported,
not enforced by the harness. If validation fails, report the scratch wheel as
unverified and do not adopt it in the parent. This checks selected installed-wheel
typing and runtime promises; it does not replace the source gate in `verify`.

Check the exact wheel's METADATA and installed CLI version against the workspace
version, using that wheel from an external working directory, not a potentially
stale editable extension. Exercise the parent's reduced-record serializer on a
real native plan and compare its plan/suggestion fields; its synthetic tests
alone cannot detect changes in this fork's API.

Only after those checks pass, and when the parent edit is in scope, copy the
verified wheel into its wheelhouse and update its current entry in the parent's
own format, including the source commit and `wheel_sha256` from the contract
result. Remove superseded current-wheel records rather than adding a history
archive or another receipt system. Environment installation and gitlink adoption
are separate steps, not implied by building. The current parent serializer
already omits `doc_url` and `references`; check compatibility again when the
native schema changes. Do not change the parent as a side effect unless that
work was requested.
