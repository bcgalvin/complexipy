"""Check the installed-wheel typing and runtime contract of complexipy.

Builds a wheel (or takes one via --wheel), installs it into a fresh virtual
environment, verifies the installed stub matches the wheel, type-checks each
case file from a neutral directory with ty, and confirms runtime agreement.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
CASES_DIR = Path(__file__).resolve().parent / "cases"

EXPECTED_DIAGNOSTICS: dict[str, list[tuple[int, str]]] = {
    "valid_usage.py": [],
    "result_usage.py": [],
    "assign_results.py": [
        (line, "invalid-assignment") for line in range(25, 68)
    ],
    "construct_results.py": [
        (line, "missing-argument" if line % 2 == 0 else "invalid-argument-type")
        for line in range(12, 28)
    ],
    "reassign_enums.py": [
        (3, "invalid-assignment"),
        (4, "invalid-assignment"),
        (5, "invalid-assignment"),
    ],
    "subclass_native.py": [
        (line, "subclass-of-final-class") for line in range(17, 62, 4)
    ],
    "assign_readonly.py": [
        (4, "invalid-assignment"),
        (5, "invalid-assignment"),
    ],
    "bad_keyword.py": [(3, "missing-argument"), (3, "unknown-argument")],
    "positional_base_path.py": [(3, "too-many-positional-arguments")],
    "phantom_import.py": [(1, "unresolved-import")],
    "phantom_plan_fields.py": [
        (9, "unresolved-attribute"),
        (10, "unresolved-attribute"),
    ],
}

RUNTIME_CHECKS = """
import ast
from enum import Enum
from functools import partial
from pathlib import Path
from tempfile import TemporaryDirectory
from types import GetSetDescriptorType, UnionType, new_class
from typing import get_args, get_origin

from result_usage import objects

import complexipy
import complexipy._complexipy as native
from complexipy import (
    Applicability,
    DiffEntry,
    DiffStatus,
    RefactorPlan,
    RuleCategory,
    code_complexity,
    compute_diff,
)

native_types = [
    getattr(complexipy, name)
    for name in complexipy.__all__
    if isinstance(getattr(complexipy, name), type)
]
if len(native_types) != 12:
    raise SystemExit(f"expected 12 exported native types, found {native_types}")
for cls in native_types:
    try:
        new_class(f"{cls.__name__}Child", (cls,))
    except TypeError:
        pass
    else:
        raise SystemExit(f"{cls.__name__} unexpectedly allowed subclassing")
for cls in (RuleCategory, Applicability, DiffStatus):
    if not issubclass(cls, Enum) or cls.__module__ != "complexipy":
        raise SystemExit(f"{cls.__name__} is not a complexipy enum.Enum")
    for member in cls:
        if type(member) is not cls or member.value != member.name:
            raise SystemExit(f"{cls.__name__}.{member.name} has a foreign value")
        if cls(member.value) is not member or cls[member.name] is not member:
            raise SystemExit(f"{cls.__name__} lookup missed {member.name}")
        try:
            setattr(cls, member.name, member)
        except AttributeError:
            pass
        else:
            raise SystemExit(f"{cls.__name__}.{member.name} was reassignable")
    for args in ((), (None,), (0,), ("missing",)):
        try:
            cls(*args)
        except (TypeError, ValueError):
            pass
        else:
            raise SystemExit(f"{cls.__name__} accepted {args!r}")

entry = DiffEntry(
    file_path="a.py", func_name="f", old_complexity=2, new_complexity=6
)
positional = DiffEntry("b.py", "g", None, 3)
assert (entry.file_path, entry.func_name) == ("a.py", "f")
assert (entry.old_complexity, entry.new_complexity) == (2, 6)
assert entry.status == DiffStatus.REGRESSED
assert positional.status == DiffStatus.NEW
for name in ("file_path", "old_complexity"):
    try:
        setattr(entry, name, None)
    except AttributeError:
        pass
    else:
        raise SystemExit(f"DiffEntry.{name} was writable")
for name in (
    "main",
    "output_csv",
    "output_json",
    "create_snapshot_file",
    "load_snapshot_file",
):
    if hasattr(native, name):
        raise SystemExit(f"native module unexpectedly exposes {name}")
