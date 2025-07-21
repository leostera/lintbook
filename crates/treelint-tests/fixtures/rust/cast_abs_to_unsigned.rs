fn main() {
    let x: i32 = -42;
    let y: i64 = -100;
    
    // Violations - casting abs() to unsigned types
    let abs_u32 = x.abs() as u32;  // Dangerous: could panic on i32::MIN
    let abs_u64 = y.abs() as u64;  // Dangerous: could panic on i64::MIN
    let abs_usize = x.abs() as usize;  // Dangerous: could panic on i32::MIN
    
    // More violations with different abs call patterns
    let abs_fn = abs(x) as u32;  // If abs function exists
    let abs_i32 = i32::abs(x) as u32;  // Explicit method call
    
    // Different unsigned types
    let abs_u8 = (x.abs() as i16) as u8;  // Still dangerous if original was abs()
    let abs_u16 = x.abs() as u16;
    let abs_u128 = y.abs() as u128;
    
    // No violations - safe operations
    let abs_i32_safe = x.abs();  // Not casting to unsigned
    let unsigned_abs = x.unsigned_abs();  // Using unsigned_abs (safe)
    let literal_cast = 42 as u32;  // Casting literal, not abs result
    let safe_cast = (x + 1) as u32;  // Casting expression, not abs
    
    // No violation - casting unsigned_abs result is safe
    let safe_unsigned = x.unsigned_abs() as u64;
    
    // No violation - abs() result used in other ways
    let abs_result = x.abs();
    let double_abs = abs_result * 2;
    let compared = abs_result > 10;
    
    // No violation - casting to signed types
    let abs_i64 = x.abs() as i64;  // Casting to signed type is safer
    let abs_isize = x.abs() as isize;
}