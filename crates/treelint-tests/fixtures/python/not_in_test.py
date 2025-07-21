# Test fixtures for PY007 (E713) - not-in-test

# Bad: using 'not x in y' (should trigger PY007)
if not x in items:
    pass

# Bad: using 'not (x in y)' (should trigger PY007)  
if not (value in collection):
    pass

# Bad: more complex case
if not item in valid_items:
    result = False

# Bad: with variables
if not key in dictionary:
    add_key(key)

# Good: using 'x not in y'
if x not in items:
    pass

# Good: using correct syntax
if value not in collection:
    pass

# Good: not with other operators (should not trigger)
if not x == y:
    pass

if not result > 5:
    pass

# Good: not with boolean values (should not trigger)
if not is_valid:
    pass

# Good: double negative cases (should not trigger this rule)
if not not_found:
    pass