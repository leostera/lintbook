# Test fixtures for PY009 (E721) - type-comparison

# Bad: using type() == comparison (should trigger PY009)
if type(obj) == str:
    pass

# Bad: using type() is comparison (should trigger PY009)
if type(value) is int:
    pass

# Bad: using type() != comparison (should trigger PY009)
if type(data) != list:
    pass

# Bad: using type() is not comparison (should trigger PY009)
if type(result) is not dict:
    pass

# Bad: reversed comparison (should trigger PY009)
if str == type(text):
    pass

# Bad: with complex expressions (should trigger PY009)
if type(obj.attribute) == tuple:
    pass

if type(results[0]) is dict:
    pass

# Bad: multiple type checks (should trigger PY009)
if type(x) == int and type(y) == str:
    pass

# Good: using isinstance() (should not trigger)
if isinstance(obj, str):
    pass

if isinstance(value, int):
    pass

if isinstance(data, (list, tuple)):
    pass

# Good: not isinstance() (should not trigger)
if not isinstance(obj, str):
    pass

# Good: other comparisons (should not trigger)
if obj == "hello":
    pass

if value > 5:
    pass

if data is None:
    pass

# Good: non-type function calls (should not trigger)
if len(obj) == 5:
    pass

if str(value) == "test":
    pass