fn main() {
    let text = "hello,world,rust";
    
    // Violations - suspicious splitn calls
    let parts0 = text.splitn(0, ',');  // Returns empty iterator
    let parts1 = text.splitn(1, ',');  // Returns original string as single element
    
    // Also check rsplitn
    let rparts0 = text.rsplitn(0, ','); // Returns empty iterator
    let rparts1 = text.rsplitn(1, ','); // Returns original string as single element
    
    // More violations with different strings
    let data = "a:b:c";
    let empty_split = data.splitn(0, ':'); // Violation
    let single_split = data.splitn(1, ':'); // Violation
    
    // No violations - normal splitn usage
    let parts2 = text.splitn(2, ','); // Split into at most 2 parts
    let parts3 = text.splitn(3, ','); // Split into at most 3 parts
    let all_parts = text.split(',');  // Normal split, not splitn
    
    // Normal rsplitn usage
    let rparts2 = text.rsplitn(2, ','); // Split from right into at most 2 parts
    let rparts5 = text.rsplitn(5, ','); // Split from right into at most 5 parts
    
    // No violations with variables (we only check literals)
    let n = 0;
    let var_split = text.splitn(n, ','); // Not a violation (variable, not literal)
}