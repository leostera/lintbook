// Test cases for RS004: async_yields_async

use std::future::Future;

async fn some_async_function() -> i32 {
    42
}

fn test_async_blocks_returning_futures() {
    // These should trigger violations - async blocks returning Futures
    let future1 = async {
        return some_async_function(); // Returns Future<i32> instead of i32
    };
    
    let future2 = async {
        tokio::spawn(async { 42 }) // Returns JoinHandle, not the value
    };
    
    let future3 = async {
        tokio::time::sleep(std::time::Duration::from_millis(100)) // Returns Future<()>
    };
}

fn test_proper_async_blocks() {
    // These should NOT trigger violations - properly awaited
    let future1 = async {
        some_async_function().await // Properly awaited
    };
    
    let future2 = async {
        tokio::spawn(async { 42 }).await.unwrap() // Properly awaited
    };
    
    let future3 = async {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await; // Properly awaited
        42
    };
}

fn test_sync_blocks() {
    // These should NOT trigger violations - not async blocks
    let value1 = {
        some_sync_function() // Not an async block
    };
    
    let value2 = {
        return 42; // Regular return in sync block
    };
}

fn some_sync_function() -> i32 {
    100
}

fn test_async_functions_not_blocks() {
    // These should NOT trigger violations - async functions, not blocks
    async fn inner_async() -> i32 {
        some_async_function().await
    }
    
    async fn another_async() -> i32 {
        return some_async_function().await;
    }
}

fn test_mixed_cases() {
    // This should trigger a violation
    let bad_future = async {
        if true {
            some_async_function().await
        } else {
            some_async_function() // This returns Future without await
        }
    };
    
    // This should NOT trigger - all paths are awaited
    let good_future = async {
        if true {
            some_async_function().await
        } else {
            some_async_function().await
        }
    };
}

fn test_future_constructors() {
    // These should trigger violations - creating Futures without awaiting
    let future1 = async {
        Future::ready(42) // Returns Future without await
    };
    
    let future2 = async {
        std::future::pending::<i32>() // Returns Future without await
    };
}