// Test cases for RS002: almost_swapped

fn test_almost_swap() {
    let mut a = 5;
    let mut b = 10;

    // This should trigger a violation - looks like attempted swap
    a = b;
    b = a; // This assigns the same value to both variables

    println!("a: {}, b: {}", a, b); // Both will be 10
}

fn test_proper_swap() {
    let mut x = 1;
    let mut y = 2;

    // These should NOT trigger violations - proper swapping methods
    std::mem::swap(&mut x, &mut y);

    let mut p = 3;
    let mut q = 4;
    (p, q) = (q, p); // Tuple swap
}

fn test_normal_assignments() {
    let mut value1 = 100;
    let mut value2 = 200;
    let temp = 300;

    // These should NOT trigger violations - normal assignments
    value1 = temp;
    value2 = temp;

    // Separate assignments that aren't swaps
    value1 = value2;
    // ... some other code ...
    value2 = value1; // This might be intentional, not consecutive
}

fn test_consecutive_almost_swap() {
    let mut first = "hello";
    let mut second = "world";

    // This should trigger a violation
    first = second;
    second = first; // Both will be "world"
}

fn test_different_variables() {
    let mut var_a = 1;
    let mut var_b = 2;
    let mut var_c = 3;

    // This should NOT trigger - different variable pattern
    var_a = var_b;
    var_c = var_a;
}