# Test fixtures for PY029 (E0241) - duplicate-bases

# Bad: Duplicate base classes (should trigger PY029)
class BadClass(BaseClass, BaseClass):  # Duplicate BaseClass
    pass

class AnotherBad(A, B, A):  # Duplicate A
    def method(self):
        pass

class MultipleDuplicates(A, B, C, A, B):  # A and B are duplicated
    pass

# Bad: Duplicate with different whitespace/formatting
class FormattingIssue(
    BaseClass,
    AnotherBase,
    BaseClass  # Duplicate BaseClass
):
    pass

# Bad: Mixed object and specific class duplicates
class MixedDuplicates(object, MyBase, object):  # Duplicate object
    pass

# Bad: Duplicate in complex inheritance
class ComplexInheritance(
    FirstBase,
    SecondBase,
    ThirdBase,
    FirstBase,  # Duplicate FirstBase
    FourthBase
):
    def __init__(self):
        super().__init__()

# Good: No duplicate base classes
class GoodClass(BaseClass):
    pass

class MultipleUnique(A, B, C):
    pass

class ComplexUnique(
    FirstBase,
    SecondBase,
    ThirdBase,
    FourthBase
):
    pass

# Good: No inheritance
class NoInheritance:
    pass

# Good: Single inheritance
class SingleInheritance(BaseClass):
    def method(self):
        return super().method()

# Good: Multiple inheritance without duplicates
class MultipleInheritance(Mixin1, Mixin2, BaseClass):
    def __init__(self):
        super().__init__()

# Good: Generic types (if syntax allows)
from typing import Generic, TypeVar

T = TypeVar('T')
U = TypeVar('U')

class GenericClass(Generic[T], BaseClass):
    pass

# Edge case: Identical names from different modules (should be flagged if same string)
import module1
import module2

class ModuleClasses(module1.Base, module2.Base):  # Different modules - OK
    pass

class SameModuleBase(module1.Base, module1.Base):  # Same module, same class - Bad
    pass

# Bad: Duplicate with attribute access
class AttributeAccess(package.module.Class, SomeOther, package.module.Class):
    pass

# Good: Similar names but not identical
class SimilarNames(BaseClass, BaseClass2):  # Different classes
    pass

class VersionedClasses(ClassV1, ClassV2):  # Different versions
    pass

# Bad: Nested class inheritance with duplicates
class Outer:
    class Inner(Base, Base):  # Duplicate in nested class
        pass

# Good: Nested class inheritance without duplicates
class OuterGood:
    class InnerGood(Base1, Base2):
        pass

# Bad: Multiple inheritance chains with duplicates
class ChainA(Base):
    pass

class ChainB(Base):
    pass

class ChainC(ChainA, ChainB, Base):  # Base is duplicate through inheritance chain
    # Note: This is more complex duplicate detection that might not be caught
    # by simple string matching
    pass

# Good: Proper multiple inheritance
class ProperMultiple(Serializable, Hashable, Comparable):
    def serialize(self):
        pass
    
    def __hash__(self):
        return hash(self.id)
    
    def __eq__(self, other):
        return self.id == other.id

# Bad: Abstract base class duplicates
from abc import ABC, abstractmethod

class AbstractDuplicate(ABC, SomeBase, ABC):  # ABC duplicated
    @abstractmethod
    def abstract_method(self):
        pass

# Good: Abstract base class without duplicates
class AbstractGood(ABC, SomeBase):
    @abstractmethod
    def abstract_method(self):
        pass

# Edge case: Class with same name as variable
Base = object

class VariableNameConflict(Base, object):  # Base refers to the variable 'object'
    pass

# Bad: Exception class with duplicate bases
class CustomException(Exception, ValueError, Exception):  # Exception duplicated
    pass

# Good: Exception class without duplicates
class GoodCustomException(ValueError):
    pass

# Bad: Metaclass with duplicates (if syntax allows)
class MetaClass(type):
    pass

class WithMetaclass(BaseClass, metaclass=MetaClass):  # This is fine
    pass

# Edge case: Forward references (strings)
class ForwardRef('Future', BaseClass, 'Future'):  # String duplicates
    pass

# Good: No duplicates in complex scenario
class ComplexGood(
    SerializableMixin,
    CacheableMixin,
    TimestampedMixin,
    BaseModel
):
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.timestamp = time.now()

# Bad: Simple case for easy testing
class Simple(A, A):
    pass

# Bad: Three-way duplicate
class Triple(X, Y, X, Z, Y):
    pass