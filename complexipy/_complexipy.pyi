"""Python bindings for cognitive complexity analysis."""

from typing import Final, Never, Self, final

@final
class RuleCategory:
    """PyO3 enum with class members, no name/value attributes or iteration."""

    def __new__(cls, _token: Never, /) -> Self: ...
    Complexity: Final[RuleCategory]
    Readability: Final[RuleCategory]

@final
class Applicability:
    """PyO3 enum describing a rule's ceiling or a suggestion's applicability.

    A MachineApplicable rule can produce help without a suggestion. Check for
    a suggestion before reading its applicability. Members have no name/value
    attributes and the class is not iterable.
    """

    def __new__(cls, _token: Never, /) -> Self: ...
    MachineApplicable: Final[Applicability]
    MaybeIncorrect: Final[Applicability]
    Informational: Final[Applicability]

@final
class DiffStatus:
    """PyO3 enum, not enum.Enum or str; has no name/value or iteration.

    Formatting a member yields a qualified string such as DiffStatus.REGRESSED.
    Compare members directly rather than comparing to unqualified strings.
    """

    def __new__(cls, _token: Never, /) -> Self: ...
    REGRESSED: Final[DiffStatus]
    IMPROVED: Final[DiffStatus]
    UNCHANGED: Final[DiffStatus]
    NEW: Final[DiffStatus]
    REMOVED: Final[DiffStatus]

@final
class DiffEntry:
    """Constructible comparison result with read-only attributes."""

    def __init__(
        self,
        file_path: str,
        func_name: str,
        old_complexity: int | None,
        new_complexity: int | None,
    ) -> None: ...
    @property
    def file_path(self) -> str: ...
    @property
    def func_name(self) -> str: ...
    @property
    def old_complexity(self) -> int | None: ...
    @property
    def new_complexity(self) -> int | None: ...
    @property
    def status(self) -> DiffStatus: ...

@final
class CodeSuggestion:
    """Replacement returned by analysis; cannot be constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def replacement(self) -> str: ...
    @property
    def applicability(self) -> Applicability: ...
    @property
    def description(self) -> str: ...
    @property
    def spliceable(self) -> bool:
        """Whether the replacement is a source splice eligible for measurement."""
        ...

@final
class LineComplexity:
    """One line's score contribution, returned by analysis, not constructed.

    Boolean complexity counts operator runs, not individual operators. See
    docs/scoring.md for the scoring rules and expression-walker limits.
    """

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def line(self) -> int:
        """One-based source line number."""
        ...
    @property
    def complexity(self) -> int: ...

@final
class RefactorPlan:
    """Ranked refactoring plan returned by analysis, not constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def kind(self) -> str: ...
    @property
    def title(self) -> str: ...
    @property
    def line_start(self) -> int: ...
    @property
    def line_end(self) -> int: ...
    @property
    def column_start(self) -> int:
        """One-based column of the construct on line_start."""
        ...
    @property
    def current_complexity(self) -> int: ...
    @property
    def estimated_reduction(self) -> int: ...
    @property
    def estimated_complexity_after(self) -> int: ...
    @property
    def reduction_is_measured(self) -> bool:
        """True for a measured source splice; false for a formula estimate."""
        ...
    @property
    def rule_id(self) -> str: ...
    @property
    def category(self) -> RuleCategory: ...
    @property
    def applicability(self) -> Applicability:
        """Rule's declared ceiling, not the outcome of this plan."""
        ...
    @property
    def description(self) -> str: ...
    @property
    def explanation(self) -> str: ...
    @property
    def suggestion(self) -> CodeSuggestion | None: ...
    @property
    def help(self) -> str | None: ...

