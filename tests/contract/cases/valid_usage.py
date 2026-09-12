from complexipy import (
    Applicability,
    DiffEntry,
    DiffStatus,
    RuleCategory,
    code_complexity,
    compute_diff,
    has_regressions,
)

entry = DiffEntry(
    file_path="a.py", func_name="f", old_complexity=2, new_complexity=6
)
positional = DiffEntry("b.py", "g", None, 3)
path: str = entry.file_path
name: str = entry.func_name
old = entry.old_complexity
new = entry.new_complexity
status: DiffStatus = entry.status
regressed_status: DiffStatus = DiffStatus.REGRESSED
improved_status: DiffStatus = DiffStatus.IMPROVED
unchanged_status: DiffStatus = DiffStatus.UNCHANGED
new_status: DiffStatus = DiffStatus.NEW
removed_status: DiffStatus = DiffStatus.REMOVED
category: RuleCategory = RuleCategory.Complexity
readability: RuleCategory = RuleCategory.Readability
applicability: Applicability = Applicability.MachineApplicable
maybe_incorrect: Applicability = Applicability.MaybeIncorrect
informational: Applicability = Applicability.Informational
regressed: bool = has_regressions([entry], 5)
entries = compute_diff([], "HEAD", None)
result = code_complexity(
    "def f():\n    pass\n", check_script=False, no_ignore=False
)
total: int = result.complexity
