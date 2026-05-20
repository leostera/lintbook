# Test fixtures for PY030 (E0604) - invalid-all-object

# Bad: Invalid objects in __all__ (should trigger PY030)
__all__ = [
    "ValidName",
    123,           # Wrong: number instead of string
    valid_var,     # Wrong: variable reference instead of string
    None,          # Wrong: None instead of string
]

# Bad: Mixed valid and invalid entries
__all__ = [
    "good_function",
    "GoodClass", 
    42,            # Wrong: number
    True,          # Wrong: boolean
    "another_good_one",
]

# Bad: Single invalid entry
__all__ = [some_variable]  # Wrong: variable instead of string

# Bad: Complex expressions
__all__ = [
    "valid_string",
    f"formatted_{var}",    # Wrong: f-string
    "prefix" + "suffix",   # Wrong: string concatenation
    func(),                # Wrong: function call
]

# Bad: Tuple instead of list (different issue but also invalid)
__all__ = (
    "item1",
    "item2",
    42,            # Wrong: number in tuple
)

# Bad: Invalid types in various contexts
__all__ = [
    "ValidExport",
    {},            # Wrong: dictionary
    [],            # Wrong: list
    set(),         # Wrong: set
    len,           # Wrong: builtin function reference
]

# Good: Valid __all__ declarations
__all__ = [
    "public_function",
    "PublicClass",
    "PUBLIC_CONSTANT",
]

__all__ = ["single_export"]

__all__ = [
    "export_one",
    "export_two", 
    "export_three",
]

# Good: Empty __all__
__all__ = []

# Good: Module-level variables that are strings
valid_export = "some_function"
__all__ = ["valid_export"]  # This is correct - string literal

# Good: Proper naming patterns
__all__ = [
    "snake_case_function",
    "CamelCaseClass", 
    "UPPERCASE_CONSTANT",
    "_protected_but_exported",
]

# Bad: Mixed with conditionals
if condition:
    __all__ = [
        "conditional_export",
        variable_name,  # Wrong: variable reference
    ]

# Bad: Dynamic construction
__all__ = []
__all__.append("valid_string")
__all__.append(invalid_var)  # Wrong: variable reference

# Bad: List comprehension with invalid elements
__all__ = [name for name in [
    "valid1",
    "valid2", 
    123,        # Wrong: number in source list
]]

# Good: Using string variables correctly (but not recommended)
export_name = "my_function"
__all__ = ["my_function"]  # Correct: string literal, not variable

# Bad: Using actual variable names instead of strings
def my_function():
    pass

class MyClass:
    pass

MY_CONSTANT = 42

# Wrong way: using variables instead of strings
__all__ = [
    my_function,   # Wrong: should be "my_function"
    MyClass,       # Wrong: should be "MyClass" 
    MY_CONSTANT,   # Wrong: should be "MY_CONSTANT"
]

# Correct way:
__all__ = [
    "my_function",
    "MyClass",
    "MY_CONSTANT",
]

# Bad: Complex nested structures
__all__ = [
    "valid_name",
    ["nested", "list"],  # Wrong: nested list
    ("tuple", "values"), # Wrong: tuple
]

# Bad: Using built-in types
__all__ = [
    "custom_function",
    str,           # Wrong: built-in type
    int,           # Wrong: built-in type
    list,          # Wrong: built-in type
]

# Bad: Using imported names directly
import os
from sys import path

__all__ = [
    "local_function",
    os,            # Wrong: imported module
    path,          # Wrong: imported name
]

# Good: Exporting imported names as strings
__all__ = [
    "local_function", 
    "os",          # Correct: if re-exporting os
    "path",        # Correct: if re-exporting path
]

# Edge case: __all__ reassignment
__all__ = ["initial"]
__all__ = [
    "reassigned",
    invalid_ref,   # Wrong: variable reference
]

# Edge case: Multiple __all__ definitions
__all__ = ["first_definition"]

# Later in file:
__all__ = [
    "second_definition",
    42,            # Wrong: number
]

# Bad: Using expressions
def get_name():
    return "dynamic_name"

__all__ = [
    "static_name",
    get_name(),    # Wrong: function call
]

# Bad: Attribute access
class Config:
    export_name = "some_export"

__all__ = [
    "valid_export",
    Config.export_name,  # Wrong: attribute access
]