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

    println!("\nEnd of main");
}

/// Simulates some asynchronous work by sleeping.
async fn do_work(name: &str, seconds: u64) -> u64 {
    println!("{} starting... (will take {}s)", name, seconds);
    sleep(Duration::from_secs(seconds)).await;
    println!("{} done!", name);
    seconds * 10 // Return some dummy value
}
