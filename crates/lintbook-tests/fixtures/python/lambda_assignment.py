# Test fixtures for PY010 (E731) - lambda-assignment

# Bad: assigning lambda to variable (should trigger PY010)
increment = lambda x: x + 1
add = lambda x, y: x + y
multiply = lambda a, b: a * b

# Bad: lambda with default arguments
greet = lambda name="World": f"Hello, {name}!"

# Bad: lambda with complex expression
process = lambda data: data.strip().lower() if data else ""

# Bad: lambda in class
class Calculator:
    square = lambda self, x: x ** 2

# Bad: lambda with multiple statements (using tuple)
compute = lambda x, y: (x + y, x * y)

# Good: using def instead of lambda assignment
def increment_good(x):
    return x + 1

def add_good(x, y):
    return x + y

# Good: lambda used directly in expressions (not assigned)
numbers = [1, 2, 3, 4, 5]
squared = list(map(lambda x: x ** 2, numbers))
evens = list(filter(lambda x: x % 2 == 0, numbers))

# Good: lambda in sorted/sort key
people = [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
sorted_people = sorted(people, key=lambda p: p["age"])

# Good: lambda passed as argument
def apply_operation(func, value):
    return func(value)

result = apply_operation(lambda x: x * 2, 10)

# Good: lambda in comprehensions
squared_dict = {x: (lambda y: y ** 2)(x) for x in range(5)}

# Good: regular assignments
x = 10
name = "test"
calculation = add_good(5, 3)