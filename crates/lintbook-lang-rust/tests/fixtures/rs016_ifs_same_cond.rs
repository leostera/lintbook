// Test cases for RS016: ifs_same_cond

fn test_consecutive_same_conditions() {
    let x = 10;
    let y = 20;

    // These should trigger violations - same conditions
    if x > 5 {
        println!("First condition");
    }
    if x > 5 {  // Same condition as above
        println!("Second condition - will never execute if first didn't");
    }

    // Another example with complex condition
    if x > 0 && y < 30 {
        println!("Complex condition 1");
    }
    if x > 0 && y < 30 {  // Exact same condition
        println!("Complex condition 2 - redundant");
    }
}

fn test_proper_else_if() {
    let value = 42;

    // This should NOT trigger - proper else if chain
    if value > 50 {
        println!("Greater than 50");
    } else if value > 30 {
        println!("Greater than 30");
    } else if value > 10 {
        println!("Greater than 10");
    }
}

fn test_different_conditions() {
    let a = 1;
    let b = 2;

    // These should NOT trigger - different conditions
    if a > 0 {
        println!("a is positive");
    }
    if b > 0 {  // Different variable
        println!("b is positive");
    }

    if a > 5 {
        println!("a greater than 5");
    }
    if a > 10 {  // Different threshold
        println!("a greater than 10");
    }
}

fn test_with_intermediate_code() {
    let flag = true;

    // These should still trigger violations even with code in between
    if flag == true {
        println!("Flag is true");
    }

    let _intermediate = 42;  // Some code in between

    if flag == true {  // Same condition as before
        println!("Flag is still true - redundant check");
    }
}

fn test_whitespace_variations() {
    let num = 100;

    // These should trigger violations - same conditions with different whitespace
    if num>50 {
        println!("Without spaces");
    }
    if num > 50 {  // Same condition with spaces
        println!("With spaces - should be detected as same");
    }

    if (num == 100) {
        println!("With parentheses");
    }
    if num == 100 {  // Same logic, different formatting
        println!("Without parentheses - should be detected as same");
    }
}

fn test_boolean_expressions() {
    let is_valid = true;
    let is_ready = false;

    // These should trigger violations
    if is_valid && !is_ready {
        println!("First boolean check");
    }
    if is_valid && !is_ready {  // Exact same boolean expression
        println!("Second boolean check - redundant");
    }
}

fn test_nested_ifs() {
    let x = 5;
    let y = 10;

    // Nested ifs - the inner ones should be checked too
    if x > 0 {
        if y > 5 {
            println!("Nested condition 1");
        }
        if y > 5 {  // Same nested condition - should trigger
            println!("Nested condition 2 - redundant");
        }
    }
}

fn test_method_calls() {
    let vec = vec![1, 2, 3];

    // These should trigger violations - same method call conditions
    if vec.is_empty() {
        println!("Vector is empty");
    }
    if vec.is_empty() {  // Same method call
        println!("Vector is still empty - redundant");
    }
}

fn test_complex_expressions() {
    let data = Some(42);

    // These should trigger violations - same complex conditions
    if data.is_some() && data.unwrap() > 40 {
        println!("Complex condition 1");
    }
    if data.is_some() && data.unwrap() > 40 {  // Exact same complex condition
        println!("Complex condition 2 - redundant");
    }
}