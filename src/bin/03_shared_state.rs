//! 03_shared_state.rs
//!
//! This example shows how to share mutable state across multiple tasks.
//!
//! Concepts:
//! 1. `std::sync::Arc`: Atomic Reference Counting. Allows multiple owners of the same data.
//!    Required because tasks might outlive the function that spawned them.
//! 2. `tokio::sync::Mutex`: An async-aware Mutex.
//!    - Unlike `std::sync::Mutex`, holding this lock across an `.await` point is safe (mostly).
//!    - However, for simple data, `std::sync::Mutex` is often fine if you don't await while holding it.
//!    - We use `tokio::sync::Mutex` here to demonstrate async locking.
//!    - **WARNING**: Holding a lock across an `.await` (like we do here with `sleep`) prevents
//!      other tasks from accessing the lock. This simulates a "long critical section".
//!      In real code, try to minimize the time you hold locks!
//! 3. `tokio::sync::RwLock`: Read-Write lock. Allows multiple readers or one writer.
//!
//! Scenario:
//! We have a "Database" (a HashMap) protected by a Mutex/RwLock.
//! Multiple tasks will try to read and write to it concurrently.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // We wrap our data in an RwLock (for interior mutability) and then in an Arc (for shared ownership).
    // RwLock is great when you have many readers and few writers.
    let db = Arc::new(RwLock::new(HashMap::new()));

    // Spawn 5 "Reader" tasks
    let mut handles = vec![];
    for i in 0..5 {
        let db_clone = db.clone();
        handles.push(tokio::spawn(async move {
            // Simulate reading periodically
            for _ in 0..3 {
                // Acquire read lock
                let read_guard = db_clone.read().await;
                println!("[Reader {}] Keys: {:?}", i, read_guard.keys());
                // Drop the guard automatically when it goes out of scope
                drop(read_guard); 
                sleep(Duration::from_millis(100)).await;
            }
        }));
    }

    // Spawn 1 "Writer" task
    let db_clone = db.clone();
    handles.push(tokio::spawn(async move {
        for i in 0..5 {
            // Acquire write lock
            let mut write_guard = db_clone.write().await;
            let key = format!("key_{}", i);
            let val = i * 10;
            println!("[Writer] Inserting {}: {}", key, val);
            write_guard.insert(key, val);
            // Drop guard
            drop(write_guard);
            sleep(Duration::from_millis(150)).await;
        }
    }));

    // Wait for all tasks to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Final state
    let final_db = db.read().await;
    println!("Final DB state: {:?}", *final_db);
}
