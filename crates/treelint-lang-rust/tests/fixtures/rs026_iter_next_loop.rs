// Test cases for RS026: iter_next_loop

fn test_direct_next_loop() {
    let mut iter = vec![1, 2, 3, 4, 5].into_iter();
    
    // This should trigger violation - iterating over .next()
    for item in iter.next() {
        println!("Item: {}", item); // Will only run once, if at all
    }
}

fn test_chained_iterator_next() {
    let data = vec![1, 2, 3, 4, 5];
    
    // These should trigger violations - iterator chains ending with .next()
    for item in data.iter().next() {
        println!("Item: {}", item); // Will only iterate once
    }
    
    for item in data.iter().skip(1).next() {
        println!("Item: {}", item); // Will only iterate once
    }
    
    for item in data.iter().filter(|&&x| x > 2).next() {
        println!("Item: {}", item); // Will only iterate once
    }
}

fn test_correct_iterator_usage() {
    let data = vec![1, 2, 3, 4, 5];
    
    // These should NOT trigger violations - correct iterator usage
    for item in data.iter() {
        println!("Item: {}", item); // Correct: iterates over all items
    }
    
    for item in data.iter().skip(1) {
        println!("Item: {}", item); // Correct: iterates over remaining items
    }
    
    for item in data.iter().filter(|&&x| x > 2) {
        println!("Item: {}", item); // Correct: iterates over filtered items
    }
}

fn test_while_let_correct_usage() {
    let mut iter = vec![1, 2, 3].into_iter();
    
    // This should NOT trigger - correct use of .next() with while let
    while let Some(item) = iter.next() {
        println!("Item: {}", item); // Correct: properly consuming iterator
    }
}

fn test_various_iterator_types() {
    let text = "hello world";
    let range = 0..10;
    
    // These should trigger violations
    for ch in text.chars().next() {
        println!("Char: {}", ch); // Will only get first char
    }
    
    for num in range.next() {
        println!("Num: {}", num); // Will only get first number
    }
}

fn test_method_calls_with_args() {
    let data = vec![1, 2, 3, 4, 5];
    
    // This should NOT trigger - .next() is not being called
    for item in data.iter().take(3) {
        println!("Item: {}", item);
    }
    
    // This should NOT trigger - not a for loop over .next()
    let first = data.iter().next();
    if let Some(value) = first {
        println!("First: {}", value);
    }
}

fn test_complex_iterator_chains() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // These should trigger violations - complex chains ending with .next()
    for item in data.iter().filter(|&&x| x % 2 == 0).map(|x| x * 2).next() {
        println!("Even doubled: {}", item); // Only gets first result
    }
    
    for item in data.iter().skip(2).take(5).enumerate().next() {
        println!("Enumerated: {:?}", item); // Only gets first enumerated item
    }
}

fn test_iterator_variables() {
    let data = vec![1, 2, 3];
    let my_iter = data.iter();
    let custom_iterator = data.into_iter();
    
    // These should trigger violations - iterating over .next() on iterator variables
    for item in my_iter.next() {
        println!("Item: {}", item);
    }
    
    for item in custom_iterator.next() {
        println!("Item: {}", item);
    }
}

fn test_non_iterator_next() {
    struct CustomStruct {
        value: i32,
    }
    
    impl CustomStruct {
        fn next(&self) -> i32 {
            self.value + 1
        }
    }
    
    let custom = CustomStruct { value: 42 };
    
    // This should NOT trigger - not an iterator's .next() method
    for _item in 0..custom.next() {
        println!("Using custom.next() as range end");
    }
}

fn test_string_iteration() {
    let text = "hello";
    
    // These should trigger violations
    for ch in text.chars().next() {
        println!("First char: {}", ch);
    }
    
    for byte in text.bytes().next() {
        println!("First byte: {}", byte);
    }
    
    for line in text.lines().next() {
        println!("First line: {}", line);
    }
}

fn test_correct_alternatives() {
    let data = vec![1, 2, 3, 4, 5];
    let mut iter = data.iter();
    
    // These should NOT trigger - correct alternatives to for loops with .next()
    
    // Option 1: Use the iterator directly
    for item in data.iter() {
        println!("All items: {}", item);
    }
    
    // Option 2: Use while let with .next()
    while let Some(item) = iter.next() {
        println!("While let item: {}", item);
    }
    
    // Option 3: Use if let for single item
    if let Some(first) = data.iter().next() {
        println!("First item: {}", first);
    }
}