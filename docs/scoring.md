# Scoring contract

Cognitive complexity after G. Ann Campbell's SonarSource white paper (v1.7).
The authoritative statement of this contract is `tests/main.py`:
`TestPaperConformance` pins the rules the paper prescribes, and
`TestScorerContract` pins the rules this scorer adds for Python constructs the
paper does not cover. This page describes those assertions; where the two
disagree, the tests win and this page is wrong. Every claim below names the test
that pins it, or says that it rests on a source read of
`crates/complexipy-core/src/cognitive_complexity.rs` and `utils.rs` alone.

## The increment model

A structural increment is computed as

```
own = 1 + nesting_level + boolean
```

- **structural** - a fixed +1 for the construct itself.
- **nesting** - the current nesting level, so the same construct costs more the
  deeper it sits.
- **boolean** - the cost of logical operator sequences in the construct's own
  condition, counted per *run* of one operator rather than per operator. So
  `if a and b` and `if a and b and c` both cost 2, while `if a and b or c` costs 3
  (`test_boolean_runs_are_counted_per_operator_sequence`).

Two exceptions to the formula:

- **`elif` and `else` clauses of an `if` take no nesting increment.** They cost 1
  (plus boolean for an `elif` condition) at any depth, because the `if` already
  paid the nesting increment - while their bodies still sit one level deeper.
  `for` wrapping `if`/`elif`/`else` scores 5, not 7
  (`test_elif_and_else_take_no_nesting_increment`).
