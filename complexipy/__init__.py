"""complexipy - cognitive complexity analyzer for Python.

The analysis engine, the diff comparison, and the ratchet gates are
implemented in Rust and exposed through the ``complexipy._complexipy``
extension module. This package is a thin re-export layer only.
"""

import complexipy._complexipy as _complexipy
from complexipy._complexipy import (
    Applicability,
    CodeComplexity,
    CodeSuggestion,
    DiffEntry,
    DiffStatus,
    FileComplexity,
    FunctionComplexity,
    IgnoredLocation,
    LineComplexity,
    RefactorPlan,
    RemovableIgnore,
    RuleCategory,
    code_complexity,
    collect_all_ignored_locations,
    collect_removable_ignored_locations,
    compute_diff,
    has_regressions,
)

__all__ = [
    "Applicability",
    "CodeComplexity",
    "CodeSuggestion",
    "DiffEntry",
    "DiffStatus",
    "FileComplexity",
    "FunctionComplexity",
    "IgnoredLocation",
    "LineComplexity",
    "RefactorPlan",
    "RemovableIgnore",
    "RuleCategory",
    "code_complexity",
    "collect_all_ignored_locations",
    "collect_removable_ignored_locations",
    "compute_diff",
    "file_complexity",
    "has_regressions",
]


def file_complexity(
    file_path: str,
    check_script: bool = False,
    no_ignore: bool = False,
    *,
    base_path: str = ".",
) -> FileComplexity:
    """Analyze the cognitive complexity of a single Python source file.

    Args:
        file_path: Absolute path or a path relative to base_path. The file
            must exist and be readable.
        base_path: Existing directory for resolving inputs and reporting
            paths, relative to the process working directory. Defaults to
            that working directory. Results outside it have absolute paths.
        check_script: If True, also report cognitive complexity of
            module-level (script) code as a '<module>' entry.
        no_ignore: If True, disregard all '# complexipy: ignore' and
            '# noqa: complexipy' comments, analyzing every function.

    Returns:
        FileComplexity object containing complete analysis results for the
        file, including all functions found and their complexity scores.

    Raises:
        ValueError: If the base is not an existing directory, or if reading,
            UTF-8 decoding, or parsing the file fails.
    """
    return _complexipy.file_complexity(
        file_path, base_path, check_script, no_ignore
    )
