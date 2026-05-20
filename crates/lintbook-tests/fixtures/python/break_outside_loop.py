# Test fixtures for PY020 (F701) - break-outside-loop

# Bad: Break statement outside loop (should trigger PY020)
def function_with_break():
    if condition:
        break  # Break outside any loop

def another_function():
    try:
        process_data()
        break  # Break not in a loop
    except Exception:
        handle_error()

# Bad: Break in function but not in loop
def process_items():
    for item in items:
        validate(item)
    
    if failed:
        break  # Break outside the loop

# Bad: Break in nested function
def outer_function():
    def inner_function():
        if error:
            break  # Break in inner function, not in loop
        return result
    return inner_function()

# Bad: Break in class method
class DataProcessor:
    def process(self):
        if self.should_stop:
            break  # Break outside loop in method

# Bad: Break at module level
if global_condition:
    break  # Break at module level

# Bad: Break in try/except but not in loop
try:
    risky_operation()
    if should_exit:
        break  # Break in try block, not in loop
except Exception:
    if critical_error:
        break  # Break in except block, not in loop
finally:
    if cleanup_failed:
        break  # Break in finally block, not in loop

# Bad: Break in with statement but not in loop
with open("file.txt") as f:
    content = f.read()
    if not content:
        break  # Break in with block, not in loop

# Good: Break in for loop (correct usage)
for item in items:
    if condition:
        break  # Correct: break inside for loop

for i in range(10):
    if i == 5:
        break  # Correct: break inside for loop

# Good: Break in while loop (correct usage)
while condition:
    if should_exit:
        break  # Correct: break inside while loop

count = 0
while True:
    count += 1
    if count > 10:
        break  # Correct: break inside while loop

# Good: Break in nested loops
for outer in outer_items:
    for inner in inner_items:
        if inner.matches(outer):
            break  # Correct: break inside inner loop
    if outer.is_complete():
        break  # Correct: break inside outer loop

# Good: Break in loop inside function
def process_data():
    for item in data:
        if item.is_invalid():
            break  # Correct: break inside loop within function

# Good: Break in loop inside try/except
try:
    for item in items:
        if item.causes_error():
            break  # Correct: break inside loop within try block
except Exception:
    for recovery_item in recovery_items:
        if recovery_item.works():
            break  # Correct: break inside loop within except block

# Good: Break in loop inside with statement
with database_connection() as conn:
    for record in conn.get_records():
        if record.is_target():
            break  # Correct: break inside loop within with block

# Good: Break in loop inside class method
class Processor:
    def run(self):
        for task in self.tasks:
            if task.is_done():
                break  # Correct: break inside loop within method

# Edge case: Break in comprehension (different context)
# This is actually a syntax error in Python, but our linter focuses on break statements
result = [x for x in items if not break_condition(x)]

# Edge case: Break in nested function inside loop
def outer_with_loop():
    for item in items:
        def inner():
            # This break would be an error - not directly in the loop
            if condition:
                pass  # We avoid break here to prevent syntax error
        inner()
        if item.done():
            break  # This break is correct - in the loop