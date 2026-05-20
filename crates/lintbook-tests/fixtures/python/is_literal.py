# Test fixtures for PY017 (F632) - is-literal

# Bad: Using 'is' with string literals (should trigger PY017)
if name is "admin":
    grant_access()

if user is not "guest":
    allow_modification()

# Bad: Using 'is' with numeric literals
if count is 0:
    print("Empty")

if value is 42:
    print("The answer")

if score is not 100:
    print("Not perfect")

if price is 99.99:
    apply_discount()

# Bad: Using 'is' with boolean literals
if result is True:
    continue_process()

if success is False:
    retry()

if enabled is not True:
    disable_feature()

# Bad: Using 'is' with collection literals
if items is []:
    initialize_defaults()

if config is {}:
    load_default_config()

if values is ():
    create_empty_tuple()

if unique_items is {1, 2, 3}:
    process_set()

# Bad: Complex cases
if response.status is 200:
    handle_success()

if error_code is not 404:
    log_error()

# Good: Using '==' with literals (correct)
if name == "admin":
    grant_access()

if count == 0:
    print("Empty")

if result != True:
    handle_failure()

if items == []:
    initialize_defaults()

# Good: Using 'is' with None (correct)
if value is None:
    set_default()

if result is not None:
    process_result()

# Good: Using 'is' for object identity (correct)
if obj1 is obj2:
    print("Same object")

if current_user is authenticated_user:
    allow_access()

# Good: Using 'is' with singleton constants
if direction is Ellipsis:
    handle_ellipsis()

# Note: Small integers (-5 to 256) and short strings might work with 'is'
# due to Python's interning, but this is implementation-dependent
# and should not be relied upon:

# This might work in CPython but is still wrong:
x = 5
if x is 5:  # Bad practice, even if it works
    pass

# This is unreliable:
s = "hello"
if s is "hello":  # May or may not work depending on interning
    pass