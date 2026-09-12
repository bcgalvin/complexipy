# Refactor rules

`--suggest-refactors` runs a Clippy-style rule set over the complexity regions
the scorer produces. Seven rules are registered. The ID space is not contiguous -
there is no C006, C008, C009 or C010.

## The catalog

| ID | `kind` | Category | Applicability | Effectiveness |
| -- | -- | -- | -- | -- |
| C001 | `flatten_condition` | Complexity | Informational | 4 |
| C002 | `loop_guards` | Complexity | MachineApplicable | 3 |
| C003 | `extract_helper` | Complexity | Informational | 2 |
| C004 | `split_dispatcher` | Complexity | Informational | 2 |
| C005 | `extract_predicate` | Readability | MachineApplicable | 2 |
| C007 | `collapsible_if` | Readability | MachineApplicable | 5 |
| C011 | `flatten_try` | Complexity | Informational | 2 |

`kind` is `plan.kind`, the value consumers filter on; `plan.title` is a longer
per-plan sentence. The values come from each rule's `RuleMetadata` in
`crates/complexipy-core/src/rules/complexity.rs`. What the tests pin:
`effectiveness_matches_documented_tiers` in `rules/registry/tests.rs` pins every
effectiveness value, and `every_registered_rule_produces_a_plan_consistent_with_its_own_metadata`
pins that a plan's `rule_id`, `kind`, category and applicability are the
metadata's rather than a second literal - but not what the metadata says.
`tests/test_refactor_plans.py` pins the applicability of C004, C005, C007 and C011
against real fixtures; the C001, C002 and C003 values, and every category, rest on
the metadata literals alone.

- **C001** - deeply nested conditions that can be inverted into early returns.
- **C002** - use `continue` guards at the top of a loop to reduce nesting. Refuses
  to emit a machine suggestion when statements follow the guarded chain inside the
  loop body, because re-emitting them below the inserted guards would change when
  they run.
- **C003** - extract a complex block into a helper function.
- **C004** - split a long `elif` chain into separate handlers.
- **C005** - extract a complex boolean condition into a named predicate. Its gate
  counts *runs* of one operator, so `a and b and c` is a single run and never
  fires it, while `a and b or c` does
  (`test_extract_predicate_needs_two_operator_runs`).
- **C007** - merge nested `if` statements into one with a combined condition.
- **C011** - flatten nested `try`/`except` by combining or restructuring.

`Applicability` is `MachineApplicable` when the rule can produce a replacement the
tool stands behind, and `Informational` when it can only explain. A rule that is
not confident emits `help` text rather than a wrong `suggestion`, and never prints
a complexity number it knows is fabricated.

**`plan.applicability` is a declared ceiling, not a promise.** It always comes
from the rule's metadata, and no rule overrides it per-plan. C002, C005 and C007
all declare `MachineApplicable` but fall back to help-only text in several
documented cases, each pinned in `tests/test_refactor_plans.py`: a multiline
string in the shifted body (`test_collapsible_if_skips_multiline_string_body`,
`test_loop_guard_skips_multiline_string_between_members`), an `elif` condition
(`test_predicate_elif_emits_help_only`), a condition that cannot be extracted
(`test_predicate_walrus_emits_help_only`), and statements following the guarded
chain (`test_loop_guard_trailing_statement_emits_help_only`). The console
renderer prints the plan's applicability in the header and the suggestion's in
the body (`output/refactor.rs`, source read), so a plan can display as safe to
apply directly above a `Help:` block with nothing to apply. **Consumers should
test `suggestion is not None` first and read `suggestion.applicability`.**

Plans are computed for every function the Python API returns, whatever its
score: `code_complexity_shared` always asks for plans and `build_refactor_plans`
in `refactor_plans.rs` passes the score through to the registry without gating on
it. The CLI's `--suggest-refactors` only controls whether they are rendered and
exported, for every listed function rather than only failing ones (source read of
`output/render.rs` `output_file_entries` and `output_json_shared`).

Each plan's `estimated_reduction` assumes that suggestion is applied alone. They
do not sum.

## How plans are selected

`RuleRegistry::analyze()` collects plans over the region tree, then, in order:

1. Drops any plan with `estimated_reduction < 1` as noise.
1. Sorts by spliceable first, then `effectiveness`, then reduction, then line - so
   a machine-applicable replacement beats a help-only plan of higher
   effectiveness.
1. Resolves overlapping line ranges, keeping the higher
   spliceable/effectiveness/reduction plan.
1. Caps the result at five plans per function.

In `rules/registry/tests.rs`,
`select_non_overlapping_never_returns_overlapping_plans` pins step 3 together
with the effectiveness and reduction tie-breaks,
`spliceable_plan_beats_help_only_plan_of_higher_effectiveness` pins the first
sort key, and `select_non_overlapping_caps_at_five_and_reports_the_rest` pins
step 4; the noise drop rests on a source read of `RuleRegistry::analyze`. An
overlap between two plans of equal spliceability, effectiveness and reduction
keeps the one selected first, which after the sort is the earlier line.

`effectiveness` in `RuleMetadata` is the single source of truth for ranking; the
registry reads it through `effectiveness_by_rule_id()`, so there is no
`match rule_id` anywhere.

`additional_refactor_plans` counts what the cap dropped plus plans removed when
measurement put their reduction below one. It excludes earlier noise and overlap
rejections, so it is not a count of all candidate plans. (Source read of
`analyze`; the stub's docstring mentions only the cap.)

## Measured versus estimated reduction

`reduction_is_measured` distinguishes the two. When a plan is spliceable, the
registry applies the replacement to a copy of the source, re-scores it, and
reports the real difference. When it is not, the reduction is the rule's estimate.
A spliceable plan whose spliced source fails to parse, or whose containing
function cannot be found afterwards, keeps the estimate with
`reduction_is_measured` false rather than reporting a fabricated measurement
(`unparseable_splice_measures_to_none`). A consumer that ranks by reduction
should read this flag.

## Adding a rule

Write the struct and `impl RefactorRule` in
`crates/complexipy-core/src/rules/complexity.rs`, set its `effectiveness` tier,
register it in `RuleRegistry::register_defaults()`, and document it here. Three
gates in `crates/complexipy-core/src/rules/registry/tests.rs` are hardcoded and
will fail otherwise: `fixture_for()` panics on an unknown rule id, the fixture
count is a literal, and `effectiveness_matches_documented_tiers` compares its
expected table against the registry's size before checking each tier.

Rules consume regions. They never re-parse source to *find structure* - though
some do parse expressions to validate a candidate replacement.
