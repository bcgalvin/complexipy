---
name: add-refactor-rule
description: Add or change a complexipy refactor rule while keeping metadata, registration, ranking tests, fixtures and rule documentation consistent.
---

# Change a refactor rule

Read `AGENTS.md`'s refactor-rule and testing sections, `docs/rules.md`, and the
neighboring rule implementation before editing. Keep this a rule change, not a
registry redesign. Rules consume scorer regions; do not change scoring totals
to make a suggestion look effective.

- Implement the struct and `RefactorRule` in
  `crates/complexipy-core/src/rules/complexity.rs`. Read the actual trait in
  `rules/types.rs` for its inputs, including `LineIndex` and definition names.
  Set identity, category, applicability and effectiveness in metadata; construct
  plans with `..self.metadata().new_plan()` rather than copying identity fields.
  The registry drops reductions below 1 before selection and after measurement.
  Set `spliceable` only for a faithful replacement: it takes ranking priority
  over effectiveness and enables the registry's re-parse/measurement attempt.
- Register a new rule in `RuleRegistry::register_defaults()` in
  `rules/registry.rs`. Ranking reads metadata, so do not add a separate
  rule-ID-to-effectiveness switch.
- Update the three explicit gates in `rules/registry/tests.rs`: the `fixture_for`
  arm; the checked-rule count and message in
  `every_registered_rule_produces_a_plan_consistent_with_its_own_metadata`; and
  the expected table in `effectiveness_matches_documented_tiers`. Derive the
  count from the rules you actually registered, not the stale "9th rule" comment.
  Follow the no-explanatory-code-comments convention instead of copying it.
- Add a behavioral fixture under `tests/fixtures/refactor_plans/` and assertions
  in `tests/test_refactor_plans.py`. Include a case that should fire and a refusal
  case where a rewrite would be unsafe. Assert relevant identity, applicability,
  replacement and reduction behavior, not just that some plan exists. Keep these
  fixtures out of the `tests/src/` complexity corpus.
- Update `docs/rules.md` with the ID, `plan.kind`, category, applicability,
  effectiveness and behavior. Keep examples and coverage claims tied to the
  actual tests. If uncertain about a rewrite, emit help rather than an unsafe
  replacement or a fabricated reduction.

Run the focused registry and refactor-plan tests, then `verify`. Rebuild the
extension before any Python test after Rust edits. Record a code defect found
outside the rule change in `docs/realignment/design-issues-and-bugs.md` rather
than expanding the task. G finalization moves that catalog to
`docs/maintenance/design-issues-and-bugs.md`.