@final
class FunctionComplexity:
    """Function result returned by analysis; cannot be constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def name(self) -> str:
        """Function name, Class::method for methods, or <module> in script mode.

        Nested functions contribute to their enclosing function and are not
        separately reported.
        """
        ...
    @property
    def complexity(self) -> int: ...
    @property
    def line_start(self) -> int:
        """One-based declaration start, including decorators."""
        ...
    @property
    def line_end(self) -> int: ...
    @property
    def line_complexities(self) -> list[LineComplexity]: ...
    @property
    def refactor_plans(self) -> list[RefactorPlan]:
        """Ranked refactoring plans, capped at five."""
        ...
    @property
    def additional_refactor_plans(self) -> int:
        """Cap drops plus selected plans whose measured reduction fell below one.

        Does not include earlier noise or overlap rejections.
        """
        ...

@final
class FileComplexity:
    """File result returned by analysis; cannot be constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def path(self) -> str:
        """Path relative to the analysis base where possible.

        The public wrapper uses the CWD for files beneath it and the file's
        parent otherwise, so out-of-tree files use their basename. The native
        binding retains file_path if it cannot strip base_path.
        """
        ...
    @property
    def file_name(self) -> str: ...
    @property
    def functions(self) -> list[FunctionComplexity]:
        """Top-level functions and methods, with <module> appended in script mode."""
        ...
    @property
    def complexity(self) -> int:
        """Function totals plus module-level complexity, regardless of script mode."""
        ...

@final
class CodeComplexity:
    """Source-string result returned by analysis, not constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def functions(self) -> list[FunctionComplexity]: ...
    @property
    def complexity(self) -> int:
        """Function totals plus module-level complexity, regardless of script mode."""
        ...

@final
class IgnoredLocation:
    """Reported marker returned by a collector, not constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def path(self) -> str: ...
    @property
    def line(self) -> int: ...
    @property
    def comment(self) -> str: ...

@final
class RemovableIgnore:
    """Marker returned by the removable-ignore collector, not constructed directly."""

    def __new__(cls, _token: Never, /) -> Self: ...
    @property
    def path(self) -> str: ...
    @property
    def line(self) -> int: ...
    @property
    def comment(self) -> str: ...
    @property
    def function(self) -> str: ...
    @property
    def complexity(self) -> int: ...

def code_complexity(
    code: str, check_script: bool = False, no_ignore: bool = False
) -> CodeComplexity:
    """Analyze a source string.

    Args:
        code: Python source to analyze.
        check_script: Also report module-level complexity as a <module> entry.
        no_ignore: Analyze functions even when they carry an ignore marker.

    Raises:
        ValueError: If parsing fails.
    """
    ...

def file_complexity(
    file_path: str,
    base_path: str,
    check_script: bool = False,
    no_ignore: bool = False,
) -> FileComplexity:
    """Analyze a file using an explicit base for the reported path.

    Args:
        file_path: Path to the Python source file.
        base_path: Prefix to strip from the reported path when possible.
        check_script: Also report module-level complexity as a <module> entry.
        no_ignore: Analyze functions even when they carry an ignore marker.

    Raises:
        ValueError: If reading, UTF-8 decoding, or parsing fails.
    """
    ...

def run_cli(argv: list[str], invocation_path: str | None = None) -> int:
    """Run the CLI with argv excluding the program name and return its exit code.

    invocation_path defaults to the current working directory. This is the
    console-script bootstrap, not a public complexipy package export.
    """
    ...

def compute_diff(
    current_files: list[FileComplexity],
    git_ref: str,
    invocation_path: str | None = None,
) -> list[DiffEntry]:
    """Compare current results to a Git reference.

    invocation_path is the working directory for Git commands, defaulting to
    the current working directory. A missing reference is treated as absent
    old content, making the supplied functions NEW rather than raising.
    """
    ...

def has_regressions(entries: list[DiffEntry], max_complexity: int) -> bool:
    """Whether any REGRESSED or NEW function exceeds max_complexity."""
    ...

def collect_all_ignored_locations(
    paths: list[str], exclude: list[str], invocation_path: str = "."
) -> tuple[list[IgnoredLocation], list[str]]:
    """Return recognized ignore locations and paths that could not be processed.

    paths contains local files or directories; exclude contains exclusion
    patterns. invocation_path is accepted but ignored: relative inputs resolve
    against the process CWD. Reporting scans def lines, so markers above the
    first decorator and markers on async def are not reported, even when they
    suppress analysis. A bare noqa is not recognized.
    """
    ...

def collect_removable_ignored_locations(
    paths: list[str],
    exclude: list[str],
    max_complexity_allowed: int,
    invocation_path: str = ".",
) -> tuple[list[RemovableIgnore], list[str]]:
    """Return recognized markers no longer needed and paths that failed.

    paths contains local files or directories; exclude contains exclusion
    patterns. A marker is removable when its function scores at or below
    max_complexity_allowed without suppression. invocation_path is accepted
    but ignored: relative inputs resolve against the process CWD. Reporting
    has the same placement limits as collect_all_ignored_locations.
    """
    ...
