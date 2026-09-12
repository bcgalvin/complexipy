---
name: ffi-change
description: Change complexipy's Python/native API or stubs with matching Rust definitions, bindings, exports and installed-wheel contract coverage.
---

# Change the FFI contract

Read `AGENTS.md`'s FFI contract and `docs/python-api.md`. Identify whether this is
a type, field, function or export change; do not apply a type-addition checklist
blindly to every edit.

- **Shared type:** definitions live in `crates/complexipy-core/src/classes.rs`.
  A new Python-visible type needs the `#[pymodule]` export in
  `crates/complexipy-python/src/lib.rs` and a declaration in
  `complexipy/_complexipy.pyi`. If publicly exported by the package, update
  `complexipy/__init__.py`, including `__all__`, and `docs/python-api.md`.
- **Field:** update its Rust definition, the stub and all affected Rust literals
  and conversions. `add_class` does not enumerate fields. Preserve read-only
  properties unless setters really exist. Core's `python` feature gates both
  PyO3 annotations and `serde(skip)`; consider both serialization shapes.
- **Diff types:** Python `DiffEntry` and `DiffStatus` live in `py_diff` in the
  Python crate's `lib.rs`, with conversions to distinct core diff types. Update
  those conversions when affected, rather than looking only in `classes.rs`.
- **Function:** keep the implementation, PyO3 binding, stub and any Python
  wrapper/exports synchronized. Use explicit `#[pyo3(signature = ...)]` for
  defaults. Test omitted arguments and keyword calls at runtime; `Option` alone
  does not make an argument optional in Python.
- **Stable Rust surface:** inspect `crates/complexipy-core/src/lib.rs` and
  `crates/complexipy-core/tests/lib_surface.rs` when changing public Rust
  re-exports. They have their own contract, not a copy of Python's `__all__`.

Keep constructor promises honest: the eight native result structs have no
Python constructors; the stub's required `Never` argument to `__new__` is only
a typing guard. `DiffEntry` has a real constructor. Do not document the guard
as a runtime API.

Add or update consumer cases in `tests/contract/cases/` and their expectations
in `tests/contract/check_stub_contract.py`, plus the relevant runtime checks.
New case files need an `EXPECTED_DIAGNOSTICS` entry to run. New native result
types also need an instance in `cases/result_usage.py`'s `objects` tuple, which
feeds the runtime getter/read-only/constructor sweep.
Prove both valid use and rejected use for the changed promise; keep intentional
type errors Ruff-clean. The root ty check does not establish stub/native parity.
Run `verify`, rebuilding before pytest after Rust changes. Report compatibility
breaks, including parent consumers, without changing the parent unless asked.
