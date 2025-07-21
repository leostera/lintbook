// Test cases for RS034: mistyped_literal_suffixes

fn test_mistyped_underscore_suffixes() {
    // These should trigger violations - mistyped suffixes with underscore
    let val1 = 2_32; // Violation: should be 2_i32
    let val2 = 250_8; // Violation: should be 250_u8
    let val3 = 65535_16; // Violation: should be 65535_u16
    let val4 = 4294967295_64; // Violation: should be 4294967295_u64
    let val5 = 100_128; // Violation: should be 100_u128
    
    println!("Mistyped values: {}, {}, {}, {}, {}", val1, val2, val3, val4, val5);
}

fn test_mistyped_no_underscore_suffixes() {
    // These should trigger violations - numbers ending with size digits
    let val1 = 232; // Violation: should be 2_i32 if intended as i32
    let val2 = 2508; // Violation: should be 250_u8 if intended as u8
    let val3 = 12316; // Violation: should be 123_u16 if intended as u16
    let val4 = 42364; // Violation: should be 423_i64 if intended as i64
    
    println!("No underscore mistyped: {}, {}, {}, {}", val1, val2, val3, val4);
}

fn test_floating_point_mistyped() {
    // These should trigger violations - floating point with mistyped suffixes
    let val1 = 3.14_32; // Violation: should be 3.14_f32
    let val2 = 2.718_64; // Violation: should be 2.718_f64
    
    println!("Float mistyped: {}, {}", val1, val2);
}

fn test_size_mistyped() {
    // This should trigger violation - size suffix mistyped
    let val1 = 1024_size; // Violation: should be 1024_usize
    
    println!("Size mistyped: {}", val1);
}

fn test_correct_literal_suffixes() {
    // These should NOT trigger violations - correct suffixes
    let val1 = 2_i32; // Correct
    let val2 = 250_u8; // Correct
    let val3 = 65535_u16; // Correct
    let val4 = 4294967295_u64; // Correct
    let val5 = 100_u128; // Correct
    let val6 = 3.14_f32; // Correct
    let val7 = 2.718_f64; // Correct
    let val8 = 1024_usize; // Correct
    let val9 = 1024_isize; // Correct
    
    println!("Correct suffixes: {}, {}, {}, {}, {}, {}, {}, {}, {}", 
             val1, val2, val3, val4, val5, val6, val7, val8, val9);
}

fn test_no_suffix_literals() {
    // These should NOT trigger violations - no suffixes
    let val1 = 42; // No suffix, fine
    let val2 = 3.14; // No suffix, fine
    let val3 = 0xFF; // Hex, fine
    let val4 = 0o755; // Octal, fine
    let val5 = 0b1010; // Binary, fine
    
    println!("No suffix literals: {}, {}, {}, {}, {}", val1, val2, val3, val4, val5);
}

fn test_hex_octal_binary_literals() {
    // These should NOT trigger violations - different bases
    let val1 = 0xFF32; // Hex literal, not mistyped suffix
    let val2 = 0o64; // Octal literal, not mistyped suffix
    let val3 = 0b11110000; // Binary literal, not mistyped suffix
    
    println!("Different bases: {}, {}, {}", val1, val2, val3);
}

fn test_underscores_in_numbers() {
    // These should NOT trigger violations - underscores for readability
    let val1 = 1_000_000; // Underscores for readability, fine
    let val2 = 1_234_567_890; // Multiple underscores, fine
    let val3 = 0xFF_FF_FF_FF; // Hex with underscores, fine
    
    println!("Underscores for readability: {}, {}, {}", val1, val2, val3);
}

fn test_edge_cases() {
    // These might be ambiguous but should be handled carefully
    let val1 = 8; // Single digit 8, should NOT trigger
    let val2 = 32; // Just 32, could be mistyped but probably not
    let val3 = 64; // Just 64, could be mistyped but probably not
    let val4 = 128; // Just 128, could be mistyped but probably not
    
    println!("Edge cases: {}, {}, {}, {}", val1, val2, val3, val4);
}

fn test_array_contexts() {
    // Test in array contexts
    let arr1 = [1_32, 2_32, 3_32]; // Violations: should be i32
    let arr2 = [255_8, 127_8, 0_8]; // Violations: should be u8
    
    println!("Arrays: {:?}, {:?}", arr1, arr2);
}

fn test_function_call_contexts() {
    // Test in function call contexts
    some_function(42_32); // Violation: should be 42_i32
    another_function(255_8, 65535_16); // Violations: should be u8, u16
}

fn some_function(x: i32) {
    println!("Function: {}", x);
}

fn another_function(a: u8, b: u16) {
    println!("Another function: {}, {}", a, b);
}

fn test_assignment_contexts() {
    let mut x = 0;
    
    // Test in assignment contexts
    x = 100_32; // Violation: should be 100_i32
    x += 50_32; // Violation: should be 50_i32
}

fn test_comparison_contexts() {
    let x = 100;
    
    // Test in comparison contexts
    if x > 50_32 { // Violation: should be 50_i32
        println!("Greater");
    }
    
    if x == 100_32 { // Violation: should be 100_i32
        println!("Equal");
    }
}

fn test_match_contexts() {
    let x = 42;
    
    // Test in match contexts
    match x {
        42_32 => println!("Match"), // Violation: should be 42_i32
        _ => println!("No match"),
    }
}

fn test_range_contexts() {
    // Test in range contexts
    for i in 1_32..10_32 { // Violations: should be i32
        println!("Range: {}", i);
    }
}

fn test_struct_field_contexts() {
    struct MyStruct {
        field1: u8,
        field2: u16,
    }
    
    let s = MyStruct {
        field1: 255_8, // Violation: should be 255_u8
        field2: 65535_16, // Violation: should be 65535_u16
    };
    
    println!("Struct: {}, {}", s.field1, s.field2);
}

fn test_tuple_contexts() {
    // Test in tuple contexts
    let tuple = (42_32, 255_8, 65535_16); // Violations: should have proper suffixes
    
    println!("Tuple: {:?}", tuple);
}

fn test_complex_expressions() {
    // Test in complex expressions
    let result = 2_32 + 3_32 * 4_32; // Violations: should be i32
    let another = (10_32 - 5_32) / 2_32; // Violations: should be i32
    
    println!("Complex: {}, {}", result, another);
}

fn test_generic_contexts() {
    // Test with generics
    let vec: Vec<u8> = vec![1_8, 2_8, 3_8]; // Violations: should be u8
    
    println!("Generic: {:?}", vec);
}

fn test_const_contexts() {
    // Test in const contexts
    const CONST1: i32 = 42_32; // Violation: should be 42_i32
    const CONST2: u8 = 255_8; // Violation: should be 255_u8
    
    println!("Constants: {}, {}", CONST1, CONST2);
}

fn test_static_contexts() {
    // Test in static contexts
    static STATIC1: i32 = 42_32; // Violation: should be 42_i32
    static STATIC2: u8 = 255_8; // Violation: should be 255_u8
    
    println!("Statics: {}, {}", STATIC1, STATIC2);
}