fn main() {
    // Violations - approximate constants that should use std::consts
    let pi = 3.14159; // Should use std::f64::consts::PI
    let e = 2.71828; // Should use std::f64::consts::E
    let sqrt2 = 1.41421; // Should use std::f64::consts::SQRT_2
    let half_pi = 1.57079; // Should use std::f64::consts::FRAC_PI_2
    let quarter_pi = 0.78539; // Should use std::f64::consts::FRAC_PI_4
    let ln_2 = 0.69314; // Should use std::f64::consts::LN_2
    let ln_10 = 2.30258; // Should use std::f64::consts::LN_10

    // With type suffixes
    let pi_f32 = 3.14159f32; // Should use std::f32::consts::PI
    let e_f64 = 2.71828f64; // Should use std::f64::consts::E

    // No violations - exact constants or different values
    let actual_pi = std::f64::consts::PI; // Correct usage
    let actual_e = std::f64::consts::E; // Correct usage
    let random_value = 3.0; // Not an approximate constant
    let another_value = 2.5; // Not an approximate constant
    let close_but_not_pi = 3.2; // Too far from pi
}