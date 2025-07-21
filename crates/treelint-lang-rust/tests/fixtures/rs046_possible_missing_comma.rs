// Test cases for RS046: possible_missing_comma

fn test_missing_comma_negative_numbers() {
    // These should trigger violations - negative numbers on separate lines
    let arr1 = [
        -1, -2, -3  // Missing comma after -3
        -4, -5, -6  // Violation: could be -3, -4 or -3 - 4
    ];
    
    let arr2 = [
        -10
        -20         // Violation: missing comma between -10 and -20
        -30
    ];
    
    let arr3 = [
        1, 2, 3
        -1, -2, -3  // Violation: missing comma after 3
    ];
    
    println!("Arrays: {:?}, {:?}, {:?}", arr1, arr2, arr3);
}

fn test_missing_comma_positive_numbers() {
    // These should trigger violations - positive numbers on separate lines
    let arr1 = [
        1, 2, 3     // Missing comma after 3
        4, 5, 6     // Violation: should be 3, 4
    ];
    
    let arr2 = [
        10
        20          // Violation: missing comma between 10 and 20
        30
    ];
    
    println!("Arrays: {:?}, {:?}", arr1, arr2);
}

fn test_missing_comma_identifiers() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    
    // These should trigger violations - identifiers on separate lines
    let arr1 = [
        a, b, c     // Missing comma after c
        d           // Violation: should be c, d
    ];
    
    let arr2 = [
        a
        b           // Violation: missing comma between a and b
    ];
    
    println!("Arrays: {:?}, {:?}", arr1, arr2);
}

fn test_missing_comma_expressions() {
    let x = 10;
    let y = 20;
    
    // These should trigger violations - expressions on separate lines
    let arr1 = [
        x + y       // Missing comma
        x - y       // Violation: could be (x + y), (x - y) or (x + y) - (x - y)
    ];
    
    let arr2 = [
        x * 2
        y * 3       // Violation: missing comma
    ];
    
    println!("Arrays: {:?}, {:?}", arr1, arr2);
}

fn test_missing_comma_mixed_types() {
    // These should trigger violations - mixed types on separate lines
    let arr1 = [
        "hello"
        "world"     // Violation: missing comma between strings
    ];
    
    let arr2 = [
        true
        false       // Violation: missing comma between booleans
    ];
    
    let arr3 = [
        1.5
        2.5         // Violation: missing comma between floats
    ];
    
    println!("Arrays: {:?}, {:?}, {:?}", arr1, arr2, arr3);
}

fn test_vec_macro_violations() {
    // These should trigger violations - vec! macro
    let vec1 = vec![
        1, 2, 3
        4, 5, 6     // Violation: missing comma after 3
    ];
    
    let vec2 = vec![
        -1
        -2          // Violation: missing comma
    ];
    
    let vec3 = vec![
        "a"
        "b"         // Violation: missing comma
    ];
    
    println!("Vecs: {:?}, {:?}, {:?}", vec1, vec2, vec3);
}

fn test_complex_expressions_violations() {
    let x = 5;
    
    // These should trigger violations - complex expressions
    let arr1 = [
        x + 1
        -x          // Violation: could be (x + 1), -x or (x + 1) - x
    ];
    
    let arr2 = [
        x.pow(2)
        x.pow(3)    // Violation: missing comma
    ];
    
    let arr3 = [
        format!("x={}", x).as_str()
        "y"         // Violation: missing comma
    ];
    
    println!("Arrays: {:?}, {:?}, {:?}", arr1, arr2, arr3);
}

fn test_multiline_elements() {
    // These should trigger violations - elements that span multiple lines
    let arr1 = [
        1 + 2 +
        3           // This is one element
        4           // Violation: missing comma between (1+2+3) and 4
    ];
    
    let arr2 = [
        if true { 1 } else { 2 }
        3           // Violation: missing comma after if expression
    ];
    
    println!("Arrays: {:?}, {:?}", arr1, arr2);
}

fn test_correct_comma_usage() {
    // These should NOT trigger violations - correct comma usage
    let arr1 = [1, 2, 3, 4, 5, 6];  // Single line
    
    let arr2 = [
        1, 2, 3,    // Trailing comma
        4, 5, 6,    // Trailing comma
    ];
    
    let arr3 = [
        -1, -2, -3,
        -4, -5, -6
    ];
    
    let arr4 = vec![
        "hello",
        "world",
        "foo",
        "bar"
    ];
    
    // Proper multiline expressions
    let arr5 = [
        1 + 2 + 3,  // Comma present
        4 + 5 + 6,
    ];
    
    println!("Correct arrays: {:?}, {:?}, {:?}, {:?}, {:?}", arr1, arr2, arr3, arr4, arr5);
}

