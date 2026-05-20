// Test cases for RS032: mem_replace_with_uninit

use std::mem;

fn test_mem_replace_with_uninit_violations() {
    let mut value = 42;

    // These should trigger violations - dangerous mem::replace with uninitialized
    let old1 = mem::replace(&mut value, mem::uninitialized()); // Violation
    let old2 = std::mem::replace(&mut value, std::mem::uninitialized()); // Violation
    let old3 = core::mem::replace(&mut value, core::mem::uninitialized()); // Violation

    println!("Dangerous operations: {}, {}, {}", old1, old2, old3);
}

fn test_imported_mem_functions() {
    use std::mem::{replace, uninitialized};

    let mut data = vec![1, 2, 3];

    // This should trigger violation - imported functions
    let old_data = replace(&mut data, uninitialized()); // Violation

    println!("Old data: {:?}", old_data);
}

fn test_maybe_uninit_violations() {
    use std::mem::MaybeUninit;

    let mut value = "hello".to_string();

    // This should trigger violation - using MaybeUninit unsafely
    let old = mem::replace(&mut value, MaybeUninit::uninit().assume_init()); // Violation

    println!("Old value: {}", old);
}

fn test_safe_alternatives() {
    let mut value = 42;
    let mut data = vec![1, 2, 3];
    let mut text = "hello".to_string();

    // These should NOT trigger violations - safe alternatives
    let old1 = mem::take(&mut value); // Safe: takes ownership and leaves Default
    let old2 = mem::replace(&mut value, 0); // Safe: replaces with actual value
    let old3 = mem::replace(&mut data, Vec::new()); // Safe: replaces with empty vec
    let old4 = mem::replace(&mut text, String::new()); // Safe: replaces with empty string

    // Using Default trait
    let old5 = mem::replace(&mut value, i32::default()); // Safe

    println!("Safe operations: {}, {}, {:?}, {}, {}", old1, old2, old3, old4, old5);
}

fn test_ptr_read_alternative() {
    use std::ptr;

    let mut boxed = Box::new(42);

    // This should NOT trigger violation - safe alternative
    let value = unsafe { ptr::read(&*boxed) }; // Safe when used correctly
    std::mem::forget(boxed); // Prevent double free

    println!("Read value: {}", value);
}

fn test_nested_expressions() {
    let mut value = 42;

    // This should trigger violation - nested in complex expression
    let result = if true {
        mem::replace(&mut value, mem::uninitialized()) // Violation
    } else {
        0
    };

    println!("Result: {}", result);
}

fn test_function_call_context() {
    fn dangerous_function() -> i32 {
        let mut x = 10;
        // This should trigger violation
        mem::replace(&mut x, mem::uninitialized()) // Violation
    }

    fn safe_function() -> i32 {
        let mut x = 10;
        // This should NOT trigger violation
        mem::replace(&mut x, 0) // Safe
    }

    println!("Functions: {}, {}", dangerous_function(), safe_function());
}

fn test_different_types() {
    let mut int_val = 42i32;
    let mut float_val = 3.14f64;
    let mut bool_val = true;
    let mut string_val = "test".to_string();

    // These should all trigger violations - different types with uninitialized
    let old_int = mem::replace(&mut int_val, mem::uninitialized()); // Violation
    let old_float = mem::replace(&mut float_val, mem::uninitialized()); // Violation
    let old_bool = mem::replace(&mut bool_val, mem::uninitialized()); // Violation
    let old_string = mem::replace(&mut string_val, mem::uninitialized()); // Violation

    println!("Old values: {}, {}, {}, {}", old_int, old_float, old_bool, old_string);
}

fn test_macro_calls() {
    macro_rules! replace_with_uninit {
        ($target:expr) => {
            mem::replace($target, mem::uninitialized()) // Should trigger violation
        };
    }

    let mut value = 100;
    let old = replace_with_uninit!(&mut value); // Violation

    println!("Macro result: {}", old);
}

fn test_method_chaining() {
    struct Container {
        value: i32,
    }

    impl Container {
        fn get_mut(&mut self) -> &mut i32 {
            &mut self.value
        }
    }

    let mut container = Container { value: 42 };

    // This should trigger violation - method chaining with mem::replace
    let old = mem::replace(container.get_mut(), mem::uninitialized()); // Violation

    println!("Container old value: {}", old);
}

fn test_generic_contexts() {
    fn replace_with_uninit<T>() -> T {
        let mut dummy: T = unsafe { mem::uninitialized() };
        // This should trigger violation
        mem::replace(&mut dummy, mem::uninitialized()) // Violation
    }

    // These would be problematic if called
    // let _: i32 = replace_with_uninit();
    // let _: String = replace_with_uninit();
}

fn test_valid_mem_replace_usage() {
    let mut value = 42;
    let mut option_val = Some(100);
    let mut vec_val = vec![1, 2, 3];

    // These should NOT trigger violations - valid mem::replace usage
    let old1 = mem::replace(&mut value, 999);
    let old2 = mem::replace(&mut option_val, None);
    let old3 = mem::replace(&mut vec_val, vec![4, 5, 6]);

    // Using const values
    const DEFAULT_VALUE: i32 = 0;
    let old4 = mem::replace(&mut value, DEFAULT_VALUE);

    println!("Valid replacements: {}, {:?}, {:?}, {}", old1, old2, old3, old4);
}

// Edge case: function that returns uninitialized (still dangerous)
fn get_uninitialized<T>() -> T {
    unsafe { mem::uninitialized() }
}

fn test_function_returning_uninit() {
    let mut value = 42;

    // This should trigger violation - function call that returns uninitialized
    let old = mem::replace(&mut value, get_uninitialized()); // Violation

    println!("Function uninit: {}", old);
}