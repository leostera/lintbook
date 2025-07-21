# Test fixtures for PY014 (F541) - f-string-missing-placeholders

# Bad: F-strings without placeholders (should trigger PY014)
simple_f = f"Hello, World!"
single_quotes = f'This has no variables'
uppercase_f = F"UPPERCASE F PREFIX"
raw_f_string = fr"C:\Users\name"
raw_f_string_alt = rf"C:\Program Files"

# Bad: Multiline f-strings without placeholders
multiline_f = f"""
This is a multiline string
without any placeholders
"""

triple_single = f'''
Another multiline
without variables
'''

# Bad: F-strings with escaped braces (not placeholders)
escaped_braces = f"Use {{double braces}} to show literal braces"
empty_braces = f"Empty braces: {{}}"

# Good: F-strings with placeholders (should not trigger)
name = "Alice"
age = 30
greeting = f"Hello, {name}!"
info = f'Name: {name}, Age: {age}'
calculation = f"2 + 2 = {2 + 2}"

# Good: F-strings with formatting
pi = 3.14159
formatted_pi = f"Pi: {pi:.2f}"
percentage = 0.85
formatted_pct = f"Score: {percentage:.1%}"

# Good: F-strings with expressions
items = [1, 2, 3]
count_str = f"Items: {len(items)}"
sum_str = f"Sum: {sum(items)}"

# Good: F-strings with method calls
text = "hello"
upper_str = f"Uppercase: {text.upper()}"
title_str = f"Title: {text.title()}"

# Good: F-strings with dictionary access
data = {"name": "Bob", "age": 25}
dict_access = f"Name: {data['name']}"
dict_get = f"Age: {data.get('age', 0)}"

# Good: Nested expressions
nested = f"Result: {max(min(10, 20), 5)}"
conditional = f"Status: {'Active' if True else 'Inactive'}"

# Good: Regular strings (should not trigger)
regular_double = "Just a regular string"
regular_single = 'Another regular string'
raw_string = r"Raw string \n stays literal"

# Good: Other string prefixes
bytes_string = b"Bytes string"
unicode_string = u"Unicode string"

# Good: Complex f-string expressions
import datetime
now = datetime.datetime.now()
timestamp = f"Current time: {now:%Y-%m-%d %H:%M:%S}"

# Good: F-strings with multiple placeholders
x, y = 10, 20
coords = f"Point: ({x}, {y})"
math_expr = f"{x} + {y} = {x + y}"

# Good: F-strings with f-expressions inside
value = 42
debug = f"value={value!r}"
string_repr = f"repr={value!s}"