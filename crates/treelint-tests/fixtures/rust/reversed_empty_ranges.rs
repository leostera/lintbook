fn main() {
    // Violations - reversed ranges
    for i in 10..5 {  // Reversed range: start > end
        println!("{}", i);
    }
    
    for i in 100..=50 {  // Reversed inclusive range: start > end
        println!("{}", i);
    }
    
    let range1 = 20..10;  // Reversed range
    let range2 = 30..=15; // Reversed inclusive range
    
    // More violations
    for i in 5..0 {
        println!("{}", i);
    }
    
    for i in 1..=0 {
        println!("{}", i);
    }
    
    // No violations - correct ranges
    for i in 0..10 {  // Correct range
        println!("{}", i);
    }
    
    for i in 5..=10 {  // Correct inclusive range
        println!("{}", i);
    }
    
    let range3 = 0..5;   // Correct range
    let range4 = 1..=10; // Correct inclusive range
    
    // Equal start and end (not a violation for exclusive, empty but valid)
    for i in 5..5 {  // Empty but valid range
        println!("{}", i);
    }
    
    // Equal start and end for inclusive (valid, single element)
    for i in 5..=5 { // Valid inclusive range with one element
        println!("{}", i);
    }
}