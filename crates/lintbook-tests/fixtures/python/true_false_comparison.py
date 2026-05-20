# Test fixtures for PY006 (E712) - true-false-comparison

# Bad: using == with True (should trigger PY006)
if x == True:
    pass

# Bad: using != with True (should trigger PY006)
if result != True:
    pass

# Bad: using == with False (should trigger PY006)
if value == False:
    pass

# Bad: using != with False (should trigger PY006)
if condition != False:
    pass

# Bad: True on left side (should trigger PY006)
if True == flag:
    pass

# Bad: False on left side (should trigger PY006)
if False != status:
    pass

# Good: using truthiness checks
if x:
    pass

if not result:
    pass

if value:
    pass

if not condition:
    pass

# Good: non-boolean comparisons (should not trigger)
if x == 5:
    pass

if name != "hello":
    pass

if result == "True":
    pass