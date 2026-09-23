"""Python bindings for cognitive complexity analysis."""

from enum import Enum
from typing import Never, Self, final

@final
class RuleCategory(Enum):
    """Rule category enum; each member's value equals its name."""

    Complexity = "Complexity"
    Readability = "Readability"

@final
class Applicability(Enum):
    """Enum for a rule's ceiling or a suggestion's applicability.

    Each member's value equals its name. A MachineApplicable rule can produce
    help without a suggestion. Check for a suggestion before reading its
    applicability.
    """

    MachineApplicable = "MachineApplicable"
    MaybeIncorrect = "MaybeIncorrect"
    Informational = "Informational"

@final
class DiffStatus(Enum):
    """Comparison status enum; each member's value equals its name.

    It is not a str subclass. Formatting a member yields a qualified string
    such as DiffStatus.REGRESSED; compare members, or read .name or .value.
    """

    REGRESSED = "REGRESSED"
    IMPROVED = "IMPROVED"
    UNCHANGED = "UNCHANGED"
    NEW = "NEW"
    REMOVED = "REMOVED"

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
        """Canonical file path, relative to the analysis root when inside it.

        Files outside the root keep their absolute path. Public file_complexity
        uses base_path (default CWD); collectors and directory analysis use
        invocation_path. Symlinks resolve before path labeling.
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
    """Analyze a file relative to an existing directory root.

    Args:
        file_path: Absolute path or path relative to base_path.
        base_path: Existing directory resolved from the process CWD. Results
            inside it have root-relative paths; outside results have absolute
            paths. Existing paths are canonicalized, including symlinks.
        check_script: Also report module-level complexity as a <module> entry.
        no_ignore: Analyze functions even when they carry an ignore marker.

    Raises:
        ValueError: If the base is not an existing directory, or if reading,
            UTF-8 decoding, or parsing fails.
    """
    ...

def run_cli(argv: list[str], invocation_path: str | None = None) -> int:
    """Run the CLI with argv excluding the program name and return its exit code.

    invocation_path defaults to the current working directory. This is the
    console-script bootstrap, not a public complexipy package export.
    """
    ...

def run_lsp() -> int:
    """Run the language server over stdio and return its exit code.

    This backs `complexipy lsp` and is not a public complexipy package export.
    stdout carries only protocol frames; logs go to stderr. The call blocks
    until the server exits and releases the interpreter lock while it runs.
    Returns 0 after a clean shutdown and 1 otherwise.
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

    Relative inputs resolve against invocation_path, an existing directory
    resolved from the process CWD. Successful paths are root-relative inside
    it and absolute outside it. Failed entries are plain absolute paths with
    no appended error text. Invalid exclusions identify the walked directory;
    emitted traversal errors identify their path or fall back to the walk root.
    Successful rows survive failures and retain repeated input multiplicity.
    Overlapping requests may repeat failed paths; the lists are not sets.
    Directory discovery applies excludes and ignore rules; explicit files bypass
    those filters. An invalid invocation root raises ValueError.
    The ignore walker suppresses ignore-file I/O errors internally; an empty
    failed-path list does not prove complete filter-file readability.

    Reporting scans def lines, so markers above the first decorator and markers
    on async def are not reported, even when they suppress analysis. A bare
    noqa is not recognized.
    """
    ...

def collect_removable_ignored_locations(
    paths: list[str],
    exclude: list[str],
    max_complexity_allowed: int,
    invocation_path: str = ".",
) -> tuple[list[RemovableIgnore], list[str]]:
    """Return recognized markers no longer needed and paths that failed.

    A marker is removable when its function scores at or below
    max_complexity_allowed without suppression. Input resolution, result paths,
    failures, directory filtering and marker placement limits are the same as
    collect_all_ignored_locations. An invalid invocation root raises ValueError.
    """
    ...
