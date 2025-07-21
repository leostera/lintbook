# Test fixtures for PY033 (F401) - unused-import

# Bad: Unused standard library imports
import os  # Not used
import sys  # Not used
import json  # Not used
import re  # Not used

# Bad: Unused third-party imports
import requests  # Not used
import numpy  # Not used
import pandas  # Not used

# Bad: Unused from imports
from datetime import datetime  # Not used
from collections import defaultdict  # Not used
from typing import List, Dict, Optional  # All not used

# Bad: Unused aliased imports
import os as operating_system  # Not used
import sys as system  # Not used
from json import loads as json_loads  # Not used

# Bad: Mixed used and unused imports
import logging  # Not used
import time  # Used below
from pathlib import Path  # Not used
from pathlib import PurePath  # Used below

# Good: Used imports
import math
from string import ascii_letters
from typing import Union

# Usage of good imports
result = math.sqrt(16)
alphabet = ascii_letters
my_var: Union[str, int] = "hello"

# Usage of time from mixed imports above
time.sleep(0.1)
path = PurePath("/some/path")

# Bad: Import inside function (unused)
def my_function():
    import socket  # Not used
    import threading  # Used in function
    thread = threading.Thread(target=lambda: None)
    return thread

# Bad: Import inside class (unused)
class MyClass:
    import warnings  # Not used
    
    def method(self):
        import tempfile  # Not used
        import uuid  # Used in method
        return str(uuid.uuid4())

# Bad: Conditional imports (unused)
if True:
    import pickle  # Not used
    import csv  # Used below

# Usage of csv
with open("file.csv") as f:
    csv_content = csv.reader(f)

# Bad: Import in try/except (unused)
try:
    import optional_module  # Not used
except ImportError:
    optional_module = None

# Bad: Star imports (generally problematic, but different issue)
from math import *  # This would be a different lint (star imports)

# Good: Import used in string formatting/type hints
from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from some_module import SomeType  # Used in type hints

# Good: Import used in exception handling
from requests import RequestException

def make_request():
    try:
        # requests would be imported separately and used here
        pass
    except RequestException:
        pass

# Bad: Multiple unused imports on same line
from os import path, environ, getcwd  # Only path is used below

current_path = path.dirname(__file__)

# Bad: Unused import with same name as local variable
import token  # Not used - shadowed by local variable
token = "my_token"

# Bad: Unused import in __all__ context
import base64  # Not used even though in __all__
import hashlib  # Used and in __all__

__all__ = ["hashlib", "base64", "my_function"]

# Usage of hashlib
digest = hashlib.md5(b"test").hexdigest()

# Good: Import used in decorator
from functools import wraps

def my_decorator(func):
    @wraps(func)
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

# Bad: Import used only in comment
import ast  # Used only in comment: ast.parse(code)

# Good: Import used in global variable
import platform
SYSTEM_NAME = platform.system()

# Bad: Import at different scopes (unused)
def scope_test():
    import random  # Not used in this function
    pass

class ScopeClass:
    import inspect  # Not used in this class
    pass

# Good: Import used in comprehension
import itertools
combinations = [x for x in itertools.combinations([1, 2, 3], 2)]

# Bad: Import for module that's re-imported differently later
import json as json1  # Not used
import json as json2  # Used below

data = json2.loads('{"key": "value"}')

# Good: Import used in except clause
import traceback

def error_handler():
    try:
        risky_operation()
    except Exception:
        traceback.print_exc()

# Bad: Unused relative imports
from . import sibling_module  # Not used (would be in package)
from ..parent import parent_module  # Not used (would be in package)

# Good: Import used in type annotation
from typing import Callable

def higher_order(func: Callable) -> int:
    return func()

# Bad: Unused import in multiple assignment context
import shutil, glob  # Neither used
import copy, deepcopy  # Only copy used below

backup = copy.copy({"key": "value"})

# Good: Import used in context manager
import contextlib

@contextlib.contextmanager
def my_context():
    yield

# Bad: Unused import that would be used in string
import keyword  # Not actually used, just mentioned in string
message = "The keyword module is useful"

# Good: Import actually used with getattr
import types
module_type = getattr(types, "ModuleType")

# Edge case: Import used only in __name__ check
import argparse  # Used conditionally

if __name__ == "__main__":
    parser = argparse.ArgumentParser()

# Bad: Future import that's unused
from __future__ import annotations  # Not used

# Good: Future import that's used (Python < 3.10 would need this)
from __future__ import unicode_literals
text = "This string benefits from unicode_literals"

# Bad: Import used only in unreachable code
import sys as system_module  # Not used - code below is unreachable
if False:
    system_module.exit(1)

# Good: Import used in lambda
import operator
sorter = lambda x: operator.itemgetter(0)(x)

# Bad: Aliased import where alias is unused but original name is used elsewhere
import os as filesystem  # Alias not used
import os  # This import is used

current_dir = os.getcwd()  # Uses the second import

# Good: Import used in generator expression
import string
chars = (c for c in string.ascii_lowercase if c.isalpha())

# Bad: Import in nested scope (unused)
def outer():
    def inner():
        import calendar  # Not used
        pass
    return inner

# Good: Import used in class attribute
import datetime
class TimestampedClass:
    created_at = datetime.datetime.now()

# Bad: Import where only module name is used in string
import email  # Not actually used as module
print("Send me an email message")  # Just string mention, not usage

# Edge case: Import used in eval/exec (hard to detect)
import builtins  # Used in eval below
result = eval("builtins.len([1, 2, 3])")

# Good: Import used in assert statement
import unittest
assert issubclass(unittest.TestCase, object)

# Bad: Import that conflicts with builtin
import type  # Not used, and shadows builtin type()

def risky_operation():
    """Placeholder for risky operations used in examples."""
    pass