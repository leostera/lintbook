// Test cases for RS014: erasing_op

fn test_multiplication_by_zero() {
    let x = 42;
    let y = 100;
    
    // These should trigger violations - multiplication by zero
    let result1 = x * 0;     // Always 0
    let result2 = 0 * y;     // Always 0
    let result3 = x * 0u32;  // Always 0
    let result4 = 0i64 * y;  // Always 0
    
    println!("Results: {}, {}, {}, {}", result1, result2, result3, result4);
}

fn test_bitwise_and_with_zero() {
    let bits = 0xFF;
    let mask = 0x0F;
    
    // These should trigger violations - AND with zero
    let result1 = bits & 0;     // Always 0
    let result2 = 0 & mask;     // Always 0
    let result3 = bits & 0x00;  // Always 0
    
    println!("Results: {}, {}, {}", result1, result2, result3);
}

fn test_bitwise_or_with_all_ones() {
    let value = 0x0F;
    
    // These should trigger violations - OR with all 1s
    let result1 = value | 0xFF;      // Always 0xFF
    let result2 = 0xFF | value;      // Always 0xFF
    let result3 = value | u8::MAX;   // Always u8::MAX
    let result4 = !0 | value;        // Always !0 (all 1s)
    
    println!("Results: {}, {}, {}, {}", result1, result2, result3, result4);
}

fn test_xor_with_zero() {
    let data = 0xAB;
    
    // These should trigger violations - XOR with zero
    let result1 = data ^ 0;     // Always data
    let result2 = 0 ^ data;     // Always data
    let result3 = data ^ 0x00;  // Always data
    
    println!("Results: {}, {}, {}", result1, result2, result3);
}

fn test_shift_operations() {
    let number = 42;
    
    // These should trigger violations - excessive shifts
    let result1 = number << 64;  // Undefined behavior / 0
    let result2 = number >> 65;  // Undefined behavior / 0
    let result3 = number << 128; // Definitely undefined
    
    println!("Results: {}, {}, {}", result1, result2, result3);
}

fn test_valid_operations() {
    let a = 10;
    let b = 5;
    
    // These should NOT trigger violations - normal operations
    let product = a * b;      // Normal multiplication
    let masked = a & 0x0F;    // Valid masking
    let combined = a | 0x10;  // Valid bit setting
    let flipped = a ^ 0xFF;   // Valid XOR
    let shifted = a << 2;     // Valid shift
    
    // AND with all 1s that preserves value
    let preserved = a & 0xFF; // Valid masking
    let also_valid = a & !0;  // Valid (though redundant)
    
    println!("Valid results: {}, {}, {}, {}, {}, {}, {}", 
             product, masked, combined, flipped, shifted, preserved, also_valid);
}

fn test_zero_operations() {
    let zero = 0;
    let value = 123;
    
    // These might be intentional with zero
    let zero_product = zero * value;  // Might be intentional
    let zero_and = zero & value;      // Might be intentional
    
    println!("Zero operations: {}, {}", zero_product, zero_and);
}

fn test_constants() {
    const MASK: u32 = 0x00000000;
    const ALL_BITS: u32 = 0xFFFFFFFF;
    
    let value = 0x12345678;
    
    // These should trigger violations with constants
    let masked = value & MASK;     // Always 0
    let ored = value | ALL_BITS;   // Always ALL_BITS
    
    println!("Constant operations: {}, {}", masked, ored);
}