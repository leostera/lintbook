# Test fixtures for PY005 (E711) - none-comparison

# Bad: using == with None (should trigger PY005)
if x == None:
    pass

# Bad: using != with None (should trigger PY005)
if None != result:
    pass

# Good: using is with None
if x is None:
    pass

# Good: using is not with None
if result is not None:
    pass

# Good: non-None comparisons (should not trigger)
if x == 5:
    pass

if name != "hello":
    pass