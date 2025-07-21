# Test fixtures for PY034 (F402) - import-shadowed-by-loop-var

# Good: Import not shadowed
import os
import sys

# Good: Loop variable doesn't shadow import
for item in items:
    print(item)

# Bad: Loop variable shadows import
import json
for json in data_sources:  # Wrong: 'json' shadows imported module
    process(json)

# Bad: Multiple loop variables, one shadows import
import time
for index, time in enumerate(timestamps):  # Wrong: 'time' shadows imported module
    print(index, time)

# Bad: Nested loop shadowing
import math
for outer in range(3):
    for math in calculations:  # Wrong: 'math' shadows imported module
        result = math * 2

# Bad: Loop variable shadows aliased import
import datetime as dt
for dt in date_list:  # Wrong: 'dt' shadows aliased import
    print(dt)

# Bad: Comprehension variable shadows import
import random
numbers = [random for random in range(10)]  # Wrong: 'random' shadows imported module

# Bad: Generator expression variable shadows import
import string
chars = (string for string in alphabet)  # Wrong: 'string' shadows imported module

# Bad: Set comprehension variable shadows import
import collections
unique_items = {collections for collections in item_list}  # Wrong: 'collections' shadows imported module

# Bad: Dict comprehension variable shadows import
import itertools
mapping = {itertools: i for i, itertools in enumerate(data)}  # Wrong: 'itertools' shadows imported module

# Bad: From import shadowed by loop variable
from os import path
for path in directory_paths:  # Wrong: 'path' shadows imported name
    check_path(path)

# Bad: Multiple from imports, one shadowed
from sys import argv, exit
for exit in exit_codes:  # Wrong: 'exit' shadows imported function
    handle_exit(exit)

# Bad: Aliased from import shadowed
from json import loads as json_loads
for json_loads in file_loaders:  # Wrong: 'json_loads' shadows imported function
    json_loads()

# Good: Using imports correctly before loop
import logging
logging.info("Starting process")

for log_entry in log_entries:
    logging.debug(f"Processing: {log_entry}")

# Bad: Loop variable shadows import used after loop
import pickle
data = [1, 2, 3]
for pickle in data:  # Wrong: 'pickle' shadows imported module
    print(pickle)
# Now pickle module is no longer accessible
# serialized = pickle.dumps(data)  # This would fail

# Bad: Multiple shadowing in same loop
import uuid, hashlib
for uuid, hashlib in zip(uuid_list, hash_list):  # Wrong: both shadow imported modules
    process(uuid, hashlib)

# Good: Using different variable names
import base64
import binascii

for encoded_data in base64_strings:
    decoded = base64.b64decode(encoded_data)
    
for hex_data in hex_strings:
    binary = binascii.unhexlify(hex_data)

# Bad: Shadowing in while loop (less common but possible)
import socket
while socket in socket_list:  # Wrong: 'socket' shadows imported module
    handle_socket(socket)
    socket_list.remove(socket)

# Bad: Unpacking in loop shadows import
import calendar
for year, calendar in year_calendar_pairs:  # Wrong: 'calendar' shadows imported module
    print_calendar(year, calendar)

# Good: Import used in loop body without shadowing
import re
for pattern in regex_patterns:
    if re.match(pattern, text):
        print(f"Pattern {pattern} matches")

# Bad: Complex unpacking shadows import
import struct
for (header, struct, footer) in data_triplets:  # Wrong: 'struct' shadows imported module
    process_data(header, struct, footer)

# Bad: Nested function with shadowing loop
import operator
def process_operations():
    for operator in operation_list:  # Wrong: 'operator' shadows imported module
        apply_operation(operator)

# Bad: Class method with shadowing loop
import copy
class DataProcessor:
    def process(self):
        for copy in data_copies:  # Wrong: 'copy' shadows imported module
            self.handle_copy(copy)

# Bad: Lambda with shadowing (if possible syntactically)
import functools
# This might not be valid Python syntax, but conceptually:
# process = lambda functools: functools.reduce(add, [1, 2, 3])

# Good: Import used in loop condition
import os
for filename in os.listdir("."):
    if filename.endswith(".py"):
        process_file(filename)

# Bad: Loop in function shadows module-level import
import shutil

def cleanup_files():
    for shutil in temp_files:  # Wrong: 'shutil' shadows module-level import
        remove_temp(shutil)

# Good: Different scope, no conflict
def process_items():
    import subprocess  # Local import
    for item in items:
        subprocess.run(["process", item])

# Bad: Shadowing with star import (complex case)
from math import *
for sin in trigonometric_functions:  # Wrong: 'sin' shadows imported function
    calculate(sin)

# Bad: Multiple imports, selective shadowing
import ast, dis, inspect
for ast, node in ast_nodes:  # Wrong: 'ast' shadows imported module
    analyze_node(ast, node)
# 'dis' and 'inspect' remain accessible

# Good: Import inside loop (local scope)
for data_file in data_files:
    import csv  # Good: local import, no shadowing issue
    with open(data_file) as f:
        reader = csv.reader(f)

# Bad: Exception handling variable shadows import
import traceback
try:
    risky_operation()
except Exception as traceback:  # Wrong: 'traceback' shadows imported module
    print(f"Error: {traceback}")

# Bad: With statement variable shadows import
import tempfile
with open("data.txt") as tempfile:  # Wrong: 'tempfile' shadows imported module
    content = tempfile.read()

# Good: Proper variable naming
import pathlib
for file_path in pathlib.Path(".").iterdir():
    if file_path.is_file():
        process_file(file_path)

# Bad: List comprehension with multiple variables
import warnings
pairs = [(warnings, msg) for warnings, msg in warning_list]  # Wrong: 'warnings' shadows imported module

# Edge case: Shadowing in async for loop
import asyncio
async def async_processor():
    for asyncio in async_tasks:  # Wrong: 'asyncio' shadows imported module
        await process_async(asyncio)

# Edge case: Walrus operator in comprehension
import statistics
# This might shadow if used as loop variable in comprehension
# data = [result for item in items if (statistics := calculate(item)) > 0]

# Good: Imports used properly with descriptive loop variables
import json
import yaml

for config_file in configuration_files:
    if config_file.endswith('.json'):
        with open(config_file) as f:
            config = json.load(f)
    elif config_file.endswith('.yaml'):
        with open(config_file) as f:
            config = yaml.safe_load(f)

# Helper functions referenced in examples
def process(data): pass
def process_file(filename): pass
def check_path(path): pass
def handle_exit(code): pass
def apply_operation(op): pass
def handle_copy(data): pass
def remove_temp(file): pass
def analyze_node(ast, node): pass
def risky_operation(): pass
def process_async(task): pass
def calculate(item): pass