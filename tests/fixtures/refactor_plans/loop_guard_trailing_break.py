def drain(batch, lookup, sink):
    processed = 0
    for _ in range(batch):
        result = lookup()
        if result is None:
            for pending in sink.flush():
                processed += pending
            break
        processed += sink(result)
    return processed