if not hasattr(RefactorPlan, "rule_id"):
    raise SystemExit("RefactorPlan lost its rule_id accessor")
for name in ("doc_url", "references"):
    if hasattr(RefactorPlan, name):
        raise SystemExit(f"RefactorPlan unexpectedly exposes {name}")
def matches_type(value, expected):
    if get_origin(expected) is list:
        return type(value) is list and all(
            matches_type(item, get_args(expected)[0]) for item in value
        )
    if get_origin(expected) is UnionType:
        return any(matches_type(value, item) for item in get_args(expected))
    return type(value) is expected


stub = ast.parse(Path(native.__file__).with_name("_complexipy.pyi").read_text())
for cls in (RuleCategory, Applicability, DiffStatus):
    definition = next(
        node for node in stub.body
        if isinstance(node, ast.ClassDef) and node.name == cls.__name__
    )
    declared = [
        (node.targets[0].id, node.value.value)
        for node in definition.body
        if isinstance(node, ast.Assign)
    ]
    actual = [(member.name, member.value) for member in cls]
    if declared != actual:
        raise SystemExit(f"{cls.__name__} members disagree with the installed stub")
namespace = dict(vars(native))
properties = {
    cls.name: {
        field.name: eval(ast.unparse(field.returns), namespace)
        for field in cls.body
        if isinstance(field, ast.FunctionDef)
        and any(isinstance(dec, ast.Name) and dec.id == "property" for dec in field.decorator_list)
    }
    for cls in stub.body
    if isinstance(cls, ast.ClassDef)
}
for instance in objects:
    cls = type(instance)
    getters = properties[cls.__name__]
    values = {
        name: getattr(instance, name)
        for name, descriptor in vars(cls).items()
        if isinstance(descriptor, GetSetDescriptorType) and not name.startswith("_")
    }
    if not values or set(values) != set(getters):
        raise SystemExit(f"{cls.__name__} getter names disagree with the installed stub")
    for name, value in values.items():
        if not matches_type(value, getters[name]):
            raise SystemExit(f"{cls.__name__}.{name} runtime type disagrees with the stub")
        if type(value) is list:
            returned = getattr(instance, name)
            if returned is value:
                raise SystemExit(f"{cls.__name__}.{name} reused its Python list")
            returned.clear()
            if len(getattr(instance, name)) != len(value):
                raise SystemExit(f"{cls.__name__}.{name} list mutation changed the result")
        try:
            setattr(instance, name, value)
        except AttributeError:
            pass
        else:
            raise SystemExit(f"{cls.__name__}.{name} was writable")
    positional = tuple(values[name] for name in getters)
    for args, kwargs in (((), {}), ((None,), {}), (positional, {}), ((), values)):
        try:
            cls(*args, **kwargs)
        except TypeError:
            pass
        else:
            raise SystemExit(f"{cls.__name__} unexpectedly grew a constructor")
if f"{DiffStatus.REGRESSED}" != "DiffStatus.REGRESSED":
    raise SystemExit("DiffStatus no longer formats as DiffStatus.REGRESSED")
if compute_diff([], "HEAD") != []:
    raise SystemExit("compute_diff lost its invocation_path default")
try:
    complexipy.file_complexity("file.py", False, False, ".")
except TypeError:
    pass
else:
    raise SystemExit("file_complexity accepted a positional base_path")
try:
    code_complexity(source="def f():\\n    pass\\n")
except TypeError:
    pass
else:
    raise SystemExit("code_complexity accepted an unknown keyword")
with TemporaryDirectory() as directory:
    root = Path(directory).resolve()
    (root / "good.py").write_text("def f():  # complexipy: ignore\\n    pass\\n")
    blocked = root / "blocked"
    blocked.mkdir()
    (blocked / "hidden.py").write_text("def hidden():\\n    pass\\n")
    collectors = (
        complexipy.collect_all_ignored_locations,
        partial(complexipy.collect_removable_ignored_locations, max_complexity_allowed=15),
    )
    blocked.chmod(0)
    try:
        for collector in collectors:
            rows, failed = collector(["."], [], invocation_path=str(root))
            assert [row.path for row in rows] == ["good.py"]
            assert failed == [blocked.as_posix()]
    finally:
        blocked.chmod(0o700)
    for collector in collectors:
        rows, failed = collector(["."], ["["], invocation_path=str(root))
        assert not rows
        assert failed == [root.as_posix()]
