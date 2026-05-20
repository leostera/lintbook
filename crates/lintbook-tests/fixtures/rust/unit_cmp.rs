fn main() {
    let unit_val = ();

    // Violations - comparing unit types
    if unit_val == () {  // Always true
        println!("This will always execute");
    }

    if () == unit_val {  // Always true
        println!("This will always execute");
    }

    if () != () {  // Always false
        println!("This will never execute");
    }

    // More violations with function calls that return unit
    let result1 = println!("hello");
    let result2 = println!("world");

    if result1 == result2 {  // Comparing unit values, always true
        println!("Always true");
    }

    if result1 == () {  // Comparing unit with unit literal, always true
        println!("Always true");
    }

    // No violations - normal comparisons
    let x = 5;
    let y = 10;

    if x == y {  // Normal comparison
        println!("x equals y");
    }

    if x != 0 {  // Normal comparison
        println!("x is not zero");
    }

    // No violations - comparing Options or Results containing unit
    let opt1: Option<()> = Some(());
    let opt2: Option<()> = None;

    if opt1 == opt2 {  // Comparing Options, not unit types directly
        println!("Options are equal");
    }
}