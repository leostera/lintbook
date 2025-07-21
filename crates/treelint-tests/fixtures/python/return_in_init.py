# Test fixtures for PY026 (E0101) - return-in-init

# Bad: Return statement in __init__ (should trigger PY026)
class BadClass:
    def __init__(self):
        self.value = 42
        return self.value  # Wrong: __init__ should not return a value

class AnotherBadClass:
    def __init__(self, name):
        if not name:
            return None  # Wrong: __init__ should not return anything
        self.name = name

class ConditionalReturn:
    def __init__(self, data):
        if data is None:
            return  # Wrong: even bare return is not allowed in __init__
        self.data = data

class EarlyReturn:
    def __init__(self, config):
        if not self.validate_config(config):
            return False  # Wrong: __init__ cannot return values
        self.config = config

# Bad: Return in __init__ with complex logic
class ComplexInit:
    def __init__(self, items):
        processed = []
        for item in items:
            if item.is_valid():
                processed.append(item)
            else:
                return None  # Wrong: cannot return from __init__
        self.items = processed

# Bad: Multiple returns in __init__
class MultipleReturns:
    def __init__(self, value):
        if value < 0:
            return -1  # Wrong
        elif value == 0:
            return 0   # Wrong
        else:
            self.value = value
            return 1   # Wrong

# Good: Proper __init__ without returns
class GoodClass:
    def __init__(self):
        self.value = 42
        # No return statement - correct

class ProperInit:
    def __init__(self, name):
        if not name:
            raise ValueError("Name cannot be empty")  # Correct: raise exception
        self.name = name

class ConditionalInit:
    def __init__(self, data):
        if data is None:
            self.data = []  # Correct: set default instead of returning
        else:
            self.data = data

class ValidatedInit:
    def __init__(self, config):
        if not self.validate_config(config):
            raise ValueError("Invalid config")  # Correct: raise exception
        self.config = config
    
    def validate_config(self, config):
        return config is not None

# Good: Other methods with returns (not __init__)
class MethodsWithReturns:
    def __init__(self, value):
        self.value = value
        # No return - correct

    def get_value(self):
        return self.value  # Correct: regular method can return

    def process(self):
        if self.value < 0:
            return None  # Correct: regular method can return
        return self.value * 2

    def __str__(self):
        return f"Value: {self.value}"  # Correct: __str__ should return

    def __len__(self):
        return len(str(self.value))  # Correct: __len__ should return

# Good: Complex __init__ without returns
class ComplexGoodInit:
    def __init__(self, items):
        processed = []
        for item in items:
            if item.is_valid():
                processed.append(item)
            else:
                # Instead of returning, handle the error appropriately
                raise ValueError(f"Invalid item: {item}")
        self.items = processed

# Good: Exception handling in __init__
class InitWithExceptionHandling:
    def __init__(self, data):
        try:
            self.processed_data = self.process_data(data)
            # No return - correct
        except Exception as e:
            # Handle exception, but don't return
            self.processed_data = None
            self.error = str(e)
    
    def process_data(self, data):
        return data.upper() if data else ""

# Good: __init__ that calls other methods
class InitCallingMethods:
    def __init__(self, config):
        self.setup(config)
        self.initialize()
        # No return - correct
    
    def setup(self, config):
        self.config = config
        return True  # Correct: helper method can return
    
    def initialize(self):
        self.state = "initialized"

# Edge case: Nested class with __init__
class OuterClass:
    def __init__(self, outer_value):
        self.outer_value = outer_value
        # No return - correct
        
    class InnerClass:
        def __init__(self, inner_value):
            return inner_value  # Wrong: __init__ cannot return even in nested class

# Edge case: __init__ in inherited class
class BaseClass:
    def __init__(self, base_value):
        self.base_value = base_value
        # No return - correct

class DerivedClass(BaseClass):
    def __init__(self, base_value, derived_value):
        super().__init__(base_value)
        if derived_value is None:
            return  # Wrong: derived __init__ also cannot return
        self.derived_value = derived_value

# Good: Metaclass __init__ (different context)
class MetaClass(type):
    def __init__(cls, name, bases, attrs):
        super().__init__(name, bases, attrs)
        # No return - correct for metaclass __init__

# Edge case: Function named __init__ (not a class method)
def __init__(value):
    if value:
        return value  # This is fine - it's a regular function, not a class __init__
    return None

# Good: __new__ method (can return)
class WithNew:
    def __new__(cls, value):
        if value < 0:
            return None  # Correct: __new__ can return
        return super().__new__(cls)
    
    def __init__(self, value):
        self.value = value
        # No return - correct

# Bad: Return in __init__ with finally
class InitWithFinally:
    def __init__(self, resource):
        try:
            self.resource = self.acquire_resource(resource)
        except Exception:
            return None  # Wrong: cannot return from __init__
        finally:
            self.cleanup()
    
    def acquire_resource(self, resource):
        return resource
    
    def cleanup(self):
        pass