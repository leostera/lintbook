# Test fixtures for PY031 (E0605) - invalid-all-format

# Bad: __all__ inside function
def my_function():
    __all__ = ["should_not_be_here"]  # Wrong: __all__ inside function

# Bad: __all__ inside class
class MyClass:
    __all__ = ["class_level_all"]  # Wrong: __all__ inside class

# Bad: __all__ inside conditional
if some_condition:
    __all__ = ["conditional_all"]  # Wrong: __all__ in conditional block

# Bad: __all__ inside loop
for item in items:
    __all__ = ["loop_all"]  # Wrong: __all__ in loop

# Bad: __all__ inside try/except
try:
    __all__ = ["try_block_all"]  # Wrong: __all__ in try block
except Exception:
    __all__ = ["except_block_all"]  # Wrong: __all__ in except block

# Bad: __all__ inside with statement
with open("file.txt") as f:
    __all__ = ["with_block_all"]  # Wrong: __all__ in with block

# Bad: Multiple __all__ assignments (reassignment pattern)
__all__ = ["first"]
if condition:
    __all__ = ["second"]  # Wrong: conditional reassignment

# Bad: Dynamic modification using append
__all__ = ["initial"]
__all__.append("dynamic")  # Wrong: dynamic modification

# Bad: Dynamic modification using extend
__all__ = ["base"]
__all__.extend(["extended1", "extended2"])  # Wrong: dynamic extension

# Bad: Dynamic modification using += operator
__all__ = ["start"]
__all__ += ["added"]  # Wrong: augmented assignment

# Bad: __all__ using complex assignment operators
__all__ = ["base"]
__all__ *= 2  # Wrong: multiplication assignment

# Bad: __all__ in nested function
def outer():
    def inner():
        __all__ = ["nested_function"]  # Wrong: __all__ in nested function
    return inner

# Bad: __all__ in lambda (if possible)
lambda_func = lambda: setattr(sys.modules[__name__], '__all__', ["lambda_all"])  # Wrong: __all__ in lambda

# Bad: __all__ in list comprehension context
results = [__all__ for __all__ in [["wrong"]]]  # Wrong: __all__ as iteration variable

# Bad: __all__ in generator expression
gen = (__all__ for __all__ in [["generator"]])  # Wrong: __all__ as iteration variable

# Bad: __all__ as function parameter
def bad_function(__all__):  # Wrong: __all__ as parameter name
    pass

# Bad: __all__ as class attribute assignment within method
class BadClass:
    def __init__(self):
        self.__all__ = ["instance_all"]  # Wrong: __all__ as instance attribute
    
    def method(self):
        __all__ = ["method_all"]  # Wrong: __all__ in method

# Bad: __all__ in finally block
try:
    pass
finally:
    __all__ = ["finally_all"]  # Wrong: __all__ in finally block

# Bad: Global __all__ but inside nested scope
def wrapper():
    global __all__
    __all__ = ["global_in_function"]  # Wrong: global __all__ in function

# Bad: Nonlocal __all__ usage
def outer_func():
    __all__ = ["outer"]
    
    def inner_func():
        nonlocal __all__
        __all__ = ["inner"]  # Wrong: nonlocal __all__

# Bad: __all__ assignment to something other than list literal at module level
import sys
__all__ = sys.modules[__name__].__dict__.keys()  # Wrong: complex expression

# Bad: __all__ using walrus operator
if (__all__ := ["walrus"]):  # Wrong: __all__ with walrus operator
    pass

# Bad: __all__ in async context
async def async_function():
    __all__ = ["async_all"]  # Wrong: __all__ in async function

# Bad: __all__ in decorator function
def decorator(func):
    __all__ = ["decorator_all"]  # Wrong: __all__ in decorator
    return func

# Bad: __all__ assignment using slice
some_list = ["a", "b", "c"]
__all__ = some_list[:]  # Wrong: slice assignment

# Bad: __all__ using dict comprehension result (converted to list)
__all__ = list({"key": "value" for _ in range(1)}.keys())  # Wrong: complex expression

# Bad: __all__ using unpacking
values = ["item1", "item2"]
__all__ = [*values]  # Wrong: unpacking in __all__

# Bad: __all__ in match/case statement (Python 3.10+)
match value:
    case 1:
        __all__ = ["match_case"]  # Wrong: __all__ in match case

# Good: Proper module-level __all__ declarations
__all__ = [
    "public_function",
    "PublicClass",
    "PUBLIC_CONSTANT",
]

# Good: Simple module-level assignment
__all__ = ["single_export"]

# Good: Empty __all__ at module level
__all__ = []

# Good: Multiple simple assignments at module level (though not recommended)
__all__ = ["first_set"]
# Later in file (this should be detected as potential issue, but format is technically valid)
__all__ = ["second_set"]

def public_function():
    """This is a public function."""
    pass

class PublicClass:
    """This is a public class."""
    pass

PUBLIC_CONSTANT = "This is a public constant"

# Good: __all__ referencing the actual defined names (even though this should use strings)
# NOTE: This is caught by PY030 (invalid-all-object), not this lint
# __all__ = [public_function, PublicClass, PUBLIC_CONSTANT]  # This would be PY030 violation

# Bad: __all__ inside async context manager
async def async_context():
    async with some_async_context():
        __all__ = ["async_context_all"]  # Wrong: __all__ in async context

# Bad: __all__ assignment in exception handler specific cases
def error_handler():
    try:
        risky_operation()
    except ValueError:
        __all__ = ["value_error_all"]  # Wrong: __all__ in specific exception handler
    except Exception as e:
        __all__ = ["general_error_all"]  # Wrong: __all__ in general exception handler