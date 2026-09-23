import json
from functools import partial

import pytest

from complexipy import _complexipy as native
from complexipy import (
    collect_all_ignored_locations,
    collect_removable_ignored_locations,
)

MARKED = "def f(a):  # complexipy: ignore\n    return a\n"
COLLECTORS = [
    collect_all_ignored_locations,
    partial(collect_removable_ignored_locations, max_complexity_allowed=15),
]


@pytest.mark.parametrize("collector", COLLECTORS, ids=["all", "removable"])
def test_collectors_report_real_walk_errors_and_keep_good_rows(
    tmp_path, collector
):
    (tmp_path / "good.py").write_text(MARKED)
    blocked = tmp_path / "blocked"
    blocked.mkdir()
    (blocked / "hidden.py").write_text(MARKED)
    blocked.chmod(0)
    try:
        with pytest.raises(PermissionError):
            list(blocked.iterdir())
        rows, failed = collector(["."], [], invocation_path=str(tmp_path))
    finally:
        blocked.chmod(0o700)
    assert [row.path for row in rows] == ["good.py"]
    assert failed == [blocked.resolve().as_posix()]


@pytest.mark.parametrize("collector", COLLECTORS, ids=["all", "removable"])
def test_invalid_exclusion_returns_a_path_not_a_diagnostic(tmp_path, collector):
    (tmp_path / "good.py").write_text(MARKED)
    rows, failed = collector(["."], ["["], invocation_path=str(tmp_path))
    assert not rows
    assert failed == [tmp_path.resolve().as_posix()]


@pytest.mark.parametrize("quiet", [False, True])
@pytest.mark.parametrize("custom_output", [False, True])
def test_cli_reports_walk_failure_once_and_preserves_input_multiplicity(
    tmp_path, capfd, quiet, custom_output
):
    target = tmp_path / "pkg"
    target.mkdir()
    (target / "good.py").write_text(MARKED)
    blocked = target / "blocked"
    blocked.mkdir()
    (blocked / "hidden.py").write_text(MARKED)
    output = tmp_path / "out" if custom_output else tmp_path
    output.mkdir(exist_ok=True)
    report = output / "complexipy-ignored.json"
    report.write_text('[{"stale": true}]\n')
    args = [
        "pkg",
        "pkg",
        "--report-ignored",
        "--no-ignore",
        "--output-format",
        "json",
        "--snapshot-create",
    ]
    if quiet:
        args.append("--quiet")
    if custom_output:
        args.extend(["--output", "out/"])
    blocked.chmod(0)
    try:
        exit_code = native.run_cli(args, invocation_path=str(tmp_path))
    finally:
        blocked.chmod(0o700)
    captured = capfd.readouterr()
    assert exit_code == 1
    assert captured.err.count(blocked.resolve().as_posix()) == 1
    assert "Incomplete collection" in captured.err
    assert "Ignore-marker collection incomplete" in captured.err
    assert "No ignore comments found" not in captured.out
    assert not report.exists()
    assert not (tmp_path / "complexipy-snapshot.json").exists()
    assert not (tmp_path / ".complexipy_cache").exists()
    rows = json.loads((output / "complexipy-results.json").read_text())
    assert [row["path"] for row in rows] == ["pkg/good.py", "pkg/good.py"]


@pytest.mark.parametrize("quiet", [False, True])
def test_analysis_failure_keeps_completed_marker_inventory_without_snapshot_verdict(
    tmp_path, capfd, quiet
):
    (tmp_path / "good.py").write_text(MARKED)
    assert (
        native.run_cli(
            ["good.py", "--snapshot-create"], invocation_path=str(tmp_path)
        )
        == 0
    )
    snapshot = (tmp_path / "complexipy-snapshot.json").read_bytes()
    (tmp_path / "bad.py").write_text("def broken(:\n")
    capfd.readouterr()
    args = [
        ".",
        "--no-ignore",
        "--report-ignored",
        "--output-format",
        "csv,json",
        "--output",
        "out/",
    ]
    if quiet:
        args.append("--quiet")
    assert native.run_cli(args, invocation_path=str(tmp_path)) == 1
    captured = capfd.readouterr()
    assert "Snapshot watermark" not in captured.out
    assert "Ignore-marker collection incomplete" not in captured.err
    assert (tmp_path / "complexipy-snapshot.json").read_bytes() == snapshot
    markers = json.loads((tmp_path / "out/complexipy-ignored.json").read_text())
    assert [row["path"] for row in markers] == ["good.py"]
    rows = json.loads((tmp_path / "out/complexipy-results.json").read_text())
    assert [row["path"] for row in rows] == ["good.py"]
