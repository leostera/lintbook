// Test cases for RS029: let_underscore_lock

use std::sync::{Mutex, RwLock, Arc};

fn test_let_underscore_mutex_violations() {
    let mutex = Mutex::new(42);
    let rwlock = RwLock::new(vec![1, 2, 3]);

    // These should trigger violations - immediately dropping lock guards
    let _ = mutex.lock(); // Violation: lock immediately dropped
    let _ = mutex.try_lock(); // Violation: lock immediately dropped
    let _ = rwlock.read(); // Violation: read lock immediately dropped
    let _ = rwlock.write(); // Violation: write lock immediately dropped
    let _ = rwlock.try_read(); // Violation: read lock immediately dropped
    let _ = rwlock.try_write(); // Violation: write lock immediately dropped

    println!("Locks were immediately dropped - no synchronization!");
}

fn test_proper_lock_usage() {
    let mutex = Mutex::new(42);
    let rwlock = RwLock::new(vec![1, 2, 3]);

    // These should NOT trigger violations - proper lock usage
    let guard = mutex.lock().unwrap();
    println!("Value: {}", *guard);
    drop(guard); // Explicit drop

    let _guard = mutex.lock().unwrap(); // Named with underscore prefix

    {
        let read_guard = rwlock.read().unwrap();
        println!("Data: {:?}", *read_guard);
    } // Guard dropped at end of scope

    let write_guard = rwlock.write().unwrap();
    // Use write_guard...
    drop(write_guard);
}

fn test_arc_mutex_patterns() {
    let shared_mutex = Arc::new(Mutex::new(100));
    let shared_rwlock = Arc::new(RwLock::new(String::from("data")));

    // These should trigger violations
    let _ = shared_mutex.lock(); // Violation
    let _ = shared_rwlock.read(); // Violation
    let _ = shared_rwlock.write(); // Violation

    // These should NOT trigger violations
    let guard = shared_mutex.lock().unwrap();
    println!("Shared value: {}", *guard);

    let _read_guard = shared_rwlock.read().unwrap();
}

fn test_unwrap_and_expect() {
    let mutex = Mutex::new("test");

    // These should trigger violations - even with unwrap/expect
    let _ = mutex.lock().unwrap(); // Violation
    let _ = mutex.try_lock().expect("Failed to lock"); // Violation

    // These should NOT trigger violations
    let guard = mutex.lock().unwrap();
    println!("Value: {}", *guard);

    let _guard = mutex.try_lock().expect("Failed to lock");
}

#[cfg(feature = "tokio")]
async fn test_async_locks() {
    use tokio::sync::{Mutex as AsyncMutex, RwLock as AsyncRwLock};

    let async_mutex = AsyncMutex::new(42);
    let async_rwlock = AsyncRwLock::new(vec![1, 2, 3]);

    // These should trigger violations - async locks immediately dropped
    let _ = async_mutex.lock().await; // Violation
    let _ = async_rwlock.read().await; // Violation
    let _ = async_rwlock.write().await; // Violation

    // These should NOT trigger violations
    let guard = async_mutex.lock().await;
    println!("Async value: {}", *guard);

    let _read_guard = async_rwlock.read().await;
}

fn test_parking_lot_locks() {
    // Note: These would require parking_lot dependency
    // Simulating the patterns for demonstration

    struct ParkingLotMutex<T>(std::sync::Mutex<T>);
    impl<T> ParkingLotMutex<T> {
        fn lock(&self) -> std::sync::MutexGuard<T> {
            self.0.lock().unwrap()
        }
        fn try_lock(&self) -> Option<std::sync::MutexGuard<T>> {
            self.0.try_lock().ok()
        }
    }

    let parking_mutex = ParkingLotMutex(std::sync::Mutex::new(123));

    // These should trigger violations
    let _ = parking_mutex.lock(); // Violation
    if let Some(guard) = parking_mutex.try_lock() {
        let _ = guard; // This might be a violation too, but not our focus
    }

    // This should NOT trigger violation
    let _guard = parking_mutex.lock();
}

fn test_lock_variable_names() {
    let m = Mutex::new(1);
    let mtx = Mutex::new(2);
    let my_lock = Mutex::new(3);
    let data_mutex = Mutex::new(4);
    let GLOBAL_LOCK = Mutex::new(5);

    // These should trigger violations - various lock variable names
    let _ = m.lock(); // Violation
    let _ = mtx.lock(); // Violation
    let _ = my_lock.lock(); // Violation
    let _ = data_mutex.lock(); // Violation
    let _ = GLOBAL_LOCK.lock(); // Violation

    // These should NOT trigger violations
    let _guard1 = m.lock();
    let _guard2 = mtx.lock();
    let _guard3 = my_lock.lock();
}

fn test_non_lock_underscore_assignments() {
    let data = vec![1, 2, 3];
    let result = Some(42);

    // These should NOT trigger violations - not lock calls
    let _ = data.iter().count(); // Not a lock
    let _ = result.unwrap(); // Not a lock
    let _ = calculate_something(); // Not a lock
    let _ = "hello".to_string(); // Not a lock
}

fn calculate_something() -> i32 {
    42
}

fn test_method_chaining() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));

    // This should trigger violation - lock in method chain assigned to _
    let _ = mutex.clone().lock(); // Violation

    // This should NOT trigger violation - proper usage
    let data = mutex.lock().unwrap();
    let len = data.len();
    println!("Length: {}", len);
}

fn test_complex_expressions() {
    let mutex1 = Mutex::new(1);
    let mutex2 = Mutex::new(2);

    // These should trigger violations - complex lock expressions
    let _ = if true { mutex1.lock() } else { mutex2.lock() }; // Violation
    let _ = mutex1.try_lock().or_else(|| mutex2.try_lock()); // Violation

    // These should NOT trigger violations
    let guard = if true { mutex1.lock() } else { mutex2.lock() };
    match guard {
        Ok(g) => println!("Got lock: {}", *g),
        Err(_) => println!("Failed to get lock"),
    }
}

fn test_lock_with_question_mark() {
    fn try_lock_operation() -> Result<(), Box<dyn std::error::Error>> {
        let mutex = Mutex::new(42);

        // This should trigger violation - lock with ? assigned to _
        let _ = mutex.try_lock()?; // Violation

        // This should NOT trigger violation
        let _guard = mutex.try_lock()?;

        Ok(())
    }

    let _ = try_lock_operation(); // This underscore is fine - not a lock
}

struct CustomLock {
    inner: Mutex<i32>,
}

impl CustomLock {
    fn blocking_lock(&self) -> std::sync::MutexGuard<i32> {
        self.inner.lock().unwrap()
    }

    fn blocking_read(&self) -> std::sync::MutexGuard<i32> {
        self.inner.lock().unwrap()
    }

    fn blocking_write(&self) -> std::sync::MutexGuard<i32> {
        self.inner.lock().unwrap()
    }
}

fn test_custom_lock_methods() {
    let custom = CustomLock {
        inner: Mutex::new(100),
    };

    // These should trigger violations - custom lock methods
    let _ = custom.blocking_lock(); // Violation
    let _ = custom.blocking_read(); // Violation
    let _ = custom.blocking_write(); // Violation

    // These should NOT trigger violations
    let _guard1 = custom.blocking_lock();
    let _guard2 = custom.blocking_read();
    let _guard3 = custom.blocking_write();
}