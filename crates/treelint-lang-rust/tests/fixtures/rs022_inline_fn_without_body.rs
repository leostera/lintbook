// Test cases for RS022: inline_fn_without_body

// These should trigger violations - inline on trait methods without bodies
trait MyTrait {
    #[inline]
    fn method_without_body(&self) -> i32; // Violation: inline has no effect
    
    #[inline(always)]
    fn another_method(&self, value: String); // Violation: inline has no effect
    
    // This should NOT trigger - has default implementation
    #[inline]
    fn method_with_body(&self) -> i32 {
        42
    }
    
    // This should NOT trigger - no inline attribute
    fn normal_trait_method(&self);
}

// These should trigger violations - inline on extern functions
extern "C" {
    #[inline]
    fn external_function(x: i32) -> i32; // Violation: inline has no effect on extern
    
    #[inline(always)]
    fn another_extern_fn(); // Violation: inline has no effect on extern
    
    // This should NOT trigger - no inline attribute
    fn normal_extern_fn(data: *const u8);
}

// These should NOT trigger violations - functions with bodies
impl MyTrait for i32 {
    #[inline]
    fn method_without_body(&self) -> i32 {
        *self
    }
    
    #[inline(always)]
    fn another_method(&self, _value: String) {
        // Implementation
    }
    
    #[inline]
    fn method_with_body(&self) -> i32 {
        *self + 1
    }
}

// Regular functions with inline - should NOT trigger
#[inline]
fn regular_function_with_body() -> i32 {
    100
}

#[inline(always)]
fn another_regular_function(x: i32, y: i32) -> i32 {
    x + y
}

// Function declarations in other contexts
struct MyStruct;

impl MyStruct {
    #[inline]
    fn associated_function() -> Self {
        MyStruct
    }
    
    #[inline(always)]
    fn method(&self) -> i32 {
        42
    }
}

// Abstract trait with inline attributes - should trigger violations
trait AbstractTrait {
    #[inline]
    fn abstract_method1(&self) -> bool; // Violation
    
    #[inline(never)]  // Different inline variant
    fn abstract_method2(&self, data: &str) -> String; // Violation
    
    #[inline]
    fn concrete_method(&self) -> i32 {
        // This should NOT trigger - has body
        10
    }
}

// More extern examples with different syntaxes
extern {
    #[inline]
    fn c_function() -> i32; // Violation
}

extern "system" {
    #[inline(always)]
    fn system_call(param: u32) -> u32; // Violation
}

// Edge cases
trait ComplexTrait {
    // Multiple attributes including inline
    #[cfg(feature = "test")]
    #[inline]
    #[must_use]
    fn complex_method(&self) -> Option<i32>; // Violation: inline has no effect
    
    // Inline with generic
    #[inline]
    fn generic_method<T>(&self, value: T) -> T; // Violation
    
    // Inline with where clause
    #[inline]
    fn where_clause_method<T>(&self) -> T 
    where 
        T: Default; // Violation
}

// Functions with various inline attributes that should NOT trigger
mod valid_inline_usage {
    #[inline]
    pub fn public_function() -> &'static str {
        "hello"
    }
    
    #[inline(always)]
    fn private_function(x: f64) -> f64 {
        x * 2.0
    }
    
    #[inline(never)]
    fn never_inline() {
        println!("This function should never be inlined");
    }
}