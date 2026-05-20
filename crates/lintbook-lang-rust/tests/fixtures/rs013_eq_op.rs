// Test cases for RS013: eq_op

fn test_equal_operands() {
    let x = 10;
    let y = 20;

    // These should trigger violations - equal operands
    if x == x {  // Always true
        println!("This will always execute");
    }

    if y != y {  // Always false
        println!("This will never execute");
    }

    let result = x - x;  // Always 0
    let division = y / y;  // Always 1 (unless y is 0)

    // Boolean operations with equal operands
    let flag = true;
    if flag && flag {  // Redundant
        println!("Redundant condition");
    }

    if flag || flag {  // Redundant
        println!("Redundant condition");
    }
}

fn test_bitwise_equal_operands() {
    let bits = 0xFF;

    // These should trigger violations
    let xor_result = bits ^ bits;  // Always 0
    let and_result = bits & bits;  // Always bits
    let or_result = bits | bits;   // Always bits
}

fn test_comparison_equal_operands() {
    let value = 42;

    // These should trigger violations
    if value < value {   // Always false
        println!("Never");
    }

    if value <= value {  // Always true
        println!("Always");
    }

    if value > value {   // Always false
        println!("Never");
    }

    if value >= value {  // Always true
        println!("Always");
    }
}

fn test_arithmetic_equal_operands() {
    let num = 15;

    // These should trigger violations
    let mod_result = num % num;  // Always 0 (unless num is 0)
    let add_result = num + num;  // Could be intentional (doubling)
    let mul_result = num * num;  // Could be intentional (squaring)
}

fn test_valid_cases() {
    let a = 5;
    let b = 10;

    // These should NOT trigger violations - different operands
    if a == b {
        println!("Valid comparison");
    }

    let sum = a + b;
    let difference = a - b;
    let product = a * b;

    // Self-comparison for NaN checking might be intentional
    let float_val = 3.14f64;
    if float_val != float_val {  // NaN check - might be intentional
        println!("NaN detected");
    }
}

fn test_constants() {
    const MAX_SIZE: usize = 1024;

    // These might be intentional with constants
    let combined = MAX_SIZE | MAX_SIZE;  // Might be intentional with bit flags
    let masked = MAX_SIZE & MAX_SIZE;    // Might be intentional
}