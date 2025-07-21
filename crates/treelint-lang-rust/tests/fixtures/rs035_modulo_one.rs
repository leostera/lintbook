// Test cases for RS035: modulo_one

fn test_modulo_one_violations() {
    let x = 42;
    let y = -100;
    
    // These should trigger violations - modulo by 1
    let result1 = x % 1; // Violation: will always be 0
    let result2 = y % 1; // Violation: will always be 0
    let result3 = 42 % 1; // Violation: will always be 0
    let result4 = (-50) % 1; // Violation: will always be 0
    
    println!("Modulo one results: {}, {}, {}, {}", result1, result2, result3, result4);
}

fn test_modulo_minus_one_violations() {
    let x = 42;
    let y = -100;
    
    // These should trigger violations - modulo by -1 (dangerous)
    let result1 = x % -1; // Violation: can panic/overflow or result in 0
    let result2 = y % -1; // Violation: can panic/overflow or result in 0
    let result3 = 42 % -1; // Violation: can panic/overflow or result in 0
    let result4 = (-50) % -1; // Violation: can panic/overflow or result in 0
    
    println!("Modulo minus one results: {}, {}, {}, {}", result1, result2, result3, result4);
}

fn test_parenthesized_violations() {
    let x = 42;
    
    // These should trigger violations - parenthesized 1 and -1
    let result1 = x % (1); // Violation: parenthesized 1
    let result2 = x % (-1); // Violation: parenthesized -1
    
    println!("Parenthesized results: {}, {}", result1, result2);
}

fn test_const_one_violations() {
    const ONE: i32 = 1;
    const MINUS_ONE: i32 = -1;
    
    let x = 42;
    
    // These might trigger violations - const values (depends on implementation)
    let result1 = x % ONE; // Potentially violation if we can detect const value
    let result2 = x % MINUS_ONE; // Potentially violation if we can detect const value
    
    println!("Const results: {}, {}", result1, result2);
}

fn test_variable_expressions() {
    let x = 42;
    let one = 1;
    let minus_one = -1;
    
    // These might trigger violations - variables that happen to be 1 or -1
    let result1 = x % one; // Depends on if we can track variable values
    let result2 = x % minus_one; // Depends on if we can track variable values
    
    println!("Variable results: {}, {}", result1, result2);
}

fn test_valid_modulo_operations() {
    let x = 42;
    let y = -100;
    
    // These should NOT trigger violations - valid modulo operations
    let result1 = x % 2; // Valid: modulo by 2
    let result2 = x % 3; // Valid: modulo by 3
    let result3 = x % 10; // Valid: modulo by 10
    let result4 = y % 5; // Valid: modulo by 5
    let result5 = x % -2; // Valid: modulo by -2
    let result6 = x % -5; // Valid: modulo by -5
    let result7 = 100 % 7; // Valid: literal modulo
    
    println!("Valid modulo: {}, {}, {}, {}, {}, {}, {}", 
             result1, result2, result3, result4, result5, result6, result7);
}

fn test_different_types() {
    let x_u32: u32 = 42;
    let x_i64: i64 = -100;
    let x_usize: usize = 1000;
    
    // These should trigger violations - different integer types
    let result1 = x_u32 % 1; // Violation: u32 modulo 1
    let result2 = x_i64 % -1; // Violation: i64 modulo -1
    let result3 = x_usize % 1; // Violation: usize modulo 1
    
    println!("Different types: {}, {}, {}", result1, result2, result3);
}

fn test_complex_expressions() {
    let x = 42;
    let y = 10;
    
    // These should trigger violations - complex left-hand expressions
    let result1 = (x + y) % 1; // Violation: complex expression modulo 1
    let result2 = (x * 2) % -1; // Violation: complex expression modulo -1
    let result3 = get_value() % 1; // Violation: function call modulo 1
    
    println!("Complex expressions: {}, {}, {}", result1, result2, result3);
}

fn get_value() -> i32 {
    42
}

fn test_method_chain_context() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Test in method chain context
    let results: Vec<i32> = numbers.iter()
        .map(|&x| x % 1) // Violation: modulo 1 in map
        .collect();
    
    println!("Method chain: {:?}", results);
}

