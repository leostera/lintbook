# Test fixtures for PY019 (F634) - if-tuple

# Bad: If with non-empty tuple (should trigger PY019)
# These are always True because non-empty tuples are truthy
if (1, 2):
    print("This always executes")

if (x > 0, y < 10):
    process_values()

if ("error", msg):
    handle_error()

if (True, False):
    print("Mixed booleans, still truthy")

# Bad: Single element tuple with trailing comma
if (condition,):
    execute()

if (x > 0,):
    handle_positive()

if (result is not None,):
    process_result()

# Bad: Complex tuple conditions
if (validate(x), check(y), verify(z)):
    all_checks_passed()  # Wrong! This doesn't check all conditions

if (a == 1, b == 2, c == 3):
    handle_values()  # Wrong! Only checks if tuple is non-empty

# Bad: Elif with tuple
if x < 0:
    handle_negative()
elif (x > 0, x < 10):
    handle_small_positive()
elif (x >= 10, x < 100):
    handle_medium()
else:
    handle_large()

# Good: If with parentheses (not a tuple)
if (x > 0):
    handle_positive()

if (x > 0 and y < 10):
    handle_range()

if (x or y):
    handle_either()

if (not error):
    continue_processing()

# Good: If with boolean expressions
if x > 0 and y < 10:
    handle_range()

if x or y or z:
    handle_any()

if all([x, y, z]):
    handle_all()

if any([x, y, z]):
    handle_some()

# Good: If without parentheses
if condition:
    execute()

if result:
    process()

if not failed:
    succeed()

# Good: Complex boolean expressions
if x > 0 and y < 10 and z != 0:
    process_all()

if validate(x) and check(y) and verify(z):
    all_valid()

# Edge case: Empty tuple (always False, but different issue)
# This lint doesn't flag empty tuples
if ():
    print("Never executes")

# Common mistake this lint catches:
# Developer meant to write:
#   if x > 0 and y < 10:
# But wrote:
#   if (x > 0, y < 10):
# The second form is always True if any element exists

# Another common mistake:
# Trying to check multiple conditions with commas
# Wrong:
if (username, password, email):  # Always True if any is truthy
    login()
# Right:
if username and password and email:
    login()

# Nested cases
if outer_condition:
    if (inner_x, inner_y):  # Still wrong in nested context
        nested_process()