fn test_single_line_no_violation() {
    // These should NOT trigger violations - all on one line
    let arr1 = [-1, -2, -3 -4];  // This is actually -3 - 4 = -7, but on same line
    let arr2 = [1 + 2 - 3];      // Single expression
    let arr3 = [a, b, c];         // Assuming a, b, c are defined
    
    println!("Single line: {:?}, {:?}", arr1, arr2);
}

fn test_comments_between_elements() {
    // These might trigger violations - comments between elements
    let arr1 = [
        1, 2, 3     // First row
        // Comment here
        4, 5, 6     // Violation: missing comma after 3
    ];
    
    let arr2 = [
        -1,
        /* comment */ -2,
        -3
        /* another comment */
        -4          // Violation: missing comma after -3
    ];
    
    println!("Arrays with comments: {:?}, {:?}", arr1, arr2);
}

fn test_nested_arrays() {
    // Test nested arrays
    let matrix = [
        [1, 2, 3],
        [4, 5, 6]   // No violation: array elements are properly separated
        [7, 8, 9]   // Violation: missing comma between arrays
    ];
    
    let nested = [
        vec![1, 2]
        vec![3, 4]  // Violation: missing comma between vecs
    ];
    
    println!("Nested: {:?}, {:?}", matrix, nested);
}

fn test_function_calls_in_array() {
    fn get_value() -> i32 { 42 }
    
    // These should trigger violations
    let arr1 = [
        get_value()
        get_value() // Violation: missing comma
    ];
    
    let arr2 = [
        "test".len()
        "another".len() // Violation: missing comma
    ];
    
    println!("Function calls: {:?}, {:?}", arr1, arr2);
}

fn test_macro_calls_in_array() {
    // These should trigger violations
    let arr1 = [
        format!("a")
        format!("b") // Violation: missing comma
    ];
    
    let arr2 = [
        println!("test"); 1
        println!("test2"); 2 // Violation: missing comma
    ];
    
    println!("Macro calls: {:?}", arr1);
}

fn test_operators_that_look_ambiguous() {
    let x = 5;
    
    // These are particularly ambiguous cases
    let arr1 = [
        x           // Just x
        -x          // Violation: could be x, -x or x - x
    ];
    
    let arr2 = [
        2 * x       // 2 * x
        +3          // Violation: could be (2*x), +3 or (2*x) + 3
    ];
    
    let arr3 = [
        !true       // !true
        !false      // Violation: could be two elements or some weird expression
    ];
    
    println!("Ambiguous: {:?}, {:?}, {:?}", arr1, arr2, arr3);
}

fn test_range_expressions() {
    // These might trigger violations with ranges
    let arr1 = [
        1..10
        11..20      // Violation: missing comma between ranges
    ];
    
    let arr2 = [
        1..=5,
        6..=10,     // Correct: has commas
    ];
    
    println!("Ranges: {:?}, {:?}", arr1, arr2);
}

fn test_closure_expressions() {
    // These should trigger violations with closures
    let arr1 = [
        |x| x + 1
        |y| y * 2   // Violation: missing comma between closures
    ];
    
    let arr2: [Box<dyn Fn(i32) -> i32>; 2] = [
        Box::new(|x| x + 1),
        Box::new(|y| y * 2), // Correct: has comma
    ];
    
    println!("Closures: {:?}", arr1.len());
}

fn test_tuple_expressions() {
    // These should trigger violations with tuples
    let arr1 = [
        (1, 2)
        (3, 4)      // Violation: missing comma between tuples
    ];
    
    let arr2 = [
        (1, 2),
        (3, 4),     // Correct: has commas
    ];
    
    println!("Tuples: {:?}, {:?}", arr1, arr2);
}

fn test_struct_expressions() {
    struct Point { x: i32, y: i32 }
    
    // These should trigger violations
    let arr1 = [
        Point { x: 1, y: 2 }
        Point { x: 3, y: 4 }  // Violation: missing comma
    ];
    
    let arr2 = [
        Point { x: 1, y: 2 },
        Point { x: 3, y: 4 }, // Correct: has comma
    ];
    
    println!("Structs: {:?}, {:?}", arr1.len(), arr2.len());
}

fn test_const_expressions() {
    const A: i32 = 1;
    const B: i32 = 2;
    
    // These should trigger violations
    let arr1 = [
        A
        B           // Violation: missing comma
    ];
    
    const ARRAY: [i32; 4] = [
        1, 2
        3, 4        // Violation: missing comma in const context
    ];
    
    println!("Const: {:?}, {:?}", arr1, ARRAY);
}