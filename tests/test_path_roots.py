from functools import partial
from pathlib import Path

import pytest

from complexipy import _complexipy as native
from complexipy import (
    collect_all_ignored_locations,
    collect_removable_ignored_locations,
    file_complexity,
)

MARKED = "def f(a):  # complexipy: ignore\n    if a:\n        return a\n"
COLLECTORS = [
    collect_all_ignored_locations,
    partial(collect_removable_ignored_locations, max_complexity_allowed=15),
]


def write(root: Path, name: str) -> Path:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(MARKED)
    return path


def test_public_and_native_file_paths_share_one_root(tmp_path, monkeypatch):
    root = tmp_path / "repo"
    first = write(root, "a/utils.py")
    second = write(root, "b/utils.py")
    external = tmp_path / "external"
    external.mkdir()
    monkeypatch.chdir(external)

    for path, expected in [(first, "a/utils.py"), (second, "b/utils.py")]:
        default = file_complexity(str(path), False, True)
        assert default.path == path.resolve().as_posix()
        assert default.complexity == 1
        result = file_complexity(expected, no_ignore=True, base_path="../repo")
        assert result.path == expected
        assert result.functions[0].complexity == 1
        native_result = native.file_complexity(
            expected, base_path=str(root), no_ignore=True
        )
        assert native_result.path == expected
        assert native_result.complexity == result.complexity


def test_default_root_is_cwd_and_outside_paths_remain_absolute(
    tmp_path, monkeypatch
):
    root = tmp_path / "repo"
    inside = write(root, "pkg/inside.py")
    outside = write(tmp_path / "other", "outside.py")
    monkeypatch.chdir(root)
    assert file_complexity("pkg/inside.py").path == "pkg/inside.py"
    assert file_complexity(str(inside)).path == "pkg/inside.py"
    assert (
        file_complexity("../other/outside.py").path
        == outside.resolve().as_posix()
    )


def test_base_selects_lookup_root_not_the_process_cwd(tmp_path, monkeypatch):
    root = tmp_path / "repo"
    write(root, "file.py")
    external = tmp_path / "external"
    external.mkdir()
    (external / "file.py").write_text("not valid python !!!")
    monkeypatch.chdir(external)
    assert file_complexity("file.py", base_path=str(root)).path == "file.py"
    with pytest.raises(ValueError):
        file_complexity("file.py")


@pytest.mark.parametrize("root_kind", ["missing", "file.py"])
def test_file_base_must_be_an_existing_directory(tmp_path, root_kind):
    source = write(tmp_path, "file.py")
    with pytest.raises(ValueError, match="root"):
        file_complexity(str(source), base_path=str(tmp_path / root_kind))


def test_file_path_arguments_must_be_strings(tmp_path):
    source = write(tmp_path, "file.py")
    with pytest.raises(TypeError):
        file_complexity(source)
    with pytest.raises(TypeError):
        file_complexity("file.py", base_path=tmp_path)


def test_public_base_is_keyword_only():
    with pytest.raises(TypeError):
        file_complexity("file.py", False, False, ".")


@pytest.mark.parametrize("collector", COLLECTORS, ids=["all", "removable"])
def test_collectors_resolve_inputs_and_report_consistent_paths(
    tmp_path, monkeypatch, collector
):
    root = tmp_path / "repo"
    write(root, "pkg/a/utils.py")
    write(root, "pkg/b/utils.py")
    external = tmp_path / "external"
    external.mkdir()
    monkeypatch.chdir(external)
    for inputs in [["pkg"], ["pkg/a/utils.py", "pkg/b/utils.py"]]:
        rows, failed = collector(inputs, [], invocation_path="../repo")
        assert not failed
        assert [row.path for row in rows] == [
            "pkg/a/utils.py",
            "pkg/b/utils.py",
        ]
    rows, failed = collector(
        ["pkg/a/utils.py", "missing.py"], [], invocation_path=str(root)
    )
    assert [row.path for row in rows] == ["pkg/a/utils.py"]
    assert failed == [(root.resolve() / "missing.py").as_posix()]


@pytest.mark.parametrize("collector", COLLECTORS, ids=["all", "removable"])
def test_collectors_outside_root_paths_remain_absolute(tmp_path, collector):
    root = tmp_path / "repo"
    root.mkdir()
    outside = write(tmp_path / "other", "utils.py")
    rows, failed = collector([str(outside)], [], invocation_path=str(root))
    assert not failed
    assert [row.path for row in rows] == [outside.resolve().as_posix()]


@pytest.mark.parametrize("collector", COLLECTORS, ids=["all", "removable"])
@pytest.mark.parametrize("root_kind", ["missing", "file.py"])
def test_collectors_reject_invalid_roots(tmp_path, collector, root_kind):
    write(tmp_path, "file.py")
    with pytest.raises(ValueError, match="root"):
        collector([], [], invocation_path=str(tmp_path / root_kind))
