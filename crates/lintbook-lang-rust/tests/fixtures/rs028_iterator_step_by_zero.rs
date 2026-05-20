// Test cases for RS028: iterator_step_by_zero

fn test_step_by_zero_violations() {
    // These should trigger violations - .step_by(0) will panic

    // Range with step_by(0)
    for i in (0..10).step_by(0) {
        println!("This will panic: {}", i);
    }

    // Vector iterator with step_by(0)
    let data = vec![1, 2, 3, 4, 5];
    for item in data.iter().step_by(0) {
        println!("This will panic: {}", item);
    }

    // Collecting with step_by(0)
    let result: Vec<_> = (1..20).step_by(0).collect();
    println!("This will panic: {:?}", result);
}

fn test_step_by_zero_in_chains() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // These should trigger violations - .step_by(0) in method chains
    let evens: Vec<_> = numbers
        .iter()
        .filter(|&&x| x % 2 == 0)
        .step_by(0)  // Will panic
        .collect();

    let processed: Vec<_> = (0..100)
        .map(|x| x * 2)
        .step_by(0)  // Will panic
        .take(5)
        .collect();

    println!("These will panic: {:?}, {:?}", evens, processed);
}

fn test_valid_step_by_calls() {
    // These should NOT trigger violations - valid .step_by() calls
    let step_one: Vec<_> = (0..10).step_by(1).collect();
    let step_two: Vec<_> = (0..10).step_by(2).collect();
    let step_three: Vec<_> = (0..20).step_by(3).collect();

    println!("Valid steps: {:?}, {:?}, {:?}", step_one, step_two, step_three);
}

fn test_step_by_with_variables() {
    let step_size = 0;
    let other_step = 2;

    // This should NOT trigger - using a variable (even if it's 0)
    // The lint only catches literal 0, not variables that might be 0
    let result1: Vec<_> = (0..10).step_by(step_size).collect();

    // This should NOT trigger - using a non-zero variable
    let result2: Vec<_> = (0..10).step_by(other_step).collect();

    println!("Variable steps: {:?}, {:?}", result1, result2);
}

fn test_string_iterators() {
    let text = "hello world";

    // These should trigger violations - .step_by(0) on string iterators
    let chars: Vec<_> = text.chars().step_by(0).collect();
    let bytes: Vec<_> = text.bytes().step_by(0).collect();

    // These should NOT trigger - valid step_by on string iterators
    let chars_valid: Vec<_> = text.chars().step_by(2).collect();
    let bytes_valid: Vec<_> = text.bytes().step_by(3).collect();

    println!("Will panic: {:?}, {:?}", chars, bytes);
    println!("Valid: {:?}, {:?}", chars_valid, bytes_valid);
}

fn test_range_types() {
    // These should trigger violations - various range types with step_by(0)
    let range1: Vec<_> = (0..10).step_by(0).collect();
    let range2: Vec<_> = (1..=20).step_by(0).collect();
    let range3: Vec<_> = (10..0).step_by(0).collect(); // Empty range, but still invalid

    // These should NOT trigger - valid step_by on ranges
    let range4: Vec<_> = (0..10).step_by(1).collect();
    let range5: Vec<_> = (1..=20).step_by(5).collect();

    println!("Will panic: {:?}, {:?}, {:?}", range1, range2, range3);
    println!("Valid: {:?}, {:?}", range4, range5);
}

fn test_iterator_variables() {
    let data = vec![1, 2, 3, 4, 5];
    let my_iter = data.iter();

    // This should trigger violation - .step_by(0) on iterator variable
    let result: Vec<_> = my_iter.step_by(0).collect();

    println!("Will panic: {:?}", result);
}

fn test_complex_iterator_chains() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // These should trigger violations - .step_by(0) in complex chains
    let result1: Vec<_> = data
        .iter()
        .filter(|&&x| x > 3)
        .map(|x| x * 2)
        .enumerate()
        .step_by(0)  // Will panic
        .collect();

    let result2: Vec<_> = (0..50)
        .filter(|&x| x % 2 == 0)
        .step_by(0)  // Will panic
        .take(10)
        .collect();

    println!("Will panic: {:?}, {:?}", result1, result2);
}

fn test_for_loops_with_step_by_zero() {
    let data = vec![1, 2, 3, 4, 5];

    // These should trigger violations - .step_by(0) in for loops
    for item in data.iter().step_by(0) {
        println!("Will panic: {}", item);
    }

    for i in (0..10).step_by(0) {
        println!("Will panic at index: {}", i);
    }

    // This should NOT trigger - valid step_by in for loop
    for item in data.iter().step_by(2) {
        println!("Valid item: {}", item);
    }
}

fn test_enumerate_with_step_by() {
    let data = vec!["a", "b", "c", "d", "e"];

    // This should trigger violation - .step_by(0) on enumerated iterator
    for (i, item) in data.iter().enumerate().step_by(0) {
        println!("Will panic: {} = {}", i, item);
    }

    // This should NOT trigger - valid step_by on enumerated iterator
    for (i, item) in data.iter().enumerate().step_by(2) {
        println!("Valid: {} = {}", i, item);
    }
}

fn test_step_by_with_expressions() {
    // These should NOT trigger - expressions that evaluate to 0, but not literal 0
    let result1: Vec<_> = (0..10).step_by(1 - 1).collect();
    let result2: Vec<_> = (0..10).step_by(get_step_size()).collect();

    println!("Expression steps: {:?}, {:?}", result1, result2);
}

fn get_step_size() -> usize {
    0 // This will still cause a runtime panic, but the lint only catches literal 0
}

fn test_chained_step_by() {
    // This should trigger violation - step_by(0) in a chain
    let result: Vec<_> = (0..100)
        .filter(|&x| x % 2 == 0)
        .map(|x| x / 2)
        .step_by(0)  // Will panic
        .take(5)
        .collect();

    println!("Will panic: {:?}", result);
}

fn test_step_by_on_collections() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 3);

    // These should trigger violations - .step_by(0) on collection iterators
    let keys: Vec<_> = map.keys().step_by(0).collect();
    let values: Vec<_> = map.values().step_by(0).collect();

    println!("Will panic: {:?}, {:?}", keys, values);
}

fn test_non_iterator_step_by() {
    struct CustomCollection {
        data: Vec<i32>,
    }

    impl CustomCollection {
        fn step_by(&self, _n: usize) -> &Self {
            self
        }
    }

    let custom = CustomCollection {
        data: vec![1, 2, 3],
    };

    // This should NOT trigger - not an iterator's step_by method
    let _result = custom.step_by(0);
}