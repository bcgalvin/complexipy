import enum

import pytest

import complexipy
from complexipy import _complexipy as native

NESTED_CONDITIONALS = (
    "def f(a, b):\n    if a:\n        if b:\n            return 1\n"
    "    return 0\n"
)

MEMBERS = {
    complexipy.RuleCategory: ["Complexity", "Readability"],
    complexipy.Applicability: [
        "MachineApplicable",
        "MaybeIncorrect",
        "Informational",
    ],
    complexipy.DiffStatus: [
        "REGRESSED",
        "IMPROVED",
        "UNCHANGED",
        "NEW",
        "REMOVED",
    ],
}


@pytest.mark.parametrize("enum_type", MEMBERS)
def test_the_types_are_python_enums(enum_type):
    assert issubclass(enum_type, enum.Enum)
    assert enum_type.__module__ == "complexipy"


@pytest.mark.parametrize("enum_type", MEMBERS)
def test_members_keep_their_declaration_order(enum_type):
    assert [member.name for member in enum_type] == MEMBERS[enum_type]


@pytest.mark.parametrize("enum_type", MEMBERS)
def test_the_value_equals_the_member_name(enum_type):
    for member in enum_type:
        assert member.value == member.name
        assert enum_type(member.value) is member


def test_the_native_module_holds_the_same_classes():
    for enum_type in MEMBERS:
        assert getattr(native, enum_type.__name__) is enum_type


def test_a_derived_status_is_a_member_of_the_exported_class():
    regressed = complexipy.DiffEntry("a.py", "f", 1, 2)
    added = complexipy.DiffEntry("a.py", "f", None, 2)

    assert regressed.status is complexipy.DiffStatus.REGRESSED
    assert added.status is complexipy.DiffStatus.NEW
    assert isinstance(regressed.status, enum.Enum)


def test_analysis_fields_return_members_of_the_exported_classes():
    function = complexipy.code_complexity(NESTED_CONDITIONALS).functions[0]

    assert function.refactor_plans
    plan = function.refactor_plans[0]

    assert type(plan.category) is complexipy.RuleCategory
    assert plan.suggestion is not None
    assert type(plan.suggestion.applicability) is complexipy.Applicability
