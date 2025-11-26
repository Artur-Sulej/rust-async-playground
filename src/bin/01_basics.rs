//! 01_basics.rs
//!
//! This example covers the absolute basics of async Rust with Tokio.
//!
//! Concepts:
//! 1. `#[tokio::main]`: The macro that sets up the async runtime.
//! 2. `async fn`: Defining asynchronous functions.
//! 3. `.await`: Yielding control to the runtime until a future completes.
//! 4. `tokio::spawn`: Launching a new background task (green thread).
//! 5. `tokio::join!`: Waiting for multiple futures to complete concurrently.
//!
//! Key Takeaway:
//! Async tasks are lightweight. You can spawn thousands of them.
//! They must be `Send` to be moved across threads by the work-stealing runtime.

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("Start of main");

    // 1. Calling an async function does nothing until you `.await` it.
    // This runs synchronously on the current task.
    let result = do_work("Task A", 1).await;
    println!("Task A finished with result: {}", result);

    // 2. Concurrency with `tokio::join!`
    // We can run multiple futures concurrently on the SAME task.
    println!("\nStarting Task B and Task C concurrently...");
    let (res_b, res_c) = tokio::join!(do_work("Task B", 2), do_work("Task C", 1));
    println!("Joined tasks finished: B={}, C={}", res_b, res_c);

    // 3. Spawning background tasks with `tokio::spawn`
    // This creates a NEW task that can run in parallel on a different thread.
    // The return value is a `JoinHandle`.
    println!("\nSpawning background Task D...");
    let handle_d = tokio::spawn(async { do_work("Task D (background)", 3).await });

    // We can continue doing other work in main while Task D runs.
    println!("Main is doing other work...");
    sleep(Duration::from_millis(500)).await;

    // Await the JoinHandle to get the result of the spawned task.
    // Note: `await`ing a handle returns a Result (it might have panicked).
    match handle_d.await {
        Ok(val) => println!("Task D joined successfully with result: {}", val),
        Err(e) => println!("Task D failed: {:?}", e),
    }

    // 4. The `Sync` Trait
    // `Sync` means it's safe for multiple threads to access `&T` concurrently.
    // `Send` means it's safe to move `T` to another thread.
    //
    // To share data between tasks (which might run on different threads), we often use `Arc<T>`.
    // `Arc<T>` is `Send` ONLY if `T` is `Sync`.
    //
    // Example: `String` is `Sync`, so `Arc<String>` is `Send`. We can share it.
    use std::sync::Arc;
    let shared_data = Arc::new("Immutable shared state".to_string());

    let data_clone = shared_data.clone();
    let handle_e = tokio::spawn(async move {
        // We are accessing `data_clone` (which is an Arc) on a different thread.
        // This works because `String` is `Sync`.
        println!("Task E reading shared data: '{}'", data_clone);
    });
    handle_e.await.unwrap();

    // Note: `std::cell::RefCell` is NOT `Sync`.
    // If we tried to spawn a task with `Arc<RefCell<i32>>`, it would fail to compile!
    // Because `RefCell` cannot be safely accessed from multiple threads.

    println!("\nEnd of main");
}

/// Simulates some asynchronous work by sleeping.
async fn do_work(name: &str, seconds: u64) -> u64 {
    println!("{} starting... (will take {}s)", name, seconds);
    sleep(Duration::from_secs(seconds)).await;
    println!("{} done!", name);
    seconds * 10 // Return some dummy value
}