fn test_assignment_contexts() {
    let mut x = 42;
    
    // Test in assignment contexts
    x %= 1; // Violation: modulo-assign by 1
    
    let mut y = 100;
    y %= -1; // Violation: modulo-assign by -1
    
    println!("Assignment results: {}, {}", x, y);
}

fn test_conditional_contexts() {
    let x = 42;
    
    // Test in conditional contexts
    if x % 1 == 0 { // Violation: modulo 1 in condition (always true)
        println!("Always true condition");
    }
    
    if x % -1 == 0 { // Violation: modulo -1 in condition
        println!("Dangerous condition");
    }
}

fn test_match_contexts() {
    let x = 42;
    
    // Test in match contexts
    match x % 1 { // Violation: modulo 1 in match (always 0)
        0 => println!("Always matches"),
        _ => println!("Never reached"),
    }
}

fn test_return_contexts() {
    fn returns_modulo_one(x: i32) -> i32 {
        x % 1 // Violation: returning modulo 1
    }
    
    fn returns_modulo_minus_one(x: i32) -> i32 {
        x % -1 // Violation: returning modulo -1
    }
    
    println!("Returns: {}, {}", returns_modulo_one(42), returns_modulo_minus_one(42));
}

fn test_array_index_contexts() {
    let arr = [10, 20, 30, 40, 50];
    let x = 42;
    
    // Test in array indexing (though this would be problematic anyway)
    // Note: This might not compile due to type issues, but shows the pattern
    // let value = arr[x % 1]; // Would be violation: modulo 1 for indexing
    
    println!("Array: {:?}", arr);
}

fn test_loop_contexts() {
    // Test in loop contexts
    for i in 0..10 {
        let remainder = i % 1; // Violation: modulo 1 in loop
        println!("Loop remainder: {}", remainder);
    }
}

fn test_closure_contexts() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Test in closure contexts
    numbers.iter().for_each(|&x| {
        let remainder = x % 1; // Violation: modulo 1 in closure
        println!("Closure remainder: {}", remainder);
    });
}

fn test_struct_field_contexts() {
    struct Point {
        x: i32,
        y: i32,
    }
    
    let point = Point { x: 42, y: 24 };
    
    // Test with struct fields
    let result1 = point.x % 1; // Violation: field modulo 1
    let result2 = point.y % -1; // Violation: field modulo -1
    
    println!("Struct field results: {}, {}", result1, result2);
}

fn test_tuple_contexts() {
    let tuple = (42, 24);
    
    // Test with tuple elements
    let result1 = tuple.0 % 1; // Violation: tuple element modulo 1
    let result2 = tuple.1 % -1; // Violation: tuple element modulo -1
    
    println!("Tuple results: {}, {}", result1, result2);
}

fn test_overflow_prone_values() {
    // These are particularly dangerous with -1 modulo
    let min_i32 = i32::MIN;
    let min_i64 = i64::MIN;
    
    // These should trigger violations and are especially dangerous
    let result1 = min_i32 % -1; // Violation: can overflow/panic
    let result2 = min_i64 % -1; // Violation: can overflow/panic
    
    println!("Overflow prone: {}, {}", result1, result2);
}

fn test_literal_suffixes() {
    let x = 42;
    
    // Test with different literal suffixes
    let result1 = x % 1i32; // Violation: modulo 1 with i32 suffix
    let result2 = x % 1u32; // Violation: modulo 1 with u32 suffix
    let result3 = x % -1i64; // Violation: modulo -1 with i64 suffix
    
    println!("Literal suffixes: {}, {}, {}", result1, result2, result3);
}

fn test_unicode_and_formatting() {
    let x = 42;
    
    // These should still be caught even with different formatting
    let result1 = x%1; // Violation: no spaces around operator
    let result2 = x % 1 ; // Violation: extra space before semicolon
    
    println!("Formatting variations: {}, {}", result1, result2);
}

// Test with generic types
fn test_generic_types<T>(x: T) -> T 
where 
    T: std::ops::Rem<Output = T> + Copy,
{
    // This might be harder to detect without type information
    // but if we see literal 1, it should still trigger
    let one = 1;
    // x % one  // Would depend on type analysis
    x
}