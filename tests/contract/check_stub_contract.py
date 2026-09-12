"""Check the installed-wheel typing and runtime contract of complexipy.

Builds a wheel (or takes one via --wheel), installs it into a fresh virtual
environment, verifies the installed stub matches the wheel, type-checks each
case file from a neutral directory with ty, and confirms runtime agreement.
"""

from __future__ import annotations

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
    "assign_readonly.py": [
        (4, "invalid-assignment"),
        (5, "invalid-assignment"),
    ],
    "bad_keyword.py": [(3, "missing-argument"), (3, "unknown-argument")],
    "phantom_import.py": [(1, "unresolved-import")],
}

RUNTIME_CHECKS = """
import complexipy._complexipy as native
from complexipy import DiffEntry, DiffStatus, code_complexity

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
try:
    code_complexity(source="def f():\\n    pass\\n")
except TypeError:
    pass
else:
    raise SystemExit("code_complexity accepted an unknown keyword")
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
    valid_case = run(
        [str(python), str(case_dir / "valid_usage.py")], cwd=case_dir
    )
    if valid_case.returncode:
        problems.append(
            f"valid_usage.py failed at runtime:\n{valid_case.stderr}"
        )
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
