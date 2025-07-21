// Test cases for RS041: option_env_unwrap

fn test_option_env_unwrap_violations() {
    // These should trigger violations - option_env!().unwrap()
    let home = option_env!("HOME").unwrap(); // Violation: use env!("HOME") instead
    let path = option_env!("PATH").unwrap(); // Violation: use env!("PATH") instead
    let user = option_env!("USER").unwrap(); // Violation: use env!("USER") instead
    
    println!("Environment variables: {}, {}, {}", home, path, user);
}

fn test_different_env_vars() {
    // These should trigger violations - various environment variables
    let cargo_home = option_env!("CARGO_HOME").unwrap(); // Violation
    let rust_log = option_env!("RUST_LOG").unwrap(); // Violation
    let tmpdir = option_env!("TMPDIR").unwrap(); // Violation
    let lang = option_env!("LANG").unwrap(); // Violation
    
    println!("More env vars: {}, {}, {}, {}", cargo_home, rust_log, tmpdir, lang);
}

fn test_custom_env_vars() {
    // These should trigger violations - custom environment variables
    let api_key = option_env!("API_KEY").unwrap(); // Violation
    let db_url = option_env!("DATABASE_URL").unwrap(); // Violation
    let secret = option_env!("SECRET_TOKEN").unwrap(); // Violation
    
    println!("Custom vars: {}, {}, {}", api_key, db_url, secret);
}

fn test_chained_unwrap() {
    // These should trigger violations - method chaining with unwrap
    let result1 = option_env!("HOME").unwrap().to_string(); // Violation
    let result2 = option_env!("PATH").unwrap().len(); // Violation
    
    println!("Chained: {}, {}", result1, result2);
}

fn test_assignment_contexts() {
    // These should trigger violations - in assignment contexts
    let mut var = String::new();
    var = option_env!("HOME").unwrap(); // Violation
    
    let tuple = (option_env!("PATH").unwrap(), option_env!("USER").unwrap()); // Violations
    
    println!("Assignments: {}, {:?}", var, tuple);
}

fn test_function_call_contexts() {
    // These should trigger violations - as function arguments
    some_function(option_env!("HOME").unwrap()); // Violation
    another_function(option_env!("PATH").unwrap(), option_env!("USER").unwrap()); // Violations
}

fn some_function(path: &str) {
    println!("Function: {}", path);
}

fn another_function(path: &str, user: &str) {
    println!("Another function: {}, {}", path, user);
}

fn test_struct_field_contexts() {
    struct Config {
        home: String,
        path: String,
    }
    
    // These should trigger violations - in struct initialization
    let config = Config {
        home: option_env!("HOME").unwrap(), // Violation
        path: option_env!("PATH").unwrap(), // Violation
    };
    
    println!("Config: {}, {}", config.home, config.path);
}

fn test_array_contexts() {
    // These should trigger violations - in array/vec contexts
    let env_vars = vec![
        option_env!("HOME").unwrap(), // Violation
        option_env!("PATH").unwrap(), // Violation
        option_env!("USER").unwrap(), // Violation
    ];
    
    let array = [
        option_env!("LANG").unwrap(), // Violation
        option_env!("SHELL").unwrap(), // Violation
    ];
    
    println!("Arrays: {:?}, {:?}", env_vars, array);
}

fn test_match_contexts() {
    let home = option_env!("HOME").unwrap(); // Violation
    
    // Test in match contexts
    match home.as_str() {
        "/home/user" => println!("User home"),
        _ => println!("Other home"),
    }
}

fn test_if_contexts() {
    // These should trigger violations - in if conditions
    if option_env!("HOME").unwrap().starts_with("/home") { // Violation
        println!("Linux-style home");
    }
    
    if !option_env!("PATH").unwrap().is_empty() { // Violation
        println!("Path is set");
    }
}

fn test_return_contexts() {
    fn get_home() -> String {
        option_env!("HOME").unwrap() // Violation
    }
    
    fn get_path() -> &'static str {
        option_env!("PATH").unwrap() // Violation
    }
    
    println!("Returns: {}, {}", get_home(), get_path());
}

fn test_closure_contexts() {
    let env_vars = vec!["HOME", "PATH", "USER"];
    
    // These should trigger violations - in closures
    let values: Vec<String> = env_vars.iter()
        .map(|&var| option_env!(var).unwrap()) // This is tricky - macro with variable
        .collect();
    
    // More realistic closure example
    let get_home = || option_env!("HOME").unwrap(); // Violation
    
    println!("Closure: {:?}, {}", values, get_home());
}

