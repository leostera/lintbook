# Test fixtures for PY025 (F901) - raise-not-implemented

# Bad: Raising NotImplemented (should trigger PY025)
def abstract_method():
    raise NotImplemented

class BaseClass:
    def virtual_method(self):
        raise NotImplemented
    
    def another_method(self):
        # Still wrong even with parentheses
        raise NotImplemented()

# Bad: In various contexts
def process_data(data_type):
    if data_type == "json":
        return process_json()
    elif data_type == "xml":
        return process_xml()
    else:
        raise NotImplemented

# Bad: With raise from
def convert(value):
    try:
        return int(value)
    except ValueError as e:
        raise NotImplemented from e

# Good: Using NotImplementedError (correct)
def proper_abstract_method():
    raise NotImplementedError()

class ProperBaseClass:
    def virtual_method(self):
        raise NotImplementedError("Subclasses must implement this method")
    
    def another_method(self):
        raise NotImplementedError()

# Good: With custom messages
def unimplemented_feature():
    raise NotImplementedError("This feature will be available in v2.0")

def platform_specific():
    raise NotImplementedError(f"Not supported on {platform.system()}")

# Good: Other exceptions
def validate(value):
    if not value:
        raise ValueError("Value cannot be empty")
    if value < 0:
        raise ValueError("Value must be positive")

# NotImplemented in other contexts (not a raise statement)
# These should NOT trigger PY025
def compare(other):
    if not isinstance(other, MyClass):
        return NotImplemented
    return self.value == other.value

# Returning NotImplemented (correct for comparison methods)
class MyNumber:
    def __add__(self, other):
        if not isinstance(other, MyNumber):
            return NotImplemented
        return MyNumber(self.value + other.value)
    
    def __eq__(self, other):
        if not isinstance(other, MyNumber):
            return NotImplemented
        return self.value == other.value

# Checking for NotImplemented
result = some_operation()
if result is NotImplemented:
    handle_not_implemented()

# Common mistake: Developer confuses NotImplemented with NotImplementedError
# NotImplemented is a singleton used in rich comparison methods
# NotImplementedError is the exception to raise for unimplemented functionality