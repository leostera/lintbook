# Test fixtures for PY023 (F706) - return-outside-function

# Bad: Return statement outside function (should trigger PY023)
# Module level return - invalid
if condition:
    return value  # Return at module level

try:
    result = calculate()
    return result  # Return in module-level try block
except Exception:
    return error  # Return in module-level except block

# Bad: Return in class definition (not in method)
class MyClass:
    if debug:
        return "debug info"  # Return in class body
    
    value = get_default()
    if not value:
        return None  # Return in class attribute initialization

# Bad: Return in comprehensions at module level
# Note: This would be a syntax error, but we test for completeness

# Good: Return in function (correct usage)
def my_function():
    if condition:
        return "early return"  # Correct: return inside function
    return "normal return"

def fibonacci(n):
    if n <= 1:
        return n  # Correct: return inside function
    return fibonacci(n-1) + fibonacci(n-2)

def process_data(data):
    if not data:
        return None  # Correct: return inside function
    return process(data)

# Good: Return in method (methods are functions)
class DataProcessor:
    def get_data(self):
        return self.data  # Correct: return inside method
    
    def process_item(self, item):
        if not item.is_valid():
            return None  # Correct: return inside method
        return self.process(item)

# Good: Return in nested function
def outer_function():
    def inner_function():
        return "inner result"  # Correct: return inside nested function
    return inner_function()

# Good: Return in lambda
result = lambda x: x * 2  # Correct: implicit return in lambda
explicit_lambda = lambda x: (return x * 2)  # This would be syntax error

# Good: Return with various expressions
def complex_function():
    return  # Correct: bare return
    return 42  # Correct: return with value
    return compute()  # Correct: return with computation
    return x, y, z  # Correct: return tuple
    return {"key": "value"}  # Correct: return dict
    return [1, 2, 3]  # Correct: return list

# Good: Return in different control structures within function
def control_flow_function():
    if condition:
        return "conditional"  # Correct: return in if inside function
    
    try:
        risky_operation()
        return "success"  # Correct: return in try inside function
    except Exception as e:
        return f"error: {e}"  # Correct: return in except inside function
    finally:
        pass  # Note: return in finally is unusual but valid
    
    with context_manager() as ctx:
        if ctx.should_return():
            return ctx.value  # Correct: return in with inside function
    
    for item in items:
        if item.is_target():
            return item  # Correct: return in for inside function
    
    while condition:
        if ready:
            return result()  # Correct: return in while inside function

# Good: Return in async function
async def async_function():
    await some_operation()
    return "async result"  # Correct: return inside async function

# Good: Multiple returns in function
def multi_return_function():
    if case1:
        return 1  # Correct
    elif case2:
        return 2  # Correct
    else:
        return 3  # Correct

# Good: Return in class method with various access modifiers
class ProcessorClass:
    def public_method(self):
        return "public"  # Correct
    
    def _protected_method(self):
        return "protected"  # Correct
    
    def __private_method(self):
        return "private"  # Correct
    
    @staticmethod
    def static_method():
        return "static"  # Correct
    
    @classmethod
    def class_method(cls):
        return "class"  # Correct
    
    @property
    def property_method(self):
        return self._value  # Correct

# Good: Return in nested class and method
class OuterClass:
    def outer_method(self):
        class InnerClass:
            def inner_method(self):
                return "inner"  # Correct: return in nested class method
        return InnerClass().inner_method()

# Good: Return in decorator
def my_decorator(func):
    def wrapper(*args, **kwargs):
        result = func(*args, **kwargs)
        return result  # Correct: return inside wrapper function
    return wrapper

# Good: Return in generator (though unusual, it's valid)
def generator_with_return():
    yield 1
    yield 2
    return "done"  # Correct: return inside generator function

# Good: Return in recursive function
def factorial(n):
    if n <= 1:
        return 1  # Correct: base case return
    return n * factorial(n - 1)  # Correct: recursive return

# Good: Return in function with exception handling
def safe_operation():
    try:
        result = risky_operation()
        return result  # Correct
    except SpecificError:
        return default_value()  # Correct
    except Exception:
        return None  # Correct
    finally:
        cleanup()
        # return in finally is valid but unusual

# Module-level control structures with invalid returns
for item in module_items:
    if item.done():
        return item  # Invalid: return at module level

while module_condition:
    if should_exit:
        return "exit"  # Invalid: return at module level

with module_context() as ctx:
    if ctx.error:
        return None  # Invalid: return at module level