print(native.__file__)
"""

DIAGNOSTIC = re.compile(
    r"^(?P<file>[^:]+):(?P<line>\d+):\d+: "
    r"(?P<level>error|warning)\[(?P<rule>[^\]]+)\]"
)

Diagnostics = list[tuple[int, str]]


def run(
    args: list[str], cwd: Path | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=cwd, text=True, capture_output=True)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def venv_python(venv: Path) -> Path:
    if os.name == "nt":
        return venv / "Scripts" / "python.exe"
    return venv / "bin" / "python"


def ty(*args: str) -> list[str]:
    return [
        "uv",
        "run",
        "--no-sync",
        "--project",
        str(PROJECT_ROOT),
        "ty",
        *args,
    ]


def build_wheel(out_dir: Path) -> Path:
    result = run(
        [
            "uv",
            "run",
            "--no-sync",
            "maturin",
            "build",
            "--profile",
            "dev",
            "--locked",
            "--interpreter",
            sys.executable,
            "--out",
            str(out_dir),
        ],
        cwd=PROJECT_ROOT,
    )
    if result.returncode:
        raise SystemExit(f"wheel build failed:\n{result.stderr}")
    wheels = sorted(out_dir.glob("complexipy-*.whl"))
    if len(wheels) != 1:
        raise SystemExit(f"expected exactly one wheel, found {wheels}")
    return wheels[0]


def install_wheel(wheel: Path, venv: Path) -> Path:
    python = venv_python(venv)
    for args in (
        ["uv", "venv", "--python", sys.executable, str(venv)],
        ["uv", "pip", "install", "--python", str(python), str(wheel)],
    ):
        result = run(args)
        if result.returncode:
            raise SystemExit(f"{' '.join(args)} failed:\n{result.stderr}")
    return python


def verify_origin(wheel: Path, venv: Path, python: Path) -> dict[str, object]:
    with zipfile.ZipFile(wheel) as archive:
        wheel_stub = archive.read("complexipy/_complexipy.pyi")
    result = run(
        [str(python), "-c", "import complexipy; print(complexipy.__file__)"],
        cwd=venv,
    )
    if result.returncode:
        raise SystemExit(
            f"import from installed wheel failed:\n{result.stderr}"
        )
    package_dir = Path(result.stdout.strip()).parent
    if venv.resolve() not in package_dir.resolve().parents:
        raise SystemExit(f"complexipy resolved outside the venv: {package_dir}")
    installed_stub = (package_dir / "_complexipy.pyi").read_bytes()
    if sha256(installed_stub) != sha256(wheel_stub):
        raise SystemExit("installed stub bytes differ from the wheel")
    source_stub = (PROJECT_ROOT / "complexipy" / "_complexipy.pyi").read_bytes()
    return {
        "package_dir": str(package_dir),
        "stub_sha256": sha256(installed_stub),
        "source_stub_matches_wheel": sha256(source_stub) == sha256(wheel_stub),
    }


def ty_version() -> str:
    result = run(ty("--version"), cwd=PROJECT_ROOT)
    if result.returncode:
        raise SystemExit(f"ty is not runnable:\n{result.stderr}")
    return result.stdout.strip()


def type_check_case(
    case_dir: Path, venv: Path, case: str
) -> tuple[int, Diagnostics, list[str]]:
    result = run(
        ty("check", "--python", str(venv), "--output-format", "concise", case),
        cwd=case_dir,
    )
    if result.returncode > 1:
        raise SystemExit(
            f"ty failed on {case} with exit {result.returncode}:\n"
            f"{result.stdout}{result.stderr}"
        )
    found: Diagnostics = []
    elsewhere: list[str] = []
    for line in result.stdout.splitlines() + result.stderr.splitlines():
        match = DIAGNOSTIC.match(line)
        if not match:
            continue
        if match.group("file") == case:
            found.append((int(match.group("line")), match.group("rule")))
        else:
            elsewhere.append(line)
    return result.returncode, sorted(found), elsewhere


def check_cases(
    case_dir: Path, venv: Path
) -> tuple[dict[str, Diagnostics], list[str]]:
    actual: dict[str, Diagnostics] = {}
    problems: list[str] = []
    for case in EXPECTED_DIAGNOSTICS:
        exit_code, found, elsewhere = type_check_case(case_dir, venv, case)
        actual[case] = found
        expected_exit = 1 if EXPECTED_DIAGNOSTICS[case] else 0
        if exit_code != expected_exit:
            problems.append(
                f"{case}: ty exited {exit_code}, expected {expected_exit}"
            )
        if elsewhere:
            problems.append(
                f"{case}: diagnostics outside the case file: {elsewhere}"
            )
    return actual, problems


def compare(
    expected: dict[str, Diagnostics], actual: dict[str, Diagnostics]
) -> list[str]:
    mismatches = []
    for case, want in expected.items():
        got = actual.get(case)
        if got != sorted(want):
            mismatches.append(f"{case}: expected {sorted(want)}, got {got}")
    return mismatches


def self_test(actual: dict[str, Diagnostics]) -> list[str]:
    corrupted = {
        case: list(rules) for case, rules in EXPECTED_DIAGNOSTICS.items()
    }
    corrupted["valid_usage.py"].append((1, "unresolved-import"))
    corrupted["bad_keyword.py"].pop()
    mismatches = compare(corrupted, actual)
    if len(mismatches) != 2:
        return [f"self-test expected 2 mismatches, got {mismatches}"]
    return []


def runtime_checks(
    python: Path, venv: Path, case_dir: Path
) -> tuple[str, list[str]]:
    problems: list[str] = []
    for case, expected in EXPECTED_DIAGNOSTICS.items():
        if expected:
            continue
        valid_case = run([str(python), str(case_dir / case)], cwd=case_dir)
        if valid_case.returncode:
            problems.append(f"{case} failed at runtime:\n{valid_case.stderr}")
    runtime = run([str(python), "-c", RUNTIME_CHECKS], cwd=case_dir)
    native_origin = runtime.stdout.strip()
    if runtime.returncode:
        problems.append(
            f"runtime checks failed:\n{runtime.stderr or runtime.stdout}"
        )
    elif venv.resolve() not in Path(native_origin).resolve().parents:
        problems.append(
            f"native module resolved outside the venv: {native_origin}"
        )
    return native_origin, problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--wheel", type=Path, help="use this wheel instead of building one"
    )
    parser.add_argument(
        "--receipt", type=Path, help="write a JSON receipt here"
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="also verify that wrong expectations fail",
    )
    options = parser.parse_args()

    workspace = Path(tempfile.mkdtemp(prefix="complexipy-contract-"))
    receipt: dict[str, object] = {"self_test": options.self_test}
    problems: list[str] = []
    try:
        receipt["ty_version"] = ty_version()
        wheel = (
            options.wheel.resolve()
            if options.wheel
            else build_wheel(workspace / "wheels")
        )
        receipt["wheel"] = wheel.name
        receipt["wheel_sha256"] = sha256(wheel.read_bytes())
        venv = workspace / "venv"
        python = install_wheel(wheel, venv)
        receipt["origin"] = verify_origin(wheel, venv, python)

        case_dir = workspace / "cases"
        shutil.copytree(CASES_DIR, case_dir)
        actual, problems = check_cases(case_dir, venv)
        receipt["diagnostics"] = {
            case: [list(d) for d in rules] for case, rules in actual.items()
        }
        problems.extend(compare(EXPECTED_DIAGNOSTICS, actual))
        if options.self_test and not problems:
            problems.extend(self_test(actual))

        native_origin, runtime_problems = runtime_checks(python, venv, case_dir)
        receipt["native_origin"] = native_origin
        problems.extend(runtime_problems)
    except SystemExit as failure:
        problems.append(str(failure))
    finally:
        shutil.rmtree(workspace, ignore_errors=True)
        receipt["problems"] = problems
        receipt["passed"] = not problems
        rendered = json.dumps(receipt, indent=2)
        if options.receipt:
            options.receipt.write_text(rendered + "\n")
        print(rendered)
    return 0 if not problems else 1


if __name__ == "__main__":
    raise SystemExit(main())
