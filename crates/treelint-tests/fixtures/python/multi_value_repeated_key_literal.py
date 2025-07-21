# Test fixtures for PY015 (F601) - multi-value-repeated-key-literal

# Bad: Dictionary with duplicate string keys (should trigger PY015)
user_data = {
    "name": "Alice",
    "age": 30,
    "name": "Bob",  # This overwrites "Alice"
    "email": "bob@example.com"
}

# Bad: Dictionary with duplicate numeric keys
scores = {
    1: "first",
    2: "second", 
    1: "first_again",  # This overwrites "first"
    3: "third"
}

# Bad: Multiple duplicate keys
config = {
    "debug": True,
    "verbose": False,
    "timeout": 30,
    "debug": False,     # Overwrites True
    "port": 8080,
    "verbose": True,    # Overwrites False
    "host": "localhost"
}

# Bad: Mixed quotes but same key
settings = {
    'mode': 'production',
    "mode": 'development',  # Same key, overwrites
    'level': 5
}

# Bad: Dictionary comprehension result (literal dict)
literal_comp = {
    "a": 1,
    "b": 2,
    "a": 3  # Duplicate
}

# Good: No duplicate keys
person = {
    "first_name": "John",
    "last_name": "Doe",
    "age": 25,
    "city": "New York"
}

# Good: Similar but different keys
similar_keys = {
    "name": "value1",
    "Name": "value2",      # Different key (case sensitive)
    "NAME": "value3",      # Different key
    "_name": "value4",     # Different key
    "name_": "value5"      # Different key
}

# Good: Numeric vs string keys (different types)
mixed_types = {
    "1": "string key one",
    1: "numeric key one",   # Different from "1"
    2.0: "float key",
    "2.0": "string key two" # Different from 2.0
}

# Good: Nested dictionaries with same keys in different scopes
nested = {
    "user": {
        "id": 1,
        "name": "Alice"
    },
    "admin": {
        "id": 2,
        "name": "Bob"  # Not a duplicate (different dictionary)
    }
}

# Good: Using dict() constructor
dict_constructor = dict(
    name="Charlie",
    age=35,
    city="Boston"
)

# Good: Dictionary merge (not a literal)
base = {"a": 1, "b": 2}
extended = {**base, "c": 3}

# Bad: Complex case with many duplicates
complex_dict = {
    "key1": "value1",
    "key2": "value2",
    "key3": "value3",
    "key1": "updated1",  # Duplicate
    "key4": "value4",
    "key2": "updated2",  # Duplicate
    "key5": "value5",
    "key3": "updated3",  # Duplicate
}

# Note: In Python, True == 1 and False == 0 at runtime,
# but for linting purposes we treat them as different keys
bool_numeric = {
    True: "boolean true",
    1: "number one",      # Treated as different for linting
    False: "boolean false",
    0: "number zero"      # Treated as different for linting
}