fn test_loop_contexts() {
    let vars = ["HOME", "PATH", "USER"];
    
    // These should trigger violations - in loops
    for var in &vars {
        // This is complex - macro with runtime variable name
        // let value = option_env!(var).unwrap(); // This won't compile
        println!("Var: {}", var);
    }
    
    // More realistic loop example
    for _i in 0..3 {
        let home = option_env!("HOME").unwrap(); // Violation
        println!("Loop home: {}", home);
    }
}

fn test_complex_expressions() {
    // These should trigger violations - in complex expressions
    let result = format!("Home: {}", option_env!("HOME").unwrap()); // Violation
    let length = option_env!("PATH").unwrap().len() + 10; // Violation
    let uppercase = option_env!("USER").unwrap().to_uppercase(); // Violation
    
    println!("Complex: {}, {}, {}", result, length, uppercase);
}

fn test_correct_env_usage() {
    // These should NOT trigger violations - correct usage with env!
    let home = env!("HOME"); // Correct: compile-time check
    let path = env!("PATH"); // Correct: compile-time check
    
    println!("Correct env: {}, {}", home, path);
}

fn test_correct_option_env_handling() {
    // These should NOT trigger violations - proper Option handling
    let home = option_env!("HOME"); // Correct: returns Option
    let path = option_env!("PATH").unwrap_or("/default/path"); // Correct: provides default
    let user = option_env!("USER").unwrap_or_else(|| "unknown".to_string()); // Correct: lazy default
    
    if let Some(cargo_home) = option_env!("CARGO_HOME") { // Correct: pattern matching
        println!("Cargo home: {}", cargo_home);
    }
    
    match option_env!("RUST_LOG") { // Correct: match handling
        Some(level) => println!("Rust log: {}", level),
        None => println!("No rust log set"),
    }
    
    println!("Correct option handling: {:?}, {}, {}", home, path, user);
}

fn test_map_and_other_methods() {
    // These should NOT trigger violations - other Option methods
    let home_len = option_env!("HOME").map(|h| h.len()); // Correct: map
    let path_exists = option_env!("PATH").is_some(); // Correct: is_some
    let user_default = option_env!("USER").or(Some("default")); // Correct: or
    
    println!("Other methods: {:?}, {}, {:?}", home_len, path_exists, user_default);
}

fn test_expect_method() {
    // These might be similar violations but different method
    let home = option_env!("HOME").expect("HOME must be set"); // Similar issue but different method
    let path = option_env!("PATH").expect("PATH required"); // Similar issue but different method
    
    println!("Expect: {}, {}", home, path);
}

fn test_question_mark_operator() {
    // These should NOT trigger violations - using ? operator
    fn get_env_var() -> Option<String> {
        let home = option_env!("HOME")?; // Correct: early return if None
        Some(home)
    }
    
    if let Some(var) = get_env_var() {
        println!("Question mark: {}", var);
    }
}

fn test_nested_unwrap() {
    // This should trigger violation - nested in other expressions
    let result = Some(option_env!("HOME").unwrap()); // Violation: still unwrapping option_env
    
    println!("Nested: {:?}", result);
}

fn test_macro_combinations() {
    // Test with other macros
    println!("Macro combo: {}", option_env!("HOME").unwrap()); // Violation
    eprintln!("Error: {}", option_env!("PATH").unwrap()); // Violation
    
    let formatted = format!("User: {}", option_env!("USER").unwrap()); // Violation
    println!("{}", formatted);
}

fn test_const_contexts() {
    // These might be violations in const contexts
    const HOME: &str = env!("HOME"); // Correct: env! works in const
    // const PATH: &str = option_env!("PATH").unwrap(); // Would be violation but might not compile
    
    println!("Const: {}", HOME);
}

fn test_static_contexts() {
    // These might be violations in static contexts
    static HOME: &str = env!("HOME"); // Correct: env! works in static
    // static PATH: &str = option_env!("PATH").unwrap(); // Would be violation but might not compile
    
    println!("Static: {}", HOME);
}

fn test_thread_local() {
    use std::thread_local;
    
    thread_local! {
        static HOME: String = option_env!("HOME").unwrap(); // Violation
    }
    
    HOME.with(|h| println!("Thread local: {}", h));
}

fn test_lazy_static() {
    // If using lazy_static crate
    // lazy_static! {
    //     static ref HOME: String = option_env!("HOME").unwrap(); // Would be violation
    // }
}

fn test_different_unwrap_styles() {
    // Test different ways of calling unwrap
    let home1 = option_env!("HOME").unwrap(); // Violation: direct
    let home2 = (option_env!("HOME")).unwrap(); // Violation: parenthesized
    
    let opt = option_env!("PATH");
    let path = opt.unwrap(); // This won't be caught as it's separated
    
    println!("Different styles: {}, {}, {}", home1, home2, path);
}