from pathlib import PurePath

from complexipy import (
    collect_all_ignored_locations,
    collect_removable_ignored_locations,
)


def test_collectors_report_per_file_failures_and_keep_valid_rows(tmp_path):
    (tmp_path / "valid.py").write_text(
        "def simple(a):  # complexipy: ignore\n    return a\n",
        encoding="utf-8",
    )
    (tmp_path / "malformed.py").write_text(
        "def broken(:  # complexipy: ignore\n    return 1\n",
        encoding="utf-8",
    )
    (tmp_path / "bad.py").write_bytes(
        b"def f():  # complexipy: ignore\n    return '\xff'\n"
    )

    locations, failed = collect_all_ignored_locations([str(tmp_path)], [])
    assert sorted(PurePath(loc.path).name for loc in locations) == [
        "malformed.py",
        "valid.py",
    ]
    assert [PurePath(p).name for p in failed] == ["bad.py"]

    removable, failed = collect_removable_ignored_locations(
        [str(tmp_path)], [], 15
    )
    assert [r.function for r in removable] == ["simple"]
    assert sorted(PurePath(p).name for p in failed) == [
        "bad.py",
        "malformed.py",
    ]
