# Test fixtures for PY022 (F704) - yield-outside-function

# Bad: Yield statement outside function (should trigger PY022)
# Module level yield - invalid
if condition:
    yield value  # Yield at module level

try:
    result = calculate()
    yield result  # Yield in module-level try block
except Exception:
    yield error  # Yield in module-level except block

# Bad: Yield in class definition (not in method)
class MyClass:
    if debug:
        yield "debug info"  # Yield in class body
    
    value = yield get_value()  # Yield in class attribute

# Bad: Yield in comprehensions at module level
# Note: This would be a syntax error, but we test for completeness
data = [yield x for x in items]  # Yield in list comprehension

# Good: Yield in function (correct usage)
def generator_function():
    for i in range(10):
        yield i  # Correct: yield inside function

def fibonacci():
    a, b = 0, 1
    while True:
        yield a  # Correct: yield inside function
        a, b = b, a + b

def process_items(items):
    for item in items:
        if item.is_valid():
            yield item  # Correct: yield inside function

# Good: Yield in method (methods are functions)
class DataProcessor:
    def generate_data(self):
        for i in range(10):
            yield i * 2  # Correct: yield inside method
    
    def process_stream(self, stream):
        for item in stream:
            processed = self.process_item(item)
            yield processed  # Correct: yield inside method

# Good: Yield in nested function
def outer_function():
    def inner_generator():
        for i in range(5):
            yield i  # Correct: yield inside nested function
    return inner_generator()

# Good: Yield in lambda (though unusual)
generator_lambda = lambda: (yield x for x in range(5))

# Good: Yield with various expressions
def complex_generator():
    yield  # Correct: bare yield
    yield 42  # Correct: yield with value
    yield from range(10)  # Correct: yield from
    x = yield  # Correct: yield as expression
    value = yield compute()  # Correct: yield with computation

# Good: Yield in different control structures within function
def control_flow_generator():
    if condition:
        yield "conditional"  # Correct: yield in if inside function
    
    try:
        risky_operation()
        yield "success"  # Correct: yield in try inside function
    except Exception as e:
        yield f"error: {e}"  # Correct: yield in except inside function
    finally:
        yield "cleanup"  # Correct: yield in finally inside function
    
    with context_manager() as ctx:
        yield ctx.value  # Correct: yield in with inside function
    
    for item in items:
        yield item  # Correct: yield in for inside function
    
    while condition:
        yield next_value()  # Correct: yield in while inside function

# Good: Async generator (yield in async function)
async def async_generator():
    for i in range(10):
        yield i  # Correct: yield inside async function
        await asyncio.sleep(0.1)

# Edge case: Yield in generator expression (different context)
# Generator expressions create their own scope
gen = (yield x for x in items)  # This is complex - generator expression scope

# Edge case: Yield in function default argument
# This would be a syntax error in most cases
def func_with_generator_default(gen=None):
    if gen is None:
        def default_gen():
            yield 1  # Correct: yield inside nested function
        gen = default_gen()
    return gen

# Edge case: Yield in decorator
def generator_decorator(func):
    def wrapper(*args, **kwargs):
        yield "before"  # Correct: yield inside wrapper function
        result = func(*args, **kwargs)
        yield result
        yield "after"
    return wrapper

# Multiple yields in function
def multi_yield_function():
    yield 1  # Correct
    yield 2  # Correct
    if condition:
        yield 3  # Correct
    else:
        yield 4  # Correct

# Yield in class method with various access modifiers
class GeneratorClass:
    def public_generator(self):
        yield "public"  # Correct
    
    def _protected_generator(self):
        yield "protected"  # Correct
    
    def __private_generator(self):
        yield "private"  # Correct
    
    @staticmethod
    def static_generator():
        yield "static"  # Correct
    
    @classmethod
    def class_generator(cls):
        yield "class"  # Correct
    
    @property
    def generator_property(self):
        # This would be unusual but syntactically valid
        def inner():
            yield "property"  # Correct: yield inside nested function
        return inner()

# Yield from variations
def yield_from_examples():
    yield from range(10)  # Correct
    yield from other_generator()  # Correct
    yield from (x * 2 for x in range(5))  # Correct