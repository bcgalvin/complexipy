# Scoring contract

Cognitive complexity after G. Ann Campbell's SonarSource white paper (v1.7).
The authoritative statement of this contract is
`tests/main.py::TestPaperConformance`, which asserts an exact score for each rule
below. This page describes those assertions; where the two disagree, the tests
win and this page is wrong.

## The increment model

Every structural increment is computed the same way
(`crates/complexipy-core/src/cognitive_complexity.rs`):

```
own = 1 + nesting_level + boolean
```

- **structural** - a fixed +1 for the construct itself.
- **nesting** - the current nesting level, so the same construct costs more the
  deeper it sits.
- **boolean** - the cost of logical operator sequences in the construct's own
  condition, counted per *run* of one operator rather than per operator. So
  `if a and b` and `if a and b and c` both cost 2, while `if a and b or c` costs 3.

A construct that increments does not automatically raise the nesting level for
what it contains, and a construct that raises nesting does not automatically
increment. The two are independent, which is where most of the surprises live.

Two increments never scale with nesting, however deep they sit: **boolean runs**
and **recursion**. `lambda x: x and x` scores 1 even though the lambda has already
raised the nesting level, and a recursive call inside an `if` scores 1, not 2.
When this page says a construct "raises nesting", it means for the nesting-scaled
increments inside it - not for these two.

## What increments

| Construct | Structural | Raises nesting for its body |
| -- | -- | -- |
| `if`, and each `elif` clause | yes | yes |
| `else` of an `if` | yes | yes |
| `for`, `while` | yes | yes |
| `except` handler | yes | yes |
| `match` - once for the whole statement including all `case` arms | yes | yes |
| Direct self-recursion | yes | no |
| Ternary `a if c else b` | yes | yes |
| `lambda` | no | yes |
| `try` | no | no |
| Comprehension `for` clause, and each `if` filter | yes | yes |
| A run of `and` / `or` | yes | no |

## What does not increment

These are the cases most often gotten wrong, each pinned by a named test:

- **`with` costs nothing and does not nest.** `with open(x):` wrapping `if y:`
  scores **1**, not 2 (`test_with_does_not_nest`).
- **A `try` body does not nest.** An `if` directly inside `try:` is charged at
  nesting 0, so `try` + inner `if` + one `except` scores **2**
  (`test_try_body_is_not_nested`).
- **`finally` does not nest.** A `try` whose `finally` contains an `if` scores
  **1** (`test_finally_is_not_nested`).
- **A loop `else` neither increments nor nests.** This is the opposite of an
  `if`'s `else`, which does both - the asymmetry is real and easy to misread.
  `for ... else:` containing an `if` scores 2, where the same shape under
  `if ... else:` scores 4 (`test_loop_else_is_not_nested`).
- **`elif` and `else:` + `if` are not equivalent.** Python's AST represents `elif`
  as `orelse=[If(...)]`, the same shape as a nested `if` inside an `else`, but the
  scorer treats the `elif` chain as sibling clauses: `if/elif` scores 2 while
  `if/else:` wrapping an `if` scores 4.
- **`try` itself never increments.** Only its `except` handlers do.
- **`raise`, `break` and `continue` cost nothing.** A `raise` contributes only the
  boolean cost of its own expressions; `break` and `continue` are not scored at
  all. The paper charges +1 for a break or continue *to a label*, which Python
  does not have.
- **A closure calling its enclosing function is not recursion.** It is a separate
  scope, so `def foo()` containing `def bar()` that calls `foo()` scores **0**
  (`test_nested_function_calling_outer_is_not_recursion`).

## Worked examples

Each is a conformance test, reproduced verbatim.

`match` is a single structural increment for the whole statement:

```python
def f(x):
    match x:            # +1
        case 1:
            return 'one'
        case _:
            return 'other'
# total: 1
```

Nesting compounds, and `match` takes the nesting increment like anything else:

```python
def f(xs, x):
    for i in xs:        # +1
        match x:        # +1 structural, +1 nesting
            case 1:
                pass
# total: 3
```

An `except` handler nested in a loop:

```python
def f(xs):
    for i in xs:        # +1
        try:
            pass
        except Exception:   # +1 structural, +1 nesting
            pass
# total: 3
```

Recursion increments once, and pays nesting like any other construct:

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

Expressions are scored by their own walker, which is why lambdas,
comprehensions and ternaries behave the way they do:

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

A file's complexity is the sum of its function complexities. Methods are named
`Class::method`. With `check_script` enabled, module-level code is reported as an
additional `<module>` entry, and module-level complexity is included in file and
code totals whether or not a `<module>` record appears.
