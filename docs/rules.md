# Refactor rules

`--suggest-refactors` runs a Clippy-style rule set over the complexity regions
the scorer produces. Seven rules are registered. The ID space is not contiguous -
there is no C006, C008, C009 or C010.

## The catalog

| ID | Title | Category | Applicability | Effectiveness |
| -- | -- | -- | -- | -- |
| C001 | Flatten nested conditions | Complexity | Informational | 4 |
| C002 | Loop guards | Complexity | MachineApplicable | 3 |
| C003 | Extract helper function | Complexity | Informational | 2 |
| C004 | Split dispatcher | Complexity | Informational | 2 |
| C005 | Extract predicate | Readability | MachineApplicable | 2 |
| C007 | Collapsible if | Readability | MachineApplicable | 5 |
| C011 | Flatten try/except | Complexity | Informational | 2 |

- **C001** - deeply nested conditions that can be inverted into early returns.
- **C002** - use `continue` guards at the top of a loop to reduce nesting. Refuses
  to emit a machine suggestion when statements follow the guarded chain inside the
  loop body, because re-emitting them below the inserted guards would change when
  they run.
- **C003** - extract a complex block into a helper function.
- **C004** - split a long `elif` chain into separate handlers.
- **C005** - extract a complex boolean condition into a named predicate. Its gate
  counts *runs* of one operator, so `a and b and c` is a single run and never
  fires it.
- **C007** - merge nested `if` statements into one with a combined condition.
- **C011** - flatten nested `try`/`except` by combining or restructuring.

`Applicability` is `MachineApplicable` when the rule can produce a replacement the
tool stands behind, and `Informational` when it can only explain. A rule that is
not confident emits `help` text rather than a wrong `suggestion`, and never prints
a complexity number it knows is fabricated.

**`plan.applicability` is a declared ceiling, not a promise.** It always comes
from the rule's metadata, and no rule overrides it per-plan. C002, C005 and C007
all declare `MachineApplicable` but fall back to help-only text in several
documented cases - a multiline string in the shifted body, an `elif` condition, a
condition that cannot be extracted, statements following the guarded chain. The
console renderer prints the plan's applicability in the header and the
suggestion's in the body, so a plan can display as safe to apply directly above a
`Help:` block with nothing to apply. **Consumers should test
`suggestion is not None` first and read `suggestion.applicability`.**

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

`effectiveness` in `RuleMetadata` is the single source of truth for ranking; the
registry reads it through `effectiveness_by_rule_id()`, so there is no
`match rule_id` anywhere.

`additional_refactor_plans` counts what the cap dropped plus plans removed when
measurement put their reduction below one. It excludes earlier noise and overlap
rejections, so it is not a count of all candidate plans.

## Measured versus estimated reduction

`reduction_is_measured` distinguishes the two. When a plan is spliceable, the
registry applies the replacement to a copy of the source, re-scores it, and
reports the real difference. When it is not, the reduction is the rule's estimate.
A consumer that ranks by reduction should read this flag.

## Adding a rule

Write the struct and `impl RefactorRule` in
`crates/complexipy-core/src/rules/complexity.rs`, set its `effectiveness` tier,
register it in `RuleRegistry::register_defaults()`, and document it here. Three
gates in `crates/complexipy-core/src/rules/registry/tests.rs` are hardcoded and
will fail otherwise: `fixture_for()` panics on an unknown rule id, the fixture
count is a literal, and `effectiveness_matches_documented_tiers` carries an
expected table.

Rules consume regions. They never re-parse source to *find structure* - though
some do parse expressions to validate a candidate replacement.
