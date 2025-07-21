# Test fixtures for PY021 (F702) - continue-outside-loop

# Bad: Continue statement outside loop (should trigger PY021)
def function_with_continue():
    if condition:
        continue  # Continue outside any loop

def another_function():
    try:
        process_data()
        continue  # Continue not in a loop
    except Exception:
        handle_error()

# Bad: Continue in function but not in loop
def process_items():
    for item in items:
        validate(item)
    
    if need_retry:
        continue  # Continue outside the loop

# Bad: Continue in nested function
def outer_function():
    def inner_function():
        if should_skip:
            continue  # Continue in inner function, not in loop
        return result
    return inner_function()

# Bad: Continue in class method
class DataProcessor:
    def process(self):
        if self.should_skip:
            continue  # Continue outside loop in method

# Bad: Continue at module level
if global_condition:
    continue  # Continue at module level

# Bad: Continue in try/except but not in loop
try:
    risky_operation()
    if should_skip:
        continue  # Continue in try block, not in loop
except Exception:
    if recoverable_error:
        continue  # Continue in except block, not in loop
finally:
    if cleanup_needed:
        continue  # Continue in finally block, not in loop

# Bad: Continue in with statement but not in loop
with open("file.txt") as f:
    content = f.read()
    if not content:
        continue  # Continue in with block, not in loop

# Good: Continue in for loop (correct usage)
for item in items:
    if should_skip(item):
        continue  # Correct: continue inside for loop
    process(item)

for i in range(10):
    if i % 2 == 0:
        continue  # Correct: continue inside for loop
    print(f"Odd: {i}")

# Good: Continue in while loop (correct usage)
while condition:
    if should_skip:
        continue  # Correct: continue inside while loop
    process_item()

count = 0
while count < 10:
    count += 1
    if count % 3 == 0:
        continue  # Correct: continue inside while loop
    print(count)

# Good: Continue in nested loops
for outer in outer_items:
    if outer.skip():
        continue  # Correct: continue inside outer loop
    for inner in inner_items:
        if inner.skip():
            continue  # Correct: continue inside inner loop
        process(outer, inner)

# Good: Continue in loop inside function
def process_data():
    for item in data:
        if item.is_invalid():
            continue  # Correct: continue inside loop within function
        process_valid_item(item)

# Good: Continue in loop inside try/except
try:
    for item in items:
        if item.causes_error():
            continue  # Correct: continue inside loop within try block
        process_item(item)
except Exception:
    for recovery_item in recovery_items:
        if not recovery_item.is_usable():
            continue  # Correct: continue inside loop within except block
        use_recovery_item(recovery_item)

# Good: Continue in loop inside with statement
with database_connection() as conn:
    for record in conn.get_records():
        if record.is_deleted():
            continue  # Correct: continue inside loop within with block
        process_record(record)

# Good: Continue in loop inside class method
class Processor:
    def run(self):
        for task in self.tasks:
            if task.is_skipped():
                continue  # Correct: continue inside loop within method
            self.execute_task(task)

# Edge case: Continue in nested function inside loop
def outer_with_loop():
    for item in items:
        def inner():
            # This continue would be an error - not directly in the loop
            if condition:
                pass  # We avoid continue here to prevent syntax error
        inner()
        if item.skip():
            continue  # This continue is correct - in the loop

# Edge case: Continue in comprehension (different context)
# This is actually a syntax error in Python, but our linter focuses on continue statements
result = [process(x) for x in items if not should_continue(x)]

# Multiple nested structures
def complex_function():
    if outer_condition:
        try:
            with resource_manager() as resource:
                if inner_condition:
                    continue  # Invalid: not in any loop
        except Exception:
            continue  # Invalid: not in any loop
    else:
        continue  # Invalid: not in any loop