from complexipy import code_complexity

result = code_complexity(
    "def f(a, b):\n    if a:\n        if b:\n            pass\n"
)
plan = result.functions[0].refactor_plans[0]

print(plan.rule_id)
print(plan.doc_url)
print(plan.references)
