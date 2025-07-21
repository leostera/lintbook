# Test fixtures for PY016 (F631) - assert-tuple

# Bad: Assert with non-empty tuple (should trigger PY016)
# These are always True because non-empty tuples are truthy
assert (1, 2)
assert (x > 0, y < 10)
assert ("error", msg)
assert (True, False)

# Bad: Single element tuple with trailing comma
assert (condition,)
assert (x > 0,)
assert (result is not None,)

# Bad: Complex tuple assertions
assert (validate(x), check(y), verify(z))
assert (a == 1, b == 2, c == 3)

# Good: Assert with parentheses (not a tuple)
assert (x > 0)
assert (x > 0 and y < 10)
assert (x or y)
assert (not error)

# Good: Assert with boolean expressions
assert x > 0 and y < 10
assert x or y or z
assert all([x, y, z])
assert any([x, y, z])

# Good: Assert without parentheses
assert condition
assert result
assert not failed

# Good: Assert with message (second argument)
assert x > 0, "x must be positive"
assert result, f"Expected result, got {result}"
assert data, "Data should not be empty"

# Good: Assert with complex expressions
assert isinstance(x, int)
assert len(items) > 0
assert response.status_code == 200

# Edge case: Empty tuple (always False, but different issue)
# This lint doesn't flag empty tuples
assert ()

# Common mistake this lint catches:
# Developer meant to write:
#   assert x > 0 and y < 10
# But wrote:
#   assert (x > 0, y < 10)
# The second form is always True if the tuple is non-empty

# Another common mistake:
# Trying to assert multiple conditions with commas
# Wrong:
assert (username, password, email)  # Always True if non-empty
# Right:
assert username and password and email