fn main() {
    let mut x = 5;
    let mut y = 10;

    // Violations - self assignments
    x = x;  // Self-assignment, no effect
    y = y;  // Self-assignment, no effect

    // Violations with field access
    struct Point { x: i32, y: i32 }
    let mut p = Point { x: 1, y: 2 };
    p.x = p.x;  // Self-assignment of field
    p.y = p.y;  // Self-assignment of field

    // Violations with array/indexing
    let mut arr = [1, 2, 3];
    arr[0] = arr[0];  // Self-assignment of array element

    // Violations with compound assignments that are self-referential
    // Note: x += x, x *= x are mathematically meaningful but suspicious
    x += x;  // Suspicious self-assignment (doubles x)
    x *= x;  // Suspicious self-assignment (squares x)
    x -= x;  // Results in 0
    x /= x;  // Results in 1 (if x != 0)

    // No violations - normal assignments
    x = y;     // Different variables
    y = x + 1; // Expression, not self-assignment
    x = 42;    // Literal assignment

    // No violations - normal compound assignments
    x += 1;    // Adding literal
    y -= 2;    // Subtracting literal
    x *= 2;    // Multiplying by literal
}