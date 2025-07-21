# Test fixtures for PY008 (E714) - not-is-test

# Bad: using 'not x is y' (should trigger PY008)
if not x is None:
    pass

# Bad: using 'not (x is y)' (should trigger PY008)  
if not (value is None):
    pass

# Bad: more complex case
if not item is valid_item:
    result = False

# Bad: with variables
if not result is expected:
    handle_error()

# Bad: with objects
if not obj is singleton:
    create_new()

# Good: using 'x is not y'
if x is not None:
    pass

# Good: using correct syntax
if value is not None:
    pass

# Good: not with other operators (should not trigger)
if not x == y:
    pass

if not result > 5:
    pass

# Good: not with 'in' operator (should not trigger this rule)
if not x in items:
    pass

# Good: not with boolean values (should not trigger)
if not is_valid:
    pass

# Good: double negative cases (should not trigger this rule)
if not not_found:
    pass