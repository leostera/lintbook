// Test cases for RS025: invisible_characters

fn test_normal_code() {
    // This should NOT trigger - normal ASCII characters
    let normal_string = "Hello, world!";
    let number = 42;
    println!("Normal code: {} {}", normal_string, number);
}

fn test_zero_width_space() {
    // This should trigger violations - contains zero-width space
    let variable_name = "test"; // Zero-width space between 'test' and comment
    println!("Variable: {}", variable_name);
}

fn test_directional_marks() {
    // This should trigger violations - contains RTL/LTR marks
    let text = "Hello‪world‬"; // Contains LTR embedding and pop directional formatting
    println!("Text with directional marks: {}", text);
}

fn test_bom_character() {
    // This should trigger violation - contains BOM
    let bom_string = "﻿text"; // BOM at start
    println!("String with BOM: {}", bom_string);
}

fn test_soft_hyphen() {
    // This should trigger violation - contains soft hyphen
    let hyphenated = "soft­hyphen"; // Contains soft hyphen
    println!("Hyphenated: {}", hyphenated);
}

fn test_variation_selectors() {
    // This should trigger violation - contains variation selector
    let emoji_var = "🏻"; // Contains variation selector
    println!("Emoji variation: {}", emoji_var);
}

fn test_multiple_invisible_chars() {
    // This should trigger multiple violations
    let problematic = "a​b‌c‍d"; // Multiple zero-width characters
    println!("Problematic string: {}", problematic);
}

fn test_in_comments() {
    // This should trigger violation - invisible char in comment
    /* This comment has a zero-width space: ​ */
    
    // Another comment with directional mark: ‪
    let normal = "normal";
}

fn test_in_string_literals() {
    // This should trigger violations - invisible chars in strings
    let string1 = "Hello​world"; // Zero-width space in string
    let string2 = "Text‪with‬marks"; // Directional marks in string
    let string3 = "Soft­hyphen"; // Soft hyphen in string
    
    println!("{} {} {}", string1, string2, string3);
}

fn test_in_identifiers() {
    // This should trigger violations - invisible chars in identifiers
    let variable​name = 42; // Zero-width space in identifier
    let another‌var = "test"; // Zero-width non-joiner in identifier
    
    println!("{} {}", variable​name, another‌var);
}

fn test_valid_unicode() {
    // These should NOT trigger - valid Unicode characters
    let emoji = "🦀"; // Rust crab emoji
    let chinese = "你好"; // Chinese characters
    let greek = "αβγ"; // Greek letters
    let math = "∑∏∫"; // Mathematical symbols
    
    println!("Valid Unicode: {} {} {} {}", emoji, chinese, greek, math);
}

fn test_normal_whitespace() {
    // These should NOT trigger - normal whitespace
    let spaced = "word word"; // Normal space
    let tabbed = "word	word"; // Tab character
    let newline = "word
word"; // Newline
    
    println!("Normal whitespace: '{}' '{}' '{}'", spaced, tabbed, newline);
}

fn test_mixed_content() {
    // This should trigger some violations but not others
    let mixed1 = "normal text"; // Should not trigger
    let mixed2 = "text​with​problems"; // Should trigger (zero-width spaces)
    let mixed3 = "🦀 normal emoji"; // Should not trigger
    let mixed4 = "text‪direction‬problem"; // Should trigger (directional marks)
    
    println!("{} {} {} {}", mixed1, mixed2, mixed3, mixed4);
}

// Function name with invisible character - should trigger
fn function​with​invisible​chars() {
    println!("Function with invisible characters in name");
}

// Struct with invisible character - should trigger
struct Data​With​Problems {
    field: i32,
}

// Implementation with invisible characters - should trigger
impl Data​With​Problems {
    fn method​name(&self) -> i32 {
        self.field
    }
}

// Macro with invisible characters - should trigger
macro_rules! macro​with​problems {
    () => {
        println!("Macro with invisible chars");
    };
}