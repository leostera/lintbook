# Test fixtures for PY027 (E0115) - nonlocal-and-global

# Bad: Variable declared both nonlocal and global (should trigger PY027)
def outer_function():
    x = 10
    
    def inner_function():
        global x      # Wrong: x cannot be both global and nonlocal
        nonlocal x    # Wrong: x cannot be both global and nonlocal
        x = 20

# Bad: Multiple variables with conflicts
def complex_scope():
    a = 1
    b = 2
    
    def nested():
        global a, b     # Wrong: a and b declared as global
        nonlocal a, c   # Wrong: a declared as both global and nonlocal
        # c is fine as only nonlocal
        a = 10
        b = 20
        c = 30

# Bad: Variables in different order
def different_order():
    value = 42
    
    def modify_value():
        nonlocal value  # First declaration
        global value    # Wrong: conflicts with nonlocal above
        value = 100

# Bad: Same variable declared multiple times
def multiple_declarations():
    counter = 0
    
    def increment():
        global counter    # First global declaration
        nonlocal counter  # Wrong: conflicts with global
        global counter    # Redundant but not the main issue
        counter += 1

# Good: Only global declaration
def only_global():
    def inner():
        global global_var
        global_var = 42

# Good: Only nonlocal declaration
def only_nonlocal():
    local_var = 10
    
    def inner():
        nonlocal local_var
        local_var = 20

# Good: Different variables as global and nonlocal
def different_variables():
    local_var = 10
    
    def inner():
        global global_var     # Different variable - OK
        nonlocal local_var    # Different variable - OK
        global_var = 42
        local_var = 20

# Good: Nested scopes with different variables
def nested_scopes():
    x = 1
    
    def level1():
        y = 2
        nonlocal x  # OK: x from outer scope
        
        def level2():
            global z      # OK: z is global
            nonlocal y    # OK: y from level1 scope
            # x is not declared here, using enclosing scope
            z = 3
            y = 4
            return x + y + z
        
        return level2()
    
    return level1()

# Good: Multiple variables in same declaration
def multiple_variables_same_type():
    a, b, c = 1, 2, 3
    
    def inner():
        nonlocal a, b, c  # All nonlocal - OK
        a = 10
        b = 20
        c = 30

def another_good_example():
    def inner():
        global x, y, z    # All global - OK
        x = 1
        y = 2
        z = 3

# Edge case: Variable declared in different functions (should be OK)
def separate_functions():
    value = 42
    
    def func1():
        global value   # This is OK - refers to global scope
        value = 100
    
    def func2():
        nonlocal value # This is OK - refers to enclosing scope
        value = 200
    
    # These are separate function scopes, so no conflict

# Bad: Complex nested case
def complex_nested():
    outer_var = 1
    
    def middle():
        middle_var = 2
        
        def inner():
            global outer_var     # Wrong: outer_var is nonlocal in this context
            nonlocal outer_var   # Wrong: conflicts with global above
            nonlocal middle_var  # This part is OK
            outer_var = 10
            middle_var = 20

# Good: Using variables without declaration (inherits from enclosing scope)
def no_declarations():
    x = 10
    
    def inner():
        # No global or nonlocal declarations
        print(x)  # This is fine, reads from enclosing scope
        # x = 20  # This would create a local variable

# Bad: Variable in class scope
class MyClass:
    class_var = 100
    
    def method(self):
        def inner_function():
            global class_var     # Wrong: class_var is not global
            nonlocal class_var   # Wrong: conflicts with global above
            class_var = 200

# Good: Proper use in class methods
class ProperClass:
    def __init__(self):
        self.instance_var = 10
    
    def method(self):
        local_var = 20
        
        def helper():
            nonlocal local_var   # OK: refers to method's local variable
            global global_var    # OK: refers to module-level variable
            local_var = 30
            global_var = 40

# Bad: Lambda with conflicting declarations (if possible)
def lambda_case():
    x = 10
    
    # Note: lambdas can't have global/nonlocal statements, but for completeness
    def wrapper():
        global x
        nonlocal x  # Wrong: conflicts with global
        
        # Lambda would be:
        # lambda: x  # This just reads x from enclosing scope

# Good: Module-level variables properly accessed
module_var = 100

def access_module_var():
    def inner():
        global module_var  # OK: module_var is indeed global
        module_var = 200

# Bad: Same line declarations (if syntax allows)
def same_line_issue():
    var = 42
    
    def conflicted():
        global var; nonlocal var  # Wrong: both on same line
        var = 100