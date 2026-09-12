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
)


def assign_results(
    suggestion: CodeSuggestion,
    line: LineComplexity,
    plan: RefactorPlan,
    function: FunctionComplexity,
    file: FileComplexity,
    result: CodeComplexity,
    location: IgnoredLocation,
    marker: RemovableIgnore,
) -> None:
    suggestion.replacement = "pass"
    suggestion.applicability = Applicability.Informational
    suggestion.description = "x"
    suggestion.spliceable = False
    line.line = 1
    line.complexity = 0
    plan.kind = "x"
    plan.title = "x"
    plan.line_start = 1
    plan.line_end = 1
    plan.column_start = 1
    plan.current_complexity = 0
    plan.estimated_reduction = 0
    plan.estimated_complexity_after = 0
    plan.reduction_is_measured = False
    plan.rule_id = "x"
    plan.category = RuleCategory.Complexity
    plan.applicability = Applicability.Informational
    plan.description = "x"
    plan.explanation = "x"
    plan.suggestion = None
    plan.help = None
    function.name = "x"
    function.complexity = 0
    function.line_start = 1
    function.line_end = 1
    function.line_complexities = []
    function.refactor_plans = []
    function.additional_refactor_plans = 0
    file.path = "x"
    file.file_name = "x"
    file.functions = []
    file.complexity = 0
    result.functions = []
    result.complexity = 0
    location.path = "x"
    location.line = 1
    location.comment = "x"
    marker.path = "x"
    marker.line = 1
    marker.comment = "x"
    marker.function = "x"
    marker.complexity = 0
