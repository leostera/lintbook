# Test fixtures for PY034 (F404) - late-future-import

# Good: __future__ imports at the beginning (after docstring and comments)
"""Module docstring."""

# Good: Comments before future imports are allowed
from __future__ import annotations
from __future__ import unicode_literals

# Good: Regular imports after future imports
import os
import sys

# Good: Module-level variables after imports
MODULE_CONSTANT = "value"

def function_definition():
    pass

# Bad: Future import after regular import
import json
from __future__ import division  # Wrong: future import after regular import

# Bad: Future import after function definition  
def another_function():
    pass

from __future__ import print_function  # Wrong: future import after function definition

# Bad: Future import after class definition
class MyClass:
    pass

from __future__ import absolute_import  # Wrong: future import after class definition

# Bad: Future import after variable assignment
variable = "some value"
from __future__ import with_statement  # Wrong: future import after variable assignment

# Bad: Future import inside function
def bad_function():
    from __future__ import generator_stop  # Wrong: future import inside function

# Bad: Future import inside class
class BadClass:
    from __future__ import barry_as_FLUFL  # Wrong: future import inside class

# Bad: Future import inside if statement
if True:
    from __future__ import nested_scopes  # Wrong: future import in conditional

# Bad: Future import inside try/except
try:
    from __future__ import generators  # Wrong: future import in try block
except ImportError:
    pass

# Bad: Future import after docstring but after other statements
x = 1
from __future__ import braces  # Wrong: future import after assignment

# Good example of proper order:
"""
This is what the beginning of a file should look like:
1. Module docstring
2. Comments  
3. __future__ imports
4. Regular imports
5. Module-level code
"""

# Bad: Future import in except clause
try:
    import some_module
except ImportError:
    from __future__ import CO_FUTURE_DIVISION  # Wrong: future import in except

# Bad: Future import in finally clause
try:
    pass
finally:
    from __future__ import unicode_literals  # Wrong: future import in finally

# Bad: Future import after decorator
@decorator
def decorated_function():
    pass

from __future__ import annotations  # Wrong: future import after decorator

# Bad: Future import in loop
for i in range(1):
    from __future__ import print_function  # Wrong: future import in loop

# Bad: Future import in while loop
while False:
    from __future__ import division  # Wrong: future import in while loop

# Bad: Future import in with statement
with open("file.txt") as f:
    from __future__ import absolute_import  # Wrong: future import in with statement

# Bad: Future import after assert statement
assert True
from __future__ import unicode_literals  # Wrong: future import after assert

# Bad: Future import after global statement
global global_var
from __future__ import division  # Wrong: future import after global

# Bad: Future import after nonlocal statement (in function context)
def outer():
    var = 1
    def inner():
        nonlocal var
        from __future__ import annotations  # Wrong: future import after nonlocal

# Bad: Future import in async function
async def async_function():
    from __future__ import annotations  # Wrong: future import in async function

# Bad: Future import in generator
def generator():
    yield 1
    from __future__ import print_function  # Wrong: future import in generator

# Bad: Future import after yield statement
def another_generator():
    yield 1

from __future__ import division  # Wrong: future import after function with yield

# Bad: Future import in lambda (would be syntax error anyway)
# lambda: from __future__ import annotations  # Syntax error

# Bad: Future import in comprehension (would be syntax error)
# [from __future__ import x for x in []]  # Syntax error

# Bad: Multiple future imports mixed with other code
from __future__ import annotations  # Good: at beginning
import os  # Regular import
from __future__ import division  # Wrong: future import after regular import

# Good: All future imports together at beginning
# This is how it should be done:
# from __future__ import annotations
# from __future__ import unicode_literals  
# from __future__ import division
# import os
# import sys
# ... rest of module

# Bad: Future import after exec/eval
exec("some code")
from __future__ import print_function  # Wrong: future import after exec

# Bad: Future import after del statement
del some_variable
from __future__ import absolute_import  # Wrong: future import after del

# Bad: Future import after return (in function)
def function_with_early_return():
    return "early"
    from __future__ import annotations  # Wrong: unreachable future import

# Bad: Future import after raise
def function_with_raise():
    raise ValueError("error")
    from __future__ import division  # Wrong: unreachable future import

# Bad: Future import after break (in loop)
for i in range(1):
    break
    from __future__ import print_function  # Wrong: unreachable future import

# Bad: Future import after continue (in loop)  
for i in range(1):
    continue
    from __future__ import unicode_literals  # Wrong: unreachable future import

# Good: Comments and blank lines are allowed before future imports
# This is a comment

# Another comment

from __future__ import annotations  # This would be OK if at file beginning

# Bad: Future import after pass statement
pass
from __future__ import division  # Wrong: future import after pass

# Edge case: Future import in nested class
class Outer:
    class Inner:
        from __future__ import annotations  # Wrong: future import in nested class

# Edge case: Future import in method
class MyClass:
    def method(self):
        from __future__ import print_function  # Wrong: future import in method

# Edge case: Future import in property
class MyClass:
    @property
    def prop(self):
        from __future__ import division  # Wrong: future import in property

# Edge case: Future import in class variable context
class MyClass:
    from __future__ import annotations  # Wrong: future import as class variable
    
# Edge case: Future import after import alias
import os as operating_system
from __future__ import unicode_literals  # Wrong: future import after aliased import

# Edge case: Future import after from import
from os import path
from __future__ import absolute_import  # Wrong: future import after from import

def decorator(func): 
    return func

global_var = None
some_variable = "test"