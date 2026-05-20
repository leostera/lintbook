// Test cases for RS001: absurd_extreme_comparisons

fn test_u32_max_comparison() {
    let x: u32 = 100;

    // These should trigger violations
    if x >= u32::MAX {  // Always false
        println!("This will never execute");
    }

    if x == u32::MAX {  // Could be true
        println!("This is OK");
    }

    if x > u32::MAX {   // Always false
        println!("This will never execute");
    }
}

fn test_i32_min_comparison() {
    let y: i32 = -50;

    // These should trigger violations
    if y < i32::MIN {   // Always false
        println!("This will never execute");
    }

    if y <= i32::MIN {  // Could be true
        println!("This is OK");
    }
}

fn test_literal_extremes() {
    let z: u8 = 200;

    // These should trigger violations
    if z >= 255 {       // Always false for u8 (except when z == 255)
        println!("Edge case");
    }

    if z > 255 {        // Always false
        println!("This will never execute");
    }
}

fn test_normal_comparisons() {
    let a = 10;
    let b = 20;

    // These should NOT trigger violations
    if a < b {
        println!("Normal comparison");
    }

    if a >= 0 {
        println!("Normal comparison");
    }
}