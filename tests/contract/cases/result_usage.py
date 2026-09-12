from pathlib import Path
from tempfile import TemporaryDirectory
from typing import assert_type

from complexipy import (
    Applicability,
    CodeComplexity,
    CodeSuggestion,
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
    file_complexity,
)

source = "def f(a, b):\n    if a:\n        if b:\n            pass\n"
result = code_complexity(source, check_script=False, no_ignore=False)
assert_type(result, CodeComplexity)
assert_type(result.functions, list[FunctionComplexity])
assert_type(result.complexity, int)
function = result.functions[0]
assert_type(function.name, str)
assert_type(function.complexity, int)
assert_type(function.line_start, int)
assert_type(function.line_end, int)
assert_type(function.line_complexities, list[LineComplexity])
assert_type(function.refactor_plans, list[RefactorPlan])
assert_type(function.additional_refactor_plans, int)
line = function.line_complexities[0]
assert_type(line.line, int)
assert_type(line.complexity, int)
plan = function.refactor_plans[0]
assert_type(plan.kind, str)
assert_type(plan.title, str)
assert_type(plan.line_start, int)
assert_type(plan.line_end, int)
assert_type(plan.column_start, int)
assert_type(plan.current_complexity, int)
assert_type(plan.estimated_reduction, int)
assert_type(plan.estimated_complexity_after, int)
assert_type(plan.reduction_is_measured, bool)
assert_type(plan.rule_id, str)
assert_type(plan.category, RuleCategory)
assert_type(plan.applicability, Applicability)
assert_type(plan.description, str)
assert_type(plan.explanation, str)
assert_type(plan.suggestion, CodeSuggestion | None)
assert_type(plan.help, str | None)
suggestion = plan.suggestion
assert suggestion is not None
assert_type(suggestion.replacement, str)
assert_type(suggestion.applicability, Applicability)
assert_type(suggestion.description, str)
assert_type(suggestion.spliceable, bool)

with TemporaryDirectory() as directory:
    path = Path(directory) / "example.py"
    path.write_text(
        source + "\n# complexipy: ignore\ndef ignored():\n    pass\n",
        encoding="utf-8",
    )
    file = file_complexity(str(path), check_script=False, no_ignore=False)
    assert_type(file, FileComplexity)
    assert_type(file.path, str)
    assert_type(file.file_name, str)
    assert_type(file.functions, list[FunctionComplexity])
    assert_type(file.complexity, int)
    ignored, failures = collect_all_ignored_locations([str(path)], [])
    assert_type(ignored, list[IgnoredLocation])
    assert_type(failures, list[str])
    assert not failures
    location = ignored[0]
    assert_type(location.path, str)
    assert_type(location.line, int)
    assert_type(location.comment, str)
    removable, failures = collect_removable_ignored_locations(
        [str(path)], [], 15
    )
    assert_type(removable, list[RemovableIgnore])
    assert not failures
    marker = removable[0]
    assert_type(marker.path, str)
    assert_type(marker.line, int)
    assert_type(marker.comment, str)
    assert_type(marker.function, str)
    assert_type(marker.complexity, int)

objects = (suggestion, line, plan, function, file, result, location, marker)
