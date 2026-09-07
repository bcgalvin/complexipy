def tally(items, sink):
    total = 0
    for item in items:
        if item is not None:
            if sink(item):
                total += 1
            total += 2
    return total
