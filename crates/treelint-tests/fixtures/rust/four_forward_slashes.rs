fn main() {
    //// This is a violation - four forward slashes
    let x = 5;
    
    ///// This is also a violation - five forward slashes
    let y = 10;
    
    //////// This is a violation - eight forward slashes
    let z = 15;
    
    //// Some commented out code might look like this
    //// let old_code = "disabled";
    
    //////////////////////////////// Many slashes are bad style
    let separator = "bad";
    
    // This is fine - standard comment
    let normal = "ok";
    
    /// This is also fine - documentation comment
    let documented = "good";
    
    /* This is fine - block comment */
    let block_commented = "ok";
    
    /*
     * Multi-line block comment
     * is also fine
     */
    let multi_line = "good";
    
    // Normal comment with some // slashes inside is fine
    let inline_slashes = "ok";
    
    /// Documentation with some /// extra slashes is ok
    let doc_with_slashes = "good";
    
    // Even URLs like https://example.com are fine
    let url = "https://example.com";
}