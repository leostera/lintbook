// Test cases for RS027: iter_skip_zero

fn test_skip_zero_violations() {
    let data = vec![1, 2, 3, 4, 5];

    // These should trigger violations - .skip(0) is redundant
    let result1: Vec<_> = data.iter().skip(0).collect();
    let result2: Vec<_> = data.iter().skip(0).map(|x| x * 2).collect();
    let result3: Vec<_> = data.iter().filter(|&&x| x > 2).skip(0).collect();

    println!("Results: {:?}, {:?}, {:?}", result1, result2, result3);
}

fn test_skip_zero_in_chains() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // These should trigger violations - .skip(0) in method chains
    let evens: Vec<_> = numbers
        .iter()
        .skip(0)  // Redundant
        .filter(|&&x| x % 2 == 0)
        .collect();

    let processed: Vec<_> = numbers
        .iter()
        .map(|x| x * 2)
        .skip(0)  // Redundant
        .take(5)
        .collect();

    let enumerated: Vec<_> = numbers
        .iter()
        .enumerate()
        .skip(0)  // Redundant
        .collect();

    println!("Evens: {:?}, Processed: {:?}, Enumerated: {:?}", evens, processed, enumerated);
}

fn test_valid_skip_calls() {
    let data = vec![1, 2, 3, 4, 5];

    // These should NOT trigger violations - valid .skip() calls
    let skip_one: Vec<_> = data.iter().skip(1).collect();
    let skip_two: Vec<_> = data.iter().skip(2).collect();
    let skip_many: Vec<_> = data.iter().skip(10).collect();

    println!("Skip 1: {:?}, Skip 2: {:?}, Skip many: {:?}", skip_one, skip_two, skip_many);
}

fn test_skip_with_variables() {
    let data = vec![1, 2, 3, 4, 5];
    let skip_count = 0;
    let other_count = 2;

    // This should NOT trigger - using a variable (even if it's 0)
    let result1: Vec<_> = data.iter().skip(skip_count).collect();

    // This should NOT trigger - using a non-zero variable
    let result2: Vec<_> = data.iter().skip(other_count).collect();

    println!("Variable skip: {:?}, {:?}", result1, result2);
}

fn test_string_iterators() {
    let text = "hello world";

    // These should trigger violations - .skip(0) on string iterators
    let chars1: Vec<_> = text.chars().skip(0).collect();
    let bytes1: Vec<_> = text.bytes().skip(0).collect();

    // These should NOT trigger - valid skip on string iterators
    let chars2: Vec<_> = text.chars().skip(2).collect();
    let bytes2: Vec<_> = text.bytes().skip(3).collect();

    println!("Chars: {:?}, {:?}", chars1, chars2);
    println!("Bytes: {:?}, {:?}", bytes1, bytes2);
}

fn test_range_iterators() {
    // These should trigger violations - .skip(0) on ranges
    let range1: Vec<_> = (0..10).skip(0).collect();
    let range2: Vec<_> = (1..=20).skip(0).filter(|&x| x % 2 == 0).collect();

    // These should NOT trigger - valid skip on ranges
    let range3: Vec<_> = (0..10).skip(3).collect();
    let range4: Vec<_> = (1..=20).skip(5).collect();

    println!("Ranges: {:?}, {:?}, {:?}, {:?}", range1, range2, range3, range4);
}

fn test_iterator_variables() {
    let data = vec![1, 2, 3, 4, 5];
    let my_iter = data.iter();
    let custom_iterator = data.into_iter();

    // These should trigger violations - .skip(0) on iterator variables
    let result1: Vec<_> = my_iter.skip(0).collect();
    let result2: Vec<_> = custom_iterator.skip(0).map(|x| x * 2).collect();

    println!("Iterator variables: {:?}, {:?}", result1, result2);
}

fn test_complex_iterator_chains() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // These should trigger violations - .skip(0) in complex chains
    let result1: Vec<_> = data
        .iter()
        .filter(|&&x| x > 3)
        .map(|x| x * 2)
        .skip(0)  // Redundant
        .enumerate()
        .collect();

    let result2: Vec<_> = data
        .iter()
        .skip(0)  // Redundant
        .chain(data.iter())
        .skip(2)  // Valid
        .collect();

    println!("Complex chains: {:?}, {:?}", result1, result2);
}

fn test_for_loops_with_skip_zero() {
    let data = vec![1, 2, 3, 4, 5];

    // These should trigger violations - .skip(0) in for loops
    for item in data.iter().skip(0) {
        println!("Item: {}", item);
    }

    for (i, item) in data.iter().enumerate().skip(0) {
        println!("Index {}: {}", i, item);
    }

    // This should NOT trigger - valid skip in for loop
    for item in data.iter().skip(2) {
        println!("Skipped item: {}", item);
    }
}

fn test_collection_methods() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert("a", 1);
    map.insert("b", 2);
    map.insert("c", 3);

    // These should trigger violations - .skip(0) on collection iterators
    let keys: Vec<_> = map.keys().skip(0).collect();
    let values: Vec<_> = map.values().skip(0).collect();
    let pairs: Vec<_> = map.iter().skip(0).collect();

    println!("Map data: {:?}, {:?}, {:?}", keys, values, pairs);
}

fn test_skip_with_expressions() {
    let data = vec![1, 2, 3, 4, 5];

    // This should NOT trigger - expression that evaluates to 0, but not literal 0
    let result1: Vec<_> = data.iter().skip(1 - 1).collect();

    // This should NOT trigger - function call
    let result2: Vec<_> = data.iter().skip(get_skip_count()).collect();

    println!("Expression skip: {:?}, {:?}", result1, result2);
}

fn get_skip_count() -> usize {
    0
}

fn test_non_iterator_skip() {
    struct CustomCollection {
        data: Vec<i32>,
    }

    impl CustomCollection {
        fn skip(&self, _n: usize) -> &Self {
            self
        }
    }

    let custom = CustomCollection {
        data: vec![1, 2, 3],
    };

    // This should NOT trigger - not an iterator's skip method
    let _result = custom.skip(0);
}