- **`except` handlers and `match` have no boolean term.** The `Stmt::Try` and
  `Stmt::Match` arms charge `1 + nesting_level` and never walk a subject or
  guard, so `match a and b:` and `case 1 if a and b:` score nothing for the run
  (source read; see [Expression walker limits](#expression-walker-limits)).

A construct that increments does not automatically raise the nesting level for
what it contains, and a construct that raises nesting does not automatically
increment. The two are independent, which is where most of the surprises live.

Two increments never scale with nesting, however deep they sit: **boolean runs**
and **recursion**. `lambda x: x and x` scores 1 even though the lambda has already
raised the nesting level (`test_lambda_raises_nesting`), and a recursive call
inside an `if` scores 1, not 2 (`test_recursion_in_nested_control_flow`). When
this page says a construct "raises nesting", it means for the nesting-scaled
increments inside it - not for these two.

## What increments

| Construct | Structural | Raises nesting for its body | Pinned by |
| -- | -- | -- | -- |
| `if` | yes | yes | `test_if_else_increments_and_nests` |
| each `elif` clause | yes, no nesting term | yes | `test_elif_is_a_sibling_clause_not_a_nested_if`, `test_elif_and_else_take_no_nesting_increment` |
| `else` of an `if` | yes, no nesting term | yes | `test_if_else_increments_and_nests`, `test_elif_and_else_take_no_nesting_increment` |
| `for` | yes | yes | `test_match_nested_gets_nesting_increment` |
| `while` | yes | yes | `test_while_increments_and_nests` |
| `except` handler | yes, no boolean term | yes | `test_except_handler_gets_nesting_increment`, `test_except_handler_body_is_nested` |
| `match` - once for the whole statement including all `case` arms | yes, no boolean term | yes, for every `case` body | `test_match_top_level_structural_increment`, `test_match_case_body_is_nested` |
| Direct self-recursion | yes | no | `test_direct_recursion_increments` |
| Ternary `a if c else b` | yes | yes | `test_nested_ternary_gets_nesting_increment` |
| `lambda` | no | yes | `test_lambda_raises_nesting` |
| Nested `def` | no | yes | `test_nested_function_raises_nesting` |
| `try` | no | no | `test_try_body_is_not_nested` |
| Comprehension `for` clause | yes | yes, for the element expression and the contents of each filter; the `iter` expression and every further `for` clause stay at the comprehension's own level | `test_comprehension_loop_and_filter`, `test_comprehension_element_nests_but_generators_do_not`, `test_comprehension_filter_contents_are_nested` |
| Comprehension `if` filter | yes, never nesting-scaled | its contents are already one level deeper (previous row) | `test_comprehension_filter_is_never_nesting_scaled` |
| A run of `and` / `or` | yes | no | `test_boolean_runs_are_counted_per_operator_sequence` |

## What does not increment

These are the cases most often gotten wrong, each pinned by a named test unless
marked otherwise:

- **`with` neither increments nor nests.** `with open(x):` wrapping `if y:` scores
  **1**, not 2 (`test_with_does_not_nest`, `test_with_is_transparent_inside_a_loop`).
  Boolean runs in its context expressions still count: `with (a and b):` scores 1
  (`test_with_and_assert_count_only_boolean_runs`).
- **A `try` body does not nest.** An `if` directly inside `try:` is charged at
  nesting 0, so `try` + inner `if` + one `except` scores **2**
  (`test_try_body_is_not_nested`).
- **`finally` and a `try`'s `else` do not nest.** A `try` whose `finally` contains
  an `if` scores **1** (`test_finally_is_not_nested`); `try`/`except`/`else` with
  an `if` in the `else` scores **2** (`test_try_else_is_not_nested`).
- **A loop `else` neither increments nor nests.** This is the opposite of an
  `if`'s `else`, which does both - the asymmetry is real and easy to misread.
  `for ... else:` containing an `if` scores 2, where the same shape under
  `if ... else:` scores 4 (`test_loop_else_is_not_nested`,
  `test_if_else_increments_and_nests`).
- **`elif` and `else:` + `if` are not equivalent.** Python's AST represents `elif`
  as `orelse=[If(...)]`, the same shape as a nested `if` inside an `else`, but the
  scorer treats the `elif` chain as sibling clauses: `if/elif` scores 2 while
  `if/else:` wrapping an `if` scores 4
  (`test_elif_is_a_sibling_clause_not_a_nested_if`).
- **`try` itself never increments.** Only its `except` handlers do
  (`test_finally_is_not_nested`).
- **`raise`, `assert`, `break` and `continue` cost nothing.** `raise` and `assert`
  contribute only the boolean cost of their own expressions
  (`test_raise_does_not_increment`, `test_raise_counts_boolean_runs_in_its_expression`,
  `test_with_and_assert_count_only_boolean_runs`); `break` and `continue` are not
  scored at all (`test_break_and_continue_do_not_increment`). The paper charges +1
  for a break or continue *to a label*, which Python does not have.
- **A closure calling its enclosing function is not recursion.** It is a separate
  scope, so `def foo()` containing `def bar()` that calls `foo()` scores **0**
  (`test_nested_function_calling_outer_is_not_recursion`).
- **Recursion detection sees bare-name calls only.** `self.m()` inside method `m`
  is not recursion, and neither is a self-call inside a `lambda`. Source read of
  `RecursionFinder`; not pinned, and recorded as an open scoring question.
- **Statements in a class body other than methods are not scored.**
  `class A:` containing `if x: pass` scores 0 where the same `if` at module level
  scores 1, and a class nested inside a class body is dropped with its methods.
  Source read; not pinned, and recorded as an open scoring question.
- **A suppressed function is not scored at all.** `# complexipy: ignore` or
  `# noqa: complexipy` on the `def` line, on the line above the definition's first
  line (the `def`, or the first decorator), on a decorator line, or inside a
  multi-line signature before the first line containing a colon removes the
  function from the result (`test_ignore_marker_placements_that_suppress`,
  `test_noqa_complexipy_ignore`). See [CLI](cli.md#inline-ignores) for the
  placements that do not work.

## Expression walker limits

Boolean runs are found by a separate walker, `count_bool_ops` in `utils.rs`,
that descends through comparisons, a call's positional arguments, tuples, lists,
sets, dict values, ternaries, lambdas and comprehensions - and nothing else.
`not` is handled specially: it passes through a directly nested `and`/`or` or
another `not`, but not through anything else, so `not g(a and b)` and
`not (1 if a else 2)` score nothing for what they wrap - the second even loses
the ternary's structural increment. A run inside arithmetic (`(a and b) + 1`), a
subscript (`d[a and b]`), an attribute chain, a keyword or starred argument
(`g(k=a and b)`), an f-string, a walrus target (`if (n := a and b):`), a `match`
subject, or a `case` guard contributes nothing. The paper charges every operator
sequence. This is current behavior, recorded as a defect in the realignment
catalog, and it is deliberately not pinned by a test.

## Worked examples

Each is adapted from a named conformance test; the expression block merges
several one-line tests into one listing, so it is not runnable as written.

`match` is a single structural increment for the whole statement
(`test_match_top_level_structural_increment`):

```python
def f(x):
    match x:            # +1
        case 1:
            return 'one'
        case _:
            return 'other'
# total: 1
```

Nesting compounds, and `match` takes the nesting increment like anything else
(`test_match_nested_gets_nesting_increment`):

```python
def f(xs, x):
    for i in xs:        # +1
        match x:        # +1 structural, +1 nesting
            case 1:
                pass
# total: 3
```

An `except` handler nested in a loop (`test_except_handler_gets_nesting_increment`):

```python
def f(xs):
    for i in xs:        # +1
        try:
            pass
        except Exception:   # +1 structural, +1 nesting
            pass
# total: 3
```

Recursion increments once and never pays nesting
(`test_direct_recursion_increments`, `test_recursion_in_nested_control_flow`):

```python
def fact(n):
    return fact(n - 1)  # +1
# total: 1

def fact(n):
    if n:               # +1
        return fact(n - 1)  # +1
    return 1
# total: 2
```

Expressions are scored by their own walker, which is why lambdas, comprehensions
and ternaries behave the way they do (`test_lambda_raises_nesting`,
`test_comprehension_loop_and_filter`, `test_nested_ternary_gets_nesting_increment`,
`test_bare_expression_boolean_sequence`):

```python
g = lambda x: (1 if x else 2)   # lambda raises nesting; ternary +1 +1 -> 2
g = lambda x: x and x           # one boolean run -> 1

def f(xs):
    return [x for x in xs if x > 0]          # for +1, filter +1 -> 2
    return [y for x in xs for y in x if y]   # two fors + filter -> 3
    return any(x and x for x in xs)          # for +1, boolean +1 -> 2

x = 1 if a else (2 if b else 3)  # outer +1, inner +1 +1 nesting -> 3

def f(a, b):
    foo(a and b)                 # boolean run in a bare expression -> 1
```

## Function totals and module code

A file's complexity is the sum of its function complexities plus the complexity
of module-level statements, which is counted whether or not `check_script` is set
(`test_module_level_code_counts_without_check_script`,
`test_file_total_includes_module_level_code`). With `check_script`, module-level
code is also reported as a `<module>` entry (`TestScriptComplexity`).

Methods are named `Class::method` (`test_methods_are_named_class_method`). A
nested `def` is scored inside its enclosing function, one nesting level deeper,
and is not reported separately (`test_nested_function_raises_nesting`). A function
whose body is exactly an inner `def` followed by a `return` is treated as a
decorator and scored as its inner function at the outer nesting level
(`test_decorator_shaped_function_is_scored_as_its_inner_function`; the corpus
file `tests/src/test_decorator.py` pins the same rule through `TestFiles`).
