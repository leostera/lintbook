fn main() {
    let mut x = 0;
    
    // Violations - empty loops
    loop {  // Empty infinite loop
    }
    
    while x < 10 {  // Empty while loop
    }
    
    for i in 0..10 {  // Empty for loop
    }
    
    // Empty loop with just comments (still considered empty)
    loop {
        // This is just a comment
        // Still empty
    }
    
    while true {  // Another empty infinite loop
    }
    
    // No violations - loops with actual content
    loop {
        x += 1;
        if x > 10 {
            break;
        }
    }
    
    while x < 20 {
        x += 1;
        println!("x is {}", x);
    }
    
    for i in 0..5 {
        println!("i is {}", i);
    }
    
    // No violation - loop with std::hint::spin_loop (proper busy-waiting)
    loop {
        std::hint::spin_loop();
    }
    
    // No violation - loop with yield_now
    loop {
        std::thread::yield_now();
    }
    
    // No violation - loop with break
    loop {
        break;
    }
}